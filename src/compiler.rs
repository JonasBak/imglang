use super::*;

struct Variable {
    name: String,
    depth: u16,
    offset: u16,
    t: AstType,
}
pub struct Compiler {
    variables: Vec<Variable>,
    current_scope_depth: u16,
}

impl Compiler {
    pub fn compile(ast: &Ast) -> Chunk {
        let mut compiler = Compiler {
            variables: vec![],
            current_scope_depth: 0,
        };
        let mut chunk = Chunk::new();
        compiler.codegen(ast, &mut chunk);
        chunk
    }
    fn declare_variable(&mut self, name: &String, t: AstType) {
        self.variables.push(Variable {
            name: name.clone(),
            depth: self.current_scope_depth,
            offset: self
                .variables
                .iter()
                .last()
                .map(|v| {
                    v.offset
                        + match v.t {
                            AstType::Bool | AstType::Nil => 1,
                            AstType::Float => 8,
                        }
                })
                .unwrap_or(0),
            t,
        });
    }
    fn resolve_variable(&mut self, name: &String) -> Option<&Variable> {
        self.variables.iter().rev().find(|v| &v.name == name)
    }
    fn codegen(&mut self, ast: &Ast, chunk: &mut Chunk) {
        match ast {
            Ast::Program(ps) => {
                for p in ps.iter() {
                    self.codegen(p, chunk);
                }
                push_op(chunk, OpCode::Return as u8);
            }
            Ast::Block(ps) => {
                self.current_scope_depth += 1;
                for p in ps.iter() {
                    self.codegen(p, chunk);
                }
                self.current_scope_depth -= 1;
                while self.variables.last().map(|v| v.depth).unwrap_or(0) > self.current_scope_depth
                {
                    match self.variables.pop().unwrap().t {
                        AstType::Bool => push_op(chunk, OpCode::PopU8 as u8),
                        AstType::Float => push_op(chunk, OpCode::PopU64 as u8),
                        _ => todo!(),
                    };
                }
            }
            Ast::Print(expr, t) => {
                self.codegen(expr, chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OpCode::PrintF64 as u8),
                    AstType::Bool => push_op(chunk, OpCode::PrintBool as u8),
                    _ => todo!(),
                };
            }
            Ast::Declaration(name, expr, t) => {
                self.codegen(expr, chunk);
                self.declare_variable(name, t.unwrap());
            }
            Ast::Variable(name, t) => {
                let v = self
                    .resolve_variable(name)
                    .expect("TODO could not resolve variable");
                match t.unwrap() {
                    AstType::Bool => push_op(chunk, OpCode::VariableU8 as u8),
                    AstType::Float => push_op(chunk, OpCode::VariableU64 as u8),
                    _ => todo!(),
                };
                push_op_u16(chunk, v.offset);
            }
            Ast::Assign(name, expr, t) => {
                self.codegen(expr, chunk);
                let v = self
                    .resolve_variable(name)
                    .expect("TODO could not resolve variable");
                match t.unwrap() {
                    AstType::Bool => push_op(chunk, OpCode::AssignU8 as u8),
                    AstType::Float => push_op(chunk, OpCode::AssignU64 as u8),
                    _ => todo!(),
                };
                push_op_u16(chunk, v.offset);
            }
            Ast::If(expr, stmt, else_stmt) => {
                self.codegen(expr, chunk);

                push_op(chunk, OpCode::JumpIfFalse as u8);
                let else_jump = push_op_u16(chunk, 0);
                push_op(chunk, OpCode::PopU8 as u8);

                self.codegen(stmt, chunk);

                push_op(chunk, OpCode::Jump as u8);
                let if_jump = push_op_u16(chunk, 0);

                backpatch_jump(chunk, else_jump);
                push_op(chunk, OpCode::PopU8 as u8);

                if let Some(else_stmt) = else_stmt {
                    self.codegen(else_stmt, chunk);
                }

                backpatch_jump(chunk, if_jump);
            }
            Ast::ExprStatement(expr, t) => {
                self.codegen(expr, chunk);
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
                self.codegen(n, chunk);
                push_op(chunk, OpCode::NegateF64 as u8);
            }
            Ast::Not(n) => {
                self.codegen(n, chunk);
                push_op(chunk, OpCode::Not as u8);
            }
            Ast::Multiply(l, r, _) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                push_op(chunk, OpCode::MultiplyF64 as u8);
            }
            Ast::Divide(l, r, _) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                push_op(chunk, OpCode::DivideF64 as u8);
            }
            Ast::Add(l, r, _) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                push_op(chunk, OpCode::AddF64 as u8);
            }
            Ast::Sub(l, r, _) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                push_op(chunk, OpCode::SubF64 as u8);
            }
            Ast::Equal(l, r, t) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                match t.unwrap() {
                    AstType::Nil | AstType::Bool => push_op(chunk, OpCode::EqualU8 as u8),
                    AstType::Float => push_op(chunk, OpCode::EqualU64 as u8),
                };
            }
            Ast::NotEqual(l, r, t) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                match t.unwrap() {
                    AstType::Nil | AstType::Bool => push_op(chunk, OpCode::EqualU8 as u8),
                    AstType::Float => push_op(chunk, OpCode::EqualU64 as u8),
                };
                push_op(chunk, OpCode::Not as u8);
            }
            Ast::Greater(l, r, t) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OpCode::GreaterF64 as u8),
                    _ => panic!(),
                };
            }
            Ast::GreaterEqual(l, r, t) => {
                self.codegen(r, chunk);
                self.codegen(l, chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OpCode::LesserF64 as u8),
                    _ => panic!(),
                };
                push_op(chunk, OpCode::Not as u8);
            }
            Ast::Lesser(l, r, t) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OpCode::LesserF64 as u8),
                    _ => panic!(),
                };
            }
            Ast::LesserEqual(l, r, t) => {
                self.codegen(r, chunk);
                self.codegen(l, chunk);
                match t.unwrap() {
                    AstType::Float => push_op(chunk, OpCode::GreaterF64 as u8),
                    _ => panic!(),
                };
                push_op(chunk, OpCode::Not as u8);
            }
        }
    }
}
