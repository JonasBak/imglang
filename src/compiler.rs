use super::*;

impl Ast {
    pub fn codegen(&self, chunk: &mut Chunk) {
        match self {
            Ast::Program(exprs) => {
                for expr in exprs.iter() {
                    expr.codegen(chunk);
                }
                add_op(chunk, [OP_RETURN, 0, 0, 0]);
            }
            Ast::Number(n) => {
                let i = add_f64(chunk, *n);
                add_op(chunk, [OP_CONSTANT_F64, i, 0, 0]);
            }
            Ast::Add(l, r) => {
                l.codegen(chunk);
                r.codegen(chunk);
                add_op(chunk, [OP_ADD_F64, 0, 0, 0]);
            }
            _ => todo!(),
        }
    }
}
