use malachite::{Integer, base::num::basic::traits::One};

use crate::ir::types::ValueType;

/// Represents a type for the optimizer.
///
/// These are abstract types, and aren't really correlated with machine types or anything a normal
/// programming would care about such as i32, f32, etc.
///
/// This forms a [lattice](https://en.wikipedia.org/wiki/Lattice_(order)).
#[derive(Clone, Debug)]
pub enum OptType {
    Top,

    TypeIntTop,
    TypeInt { lo: Integer, hi: Integer },
    TypeIntBot,

    FloatTop,
    F32Top,
    Float(f64),
    F32Bottom,
    FloatBottom,

    CtrlTop,
    CtrlBottom,

    MemTop,
    MemBottom,

    // TODO: model struct types
    Bottom,
}

enum TypeClass {}

impl OptType {
    pub fn meet(&self, _rhs: &OptType) -> OptType {
        match self {
            OptType::Top => todo!(),
            OptType::TypeIntTop => todo!(),
            OptType::TypeInt { lo, hi } => todo!(),
            OptType::TypeIntBot => todo!(),
            OptType::FloatTop => todo!(),
            OptType::F32Top => todo!(),
            OptType::Float(_) => todo!(),
            OptType::F32Bottom => todo!(),
            OptType::FloatBottom => todo!(),
            OptType::CtrlTop => todo!(),
            OptType::CtrlBottom => todo!(),
            OptType::MemTop => todo!(),
            OptType::MemBottom => todo!(),
            OptType::Bottom => todo!(),
        }
    }

    pub fn from_vt_pessimistic(value: &ValueType) -> OptType {
        match value {
            ValueType::Int { width } => {
                let v = Integer::ONE << (width.get() - 1u32);
                OptType::TypeInt {
                    lo: -v.clone(),
                    hi: v - Integer::ONE,
                }
            }
            ValueType::F32 => todo!(),
            ValueType::F64 => todo!(),
            ValueType::Struct(struct_handle) => todo!(),
            ValueType::Ptr => todo!(),
            ValueType::Bot => todo!(),
        }
    }
}
