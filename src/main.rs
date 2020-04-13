mod lexer;
mod parser;

use lexer::*;
use parser::*;
use std::env;
use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("usage imglang [script]");
    } else if args.len() == 2 {
        let lexer = Lexer::new_from_file(&args[1]).unwrap();
        println!("{:?}", lexer);
        let tokens = lexer.parse().unwrap();
        println!("{:?}", tokens);
        let parser = Parser::new(tokens);
        parser.parse().unwrap();
    } else {
        print!("REPL\n> ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let lexer = Lexer::new(line.unwrap());
            println!("{:?}", lexer);
            let tokens = lexer.parse().unwrap();
            println!("{:?}", tokens);
            let parser = Parser::new(tokens);
            parser.parse().unwrap();

            print!("> ");
            io::stdout().flush().unwrap();
        }
    }
}
