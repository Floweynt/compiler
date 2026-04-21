use std::{num::NonZeroU32, sync::Arc};

use bitflags::bitflags;
use slotmap::{SlotMap, new_key_type};

mod function;
mod hir;
mod lir;
mod sym;
mod types;

new_key_type! { struct FunctionHandle; }

pub struct Module {
    struct_defs: SlotMap<StructHandle, StructData>,
    function_defs: SlotMap<FunctionHandle, Function>,
}
