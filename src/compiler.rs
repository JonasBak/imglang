use super::*;

fn binary_op(chunk: &mut Chunk, l: &Ast, r: &Ast, op: u8) {
    l.codegen(chunk);
    r.codegen(chunk);
    add_op(chunk, [op, 0, 0, 0]);
}

impl Ast {
    pub fn codegen(&self, chunk: &mut Chunk) {
        match self {
            Ast::Program(expr) => {
                expr.codegen(chunk);
                add_op(chunk, [OP_RETURN, 0, 0, 0]);
            }
            Ast::Float(n) => {
                let i = add_f64(chunk, *n);
                add_op(chunk, [OP_CONSTANT_F64, i, 0, 0]);
            }
            Ast::Negate(n) => {
                n.codegen(chunk);
                add_op(chunk, [OP_NEGATE_F64, 0, 0, 0]);
            }
            Ast::Multiply(l, r) => binary_op(chunk, l, r, OP_MULTIPLY_F64),
            Ast::Divide(l, r) => binary_op(chunk, l, r, OP_DIVIDE_F64),
            Ast::Add(l, r) => binary_op(chunk, l, r, OP_ADD_F64),
            Ast::Sub(l, r) => binary_op(chunk, l, r, OP_SUB_F64),
            _ => todo!(),
        }
    }
}
