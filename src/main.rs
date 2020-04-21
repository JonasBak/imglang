mod chunk;
mod compiler;
mod debugger;
mod lexer;
mod object;
mod parser;
mod types;
mod vm;

use chunk::*;
use compiler::*;
use debugger::*;
use lexer::*;
use object::*;
use parser::*;
use types::*;
use vm::*;

fn main() {
    let source = "
        fun hehe(c) 1 + c
        var a = fun(c) c + 9 + 8 + 3;
        var b = 1;
        fun lol() hehe(1)
        print lol();
        fun hehe() 1
        b = a(10) / 2;
        print b;
        a(5);
        a(5);
        a(5);
        a(5);
        a(5);
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
    let chunks = Compiler::compile(&ast);
    disassemble_chunk(&chunks);
    let mut vm = VM::new(chunks);
    vm.run();
}
