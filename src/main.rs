#![feature(new_range_api)]
#![feature(gen_blocks)]

use std::fs::read;

use crate::ir::{
    hir::parser::Parser,
    pass::{GlobalCodeMotion, LowerHirLirPass, Pass},
};

mod ir;
mod node;
mod util;

fn main() {
    println!("hi");
    let mut parser = Parser::new().unwrap();
    let str = read("test/hir/test.hir").unwrap();
    let mut x = parser.parse(str::from_utf8(&str).unwrap()).unwrap();

    LowerHirLirPass.apply(&mut x);
    // TODO: apply GVM
    GlobalCodeMotion.apply(&mut x);
}
