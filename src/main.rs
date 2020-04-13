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
        let tokens = parse_file(&args[1]).unwrap();
        println!("{:?}", tokens);
        parse_tokens(tokens).unwrap();
    } else {
        print!("REPL\n> ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let tokens = parse_string(line.unwrap()).unwrap();
            println!("{:?}", tokens);
            parse_tokens(tokens).unwrap();
            print!("> ");
            io::stdout().flush().unwrap();
        }
    }
}
