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
                let i = add_const_f64(chunk, *n);
                push_op(chunk, OP_CONSTANT_F64);
                push_op_u16(chunk, i);
            }
            Ast::Bool(a) => {
                match a {
                    true => push_op(chunk, OP_TRUE),
                    false => push_op(chunk, OP_FALSE),
                };
            }
            Ast::Nil => {
                push_nil(chunk);
            }
            Ast::Negate(n) => {
                n.codegen(chunk);
                push_op(chunk, OP_NEGATE_F64);
            }
            Ast::Not(n) => {
                n.codegen(chunk);
                push_op(chunk, OP_NOT);
            }
            Ast::Multiply(l, r, _) => binary_op(chunk, l, r, OP_MULTIPLY_F64),
            Ast::Divide(l, r, _) => binary_op(chunk, l, r, OP_DIVIDE_F64),
            Ast::Add(l, r, _) => binary_op(chunk, l, r, OP_ADD_F64),
            Ast::Sub(l, r, _) => binary_op(chunk, l, r, OP_SUB_F64),
            Ast::Equal(l, r, t) => {
                l.codegen(chunk);
                r.codegen(chunk);
                match t.unwrap() {
                    AstType::Nil | AstType::Bool => push_op(chunk, OP_EQUAL_U8),
                    AstType::Float => push_op(chunk, OP_EQUAL_U64),
                };
            }
            Ast::NotEqual(l, r, t) => {
                l.codegen(chunk);
                r.codegen(chunk);
                match t.unwrap() {
                    AstType::Nil | AstType::Bool => push_op(chunk, OP_EQUAL_U8),
                    AstType::Float => push_op(chunk, OP_EQUAL_U64),
                };
                push_op(chunk, OP_NOT);
            }
            Ast::Greater(l, r, t) => {
                l.codegen(chunk);
                r.codegen(chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OP_GREATER_F64),
                    _ => panic!(),
                };
            }
            Ast::GreaterEqual(l, r, t) => {
                r.codegen(chunk);
                l.codegen(chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OP_LESSER_F64),
                    _ => panic!(),
                };
                push_op(chunk, OP_NOT);
            }
            Ast::Lesser(l, r, t) => {
                l.codegen(chunk);
                r.codegen(chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OP_LESSER_F64),
                    _ => panic!(),
                };
            }
            Ast::LesserEqual(l, r, t) => {
                r.codegen(chunk);
                l.codegen(chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OP_GREATER_F64),
                    _ => panic!(),
                };
                push_op(chunk, OP_NOT);
            }
        }
    }
}
