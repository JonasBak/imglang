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
    let source = "
        var a = 1;
        {
            var a = a + a;
            var a = a + a;
            print a;
        }
        print a;
        "
    .to_string();
    let mut lexer = Lexer::new(&source).unwrap();
    let mut ast = match parse(&mut lexer) {
        Ok(ast) => ast,
        Err(error) => {
            print_parser_error(&source, error);
            return;
        }
    };
    println!("{:?}", ast);
    TypeChecker::annotate_types(&mut ast).unwrap();
    let chunk = Compiler::compile(&ast);
    disassemble_chunk(&chunk);
    run_vm(chunk);
}
