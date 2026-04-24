use slotmap::{SlotMap, new_key_type};
use smallvec::SmallVec;

use crate::ir::{function::Function, lir::types::OptType};

new_key_type! { pub struct NodeRef; }

pub enum NodeKind {
    Start,
    Stop,
}

struct NodeUse {
    node: NodeRef,
    out_idx: usize,
}

struct NodeDef {
    node: NodeRef,
    in_idx: usize,
}

struct Node {
    kind: NodeKind,
    inputs: SmallVec<[NodeUse; 4]>,
    outputs: SmallVec<[SmallVec<[NodeDef; 4]>; 2]>,
    ty: SmallVec<[OptType; 2]>,
}

impl Node {
    fn add_out(&mut self, ty: OptType) {
        self.outputs.push(SmallVec::default());
        self.ty.push(ty);
    }
}

pub struct LirGraph {
    nodes: SlotMap<NodeRef, Node>,
    start_node: NodeRef,
    stop_node: NodeRef,
}

impl LirGraph {
    pub fn make_graph(func: &Function) -> LirGraph {
        let mut start = Node {
            kind: NodeKind::Start,
            inputs: Default::default(),
            outputs: Default::default(),
            ty: Default::default(),
        };

        start.add_out(OptType::CtrlTop);
        start.add_out(OptType::MemTop);

        for ele in &func.params {
            start.add_out(OptType::from_vt_pessimistic(ele.ty()));
        }

        let mut nodes = SlotMap::with_key();

        let start_node = nodes.insert(start);
        let stop_node = nodes.insert(Node {
            kind: NodeKind::Stop,
            inputs: Default::default(),
            outputs: Default::default(),
            ty: Default::default(),
        });

        LirGraph {
            nodes,
            start_node,
            stop_node,
        }
    }
}
