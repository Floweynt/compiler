use std::{iter, num::NonZero, str::FromStr};

use malachite::Integer;
use tree_sitter::{Language, LanguageError, Node, Tree, TreeCursor};

use crate::ir::{
    Module,
    function::{CallingConvention, FunctionBody, FunctionParameter},
    hir::code::{BlockTerminator, HirFunctionBody, HirInstruction, HirOpc},
    sym::LinkageType,
    types::ValueType,
};

pub struct Parser {
    parser: tree_sitter::Parser,

    declare_node_id: u16,
    define_var_node_id: u16,
    define_fun_node_id: u16,
    instruction_node_id: u16,
    label_node_id: u16,
    identifier_node_id: u16,
    number_node_id: u16,
    float_node_id: u16,

    visibility_field_id: u16,
    name_field_id: u16,
    return_type_field_id: u16,
    type_field_id: u16,
    param_type_field_id: u16,
    param_name_field_id: u16,
    vars_field_id: u16,
    body_field_id: u16,
    opcode_field_name: u16,
    operands_field_name: u16,
}

#[derive(Debug)]
pub enum DiagnosticKind {
    Redeclare,
    Redefine,
    Undeclared,
    UndeclaredVariable,
    BadTypeUnknown,
    BadTypeIntParseWidth,
    BadTypeIntZeroWidth,
    RedeclareVariable,
    RedeclareLabel,
    BadOperandIntParse,
    BadOperands,
    UnknownOpc,
}

#[derive(Debug)]
pub struct SourceLocation {
    pub byte: usize,
    pub row: usize,
    pub col: usize,
}

#[derive(Debug)]
pub struct Diagnostic {
    kind: DiagnosticKind,
    range: (SourceLocation, SourceLocation),
}

