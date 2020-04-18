#![allow(dead_code)]
#![allow(unused_imports)]

mod chunk;
mod compiler;
mod debugger;
mod interpreter;
mod lexer;
mod parser;
mod vm;

use chunk::*;
use compiler::*;
use debugger::*;
use interpreter::*;
use lexer::*;
use parser::*;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use vm::*;

fn _interpreter() {
    let args: Vec<String> = env::args().collect();
    let scope = Environment::new();
    if args.len() > 2 {
        println!("usage imglang [script]");
    } else if args.len() == 2 {
        let source = fs::read_to_string(&args[1]).unwrap();
        let tokens = match parse_string(&source) {
            Ok(t) => t,
            Err(error) => {
                print_lexer_err(&source, error);
                return;
            }
        };
        println!("TOKENS: {:?}", tokens);
        let ast = match parse_program(tokens) {
            Ok(ast) => ast,
            Err(error) => {
                print_parser_err(&source, error);
                return;
            }
        };
        println!("AST: {:?}", ast);
        println!("RESULT: {:?}", ast.eval(&scope));
    } else {
        println!("REPL");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines().peekable();
        'l: while {
            print!("> ");
            io::stdout().flush().unwrap();
            lines.peek().is_some()
        } {
            let mut source = lines.next().unwrap().unwrap();
            let ast = loop {
                let tokens = match parse_string(&source) {
                    Ok(t) => t,
                    Err(error) => {
                        print_lexer_err(&source, error);
                        continue 'l;
                    }
                };
                match parse_program(tokens) {
                    Ok(ast) => break ast,
                    Err(ParserError::UnexpectedToken(Token {
                        t: TokenType::Eof, ..
                    })) => {
                        print!("... ");
                        io::stdout().flush().unwrap();
                        source += "\n";
                        source += &lines.next().unwrap().unwrap();
                        continue;
                    }
                    Err(error) => {
                        print_parser_err(&source, error);
                        continue 'l;
                    }
                };
            };
            println!("AST: {:?}", ast);
            println!("RESULT: {:?}", ast.eval(&scope));
        }
    }
}

fn main() {
    let mut chunk = Chunk::new();
    let tokens = parse_string(&"1+2+3+4+5;".to_string()).unwrap();
    let ast = parse_program(tokens).unwrap();
    ast.codegen(&mut chunk);
    disassemble_chunk(&chunk);
    run_vm(chunk);
}
