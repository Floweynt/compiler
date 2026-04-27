use crate::ir::{FunctionHandle, Module};

mod lower_hir_lir;

pub use lower_hir_lir::*;

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

pub mod dominator;
pub mod ssa;
