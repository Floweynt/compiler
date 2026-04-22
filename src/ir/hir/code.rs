use std::collections::HashMap;

use malachite::Integer;
use slotmap::{SlotMap, new_key_type};

use crate::{ir::types::ValueType, util::NamedContainer};

new_key_type! { pub struct Label; }
new_key_type! { pub struct LvtRef; }

pub enum BlockTerminator {
    Unreachable,
    Jmp(Label),
    Branch { if_true: Label, if_false: Label },
    Ret,
}

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

pub struct HirInstruction {
    opc: HirOpc,
    ty: ValueType,
}

pub struct BasicBlock {
    name: String,
    body: Vec<HirInstruction>,
    terminator: BlockTerminator,
}

pub struct HirLocal {
    name: String,
    ty: ValueType,
}

pub struct HirFunctionBody {
    lvt: NamedContainer<LvtRef, HirLocal>,
    bb: NamedContainer<Label, BasicBlock>,
}

impl HirFunctionBody {
    pub fn new() -> Self {
        Self {
            lvt: NamedContainer::new(),
            bb: NamedContainer::new(),
        }
    }
}
