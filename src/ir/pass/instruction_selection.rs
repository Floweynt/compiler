#![allow(non_camel_case_types)]

//really simple instruction selector
//lowers lir nodes into opcodes

//TODO: add this to code.rs in lir (i don't want to mess up commit stuff or add unnecessary stuff)
/*
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum opcode {
    MovImm(Integer),
    Copy,

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,

    Neg,
    BitNot,

    SignExtend,
    ZeroExtend,

    Compare(comparator_type),
    Branch,

    Load,
    Store,
    LoadSpill(spill_slot),
    StoreSpill(spill_slot),

    Jump,
    Call {
        arity: usize
    },
}
*/
use std::collections::HashMap;
use malachite::Integer;
use crate::ir::{
    FunctionHandle,
    Module,
    function::FunctionBody,
    lir::{
        chaitin_briggs::lir_function,
        code::{binary_operation_type, unary_operation_type, NodeKind, NodeRef, opcode},
    },
    pass::FunctionPass,
};

pub struct instruction_selection;

impl FunctionPass for instruction_selection {
    fn apply_function(module: &mut Module, function: FunctionHandle) {
        let function = module.get_function_mut(function).unwrap();

        let FunctionBody::Lir(body) = &mut function.body else {
            return;
        };
        apply_lir_function(body);
    }
}

fn apply_lir_function(function: &mut lir_function) {
    let blocks = function.blocks.keys().collect::<Vec<_>>();

    for block_ref in blocks {
        let instructions = function.blocks[block_ref].instructions.clone();
        for node_ref in instructions {
            select_node(function, node_ref);
        }
    }
}

//simple node selector that just lowers the lir nodes
//need to implement optimizations
fn select_node(function: &mut lir_function, node_ref: NodeRef) {
    let opcode = {
        let node = function.graph.node(node_ref);

        match &node.kind {
            NodeKind::Start | NodeKind::Stop | NodeKind::Phi | NodeKind::opcode(_) => {
                return;
            }

            NodeKind::Const(value) => {
                opcode::MovImm(value.clone())
            }

            NodeKind::Copy => {
                opcode::Copy
            }

            NodeKind::binary_operation(operation) => {
                match *operation {
                    binary_operation_type::Add => opcode::Add,
                    binary_operation_type::Sub => opcode::Sub,
                    binary_operation_type::Mul => opcode::Mul,
                    binary_operation_type::Div => opcode::Div,
                    binary_operation_type::Mod => opcode::Mod,
                    binary_operation_type::BitAnd => opcode::BitAnd,
                    binary_operation_type::BitOr => opcode::BitOr,

                }
            }

            NodeKind::unary_operation(operation) => {
                match *operation {
                    unary_operation_type::Neg => opcode::Neg,
                    unary_operation_type::Not => opcode::Not,
                }
            }

            NodeKind::sign_extend => {
                opcode::SignExtend
            }

            NodeKind::zero_extend => {
                opcode::ZeroExtend
            }

            NodeKind::comparator(cmp) => {
                opcode::Compare(*cmp)
            }

            NodeKind::memory_load => {
                opcode::Load
            }
            NodeKind::memory_store => {
                opcode::Store
            }

            NodeKind::load_spill(slot) => {
                opcode::LoadSpill(*slot)
            }

            NodeKind::store_spill(slot) => {
                opcode::StoreSpill(*slot)
            }

            NodeKind::Jump => {
                opcode::Jump
            }

            NodeKind::Branch => {
                opcode::Branch
            }

            NodeKind::Call {
                arity
            } => {
                opcode::Call {
                    arity: *arity
                }
            }
        }
    };

    function.graph.node_mut(node_ref).kind = NodeKind::opcode(opcode);
}






