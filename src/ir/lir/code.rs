use slotmap::{SlotMap, new_key_type};

use smallvec::SmallVec;

use crate::ir::{
    function::Function,
    lir::{peeps::Idealize, types::OptType},
};

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

    Nil,
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
    inputs: SmallVec<[Option<NodeUse>; 4]>,
    outputs: SmallVec<[(OptType, SmallVec<[NodeDef; 4]>); 2]>,
}

impl Node {
    fn add_in(&mut self, graph: &mut LirGraph, node: NodeUse) {
        self.add_nullable_in(graph, Some(node));
    }

    fn add_nullable_in(&mut self, graph: &mut LirGraph, node: Option<NodeUse>) {}

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

    pub fn make_phi(&mut self, merge: MergeNodeRef, ty: OptType) -> (PhiNodeRef, NodeUse) {
        let node = {
            let mut n = Node {
                kind: NodeKind::Phi,
                inputs: Default::default(),
                outputs: Default::default(),
            };

            n.add_out(ty);
            n.add_in(self, merge.out(0));

            n
        };

        let node = PhiNodeRef(self.nodes.insert(node));

        (node, node.out(0))
    }

    pub fn use_type(&self, node: NodeUse) -> OptType {
        self.get_node(node.node).outputs[node.out_idx].0.clone()
    }

    fn make_binop(&mut self, lhs: NodeUse, rhs: NodeUse, k: NodeKind, out_ind: usize) -> NodeUse {
        let node = {
            let mut n = Node {
                kind: k,
                inputs: Default::default(),
                outputs: Default::default(),
            };

            n.add_out(OptType::Bottom);
            n.add_nullable_in(self, None);
            n.add_in(self, lhs);
            n.add_in(self, rhs);

            n
        };

        let node = self.nodes.insert(node);

        Idealize::rewrite(node.out(out_ind), self)
    }

    pub fn make_add(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::Add, 0)
    }

    pub fn make_sub(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::Sub, 0)
    }

    pub fn make_mul(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::Mul, 0)
    }

    pub fn make_div(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::DivMod, 0)
    }

    pub fn make_mod(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::DivMod, 1)
    }

    pub fn make_if(&mut self, ctrl: NodeUse, pred: NodeUse) -> (NodeUse, NodeUse) {
        let node = {
            let mut n = Node {
                kind: NodeKind::If,
                inputs: Default::default(),
                outputs: Default::default(),
            };

            n.add_out(OptType::CtrlTop);
            n.add_out(OptType::CtrlTop);
            n.add_in(self, ctrl);
            n.add_in(self, pred);

            n
        };

        let node = IfNodeRef(self.nodes.insert(node));

        let proj_true = {
            let mut n = Node {
                kind: NodeKind::CProj,
                inputs: Default::default(),
                outputs: Default::default(),
            };

            n.add_out(OptType::CtrlTop);
            n.add_in(self, node.out(0));

            n
        };

        let proj_true = self.nodes.insert(proj_true).out(0);

        let proj_false = {
            let mut n = Node {
                kind: NodeKind::CProj,
                inputs: Default::default(),
                outputs: Default::default(),
            };

            n.add_out(OptType::CtrlTop);
            n.add_in(self, node.out(1));

            n
        };

        let proj_false = self.nodes.insert(proj_false).out(0);

        (proj_true, proj_false)
    }
}
