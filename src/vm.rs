use super::compiler::*;
use super::*;

macro_rules! expr {
    ($e:expr) => {
        $e
    };
}
macro_rules! binary_op_f64{
    ($chunk:ident, $op:tt) => {{
        let r = pop_f64(&mut $chunk);
        let l = pop_f64(&mut $chunk);
        push_f64(&mut $chunk, expr!(l $op r));
    }}
}

pub fn run_vm(mut chunk: Chunk) {
    let mut ip = 0;
    loop {
        print!("{:0>6}\t", ip);
        disassemble(&chunk, ip);
        ip = ip + 1;
        match OpCode::from(get_op(&chunk, ip - 1)) {
            OpCode::Return => {
                return;
            }
            OpCode::PrintF64 => {
                let a = pop_f64(&mut chunk);
                println!("< {}", a);
            }
            OpCode::PrintBool => {
                let a = pop_bool(&mut chunk);
                println!("< {}", a);
            }
            OpCode::ConstantF64 => {
                let i = get_op_u16(&mut chunk, ip);
                ip += 2;
                let v = get_f64(&chunk, i);
                push_f64(&mut chunk, v);
            }
            OpCode::NegateF64 => {
                let a = pop_f64(&mut chunk);
                push_f64(&mut chunk, -a);
            }
            OpCode::Not => {
                let a = pop_bool(&mut chunk);
                push_bool(&mut chunk, !a);
            }
            OpCode::MultiplyF64 => binary_op_f64!(chunk, *),
            OpCode::DivideF64 => binary_op_f64!(chunk, /),
            OpCode::AddF64 => binary_op_f64!(chunk, +),
            OpCode::SubF64 => binary_op_f64!(chunk, -),
            OpCode::Nil => push_nil(&mut chunk),
            OpCode::True => push_bool(&mut chunk, true),
            OpCode::False => push_bool(&mut chunk, false),
            OpCode::PopU8 => pop_nil(&mut chunk),
            OpCode::PopU64 => {
                pop_f64(&mut chunk);
            }
            OpCode::EqualU8 => {
                let r = pop_u8(&mut chunk);
                let l = pop_u8(&mut chunk);
                push_bool(&mut chunk, l == r);
            }
            OpCode::EqualU64 => {
                let r = pop_u64(&mut chunk);
                let l = pop_u64(&mut chunk);
                push_bool(&mut chunk, l == r);
            }
            OpCode::GreaterF64 => {
                let r = pop_f64(&mut chunk);
                let l = pop_f64(&mut chunk);
                push_bool(&mut chunk, l > r);
            }
            OpCode::LesserF64 => {
                let r = pop_f64(&mut chunk);
                let l = pop_f64(&mut chunk);
                push_bool(&mut chunk, l < r);
            }
        }
    }
}
