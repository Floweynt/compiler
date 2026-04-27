#![allow(non_camel_case_types)] //because you smell <3

use slotmap::{SlotMap, SecondaryMap, new_key_type};
use smallvec::SmallVec;
use malachite::Integer;

use crate::ir::{function::Function, lir::types::OptType};

new_key_type! { pub struct NodeRef; }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct virtual_register(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct spill_slot(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum binary_operation_type {
    Add, Sub, Mul, Div, Mod, BitAnd, BitOr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum unary_operation_type {
    Neg, BitNot
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum comparator_type {
    Equal, NotEqual, GreaterThan, LessThan, GreaterEqual, LessEqual,
}
pub enum NodeKind {
    Start,
    Stop,
    Const(Integer),
    Copy,
    binary_operation(binary_operation_type),
    unary_operation(unary_operation_type),
    sign_extend,
    zero_extend,
    comparator(comparator_type),
    memory_store,
    memory_load,
    load_spill(spill_slot),
    store_spill(spill_slot),
    Jump,
    Branch,
    Call {
        arity: usize
    },
    Phi,
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
    virtual_register: Option<virtual_register>,
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

    define_virtual_register: SecondaryMap<NodeRef, virtual_register>,
    next_virtual_register: u32,
    next_spill_slot: u32,
}

impl LirGraph {
    pub fn make_graph(func: &Function) -> LirGraph {
        let mut start = Node {
            kind: NodeKind::Start,
            inputs: Default::default(),
            outputs: Default::default(),
            ty: Default::default(),
            virtual_register: None,
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
            virtual_register: None,
        });

        LirGraph {
            nodes,
            start_node,
            stop_node,
            define_virtual_register: SecondaryMap::new(),
            next_virtual_register: 0,
            next_spill_slot: 0,
        }
    }

    pub fn new_spill_slot(&mut self) -> spill_slot {
        let spill = spill_slot(self.next_spill_slot);
        self.next_spill_slot += 1;
        return spill;
    }

    pub fn new_virtual_register(&mut self) -> virtual_register {
        let vreg = virtual_register(self.next_virtual_register);
        self.next_virtual_register += 1;
        return vreg;
    }

    pub fn set_virtual_register(&mut self, node: NodeRef, virtual_register: virtual_register) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.virtual_register = Some(virtual_register);
        }
        self.define_virtual_register.insert(node, virtual_register);
    }

    pub fn get_virtual_register(&self, node: NodeRef) -> Option<virtual_register> {
        match self.nodes.get(node) {
            Some(n) => n.virtual_register,
            None => return None,
        }
    }

    pub fn add_node(&mut self, kind: NodeKind) -> NodeRef {
        self.nodes.insert(Node {
            kind,
            inputs: Default::default(),
            outputs: Default::default(),
            ty: Default::default(),
            virtual_register: None,

        })
    }
    pub fn start_node(&self) -> NodeRef {
        self.start_node
    }
    pub fn stop_node(&self) -> NodeRef {
        self.stop_node
    }
}