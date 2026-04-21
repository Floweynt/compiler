use crate::ir::{
    sym::{Symbol, SymbolData}, types::ValueType,
};

pub enum FunctionBody {
    Hir(),
    Lir(),
    None,
}

pub enum CallingConvention {
    C,
}

pub struct FunctionParameter {
    name: String,
    ty: ValueType,
}

pub struct Function {
    sym: SymbolData,
    cconv: CallingConvention,
    args: Box<[FunctionParameter]>,
    return_ty: ValueType,
    body: FunctionBody,
}

impl Symbol for Function {
    fn get_symbol(&self) -> &SymbolData {
        &self.sym
    }
}
