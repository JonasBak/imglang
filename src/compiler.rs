use super::*;

fn binary_op(chunk: &mut Chunk, l: &Ast, r: &Ast, op: u8) {
    l.codegen(chunk);
    r.codegen(chunk);
    push_op(chunk, op);
}

impl Ast {
    pub fn codegen(&self, chunk: &mut Chunk) {
        match self {
            Ast::Program(ps) => {
                for p in ps.iter() {
                    p.codegen(chunk);
                }
                push_op(chunk, OpCode::Return as u8);
            }
            Ast::Print(expr, t) => {
                expr.codegen(chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OpCode::PrintF64 as u8),
                    AstType::Bool => push_op(chunk, OpCode::PrintBool as u8),
                    _ => todo!(),
                };
            }
            Ast::ExprStatement(expr, t) => {
                expr.codegen(chunk);
                match t.unwrap() {
                    AstType::Bool => push_op(chunk, OpCode::PopU8 as u8),
                    AstType::Float => push_op(chunk, OpCode::PopU64 as u8),
                    _ => todo!(),
                };
            }
            Ast::Float(n) => {
                let i = add_const_f64(chunk, *n);
                push_op(chunk, OpCode::ConstantF64 as u8);
                push_op_u16(chunk, i);
            }
            Ast::Bool(a) => {
                match a {
                    true => push_op(chunk, OpCode::True as u8),
                    false => push_op(chunk, OpCode::False as u8),
                };
            }
            Ast::Nil => {
                push_nil(chunk);
            }
            Ast::Negate(n) => {
                n.codegen(chunk);
                push_op(chunk, OpCode::NegateF64 as u8);
            }
            Ast::Not(n) => {
                n.codegen(chunk);
                push_op(chunk, OpCode::Not as u8);
            }
            Ast::Multiply(l, r, _) => binary_op(chunk, l, r, OpCode::MultiplyF64 as u8),
            Ast::Divide(l, r, _) => binary_op(chunk, l, r, OpCode::DivideF64 as u8),
            Ast::Add(l, r, _) => binary_op(chunk, l, r, OpCode::AddF64 as u8),
            Ast::Sub(l, r, _) => binary_op(chunk, l, r, OpCode::SubF64 as u8),
            Ast::Equal(l, r, t) => {
                l.codegen(chunk);
                r.codegen(chunk);
                match t.unwrap() {
                    AstType::Nil | AstType::Bool => push_op(chunk, OpCode::EqualU8 as u8),
                    AstType::Float => push_op(chunk, OpCode::EqualU64 as u8),
                };
            }
            Ast::NotEqual(l, r, t) => {
                l.codegen(chunk);
                r.codegen(chunk);
                match t.unwrap() {
                    AstType::Nil | AstType::Bool => push_op(chunk, OpCode::EqualU8 as u8),
                    AstType::Float => push_op(chunk, OpCode::EqualU64 as u8),
                };
                push_op(chunk, OpCode::Not as u8);
            }
            Ast::Greater(l, r, t) => {
                l.codegen(chunk);
                r.codegen(chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OpCode::GreaterF64 as u8),
                    _ => panic!(),
                };
            }
            Ast::GreaterEqual(l, r, t) => {
                r.codegen(chunk);
                l.codegen(chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OpCode::LesserF64 as u8),
                    _ => panic!(),
                };
                push_op(chunk, OpCode::Not as u8);
            }
            Ast::Lesser(l, r, t) => {
                l.codegen(chunk);
                r.codegen(chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OpCode::LesserF64 as u8),
                    _ => panic!(),
                };
            }
            Ast::LesserEqual(l, r, t) => {
                r.codegen(chunk);
                l.codegen(chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OpCode::GreaterF64 as u8),
                    _ => panic!(),
                };
                push_op(chunk, OpCode::Not as u8);
            }
        }
    }
}
