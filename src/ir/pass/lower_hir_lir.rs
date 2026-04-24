use slotmap::SecondaryMap;

use crate::ir::{
    FunctionHandle, Module, function::FunctionBody, hir::code::Label, pass::FunctionPass,
};

pub struct LowerHirLirPass;

impl FunctionPass for LowerHirLirPass {
    fn apply_function(module: &mut Module, func: FunctionHandle) {
        let func = module.get_function_mut(func).unwrap();

        let body = match &func.body {
            FunctionBody::Hir(body) => body,
            FunctionBody::Lir() => panic!("can't perform LowerHirLirPass on LIR Function"),
        };

        let dom_tree: SecondaryMap<Label, Label>; 
    }
}
