mod chunk;
mod compiler;
mod debugger;
mod heap;
mod lexer;
mod parser;
mod types;
mod utils;
mod vm;

use chunk::*;
use compiler::*;
use debugger::*;
use heap::*;
use lexer::*;
use parser::*;
use std::env;
use std::fs;
use std::io::stdout;
use types::*;
use utils::*;
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

    #[cfg(feature = "debug_build")]
    eprintln!("{:?}", ast);

    if let Err(error) = TypeChecker::annotate_types(&mut ast) {
        print_type_error(&source, error);
        return;
    }
    let chunks = Compiler::compile(&ast);

    #[cfg(feature = "debug_build")]
    eprintln!("{:?}", ast);

    #[cfg(feature = "debug_build")]
    disassemble_chunk(&chunks);

    let mut vm = VM::new(chunks);
    vm.run(&mut stdout());
}
