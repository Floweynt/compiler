use crate::ir::{
    FunctionHandle, Module, function::FunctionBody, hir::code::HirFunctionBody, lir::code::LirGraph,
};

mod dce;
mod dominator;
mod lower_hir_lir;
mod sched;

pub use lower_hir_lir::*;
pub use sched::*;

pub(super) fn hir_body(module: &mut Module, func: FunctionHandle) -> &mut HirFunctionBody {
    let func = module.get_function_mut(func).unwrap();

    match &mut func.body {
        FunctionBody::Hir(body) => body,
        FunctionBody::Lir(_) => panic!("can't perform pass on LIR Function"),
    }
}

pub(super) fn lir_body(module: &mut Module, func: FunctionHandle) -> &mut LirGraph {
    let func = module.get_function_mut(func).unwrap();

    match &mut func.body {
        FunctionBody::Hir(_) => panic!("can't perform pass on HIR Function"),
        FunctionBody::Lir(body) => body,
    }
}

pub trait FunctionPass {
    fn apply_function(module: &mut Module, func: FunctionHandle);
}

pub trait Pass {
    fn apply(module: &mut Module);
}

impl<T: FunctionPass> Pass for T {
    fn apply(module: &mut Module) {
        for func in module.function_defs.keys().collect::<Vec<_>>() {
            T::apply_function(module, func);
        }
    }
}
