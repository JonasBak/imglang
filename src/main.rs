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
use parser::*;
use types::*;
use vm::*;

fn main() {
    let source = "
        fun fib(a) {
            if (a <= 2) {
                return 1;
            }
            return fib(a-1) + fib(a-2);
        }
        print fib(30);
        // var n = 1;
        // while (n < 15) {
        //     print fib(n);
        //     n = n + 1;
        // }
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
