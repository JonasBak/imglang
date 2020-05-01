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
            eprint!("{:0>6}\t", ip);
            ip += disassemble(chunk, ip);
        }
    }
    eprintln!("{:*^64}", "");
}

fn print_simple(op: OpCode) -> CodeAdr {
    eprintln!("{:>3} {:<15}", op as u8, op.to_string());
    1
}
fn print_unary_u8(op: OpCode, operand: u8) -> CodeAdr {
    eprintln!("{:>3} {:<15} {:>3}", op as u8, op.to_string(), operand);
    2
}
fn print_unary_u16(op: OpCode, operand: u16) -> CodeAdr {
    eprintln!("{:>3} {:<15} {:>3}", op as u8, op.to_string(), operand);
    3
}

pub fn disassemble(chunk: &Chunk, ip: CodeAdr) -> CodeAdr {
    match OpCode::from(chunk.get_op(ip)) {
        op @ OpCode::NegateF64
        | op @ OpCode::MultiplyF64
        | op @ OpCode::DivideF64
        | op @ OpCode::AddF64
        | op @ OpCode::SubF64
        | op @ OpCode::True
        | op @ OpCode::False
        | op @ OpCode::PopU8
        | op @ OpCode::PopU16
        | op @ OpCode::PopU32
        | op @ OpCode::PopU64
        | op @ OpCode::Not
        | op @ OpCode::EqualU8
        | op @ OpCode::EqualU64
        | op @ OpCode::GreaterF64
        | op @ OpCode::LesserF64
        | op @ OpCode::PrintF64
        | op @ OpCode::PrintBool
        | op @ OpCode::PrintString
        | op @ OpCode::IncreaseRC
        | op @ OpCode::DecreaseRC
        | op @ OpCode::HeapifyFloat => print_simple(op),
        op @ OpCode::Return | op @ OpCode::Call | op @ OpCode::CallClosure => {
            print_unary_u8(op, chunk.get_op(ip + 1))
        }
        op @ OpCode::ConstantF64
        | op @ OpCode::ConstantString
        | op @ OpCode::VariableU8
        | op @ OpCode::VariableU16
        | op @ OpCode::VariableU32
        | op @ OpCode::VariableU64
        | op @ OpCode::AssignU8
        | op @ OpCode::AssignU16
        | op @ OpCode::AssignU32
        | op @ OpCode::AssignU64
        | op @ OpCode::AssignObj
        | op @ OpCode::AssignHeapFloat
        | op @ OpCode::JumpIfFalse
        | op @ OpCode::Jump
        | op @ OpCode::Function
        | op @ OpCode::PushU16
        | op @ OpCode::HeapFloat => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::Closure => {
            print_simple(op);
            // TODO
            4
        }
    }
}
