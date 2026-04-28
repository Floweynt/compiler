use std::num::NonZeroU32;

use crate::ir::StructHandle;

#[derive(Debug)]
pub struct Struct {
    name: String,
    entries: Vec<(String, ValueType)>,
}

#[derive(Debug)]
pub struct FunctionType {
    args: Box<[ValueType]>,
    return_ty: ValueType,
}

#[derive(Clone, Copy, Debug)]
pub enum ValueType {
    Int { width: NonZeroU32 },
    F32,
    F64,
    Struct(StructHandle),
    Ptr,
    Bot,
}

pub enum AbstractMachineType {}
