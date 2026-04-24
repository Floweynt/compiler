use crate::ir::{SymbolHandle, hir::code::HirFunctionBody, types::ValueType};

#[derive(Debug)]
pub enum FunctionBody {
    Hir(HirFunctionBody),
    Lir(),
}

#[derive(Debug)]
pub enum CallingConvention {
    C,
}

#[derive(Debug)]
pub struct FunctionParameter {
    name: String,
    ty: ValueType,
}

impl FunctionParameter {
    pub fn new(name: String, ty: ValueType) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug)]
pub struct Function {
    pub(super) sym: SymbolHandle,
    pub(super) cconv: CallingConvention,
    pub(super) params: Box<[FunctionParameter]>,
    pub(super) return_ty: ValueType,
    pub(super) body: FunctionBody,
}
