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
    for i in 0..chunk.len_code() / 4 {
        let ip = i * 4;
        print!("{:0>6}\t", ip);
        disassemble(get_op(&chunk, ip));
    }
    println!("{:*^64}", "DATA");
    for i in 0..chunk.len_data() / 8 {
        let p = i * 8;
        println!("{:0>6}\t\t\t\tu64{: >24}", p, get_u64(&chunk, p as u8));
        println!("\t\t\t\tf64{: >24}", get_f64(&chunk, p as u8));
    }
}

pub fn disassemble(op: [u8; 4]) {
    let op_code = match op[0] {
        OP_RETURN => "OP_RETURN",
        OP_CONSTANT_F64 => "OP_CONSTANT_F64",
        OP_NEGATE_F64 => "OP_NEGATE_F64",
        OP_MULTIPLY_F64 => "OP_MULTIPLY_F64",
        OP_DIVIDE_F64 => "OP_DIVIDE_F64",
        OP_ADD_F64 => "OP_ADD_F64",
        OP_SUB_F64 => "OP_SUB_F64",
        _ => "???",
    };
    println!(
        "{: >3} {: <24}\t{: >3}\t{: >3}\t{: >3}",
        op[0], op_code, op[1], op[2], op[3]
    );
}
