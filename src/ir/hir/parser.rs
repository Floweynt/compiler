use tree_sitter::{LanguageError, Tree};
use ts_utils::NodeChildCursorExt;

use crate::ir::Module;

pub struct Parser {
    parser: tree_sitter::Parser,
}

pub struct Diagnostic {}

pub enum ParseError {
    Internal,
    DiagnosticsParse(Vec<Diagnostic>),
    DiagnosticsSem(Vec<Diagnostic>),
}

impl Parser {
    pub fn new() -> Result<Parser, LanguageError> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_hir::LANGUAGE.into())?;
        parser.set_logger(Some(Box::new(|_, _| {})));
        Ok(Parser { parser })
    }

    fn gather_parse_errors(tree: &Tree) -> Result<(), ParseError> {
        let mut cursor = tree.walk();

        let mut reached_root = false;

        while !reached_root {
            let node = cursor.node();

            if node.is_error() || node.is_missing() {
                todo!()
            }

            if cursor.goto_first_child() {
                continue;
            }

            if cursor.goto_next_sibling() {
                continue;
            }

            loop {
                if !cursor.goto_parent() {
                    reached_root = true;
                    break;
                }

                if cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn parse(&mut self, src: &str) -> Result<Module, ParseError> {
        let tree = match self.parser.parse(src, None) {
            Some(x) => x,
            None => return Err(ParseError::Internal),
        };

        Self::gather_parse_errors(&tree)?;

        // build module
        for top_level in tree.root_node().children_cursor() {
            
        
        }

        todo!()
    }
}

/*let mut parser = ::new();

parser
    .set_language(&LANGUAGE.into())
    .expect("Error loading Nodes parser");

parser.set_logger(Some(Box::new(|log_type, str| {

})));

parser.parse(src, None);*/
