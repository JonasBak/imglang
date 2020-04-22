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
use std::io::stdout;
use types::*;
use vm::*;

fn main() {
    let source = "
        fun xor(a bool, b bool) bool {
           return (a or b) and (!a or !b);
        }
        print xor(false, true);
        fun test1() {
            print 123;
        }
        test1();
        fun test2(a float, b bool) float {
            if (b) {
                return a * 10;
            } else {
                return a - 2;
            }
        }
        print test2(10, true);
        print test2(10, false);
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

    #[cfg(feature = "debug_build")]
    eprintln!("{:?}", ast);

    TypeChecker::annotate_types(&mut ast).unwrap();
    let chunks = Compiler::compile(&ast);

    #[cfg(debug_build)]
    disassemble_chunk(&chunks);

    let mut vm = VM::new(chunks);
    vm.run(&mut stdout());
}
