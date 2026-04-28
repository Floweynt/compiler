use std::collections::{HashMap, HashSet};

use slotmap::SecondaryMap;

use crate::ir::hir::code::{HirFunctionBody, HirOpc, Label, LvtRef};

use super::dominator::{
    DominatorTree, compute_dominance_frontier, compute_predecessors,
};

/// SSA name is one version of a source local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SsaName {
    pub local: LvtRef,
    pub version: u32,
}

#[derive(Debug)]
pub struct CFG {
    pub predecessors: SecondaryMap<Label, Vec<Label>>,
    pub idom: SecondaryMap<Label, Label>,
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
    pub cfg: CFG,
    /// block -> locals that need a phi at block entry
    pub phi_nodes: SecondaryMap<Label, Vec<LvtRef>>,
    /// Placeholder until rename is implemented in SoN lowering.
    pub rename_state: RenameState,
}

/// Top-level HIR SSA (half) pass.
///
/// This computes where phi nodes are required. Actual renaming/materialization
/// is handled during HIR -> SoN lowering.
pub fn build_ssa(body: &HirFunctionBody) -> SSA {
    let scaffold = build_scaffold(body);
    let def_blocks = collect_def_blocks(body);
    let phi_nodes = place_phi_functions(body, &scaffold, &def_blocks);
    let rename_state = rename_variables(body, &scaffold, &phi_nodes);

    SSA {
        cfg: scaffold,
        phi_nodes,
        rename_state,
    }
}

/// Build scaffold needed before placing phi nodes.
pub fn build_scaffold(body: &HirFunctionBody) -> CFG {
    let predecessors = compute_predecessors(body);
    let dom = DominatorTree::make_dominator_tree(body);
    let dom_frontier = compute_dominance_frontier(body, &dom.graph, &predecessors);

    CFG {
        predecessors,
        idom: dom.graph,
        dom_frontier,
    }
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
    cfg: &CFG,
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
            if let Some(frontier) = cfg.dom_frontier.get(*block) {
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
    cfg: &CFG,
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
        cfg: &CFG,
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
        // for child in cfg.dom_tree_children.get(entry).unwrap() {
        //     search(child, body, cfg, phi_nodes, stacks, counter, rename_state);
        // }

        // for each assignment A in entry, do
        // for each var V in old_LHS(A), pop stacks[V]
    }

    search(&body.entry(), body, cfg, phi_nodes, &mut stacks, &mut counter, &mut rename_state);
    rename_state
}

