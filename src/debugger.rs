use super::*;
use std::fmt;

pub fn print_lexer_err(source: &String, error: LexerError) {
    let mut source = source.clone();
    match error {
        LexerError::Parse(i) => source.insert_str(i, ">>>"),
        LexerError::Unescaped(i) => source.insert_str(i, ">>>"),
    }
    println!("{}", source);
    match error {
        LexerError::Parse(i) => {
            println!("Error: Could not parse character at position {}", i);
        }
        LexerError::Unescaped(i) => {
            println!("Error: Unescaped string starting at {}", i);
        }
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn disassemble_chunk(chunk: &Chunk) {
    println!("{:*^64}", "BYTECODE");
    let mut ip = 0;
    while ip < chunk.len_code() {
        print!("{:0>6}\t", ip);
        ip += disassemble(chunk, ip);
    }
    println!("{:*^64}", "DATA");
    for i in 0..chunk.len_data() / 8 {
        let p = i * 8;
        println!(
            "{:0>6}\t\t\t\tu64{: >24}",
            p,
            get_const_u64(&chunk, p as u16)
        );
        println!("\t\t\t\tf64{: >24}", get_const_f64(&chunk, p as u16));
    }
    println!("{:*^64}", "");
}

fn print_simple(op: OpCode) -> usize {
    println!("{: >3} {: <24}", op as u8, op);
    1
}
fn print_unary(op: OpCode, operand: u64) {
    println!("{: >3} {: <24}\t{: >8}", op as u8, op, operand);
}

pub fn disassemble(chunk: &Chunk, ip: usize) -> usize {
    match OpCode::from(get_op(chunk, ip)) {
        op @ OpCode::Return => print_simple(op),
        op @ OpCode::ConstantF64 => {
            print_unary(op, get_op_u16(chunk, ip + 1) as u64);
            3
        }
        op @ OpCode::NegateF64 => print_simple(op),
        op @ OpCode::MultiplyF64 => print_simple(op),
        op @ OpCode::DivideF64 => print_simple(op),
        op @ OpCode::AddF64 => print_simple(op),
        op @ OpCode::SubF64 => print_simple(op),
        op @ OpCode::Nil => print_simple(op),
        op @ OpCode::True => print_simple(op),
        op @ OpCode::False => print_simple(op),
        op @ OpCode::PopU8 => print_simple(op),
        op @ OpCode::PopU64 => print_simple(op),
        op @ OpCode::Not => print_simple(op),
        op @ OpCode::EqualU8 => print_simple(op),
        op @ OpCode::EqualU64 => print_simple(op),
        op @ OpCode::GreaterF64 => print_simple(op),
        op @ OpCode::LesserF64 => print_simple(op),
        op @ OpCode::PrintF64 => print_simple(op),
        op @ OpCode::PrintBool => print_simple(op),
        op @ OpCode::VariableU8 => {
            print_unary(op, get_op_u16(chunk, ip + 1) as u64);
            3
        }
        op @ OpCode::VariableU64 => {
            print_unary(op, get_op_u16(chunk, ip + 1) as u64);
            3
        }
        op @ OpCode::AssignU8 => {
            print_unary(op, get_op_u16(chunk, ip + 1) as u64);
            3
        }
        op @ OpCode::AssignU64 => {
            print_unary(op, get_op_u16(chunk, ip + 1) as u64);
            3
        }
    }
}
