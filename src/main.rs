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
        var a = 123;
        if (a > 100) {
            a = a + 1;
        } else {
            a = 1;
        }
        print a;
        true and false and true;
        false or false or true or false;
        var b = 0;
        while (b < 10) {
            print b;
            b = b + 1;
        }
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
    let mut vm = VM::new();
    vm.run(chunk);
}
