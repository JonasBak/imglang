mod chunk;
mod compiler;
mod debugger;
mod externals;
mod heap;
mod lexer;
mod parser;
mod types;
mod utils;
mod vm;

use chunk::*;
use compiler::*;
use debugger::*;
use externals::*;
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

    let mut externals = Externals::new();
    externals.add_function(
        "testExternal".to_string(),
        ExternalFunction {
            args_t: vec![],
            ret_t: AstType::Float,
            dispatch: |args: Vec<ExternalArg>| -> ExternalArg {
                println!("called external function");
                return ExternalArg::Float(123.345);
            },
        },
    );

    #[cfg(feature = "debug_build")]
    eprintln!("{:?}", ast);

    if let Err(error) = TypeChecker::annotate_types(&mut ast, Some(&externals)) {
        print_type_error(&source, error);
        return;
    }
    let chunks = Compiler::compile(&ast, Some(&externals));

    #[cfg(feature = "debug_build")]
    eprintln!("{:?}", ast);

    #[cfg(feature = "debug_build")]
    disassemble_chunk(&chunks);

    let mut vm = VM::new(chunks, Some(&externals));
    vm.run(&mut stdout());
}
