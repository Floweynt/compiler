use slotmap::SecondaryMap;

use crate::ir::{
    FunctionHandle, Module,
    lir::code::{CFGNodeRef, LirGraph, NodeRef, NodeRefLike},
    pass::{FunctionPass, lir_body},
};

pub struct GlobalCodeMotion;

fn rpo_cfg(graph: &LirGraph) -> Vec<CFGNodeRef> {
    fn rpo_cfg(
        node: NodeRef,
        graph: &LirGraph,
        visited: &mut SecondaryMap<NodeRef, ()>,
        rpo: &mut Vec<CFGNodeRef>,
    ) {
        let Some(cfg) = CFGNodeRef::cast(node, graph) else {
            return;
        };

        if visited.insert(node, ()).is_some() {
            return;
        }

        for out in node.users(graph) {
            rpo_cfg(*out, graph, visited, rpo);
        }

        rpo.push(cfg);
    }

    let mut v = SecondaryMap::new();
    let mut rpo = Vec::new();

    rpo_cfg(graph.start_node().underlying_ref(), graph, &mut v, &mut rpo);
    rpo
}

impl FunctionPass for GlobalCodeMotion {
    fn apply_function(&mut self, module: &mut Module, func: FunctionHandle) {
        let body = lir_body(module, func);

        for cfg in rpo_cfg(body).iter().rev() {}
    }
}
