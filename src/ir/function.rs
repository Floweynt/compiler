use crate::ir::{SymbolHandle, hir::code::HirFunctionBody, types::ValueType};

pub enum FunctionBody {
    Hir(HirFunctionBody),
    Lir(),
}

pub enum CallingConvention {
    C,
}

pub struct FunctionParameter {
    name: String,
    ty: ValueType,
}

impl FunctionParameter {
    pub fn new(name: String, ty: ValueType) -> Self {
        return Self { name, ty };
    }
}

pub struct Function {
    pub(super) sym: SymbolHandle,
    pub(super) cconv: CallingConvention,
    pub(super) params: Box<[FunctionParameter]>,
    pub(super) return_ty: ValueType,
    pub(super) body: FunctionBody,
}
