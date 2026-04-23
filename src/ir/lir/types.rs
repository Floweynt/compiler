use malachite::Integer;

/// Represents a type for the optimizer.
///
/// These are abstract types, and aren't really correlated with machine types or anything a normal
/// programming would care about such as i32, f32, etc.
///
/// This forms a [lattice](https://en.wikipedia.org/wiki/Lattice_(order)).
enum OptType {
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

    // TODO: model struct types
    Bottom,
}

enum TypeClass {}

impl OptType {
    pub fn meet(&self, _rhs: &OptType) -> OptType {
        match self {
            OptType::Top => todo!(),
            OptType::TypeIntTop => todo!(),
            OptType::TypeInt { lo: _, hi: _ } => todo!(),
            OptType::TypeIntBot => todo!(),
            OptType::FloatTop => todo!(),
            OptType::F32Top => todo!(),
            OptType::Float(_) => todo!(),
            OptType::F32Bottom => todo!(),
            OptType::FloatBottom => todo!(),
            OptType::CtrlTop => todo!(),
            OptType::CtrlBottom => todo!(),
            OptType::Bottom => todo!(),
        }
    }
}
