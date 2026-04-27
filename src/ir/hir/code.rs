use malachite::Integer;
use slotmap::new_key_type;

use crate::{ir::types::ValueType, util::NamedContainer};

new_key_type! { pub struct Label; }
new_key_type! { pub struct LvtRef; }

#[derive(Debug)]
pub enum BlockTerminator {
    Unreachable,
    Jmp(Label),
    Branch { if_true: Label, if_false: Label },
    Ret,
}

impl BlockTerminator {
    pub fn labels(&self) -> impl Iterator<Item = Label> {
        gen {
            match self {
                BlockTerminator::Jmp(label) => yield *label,
                BlockTerminator::Branch { if_true, if_false } => {
                    yield *if_true;
                    yield *if_false;
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
pub enum HirOpc {
    LdcI(Integer),

    Load(LvtRef),
    Store(LvtRef),

    Add,
    Sub,
    Mul,
    Div,
    Mod,

    BitAnd,
    BitOr,
    BitNot,

    SExt,
    ZExt,

    CmpEq,
    CmpNe,
    CmpGt,
    CmpLt,
    CmpGe,
    CmpLe,

    Invoke { arity: usize },
}

#[derive(Debug)]
pub struct HirInstruction {
    opc: HirOpc,
    ty: ValueType,
}

impl HirInstruction {
    pub fn new(opc: HirOpc, ty: ValueType) -> Self {
        Self { opc, ty }
    }
}

#[derive(Debug)]
pub struct BasicBlock {
    name: String,
    body: Vec<HirInstruction>,
    terminator: BlockTerminator,
}

impl BasicBlock {
    pub fn add_insn(&mut self, code: HirInstruction) {
        self.body.push(code);
    }

    pub fn set_terminator(&mut self, terminator: BlockTerminator) {
        self.terminator = terminator;
    }

    pub fn successors(&self) -> impl Iterator<Item = Label> {
        self.terminator.labels()
    }
}

#[derive(Debug)]
pub struct HirLocal {
    name: String,
    ty: ValueType,
}

#[derive(Debug)]
pub struct HirFunctionBody {
    lvt: NamedContainer<LvtRef, HirLocal>,
    bb: NamedContainer<Label, BasicBlock>,
    entry: Option<Label>,
}

impl HirFunctionBody {
    pub fn new() -> Self {
        Self {
            lvt: NamedContainer::new(),
            bb: NamedContainer::new(),
            entry: None,
        }
    }

    pub fn blocks(&self) -> impl Iterator<Item = (Label, &BasicBlock)> {
        self.bb.iter()
    }

    pub fn entry(&self) -> Label {
        self.entry.unwrap()
    }

    pub fn define_local(&mut self, name: String, ty: ValueType) -> Result<LvtRef, LvtRef> {
        self.lvt.define(name.clone(), || HirLocal { name, ty })
    }

    pub fn local_by_name(&self, name: &str) -> Option<LvtRef> {
        self.lvt.by_name(name)
    }

    pub fn define_bb(&mut self, name: String) -> Result<Label, Label> {
        self.bb
            .define(name.clone(), || BasicBlock {
                name,
                body: Vec::new(),
                terminator: BlockTerminator::Unreachable,
            })
            .inspect(|&label| {
                if self.entry.is_none() {
                    self.entry = Some(label);
                }
            })
    }

    pub fn define_bb_unnamed(&mut self) -> Label {
        self.bb.define_unnamed(BasicBlock {
            name: "".to_owned(),
            body: Vec::new(),
            terminator: BlockTerminator::Unreachable,
        })
    }

    pub fn bb(&self, label: Label) -> Option<&BasicBlock> {
        self.bb.by_key(label)
    }

    pub fn bb_unchecked(&self, label: Label) -> &BasicBlock {
        self.bb(label).unwrap()
    }

    pub fn bb_mut(&mut self, label: Label) -> Option<&mut BasicBlock> {
        self.bb.by_key_mut(label)
    }

    pub fn bb_mut_unchecked(&mut self, label: Label) -> &mut BasicBlock {
        self.bb_mut(label).unwrap()
    }
}
