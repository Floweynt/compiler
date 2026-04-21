/// A generic opcode trait
/// Allows fetching various properties
pub trait Opcode {}

pub struct IROpcode {}

impl Opcode for IROpcode {}

macro_rules! def_opcode_class {
    ($($meow:tt)*) => {};
}

def_opcode_class! {
    const {
        trait_name = IROpcodeProps;
    }

    trait DataNode : Node[#data] {
        ins! {
            operand(0, #ctrl?, "ctrl");
            operand(1.., #data, "operands");
        }
    }

    trait BinOpNode : DataNode {
        ins! {
            limit(3);
            operand(1, data, "rhs");
            operand(2, data, "lhs");
        }

        trait commutes: bool;
    }

    impl AddNode : BinOpNode {
        constructor!(lhs, rhs) {
            operand(0, None);
            operand(1, lhs);
            operand(2, rhs);
        } 

        type_check!() {
            def("lhs") == $rhs;
            type_check(is_int($lhs));
        }

        outs! {
            out("carry", );
        }
    }
}

