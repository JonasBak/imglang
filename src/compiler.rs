use super::*;

fn binary_op(chunk: &mut Chunk, l: &Ast, r: &Ast, op: u8) {
    l.codegen(chunk);
    r.codegen(chunk);
    push_op(chunk, op);
}

impl Ast {
    pub fn codegen(&self, chunk: &mut Chunk) {
        match self {
            Ast::Program(expr) => {
                expr.codegen(chunk);
                push_op(chunk, OP_RETURN);
            }
            Ast::Float(n) => {
                let i = add_f64(chunk, *n);
                push_op(chunk, OP_CONSTANT_F64);
                push_op_u16(chunk, i);
            }
            Ast::Negate(n) => {
                n.codegen(chunk);
                push_op(chunk, OP_NEGATE_F64);
            }
            Ast::Multiply(l, r) => binary_op(chunk, l, r, OP_MULTIPLY_F64),
            Ast::Divide(l, r) => binary_op(chunk, l, r, OP_DIVIDE_F64),
            Ast::Add(l, r) => binary_op(chunk, l, r, OP_ADD_F64),
            Ast::Sub(l, r) => binary_op(chunk, l, r, OP_SUB_F64),
            _ => todo!(),
        }
    }
}
