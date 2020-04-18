use super::compiler::*;
use super::*;

pub fn run_vm(mut chunk: Chunk) {
    let mut ip = 0;
    loop {
        match get_op(&chunk, ip) {
            [OP_RETURN, _, _, _] => {
                println!("RETURN: {}", pop_f64(&mut chunk));
                return;
            }
            [OP_CONSTANT_F64, i, _, _] => {
                let v = get_f64(&chunk, i);
                push_f64(&mut chunk, v);
            }
            [OP_ADD_F64, _, _, _] => {
                let b = pop_f64(&mut chunk);
                let a = pop_f64(&mut chunk);
                push_f64(&mut chunk, a + b);
            }
            a @ _ => {
                println!("{:?}", a);
                todo!();
            }
        }

        ip += 4;
    }
}
