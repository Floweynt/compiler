use std::collections::{HashMap, HashSet};

use slotmap::SecondaryMap;

use crate::ir::hir::code::{HirFunctionBody, HirOpc, Label, LvtRef};

use super::dominator::DominatorTree;

/// SSA name is one version of a source local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SsaName {
    pub local: LvtRef,
    pub version: u32,
}

#[derive(Debug)]
pub struct SsaScaffold {
    pub predecessors: SecondaryMap<Label, Vec<Label>>,
    pub idom: SecondaryMap<Label, Label>,
    // pub dom_tree_children: SecondaryMap<Label, Vec<Label>>,
    pub dom_frontier: SecondaryMap<Label, Vec<Label>>,
}

/// Mutable state used for variable renaming.
#[derive(Debug, Default)]
pub struct RenameState {
    /// Next version counter for each local.
    pub next_version: HashMap<LvtRef, u32>,
    /// Active SSA names while traversing dominator tree.
    pub stacks: HashMap<LvtRef, Vec<SsaName>>,
}

#[derive(Debug)]
pub struct SSA {
    pub scaffold: SsaScaffold,
    pub phi_nodes: SecondaryMap<Label, Vec<LvtRef>>,
    pub rename_state: RenameState,
}

/// Top-level SSA construction.
pub fn build_ssa(body: &HirFunctionBody) -> SSA {
    let scaffold = build_scaffold(body);
    let def_blocks = collect_def_blocks(body);
    let phi_nodes = place_phi_functions(body, &scaffold, &def_blocks);
    let rename_state = rename_variables(body, &scaffold, &phi_nodes);

    SSA {
        scaffold,
        phi_nodes,
        rename_state,
    }
}

/// Build scaffold needed before placing phi nodes.
pub fn build_scaffold(body: &HirFunctionBody) -> SsaScaffold {
    let predecessors = compute_predecessors(body);
    let dom = DominatorTree::make_dominator_tree(body);
    // let dom_tree_children = compute_dom_tree_children(body, &dom.graph);
    let dom_frontier = compute_dominance_frontier(body, &dom.graph, &predecessors);

    SsaScaffold {
        predecessors,
        idom: dom.graph,
        // dom_tree_children,
        dom_frontier,
    }
}

/// Compute predecessor lists from each block's outgoing edges.
pub fn compute_predecessors(body: &HirFunctionBody) -> SecondaryMap<Label, Vec<Label>> {
    let mut preds = SecondaryMap::<Label, Vec<Label>>::new();

    for (label, _) in body.blocks() {
        preds.insert(label, Vec::new());
    }

    for (src, bb) in body.blocks() {
        for dst in bb.successors() {
            if let Some(list) = preds.get_mut(dst) {
                list.push(src);
            }
        }
    }

    preds
}

/// Convert idom relation into explicit dominator-tree children lists.
pub fn compute_dom_tree_children(
    body: &HirFunctionBody,
    idom: &SecondaryMap<Label, Label>,
) -> SecondaryMap<Label, Vec<Label>> {
    let mut children = SecondaryMap::<Label, Vec<Label>>::new();

    for (label, _) in body.blocks() {
        children.insert(label, Vec::new());
    }

    for (label, _) in body.blocks() {
        if let Some(parent) = idom.get(label).copied() {
            if parent != label {
                children.get_mut(parent).unwrap().push(label);
            }
        }
    }

    children
}

/// Compute dominance frontier (DF) for every block following Cooper et al. (2001, p. 8, fig 5)'s algorithm.
pub fn compute_dominance_frontier(
    body: &HirFunctionBody,
    idom: &SecondaryMap<Label, Label>,
    // dom_tree_children: &SecondaryMap<Label, Vec<Label>>,
    predecessors: &SecondaryMap<Label, Vec<Label>>,
) -> SecondaryMap<Label, Vec<Label>> {
    let mut df = SecondaryMap::<Label, Vec<Label>>::new();
    for (label, _) in body.blocks() {
        df.insert(label, Vec::new());
    }

    for (block, _) in body.blocks() {
        if predecessors.get(block).unwrap().len() > 1 {
            for pred in predecessors.get(block).unwrap() {
                let mut runner = pred;
                while *runner != *idom.get(block).unwrap() {
                    df.get_mut(*runner).unwrap().push(block);
                    runner = &idom.get(*runner).unwrap();
                }
            }
        }
    }

    df
}

