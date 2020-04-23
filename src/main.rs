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
use std::env;
use std::fs;
use std::io::stdout;
use types::*;
use vm::*;

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        eprintln!("usage \"imglang [script]\"");
        return;
    }
    args.next();
    let source = String::from_utf8(fs::read(args.next().unwrap()).unwrap()).unwrap();
    let mut lexer = match Lexer::new(&source) {
        Ok(tokens) => tokens,
        Err(error) => {
            print_lexer_err(&source, error);
            return;
        }
    };
    let mut ast = match parse(&mut lexer) {
        Ok(ast) => ast,
        Err(error) => {
            print_parser_error(&source, error);
            return;
        }
    };

    TypeChecker::annotate_types(&mut ast).unwrap();
    let chunks = Compiler::compile(&ast);

    #[cfg(feature = "debug_build")]
    eprintln!("{:?}", ast);

    #[cfg(feature = "debug_build")]
    disassemble_chunk(&chunks);

    let mut vm = VM::new(chunks);
    vm.run(&mut stdout());
}
