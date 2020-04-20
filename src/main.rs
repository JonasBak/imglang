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
    let mut chunk = Chunk::new();
    let mut lexer =
        Lexer::new(&"print (4-1<=2+1)==(8/2<1*6);1+1;print 10 > 10;".to_string()).unwrap();
    let mut ast = parse(&mut lexer);
    ast.annotate_type().unwrap();
    ast.codegen(&mut chunk);
    disassemble_chunk(&chunk);
    run_vm(chunk);
}
