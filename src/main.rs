#![allow(dead_code)]
#![allow(unused_imports)]

mod chunk;
mod compiler;
mod debugger;
mod lexer;
mod parser;
mod vm;

use chunk::*;
use compiler::*;
use debugger::*;
use lexer::*;
use parser::*;
use vm::*;

fn main() {
    let mut chunk = Chunk::new();
    let mut lexer = Lexer::new(&"!false".to_string()).unwrap();
    let ast = parse(&mut lexer);
    ast.codegen(&mut chunk);
    disassemble_chunk(&chunk);
    run_vm(chunk);
}
