use std::collections::VecDeque;
use slotmap::SecondaryMap;

use crate::ir::{
    function::Function,
    hir::code::{HirFunctionBody, Label},
    lir::code::LirGraph,
};
use crate::ir::hir::code::{BlockTerminator, HirOpc, LvtRef};
use crate::ir::lir::code::NodeUse;
use super::ssa::SSA;


/// Lower stack-machine HIR into Sea-of-Nodes LIR.
///
/// Phase 1 scaffolding:
/// 1) consume SSA mining output (phi locations, CFG metadata),
/// 2) create graph shell from function signature,
/// 3) leave instruction lowering / renaming / phi materialization as TODO steps.
pub fn lower_hir_to_son(
    func: &Function,
    body: &HirFunctionBody,
    ssa: &SSA,
) -> LirGraph {
    let mut graph = LirGraph::make_graph(func);
    let mut symbols = SecondaryMap::new();
    let mut stack = Vec::new();
    let mut bfs = VecDeque::new();
    let mut visited = SecondaryMap::new();

    // Seed a frame row for each block. We'll replace this placeholder index with
    // actual NodeUse handles once control lowering is implemented.
    bfs.push_back(body.entry());
    while (!bfs.is_empty()) {
        let label = bfs.pop_front().unwrap();
        let block = body.bb(label).unwrap();
        symbols.insert(label, SecondaryMap::new());

        for insn in block.body() {
            match insn.opc {
                // HirOpc::LdcI(val) => {
                //     let node = graph.make_const(val);
                //     stack.push(node);
                // }
                HirOpc::Load(lvt) => {
                    if ssa.phi_nodes.get(label).unwrap().contains(&lvt) {
                        // let node = graph.make_phi(last_used(lvt, label, symbols, ssa.scaffold.predecessors));
                        // stack.push(node);
                        // symbols.insert(lvt, node);
                        todo!()
                    } else {
                        stack.push(symbols.get(label).unwrap().get(lvt).copied().unwrap());
                    }
                }
                HirOpc::Store(lvt) => {
                    // let node = graph.make_store(lvt, stack.pop().unwrap());
                    symbols.get_mut(label).unwrap().insert(lvt, stack.pop().unwrap());
                }
                HirOpc::Add => {
                    let node = graph.make_add(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::Sub => {
                    let node = graph.make_sub(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::Mul => {
                    // need to differentiate between signed and unsigned?
                    let node = todo!(); //graph.make_mul(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::Div => {
                    let node = todo!(); //graph.make_div(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::Mod => {
                    let node = todo!(); //graph.make_mod(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::BitAnd => {
                    let node = graph.make_bit_and(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::BitOr => {
                    let node = graph.make_bit_or(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::BitNot => {
                    let node = graph.make_bit_not(stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::SExt => {
                    let node = graph.make_sext(stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::ZExt => {
                    let node = graph.make_zext(stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::CmpEq => {
                    let node = graph.make_cmp_eq(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::CmpNe => {
                    let node = graph.make_cmp_ne(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::CmpGt => {
                    let node = graph.make_cmp_gt(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::CmpLt => {
                    let node = graph.make_cmp_lt(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::CmpGe => {
                    let node = graph.make_cmp_ge(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::CmpLe => {
                    let node = graph.make_cmp_le(stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(node);
                }
                HirOpc::Invoke { arity } => {
                    let node = todo!(); // graph.make_call(arity, stack.pop().unwrap());
                    stack.push(node);
                }
                _ => todo!(),
            }
        }

        visited.insert(label, 1);
        let terminator = block.terminator();
        match terminator {
            BlockTerminator::Jmp(label) => {
                // todo!() wire up control edge in graph

                if !visited.contains_key(*label) {
                    bfs.push_back(*label);
                }
            }
            BlockTerminator::Branch{if_true, if_false} => {
                // todo!() wire up branch edge in graph

                if !visited.contains_key(*if_true) {
                    bfs.push_back(*if_true);
                }
                if !visited.contains_key(*if_false) {
                    bfs.push_back(*if_false);
                }
            }
            BlockTerminator::Ret => {
                todo!()
            }
            BlockTerminator::Unreachable => {
                panic!()
            }
        }
    }

    // TODO Phase 2: Build control skeleton
    // - single predecessor blocks: thread control through
    // - multi predecessor blocks: create Merge and map predecessor order
    // - preserve predecessor ordering for later phi operand indexing
    let _preds = &ssa.cfg.predecessors;

    // TODO Phase 3: Materialize phi nodes from mined locations.
    // for (block, locals) in ssa.phi_nodes.iter() { ... }
    let _phi_sites = &ssa.phi_nodes;

    // TODO Phase 4: Lower stack-machine instructions and perform SSA renaming.
    // - LdcI -> const node
    // - Load/Store -> local value table via rename stacks
    // - arithmetic ops -> graph.make_add/sub/mul/div/mod
    // - Ret -> control to Stop
    let _ = &mut graph;

    graph
}

fn last_used(
    symbol: LvtRef,
    label: Label,
    symbol_table: &SecondaryMap<Label, SecondaryMap<LvtRef, NodeUse>>,
    predecessors: SecondaryMap<Label, Vec<Label>>
) -> Vec<NodeUse> {
    let preds = predecessors.get(label).unwrap();
    // while not found, go back?
    todo!()
}
