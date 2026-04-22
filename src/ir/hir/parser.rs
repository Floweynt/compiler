use std::{iter, num::NonZero};

use tree_sitter::{Language, LanguageError, Node, Tree};

use crate::ir::{Module, sym::LinkageType, types::ValueType};

pub struct Parser {
    parser: tree_sitter::Parser,

    declare_node_id: u16,
    define_var_node_id: u16,
    define_fun_node_id: u16,

    visibility_field_id: u16,
    name_field_id: u16,
    return_type_field_id: u16,
    param_type_field_id: u16,
    param_name_field_id: u16,
    vars_field_id: u16,
    body_field_id: u16,
}

pub enum DiagnosticKind {
    Redeclare,
    Undeclared,
    BadTypeUnknown,
    BadTypeIntParseWidth,
    BadTypeIntZeroWidth,
}

pub struct SourceLocation {
    pub byte: usize,
    pub row: usize,
    pub col: usize,
}

pub struct Diagnostic {
    kind: DiagnosticKind,
    range: (SourceLocation, SourceLocation),
}

impl Diagnostic {
    pub fn make(node: Node<'_>, kind: DiagnosticKind) -> Self {
        return Self {
            kind,
            range: (
                SourceLocation {
                    byte: node.start_byte(),
                    row: node.start_position().row,
                    col: node.start_position().column,
                },
                SourceLocation {
                    byte: node.end_byte(),
                    row: node.end_position().row,
                    col: node.end_position().column,
                },
            ),
        };
    }
}

struct DiagnosticReporter(Vec<Diagnostic>);

impl DiagnosticReporter {
    pub fn add(&mut self, diag: Diagnostic) {
        self.0.push(diag);
    }

    pub fn report(&mut self, node: Node<'_>, kind: DiagnosticKind) {
        self.add(Diagnostic::make(node, kind));
    }
}

pub enum ParseError {
    Internal,
    DiagnosticsParse(Vec<Diagnostic>),
    DiagnosticsSem(Vec<Diagnostic>),
}

impl Parser {
    pub fn new() -> Result<Parser, LanguageError> {
        let lang: Language = tree_sitter_hir::LANGUAGE.into();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang)?;
        parser.set_logger(Some(Box::new(|_, _| {})));

        Ok(Parser {
            parser,
            declare_node_id: lang.id_for_node_kind("declare", true),
            define_var_node_id: lang.id_for_node_kind("define_var", true),
            define_fun_node_id: lang.id_for_node_kind("define_fun", true),
            visibility_field_id: lang.field_id_for_name("visibility").unwrap().into(),
            name_field_id: lang.field_id_for_name("name").unwrap().into(),
            return_type_field_id: lang.field_id_for_name("return_type").unwrap().into(),
            param_type_field_id: lang.field_id_for_name("param_type").unwrap().into(),
            param_name_field_id: lang.field_id_for_name("param_name").unwrap().into(),
            vars_field_id: lang.field_id_for_name("vars").unwrap().into(),
            body_field_id: lang.field_id_for_name("body").unwrap().into(),
        })
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

    fn utf8_text<'a>(node: Node, src: &'a str) -> &'a [u8] {
        &src.as_bytes()[node.start_byte()..node.end_byte()]
    }

    fn parse_visibility(node: Node, src: &str) -> LinkageType {
        match Self::utf8_text(node, src) {
            b"internal" => LinkageType::Internal,
            b"external" => LinkageType::External,
            b"private" => LinkageType::Private,
            _ => unreachable!("illegal linkage type"),
        }
    }

    fn parse_type(node: Node, src: &str) -> Result<ValueType, Diagnostic> {
        match Self::utf8_text(node, src) {
            x if x.starts_with(b"i") => {
                let width = u32::from_str_radix(str::from_utf8(&x[1..]).unwrap(), 10)
                    .map_err(|_| Diagnostic::make(node, DiagnosticKind::BadTypeIntParseWidth))?;

                Ok(ValueType::Int {
                    width: width
                        .try_into()
                        .map_err(|_| Diagnostic::make(node, DiagnosticKind::BadTypeIntZeroWidth))?,
                })
            }
            _ => Err(Diagnostic::make(node, DiagnosticKind::BadTypeUnknown)),
        }
    }

    pub fn parse(&mut self, src: &str) -> Result<Module, ParseError> {
        let tree = match self.parser.parse(src, None) {
            Some(x) => x,
            None => return Err(ParseError::Internal),
        };

        Self::gather_parse_errors(&tree)?;

        let mut module = Module::new();
        let mut diag = DiagnosticReporter(Vec::new());

        // just give me a few temp cursor
        let mut inner_cur_type = tree.walk();
        let mut inner_cur_name = tree.walk();

        // build module
        for top_level in tree.root_node().children(&mut tree.walk()) {
            let id = top_level.kind_id();

            if id == self.declare_node_id {
                // read properties
                let visibility = top_level
                    .child_by_field_id(self.visibility_field_id)
                    .unwrap();

                let node_name = top_level.child_by_field_id(self.name_field_id).unwrap();
                let name = String::from_utf8(Self::utf8_text(node_name, src).to_vec()).unwrap();

                let linkage = Self::parse_visibility(visibility, src);

                if module.declare(name, linkage).is_err() {
                    diag.report(node_name, DiagnosticKind::Redeclare);
                }
            } else if id == self.define_var_node_id {
                todo!()
            } else if id == self.define_fun_node_id {
                let node_name = top_level.child_by_field_id(self.name_field_id).unwrap();
                let name = String::from_utf8(Self::utf8_text(node_name, src).to_vec()).unwrap();

                let return_type = match Self::parse_type(
                    top_level
                        .child_by_field_id(self.return_type_field_id)
                        .unwrap(),
                    src,
                ) {
                    Ok(x) => x,
                    Err(e) => {
                        diag.add(e);
                        continue;
                    }
                };

                println!("{name}");

                println!("{:?}", return_type);

                for (ty, name) in iter::zip(
                    top_level.children_by_field_id(
                        NonZero::new(self.param_type_field_id).unwrap(),
                        &mut inner_cur_type,
                    ),
                    top_level.children_by_field_id(
                        NonZero::new(self.param_name_field_id).unwrap(),
                        &mut inner_cur_name,
                    ),
                ) {
                    
                }

                let Some(sym) = module.get_symbol(&name) else {
                    diag.report(node_name, DiagnosticKind::Undeclared);
                    continue;
                };

                // module.define_function(sym, CallingConvention::C, return_ty, body);

                todo!()
            } else {
                unreachable!("illegal node type");
            }
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
