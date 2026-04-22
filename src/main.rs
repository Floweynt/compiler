#![feature(new_range_api)]

use std::fs::read;

use crate::ir::hir::parser::Parser;

mod ir;
mod node;
mod util;

fn main() {
    println!("hi");
    let mut parser = Parser::new().unwrap();
    let str = read("test/hir/test.hir").unwrap();
    let _ = parser.parse(str::from_utf8(&str).unwrap());
}
