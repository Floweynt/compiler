use std::collections::{HashMap, hash_map};

use slotmap::{SlotMap, new_key_type};

use crate::{
    ir::{
        function::{CallingConvention, Function, FunctionBody, FunctionParameter},
        sym::{LinkageType, Symbol, SymbolContents},
        types::{Struct, ValueType},
    },
    util::NamedContainer,
};

pub mod function;
pub mod hir;
pub mod lir;
pub mod sym;
pub mod types;

new_key_type! { pub struct FunctionHandle; }
new_key_type! { pub struct VariableHandle; }
new_key_type! { pub struct SymbolHandle; }
new_key_type! { pub struct StructHandle; }

pub struct Module {
    symbols: NamedContainer<SymbolHandle, Symbol>,
    struct_defs: SlotMap<StructHandle, Struct>,
    function_defs: SlotMap<FunctionHandle, Function>,
    // variable_defs: SlotMap<VariableHandle, Variable>,
}

pub enum DeclareError {
    AlreadyDeclared(SymbolHandle),
}

pub enum DefineError {
    BadHandle,
    AlreadyDefined,
}

impl Module {
    pub fn new() -> Module {
        Module {
            symbols: NamedContainer::new(),
            struct_defs: SlotMap::with_key(),
            function_defs: SlotMap::with_key(),
        }
    }

    pub fn declare(
        &mut self,
        name: String,
        linkage: LinkageType,
    ) -> Result<SymbolHandle, DeclareError> {
        self.symbols
            .define(name.clone(), || Symbol::new(name, linkage))
            .map_err(DeclareError::AlreadyDeclared)
    }

    pub fn get_symbol(&self, name: &str) -> Option<SymbolHandle> {
        self.symbols.by_name(name)
    }

    pub fn define_function(
        &mut self,
        sym: SymbolHandle,
        cconv: CallingConvention,
        params: Box<[FunctionParameter]>,
        return_ty: ValueType,
        body: FunctionBody,
    ) -> Result<(FunctionHandle, &mut Function), DefineError> {
        let x = self.symbols.by_key_mut(sym).ok_or(DefineError::BadHandle)?;

        if !matches!(x.contents(), SymbolContents::Undefined) {
            return Err(DefineError::AlreadyDefined);
        }

        let key = self.function_defs.insert(Function {
            sym,
            cconv,
            params,
            return_ty,
            body,
        });

        let res = self.function_defs.get_mut(key).unwrap();

        x.replace_contents(SymbolContents::Function(key));

        Ok((key, res))
    }
}
