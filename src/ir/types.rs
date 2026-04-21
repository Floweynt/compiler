use std::num::NonZeroU32;

use slotmap::new_key_type;

new_key_type! { pub struct StructHandle; }

pub struct StructData {
    name: String,
    entries: Vec<(String, ValueType)>,
}

pub struct FunctionType {
    args: Box<[ValueType]>,
    return_ty: ValueType,
}

pub enum ValueType {
    Int { width: NonZeroU32 },
    F32,
    F64,
    Struct(StructHandle),
    Ptr,
}

pub enum AbstractMachineType {
}
