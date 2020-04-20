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
    let mut lexer = Lexer::new(&"var a = 1;{var a = false; print a;}print a;".to_string()).unwrap();
    let mut ast = parse(&mut lexer);
    TypeChecker::annotate_types(&mut ast).unwrap();
    println!("{:?}", ast);
    let chunk = Compiler::compile(&ast);
    disassemble_chunk(&chunk);
    run_vm(chunk);
}
