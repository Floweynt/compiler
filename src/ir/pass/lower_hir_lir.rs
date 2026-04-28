use crate::ir::{
    FunctionHandle, Module,
    function::FunctionBody,
    pass::{FunctionPass, dominator::DominatorTree},
};

pub struct LowerHirLirPass;

struct AnalysisFrame {}

impl FunctionPass for LowerHirLirPass {
    fn apply_function(module: &mut Module, func: FunctionHandle) {
        let func = module.get_function_mut(func).unwrap();

        let body = match &func.body {
            FunctionBody::Hir(body) => body,
            FunctionBody::Lir(_) => panic!("can't perform LowerHirLirPass on LIR Function"),
        };

        let dom_tree = DominatorTree::make_dominator_tree(body);
        // let ssa = super::ssa::build_ssa(body);
    }
}
