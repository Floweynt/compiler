use crate::ir::lir::code::{LirGraph, NodeUse};

pub struct Idealize;

impl Idealize {
    pub fn rewrite(node: NodeUse, graph: &mut LirGraph) -> NodeUse {
        todo!()
    }
}