impl Diagnostic {
    pub fn make(node: Node<'_>, kind: DiagnosticKind) -> Self {
        Self {
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
        }
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

    pub fn to_res(self, module: Module) -> Result<Module, ParseError> {
        if self.0.is_empty() {
            Ok(module)
        } else {
            Err(ParseError::DiagnosticsSem(self.0))
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    Internal,
    DiagnosticsParse(Vec<Diagnostic>),
    DiagnosticsSem(Vec<Diagnostic>),
}

enum InsnParseRes {
    Insn(HirInstruction),
    Term(BlockTerminator),
}

enum Operand<'a> {
    Name(&'a str),
    Number(Integer),
    Float(f64), // TODO
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
            instruction_node_id: lang.id_for_node_kind("instruction", true),
            label_node_id: lang.id_for_node_kind("label", true),
            identifier_node_id: lang.id_for_node_kind("identifier", true),
            number_node_id: lang.id_for_node_kind("number", true),
            float_node_id: lang.id_for_node_kind("float", true),
            visibility_field_id: lang.field_id_for_name("visibility").unwrap().into(),
            name_field_id: lang.field_id_for_name("name").unwrap().into(),
            return_type_field_id: lang.field_id_for_name("return_type").unwrap().into(),
            type_field_id: lang.field_id_for_name("type").unwrap().into(),
            param_type_field_id: lang.field_id_for_name("param_type").unwrap().into(),
            param_name_field_id: lang.field_id_for_name("param_name").unwrap().into(),
            vars_field_id: lang.field_id_for_name("vars").unwrap().into(),
            body_field_id: lang.field_id_for_name("body").unwrap().into(),
            opcode_field_name: lang.field_id_for_name("opcode").unwrap().into(),
            operands_field_name: lang.field_id_for_name("operands").unwrap().into(),
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
            b"f32" => Ok(ValueType::F32),
            b"f64" => Ok(ValueType::F64),
            b"bot" => Ok(ValueType::Bot),
            x if x.starts_with(b"i") => {
                let width = str::from_utf8(&x[1..])
                    .unwrap()
                    .parse::<u32>()
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

    fn make_insn(
        node: Node<'_>,
        opc: HirOpc,
        ty: ValueType,
        err: bool,
    ) -> Result<HirInstruction, Diagnostic> {
        if err {
            Err(Diagnostic::make(node, DiagnosticKind::BadOperands))
        } else {
            Ok(HirInstruction::new(opc, ty))
        }
    }

    fn parse_insn<'a>(
        &mut self,
        body: Node<'a>,
        src: &str,
        cursor: &mut TreeCursor<'a>,
        func: &HirFunctionBody,
    ) -> Result<InsnParseRes, Diagnostic> {
        let opc = body.child_by_field_id(self.opcode_field_name).unwrap();

        let ty = Self::parse_type(body.child_by_field_id(self.type_field_id).unwrap(), src)?;

        let mut operands = vec![];

        for node in
            body.children_by_field_id(NonZero::new(self.operands_field_name).unwrap(), cursor)
        {
            let id = node.kind_id();
            let str = str::from_utf8(Self::utf8_text(node, src)).unwrap();

            if id == self.identifier_node_id {
                operands.push(Operand::Name(str));
            } else if id == self.number_node_id {
                operands.push(Operand::Number(Integer::from_str(str).map_err(|_| {
                    Diagnostic::make(node, DiagnosticKind::BadOperandIntParse)
                })?));
            } else {
                todo!()
            }
        }

        Ok(InsnParseRes::Insn(match Self::utf8_text(opc, src) {
            b"add" => Self::make_insn(body, HirOpc::Add, ty, !operands.is_empty())?,
            b"sub" => Self::make_insn(body, HirOpc::Sub, ty, !operands.is_empty())?,
            b"mul" => Self::make_insn(body, HirOpc::Mul, ty, !operands.is_empty())?,
            b"div" => Self::make_insn(body, HirOpc::Div, ty, !operands.is_empty())?,
            b"mod" => Self::make_insn(body, HirOpc::Mod, ty, !operands.is_empty())?,
            b"ldc" => {
                let Ok([val]) = TryInto::<[Operand; 1]>::try_into(operands) else {
                    return Err(Diagnostic::make(opc, DiagnosticKind::BadOperands));
                };

                match val {
                    Operand::Name(_) => todo!(),
                    Operand::Number(integer) => HirInstruction::new(HirOpc::LdcI(integer), ty),
                    Operand::Float(_) => todo!(),
                }
            }
            b"ld" => {
                let Ok([Operand::Name(val)]) = TryInto::<[Operand; 1]>::try_into(operands) else {
                    return Err(Diagnostic::make(opc, DiagnosticKind::BadOperands));
                };

                let Some(x) = func.local_by_name(val) else {
                    return Err(Diagnostic::make(opc, DiagnosticKind::UndeclaredVariable));
                };

                HirInstruction::new(HirOpc::Load(x), ty)
            }
            b"st" => {
                let Ok([Operand::Name(val)]) = TryInto::<[Operand; 1]>::try_into(operands) else {
                    return Err(Diagnostic::make(opc, DiagnosticKind::BadOperands));
                };

                let Some(x) = func.local_by_name(val) else {
                    return Err(Diagnostic::make(opc, DiagnosticKind::UndeclaredVariable));
                };

                HirInstruction::new(HirOpc::Store(x), ty)
            }
            b"ret" => {
                return Ok(InsnParseRes::Term(BlockTerminator::Ret));
            }
            _ => {
                return Err(Diagnostic::make(opc, DiagnosticKind::UnknownOpc));
            }
        }))
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
        let mut inner_cur = tree.walk();
        let mut inner_cur_alt = tree.walk();

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
                let name_node = top_level.child_by_field_id(self.name_field_id).unwrap();
                let name = String::from_utf8(Self::utf8_text(name_node, src).to_vec()).unwrap();
                let mut body = HirFunctionBody::new();

                let return_type = match Self::parse_type(
                    top_level
                        .child_by_field_id(self.return_type_field_id)
                        .unwrap(),
                    src,
                ) {
                    Ok(x) => x,
                    Err(err) => {
                        diag.add(err);
                        continue;
                    }
                };

                let mut params = Vec::new();

                for (ty, param_name_node) in iter::zip(
                    top_level.children_by_field_id(
                        NonZero::new(self.param_type_field_id).unwrap(),
                        &mut inner_cur,
                    ),
                    top_level.children_by_field_id(
                        NonZero::new(self.param_name_field_id).unwrap(),
                        &mut inner_cur_alt,
                    ),
                ) {
                    let ty = match Self::parse_type(ty, src) {
                        Ok(x) => x,
                        Err(err) => {
                            diag.add(err);
                            continue;
                        }
                    };

                    let param_name =
                        String::from_utf8(Self::utf8_text(param_name_node, src).to_vec()).unwrap();

                    params.push(FunctionParameter::new(param_name.clone(), ty));

                    if body.define_local(param_name, ty).is_err() {
                        diag.report(param_name_node, DiagnosticKind::RedeclareVariable);
                    }
                }

                let Some(sym) = module.get_symbol(&name) else {
                    diag.report(name_node, DiagnosticKind::Undeclared);
                    continue;
                };

                for var_decl in top_level
                    .children_by_field_id(NonZero::new(self.vars_field_id).unwrap(), &mut inner_cur)
                {
                    let ty = var_decl.child_by_field_id(self.type_field_id).unwrap();
                    let name_node = var_decl.child_by_field_id(self.name_field_id).unwrap();

                    let ty = match Self::parse_type(ty, src) {
                        Ok(x) => x,
                        Err(err) => {
                            diag.add(err);
                            continue;
                        }
                    };

                    let name = String::from_utf8(Self::utf8_text(name_node, src).to_vec()).unwrap();

                    if body.define_local(name, ty).is_err() {
                        diag.report(name_node, DiagnosticKind::RedeclareVariable);
                    }
                }

                let mut current_bb = body.define_bb_unnamed();

                for body_node in top_level
                    .children_by_field_id(NonZero::new(self.body_field_id).unwrap(), &mut inner_cur)
                {
                    if body_node.kind_id() == self.instruction_node_id {
                        match self.parse_insn(body_node, src, &mut inner_cur_alt, &body) {
                            Ok(InsnParseRes::Insn(insn)) => {
                                body.bb_mut_unchecked(current_bb).add_insn(insn);
                            }
                            Ok(InsnParseRes::Term(insn)) => {
                                body.bb_mut_unchecked(current_bb).set_terminator(insn);
                                current_bb = body.define_bb_unnamed();
                            }
                            Err(err) => {
                                diag.add(err);
                                continue;
                            }
                        };
                    } else {
                        let node_name = body_node.child_by_field_id(self.name_field_id).unwrap();

                        let new_bb = match body.define_bb(
                            str::from_utf8(Self::utf8_text(node_name, src))
                                .unwrap()
                                .to_owned(),
                        ) {
                            Ok(x) => x,
                            Err(_) => {
                                diag.report(node_name, DiagnosticKind::RedeclareLabel);
                                continue;
                            }
                        };

                        body.bb_mut_unchecked(current_bb)
                            .set_terminator(BlockTerminator::Jmp(new_bb));

                        current_bb = new_bb;
                    }
                }

                if module
                    .define_function(
                        sym,
                        CallingConvention::C,
                        params.into_boxed_slice(),
                        return_type,
                        FunctionBody::Hir(body),
                    )
                    .is_err()
                {
                    diag.report(name_node, DiagnosticKind::Redefine);
                }
            } else {
                unreachable!("illegal node type");
            }
        }

        diag.to_res(module)
    }
}