/// Gather definition blocks for each local.
/// For this HIR, defs are expected from `Store(local)`.
pub fn collect_def_blocks(body: &HirFunctionBody) -> HashMap<LvtRef, HashSet<Label>> {
    let mut def_blocks = HashMap::new();

    for (lvt_ref, _) in body.lvt() {
        def_blocks.insert(lvt_ref, HashSet::new());
    }

    for (label, bb) in body.blocks() {
        for insn in bb.body() {
            if let HirOpc::Store(local) = insn.opc {
                // assert that we don't assign to the same local multiple times in one block?
                def_blocks.get_mut(&local).unwrap().insert(label);
            }
        }
    }

    def_blocks
}

/// Place phi functions in the dominance frontier according to
/// Cytron et al. (1991, p. 470, section 5.1, fig 11)'s algorithm.
pub fn place_phi_functions(
    body: &HirFunctionBody,
    scaffold: &SsaScaffold,
    def_blocks: &HashMap<LvtRef, HashSet<Label>>,
) -> SecondaryMap<Label, Vec<LvtRef>> {
    let mut phi_nodes = SecondaryMap::<Label, Vec<LvtRef>>::new();
    let mut work = SecondaryMap::<Label, usize>::new();
    let mut has_alr = SecondaryMap::<Label, usize>::new();

    for (label, _) in body.blocks() {
        phi_nodes.insert(label, Vec::new());
        work.insert(label, 0);
        has_alr.insert(label, 0);
    }

    let mut iter_count = 0;

    for (local, defs) in def_blocks {
        let mut worklist: Vec<&Label> = Vec::new();
        for label in defs.iter() {
            worklist.push(label);
            *work.get_mut(*label).unwrap() += 1;
        }
        iter_count += 1;

        while let Some(block) = worklist.pop() { // X
            if let Some(frontier) = scaffold.dom_frontier.get(*block) {
                for frontier_block in frontier { // Y
                    if *has_alr.get(*frontier_block).unwrap() < iter_count {
                        *has_alr.get_mut(*frontier_block).unwrap() = iter_count;
                        phi_nodes.get_mut(*frontier_block).unwrap().push(*local);

                        if *work.get(*frontier_block).unwrap() < iter_count {
                            *work.get_mut(*frontier_block).unwrap() = iter_count;
                            worklist.push(frontier_block);
                        }
                    }
                }
            }
        }
    }

    phi_nodes
}

/// Rename locals into SSA names according to Cytron et al. (1991, p. 472, section 5.2, fig 12)'s algorithm.
pub fn rename_variables(
    body: &HirFunctionBody,
    scaffold: &SsaScaffold,
    phi_nodes: &SecondaryMap<Label, Vec<LvtRef>>,
) -> RenameState {
    let mut rename_state = RenameState::default();
    let mut stacks = HashMap::new();
    let mut counter = HashMap::new();

    for (lvt_ref, _) in body.lvt() {
        stacks.insert(lvt_ref, Vec::new());
        counter.insert(lvt_ref, 0);
    }

    fn search(
        entry: &Label,
        body: &HirFunctionBody,
        scaffold: &SsaScaffold,
        phi_nodes: &SecondaryMap<Label, Vec<LvtRef>>,
        stacks: &mut HashMap<LvtRef, Vec<SsaName>>,
        counter: &mut HashMap<LvtRef, u32>,
        rename_state: &mut RenameState
    ) {
        // for insn in body.bb.get(entry).unwrap().body() {
            // if insn not a phi, then for each var V used in RHS(insn), do
            // replace use of V with V_i, where i = Top(stacks[V])

            // for each var V in LHS(insn), do
            // save counter[V] as i
            // replace each use of V with new V_i, in LHS(insn)
            // push i onto stacks[V]
            // set counter[V] = i + 1
        // }

        // successors of entry in CFG; ie next in execution, control flow edges
        // for successor in body.bb.get(entry).unwrap().successors() {
            // save which_predecessor(entry of successor) as j
            // for each phi_function F in successor, do
            // replace j-th operand V in RHS(F) with V_i, where i = Top(stacks[V])
        // }

        // children of entry in dominator tree; ie idom(child) = entry
        // for child in scaffold.dom_tree_children.get(entry).unwrap() {
        //     search(child, body, scaffold, phi_nodes, stacks, counter, rename_state);
        // }

        // for each assignment A in entry, do
        // for each var V in old_LHS(A), pop stacks[V]
    }

    search(&body.entry(), body, scaffold, phi_nodes, &mut stacks, &mut counter, &mut rename_state);
    rename_state
}

