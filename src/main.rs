mod debugger;
mod interpreter;
mod lexer;
mod parser;

use debugger::*;
use interpreter::*;
use lexer::*;
use parser::*;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut env = Scope::new();
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
        println!("RESULT: {:?}", ast.eval(&mut env));
    } else {
        println!("REPL");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines().peekable();
        while {
            print!("> ");
            io::stdout().flush().unwrap();
            lines.peek().is_some()
        } {
            let source = lines.next().unwrap().unwrap();
            let tokens = match parse_string(&source) {
                Ok(t) => t,
                Err(error) => {
                    print_lexer_err(&source, error);
                    continue;
                }
            };
            println!("TOKENS: {:?}", tokens);
            let ast = match parse_program(tokens) {
                Ok(ast) => ast,
                Err(error) => {
                    print_parser_err(&source, error);
                    continue;
                }
            };
            println!("AST: {:?}", ast);
            println!("RESULT: {:?}", ast.eval(&mut env));
        }
    }
}
