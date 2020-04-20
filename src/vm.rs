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
        match get_op(&chunk, ip - 1) {
            OP_RETURN => {
                return;
            }
            OP_PRINT_F64 => {
                let a = pop_f64(&mut chunk);
                println!("< {}", a);
            }
            OP_PRINT_BOOL => {
                let a = pop_bool(&mut chunk);
                println!("< {}", a);
            }
            OP_CONSTANT_F64 => {
                let i = get_op_u16(&mut chunk, ip);
                ip += 2;
                let v = get_f64(&chunk, i);
                push_f64(&mut chunk, v);
            }
            OP_NEGATE_F64 => {
                let a = pop_f64(&mut chunk);
                push_f64(&mut chunk, -a);
            }
            OP_NOT => {
                let a = pop_bool(&mut chunk);
                push_bool(&mut chunk, !a);
            }
            OP_MULTIPLY_F64 => binary_op_f64!(chunk, *),
            OP_DIVIDE_F64 => binary_op_f64!(chunk, /),
            OP_ADD_F64 => binary_op_f64!(chunk, +),
            OP_SUB_F64 => binary_op_f64!(chunk, -),
            OP_NIL => push_nil(&mut chunk),
            OP_TRUE => push_bool(&mut chunk, true),
            OP_FALSE => push_bool(&mut chunk, false),
            OP_POP_U8 => pop_nil(&mut chunk),
            OP_POP_U64 => {
                pop_f64(&mut chunk);
            }
            OP_EQUAL_U8 => {
                let r = pop_u8(&mut chunk);
                let l = pop_u8(&mut chunk);
                push_bool(&mut chunk, l == r);
            }
            OP_EQUAL_U64 => {
                let r = pop_u64(&mut chunk);
                let l = pop_u64(&mut chunk);
                push_bool(&mut chunk, l == r);
            }
            OP_GREATER_F64 => {
                let r = pop_f64(&mut chunk);
                let l = pop_f64(&mut chunk);
                push_bool(&mut chunk, l > r);
            }
            OP_LESSER_F64 => {
                let r = pop_f64(&mut chunk);
                let l = pop_f64(&mut chunk);
                push_bool(&mut chunk, l < r);
            }
            a @ _ => {
                println!("{:?}", a);
                todo!();
            }
        }
    }
}
