mod chunk;
mod compiler;
mod debugger;
mod lexer;
mod parser;
mod types;
mod vm;

use chunk::*;
use compiler::*;
use debugger::*;
use lexer::*;
use parser::*;
use types::*;
use vm::*;

fn main() {
    let mut lexer = Lexer::new(&"var a = 1;{var a = 2; print a;}print a;".to_string()).unwrap();
    let mut ast = parse(&mut lexer);
    ast.annotate_type().unwrap();
    println!("{:?}", ast);
    let chunk = Compiler::compile(&ast);
    disassemble_chunk(&chunk);
    run_vm(chunk);
}
