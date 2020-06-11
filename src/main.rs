mod chunk;
mod compiler;
mod debugger;
#[macro_use]
mod externals;
mod heap;
mod lexer;
mod parser;
mod stack;
mod types;
mod vm;

use chunk::*;
use compiler::*;
use debugger::*;
use externals::*;
use heap::*;
use lexer::*;
use parser::*;
use stack::*;
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

    let mut externals = Externals::new();
    externals.add_function(
        "testExternal".to_string(),
        ExternalFunction {
            args_t: vec![AstType::Float, AstType::Float],
            ret_t: AstType::Float,
            dispatch: |stack: &mut Stack| {
                external_pop_args!(stack, (arg0, f64), (arg1, f64));
                println!("from external: {}", arg0 / arg1);
                stack.push(12.0);
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
