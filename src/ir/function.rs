use crate::ir::{SymbolHandle, types::ValueType};

pub enum FunctionBody {
    Hir(),
    Lir(),
}

pub enum CallingConvention {
    C,
}

pub struct FunctionParameter {
    name: String,
    ty: ValueType,
}

pub struct Function {
    pub(super) sym: SymbolHandle,
    pub(super) cconv: CallingConvention,
    pub(super) args: Box<[FunctionParameter]>,
    pub(super) return_ty: ValueType,
    pub(super) body: FunctionBody,
}
