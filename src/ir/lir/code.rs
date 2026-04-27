use slotmap::{SecondaryMap, SlotMap, new_key_type};
use std::vec;

use malachite::Integer;
use malachite::base::num::basic::traits::Phi;
use slotmap::{SlotMap, new_key_type};
use smallvec::SmallVec;

use crate::ir::{function::Function, lir::types::OptType};

new_key_type! { pub struct NodeRef; }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VirtualRegister(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpillSlot(pub u32);

pub enum NodeKind {
    Start,
    Stop,
    Call { arity: usize },

    // bin ops
    Add,
    Sub,
    Mul,
    DivMod,

    Phi,

    Ret,

    // control
    If,
    CProj,
    Merge,
    Loop,
}

pub struct NodeUse {
    node: NodeRef,
    out_idx: usize,
}

pub struct NodeDef {
    node: NodeRef,
    in_idx: usize,
}

pub struct Node {
    kind: NodeKind,
    inputs: SmallVec<[NodeUse; 4]>,
    outputs: SmallVec<[(OptType, SmallVec<[NodeDef; 4]>); 2]>,
}

impl Node {
    fn add_in(&mut self, graph: &LirGraph, node: NodeUse) {}

    fn add_out(&mut self, ty: OptType) {
        self.outputs.push((ty, SmallVec::default()));
    }

    fn n_outs(&self) -> usize {
        self.outputs.len()
    }
}

pub trait NodeRefLike: Copy {
    fn do_cast(other: NodeRef, graph: &LirGraph) -> Option<Self>;

    fn underlying_ref(self) -> NodeRef;

    #[inline(always)]
    fn cast<U: NodeRefLike>(other: U, graph: &LirGraph) -> Option<Self> {
        Self::do_cast(other.underlying_ref(), graph)
    }

    #[inline(always)]
    fn is<U: NodeRefLike>(other: U, graph: &LirGraph) -> bool {
        Self::cast(other, graph).is_some()
    }

    fn out(&self, ind: usize) -> NodeUse {
        NodeUse {
            node: self.underlying_ref(),
            out_idx: ind,
        }
    }
}

impl NodeRefLike for NodeRef {
    fn underlying_ref(self) -> NodeRef {
        self
    }

    fn do_cast(other: NodeRef, graph: &LirGraph) -> Option<Self> {
        Some(other)
    }
}

macro_rules! define_struct {
    ($name: ident, $($pat:pat_param)|+ $(,)?) => {
        #[derive(Clone, Copy)]
        #[repr(transparent)]
        pub struct $name(NodeRef);

        impl NodeRefLike for $name {
            fn do_cast(other: NodeRef, graph: &LirGraph) -> Option<Self> {
                match graph.get_node(other).kind {
                    $($pat)|+ => Some(Self(other)),
                    // TODO: add patterns in macro parameters
                    _ => None
                }
            }

            fn underlying_ref(self) -> NodeRef {
                self.0
            }
        }
    };
}

define_struct!(StartNodeRef, NodeKind::Start);
define_struct!(PhiNodeRef, NodeKind::Phi);
define_struct!(IfNodeRef, NodeKind::If);
define_struct!(MergeNodeRef, NodeKind::Merge);

impl StartNodeRef {
    pub fn out_arg(&self, idx: usize) -> NodeUse {
        self.out(idx + 2)
    }

    pub fn out_ctrl(&self) -> NodeUse {
        self.out(0)
    }

    pub fn out_mem(&self) -> NodeUse {
        self.out(1)
    }
}

pub struct LirGraph {
    nodes: SlotMap<NodeRef, Node>,
    start_node: StartNodeRef,
    stop_node: NodeRef,
}

impl LirGraph {
    pub fn make_graph(func: &Function) -> LirGraph {
        let mut start = Node {
            kind: NodeKind::Start,
            inputs: Default::default(),
            outputs: Default::default(),
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
        });

        LirGraph {
            nodes,
            start_node: StartNodeRef(start_node),
            stop_node,
        }
    }

    pub fn arg(&self, arg: usize) -> Option<NodeUse> {
        Some(self.start_node.out_arg(arg))
    }

    pub fn get_node(&self, node: NodeRef) -> &Node {
        self.nodes.get(node).unwrap()
    }

    pub fn make_phi(&mut self, merge: MergeNodeRef, ty: OptType) -> PhiNodeRef {
        PhiNodeRef(self.nodes.insert({
            let mut n = Node {
                kind: NodeKind::Phi,
                inputs: Default::default(),
                outputs: Default::default(),
            };

            n.add_out(ty);

            n
        }))
    }

    pub fn use_type(&self, node: NodeUse) {}

    pub fn make_add(&mut self, lhs: NodeUse, rhs: NodeUse) {}
}
