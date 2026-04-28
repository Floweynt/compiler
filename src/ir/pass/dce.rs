use std::collections::VecDeque;

use slotmap::SecondaryMap;

use crate::ir::{
    FunctionHandle, Module,
    lir::code::NodeRefLike,
    pass::{FunctionPass, lir_body},
};

pub struct DeadCodeElimination;

impl FunctionPass for DeadCodeElimination {
    fn apply_function(module: &mut Module, func: FunctionHandle) {
        let body = lir_body(module, func);

        let stop = body.stop_node().underlying_ref();

        let mut visited = SecondaryMap::new();

        let mut queue = VecDeque::new();

        queue.push_back(stop);

        visited.insert(body.start_node().underlying_ref(), ());

        while let Some(node) = queue.pop_front() {
            for n_use in node.inputs(body).filter_map(|f| *f) {
                if visited.insert(n_use.node, ()).is_none() {
                    queue.push_back(n_use.node);
                }
            }
        }

        for ele in body.nodes.keys().collect::<Vec<_>>() {
            if !visited.contains_key(ele) {
                body.remove(ele);
            }
        }
    }
}
