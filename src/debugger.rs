use super::*;
use std::fmt;

pub fn print_lexer_err(source: &String, error: LexerError) {
    let mut source = source.clone();
    match error {
        LexerError::Parse(i) => source.insert_str(i, ">>>"),
        LexerError::Unescaped(i) => source.insert_str(i, ">>>"),
    }
    eprintln!("{}", source);
    match error {
        LexerError::Parse(i) => {
            eprintln!("Error: Could not parse character at position {}", i);
        }
        LexerError::Unescaped(i) => {
            eprintln!("Error: Unescaped string starting at {}", i);
        }
    }
}

fn flatmap_parser_error(error: ParserError, list: &mut Vec<(Token, &'static str)>) {
    match error {
        ParserError::Unexpected(token, msg) => list.push((token, msg)),
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
    let errors: Vec<(Token, &'static str, i32)> = errors
        .into_iter()
        .map(|err| {
            let line = lines_map[err.0.start];
            (err.0, err.1, line)
        })
        .collect();
    eprintln!("{} parser errors!\n", errors.len());
    for error in errors.iter() {
        eprintln!("...\n{: >3} | {}\n...", error.2, lines[error.2 as usize]);
        eprintln!("> on token {:?}: \"{}\"", error.0.t, error.1);
    }
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
        op @ OpCode::Return => print_unary_u8(op, chunk.get_op(ip + 1)),
        op @ OpCode::ConstantF64 => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::ConstantString => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::NegateF64 => print_simple(op),
        op @ OpCode::MultiplyF64 => print_simple(op),
        op @ OpCode::DivideF64 => print_simple(op),
        op @ OpCode::AddF64 => print_simple(op),
        op @ OpCode::SubF64 => print_simple(op),
        op @ OpCode::True => print_simple(op),
        op @ OpCode::False => print_simple(op),
        op @ OpCode::PopU8 => print_simple(op),
        op @ OpCode::PopU16 => print_simple(op),
        op @ OpCode::PopU32 => print_simple(op),
        op @ OpCode::PopU64 => print_simple(op),
        op @ OpCode::Not => print_simple(op),
        op @ OpCode::EqualU8 => print_simple(op),
        op @ OpCode::EqualU64 => print_simple(op),
        op @ OpCode::GreaterF64 => print_simple(op),
        op @ OpCode::LesserF64 => print_simple(op),
        op @ OpCode::PrintF64 => print_simple(op),
        op @ OpCode::PrintBool => print_simple(op),
        op @ OpCode::PrintString => print_simple(op),
        op @ OpCode::VariableU8 => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::VariableU16 => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::VariableU32 => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::VariableU64 => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::AssignU8 => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::AssignU16 => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::AssignU32 => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::AssignU64 => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::AssignObj => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::JumpIfFalse => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::Jump => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::Function => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::Call => print_unary_u8(op, chunk.get_op(ip + 1)),
        op @ OpCode::PushU16 => print_unary_u16(op, chunk.get_op_u16(ip + 1)),
        op @ OpCode::IncreaseRC => print_simple(op),
        op @ OpCode::DecreaseRC => print_simple(op),
    }
}
