use crate::ir::{
    FunctionHandle, Module,
    function::FunctionBody,
    pass::{FunctionPass, lower_hir_son::lower_hir_to_son, ssa::build_ssa},
};

pub struct LowerHirLirPass;

struct AnalysisFrame {}

impl FunctionPass for LowerHirLirPass {
    fn apply_function(&mut self, module: &mut Module, func: FunctionHandle) {
        let func = module.get_function_mut(func).unwrap();

        let body = match &func.body {
            FunctionBody::Hir(body) => body,
            FunctionBody::Lir(_) => panic!("can't perform LowerHirLirPass on LIR Function"),
        };

        let ssa = build_ssa(body);
        let graph = lower_hir_to_son(func, body, &ssa);
    }
}
