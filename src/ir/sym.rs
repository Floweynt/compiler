use std::mem::swap;

use crate::ir::{FunctionHandle, VariableHandle};

#[derive(Debug, Clone, Copy)]
pub enum LinkageType {
    Private,
    Internal,
    External,
}

#[derive(Debug)]
pub enum SymbolContents {
    Function(FunctionHandle),
    Variable(VariableHandle),
    Undefined,
}

#[derive(Debug)]
pub struct Symbol {
    linkage: LinkageType,
    name: String,
    contents: SymbolContents,
}

impl Symbol {
    pub(super) fn new(name: String, linkage: LinkageType) -> Symbol {
        Self {
            linkage,
            name,
            contents: SymbolContents::Undefined,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn contents(&self) -> &SymbolContents {
        &self.contents
    }

    pub fn replace_contents(&mut self, mut contents: SymbolContents) -> SymbolContents {
        swap(&mut contents, &mut self.contents);
        contents
    }
}
