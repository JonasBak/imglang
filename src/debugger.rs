#![allow(dead_code)]

use super::*;
use std::fmt;

fn print_errors(source: &String, errors: Vec<(usize, String)>) {
    let lines_map = source.chars().fold(vec![0], |mut acc, c| {
        acc.push(
            acc.last().unwrap()
                + match c {
                    '\n' => 1,
                    _ => 0,
                },
        );
        acc
    });
    let lines: Vec<&str> = source.lines().collect();
    let errors: Vec<(String, usize)> = errors
        .into_iter()
        .map(|err| {
            let line = lines_map[err.0];
            (err.1, line)
        })
        .collect();
    eprintln!("{} errors!\n", errors.len());
    for (msg, line) in errors.iter() {
        eprintln!(
            "...\n{: >3} | {}\n...",
            line + 1,
            lines[(*line).min(lines.len() - 1)]
        );
        eprintln!("> {}", msg);
    }
}

pub fn print_lexer_err(source: &String, error: LexerError) {
    let error = match error {
        LexerError::Parse(i) => (i, "could not parse character".to_string()),
        LexerError::Unescaped(i) => (i, "unescaped string".to_string()),
    };
    print_errors(source, vec![error]);
}

fn flatmap_parser_error(error: ParserError, list: &mut Vec<(usize, String)>) {
    match error {
        ParserError::Unexpected(token, msg) => list.push((
            token.start,
            format!("on token {:?}: {}", token.t, msg.to_string()),
        )),
        ParserError::BlockErrors(errors) => {
            for error in errors.into_iter() {
                flatmap_parser_error(error, list);
            }
        }
    }
}

pub fn print_parser_error(source: &String, error: ParserError) {
    let mut errors = vec![];
    flatmap_parser_error(error, &mut errors);
    print_errors(source, errors);
}

fn flatmap_type_error(error: TypeError, list: &mut Vec<(usize, String)>) {
    match error {
        TypeError::BlockErrors(errors) => {
            for error in errors.into_iter() {
                flatmap_type_error(error, list);
            }
        }
        TypeError::Error(msg, pos) => list.push((pos, msg)),
    }
}

pub fn print_type_error(source: &String, error: TypeError) {
    let mut errors = vec![];
    flatmap_type_error(error, &mut errors);
    print_errors(source, errors);
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn disassemble_chunk(chunks: &Vec<Chunk>) {
    for (i, chunk) in chunks.iter().enumerate() {
        eprintln!("{:*^64}", format!("BYTECODE CHUNK {}", i));
        let mut ip = 0;
        while ip < chunk.len_code() {
            eprintln!("{:0>6}\t{:?}", ip, chunk.get_op(ip));
            ip += 1;
        }
    }
    eprintln!("{:*^64}", "");
}
