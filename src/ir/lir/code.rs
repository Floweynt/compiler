use malachite::Integer;
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

#[derive(Debug, Clone, Copy)]
pub enum NodeKind {
    Start,
    Stop,
    Call { arity: usize },

    Const,// { val: Integer },

    // bin ops
    Add,
    Sub,
    SMul,
    UMul,
    SDivMod,
    UDivMod,
    BitAnd, // i don't see the point of all this; we're just repeating all the enum kinds from hir
    BitOr,
    BitXor,
    CmpEq,
    CmpNe,
    CmpGt,
    CmpLt,
    CmpGe,
    CmpLe,

    // unops
    BitNot,
    SExt,
    ZExt,

    Phi,

    Ret,

    // control
    If,
    CProj,
    Merge,

    Nil,
}

#[derive(Debug, Clone, Copy)]
pub struct NodeUse {
    pub node: NodeRef,
    pub out_idx: usize,
}

#[derive(Debug)]
pub struct Node {
    kind: NodeKind,
    inputs: SmallVec<[Option<NodeUse>; 4]>,
    outputs: SmallVec<[(OptType, SmallVec<[NodeRef; 4]>); 2]>,
}

impl Node {
    fn add_in(&mut self, graph: &mut LirGraph, node: NodeUse) {
        self.add_nullable_in(graph, Some(node));
    }

    fn add_nullable_in(&mut self, graph: &mut LirGraph, node: Option<NodeUse>) {
        todo!()
    }

    fn add_out(&mut self, ty: OptType) {
        self.outputs.push((ty, SmallVec::default()));
    }

    fn n_outs(&self) -> usize {
        self.outputs.len()
    }

    fn inputs(&self) -> impl Iterator<Item = &Option<NodeUse>> {
        self.inputs.iter()
    }

    fn users(&self) -> impl Iterator<Item = &NodeRef> {
        self.outputs.iter().flat_map(|f| f.1.iter())
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

    fn inputs<'a>(&self, graph: &'a LirGraph) -> impl Iterator<Item = &'a Option<NodeUse>> {
        graph.get_node(self.underlying_ref()).inputs()
    }

    fn users<'a>(&self, graph: &'a LirGraph) -> impl Iterator<Item = &'a NodeRef> {
        graph.get_node(self.underlying_ref()).users()
    }

    fn kind(&self, graph: &LirGraph) -> NodeKind {
        graph.get_node(self.underlying_ref()).kind
    }
}

impl NodeRefLike for NodeRef {
    fn underlying_ref(self) -> NodeRef {
        self
    }

    fn do_cast(other: NodeRef, _graph: &LirGraph) -> Option<Self> {
        Some(other)
    }
}

macro_rules! define_struct {
    ($name: ident, $($pat:pat_param)|+ $(,)?) => {
        #[derive(Clone, Copy, Debug)]
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
define_struct!(StopNodeRef, NodeKind::Stop);
define_struct!(PhiNodeRef, NodeKind::Phi);
define_struct!(IfNodeRef, NodeKind::If);
define_struct!(
    CFGNodeRef,
    NodeKind::Start
        | NodeKind::Stop
        | NodeKind::Ret
        | NodeKind::If
        | NodeKind::CProj
        | NodeKind::Merge
);

impl IfNodeRef {
    fn out_true(self) -> NodeUse {
        self.out(0)
    }

    fn out_false(self) -> NodeUse {
        self.out(1)
    }
}

// what's the point of a merge node? just make the phi node have 2+ inputs? where's constructor?
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

#[derive(Debug)]
pub struct LirGraph {
    pub nodes: SlotMap<NodeRef, Node>,
    start_node: StartNodeRef,
    stop_node: StopNodeRef,
}

impl LirGraph {
    /// Create a barebones graph with only the start and stop nodes.
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
            stop_node: StopNodeRef(stop_node),
        }
    }

    pub fn arg(&self, arg: usize) -> Option<NodeUse> {
        Some(self.start_node.out_arg(arg))
    }

    pub fn get_node(&self, node: NodeRef) -> &Node {
        self.nodes.get(node).unwrap()
    }

    // idk what i'm supposed to do with this...
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

    pub fn start_node(&self) -> StartNodeRef {
        self.start_node
    }

    pub fn stop_node(&self) -> StopNodeRef {
        self.stop_node
    }

    pub fn make_const(&mut self, val: Integer) -> NodeUse {
        let node = Node {
            kind: NodeKind::Const,// { val },
            inputs: Default::default(),
            outputs: Default::default(),
        };

        self.nodes.insert(node).out(0)
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

    fn make_unop(&mut self, val: NodeUse, k: NodeKind, out_ind: usize) -> NodeUse {
        let node = {
            let mut n = Node {
                kind: k,
                inputs: Default::default(),
                outputs: Default::default(),
            };

            n.add_out(OptType::Bottom);
            n.add_nullable_in(self, None);
            n.add_in(self, val);

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

    pub fn make_smul(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::SMul, 0)
    }

    pub fn make_sdiv(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::SDivMod, 0)
    }

    pub fn make_smod(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::SDivMod, 1)
    }

    pub fn make_umul(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::UMul, 0)
    }

    pub fn make_udiv(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::UDivMod, 0)
    }

    pub fn make_umod(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::UDivMod, 1)
    }

    pub fn make_bit_and(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::BitAnd, 0)
    }

    pub fn make_bit_or(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::BitOr, 0)
    }

    pub fn make_bit_not(&mut self, val: NodeUse) -> NodeUse {
        self.make_unop(val, NodeKind::BitNot, 0)
    }

    pub fn make_sext(&mut self, val: NodeUse) -> NodeUse {
        self.make_unop(val, NodeKind::SExt, 0)
    }

    pub fn make_zext(&mut self, val: NodeUse) -> NodeUse {
        self.make_unop(val, NodeKind::ZExt, 0)
    }

    pub fn make_cmp_eq(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::CmpEq, 0)
    }

    pub fn make_cmp_ne(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::CmpNe, 0)
    }

    pub fn make_cmp_gt(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::CmpGt, 0)
    }

    pub fn make_cmp_lt(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::CmpLt, 0)
    }

    pub fn make_cmp_ge(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::CmpGe, 0)
    }

    pub fn make_cmp_le(&mut self, lhs: NodeUse, rhs: NodeUse) -> NodeUse {
        self.make_binop(lhs, rhs, NodeKind::CmpLe, 0)
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
            n.add_in(self, node.out_true());

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
            n.add_in(self, node.out_false());

            n
        };

        let proj_false = self.nodes.insert(proj_false).out(0);

        (proj_true, proj_false)
    }

    pub fn remove(&mut self, node: NodeRef) {
        todo!()
    }
}
