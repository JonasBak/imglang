use super::*;

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
        println!("{:0>6}\t\t\t\tu64{: >24}", p, get_u64(&chunk, p as u16));
        println!("\t\t\t\tf64{: >24}", get_f64(&chunk, p as u16));
    }
    println!("{:*^64}", "");
}

fn print_simple(op: u8, op_string: &str) -> usize {
    println!("{: >3} {: <24}", op, op_string);
    1
}
fn print_unary(op: u8, op_string: &str, operand: u64) {
    println!("{: >3} {: <24}\t{: >8}", op, op_string, operand);
}

pub fn disassemble(chunk: &Chunk, ip: usize) -> usize {
    match get_op(chunk, ip) {
        op @ OP_RETURN => print_simple(op, "OP_RETURN"),
        op @ OP_CONSTANT_F64 => {
            print_unary(op, "OP_CONSTANT_F64", get_op(chunk, ip + 1) as u64);
            3
        }
        op @ OP_NEGATE_F64 => print_simple(op, "OP_NEGATE_F64"),
        op @ OP_MULTIPLY_F64 => print_simple(op, "OP_MULTIPLY_F64"),
        op @ OP_DIVIDE_F64 => print_simple(op, "OP_DIVIDE_F64"),
        op @ OP_ADD_F64 => print_simple(op, "OP_ADD_F64"),
        op @ OP_SUB_F64 => print_simple(op, "OP_SUB_F64"),
        op @ OP_NIL => print_simple(op, "OP_NIL"),
        op @ OP_TRUE => print_simple(op, "OP_TRUE"),
        op @ OP_FALSE => print_simple(op, "OP_FALSE"),
        op @ OP_POP_U8 => print_simple(op, "OP_POP_U8"),
        op @ OP_POP_U64 => print_simple(op, "OP_POP_U64"),
        op @ OP_NOT => print_simple(op, "OP_NOT"),
        op @ OP_EQUAL_U8 => print_simple(op, "OP_EQUAL_U8"),
        op @ OP_EQUAL_U64 => print_simple(op, "OP_EQUAL_U64"),
        op @ OP_GREATER_F64 => print_simple(op, "OP_GREATER_F64"),
        op @ OP_LESSER_F64 => print_simple(op, "OP_LESSER_F64"),
        op @ _ => print_simple(op, "?????"),
    }
}
