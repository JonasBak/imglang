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
                chunk.push_op(OpCode::Return as u8);
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
                        AstType::Bool => chunk.push_op(OpCode::PopU8 as u8),
                        AstType::Float => chunk.push_op(OpCode::PopU64 as u8),
                        _ => todo!(),
                    };
                }
            }
            Ast::Print(expr, t) => {
                self.codegen(expr, chunk);
                match t.unwrap() {
                    AstType::Float => chunk.push_op(OpCode::PrintF64 as u8),
                    AstType::Bool => chunk.push_op(OpCode::PrintBool as u8),
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
                    AstType::Bool => chunk.push_op(OpCode::VariableU8 as u8),
                    AstType::Float => chunk.push_op(OpCode::VariableU64 as u8),
                    _ => todo!(),
                };
                chunk.push_op_u16(v.offset);
            }
            Ast::Assign(name, expr, t) => {
                self.codegen(expr, chunk);
                let v = self
                    .resolve_variable(name)
                    .expect("TODO could not resolve variable");
                match t.unwrap() {
                    AstType::Bool => chunk.push_op(OpCode::AssignU8 as u8),
                    AstType::Float => chunk.push_op(OpCode::AssignU64 as u8),
                    _ => todo!(),
                };
                chunk.push_op_u16(v.offset);
            }
            Ast::If(expr, stmt, else_stmt) => {
                self.codegen(expr, chunk);

                chunk.push_op(OpCode::JumpIfFalse as u8);
                let else_jump = chunk.push_op_u16(0);
                chunk.push_op(OpCode::PopU8 as u8);

                self.codegen(stmt, chunk);

                chunk.push_op(OpCode::Jump as u8);
                let if_jump = chunk.push_op_u16(0);

                chunk.backpatch_jump(else_jump);
                chunk.push_op(OpCode::PopU8 as u8);

                if let Some(else_stmt) = else_stmt {
                    self.codegen(else_stmt, chunk);
                }

                chunk.backpatch_jump(if_jump);
            }
            Ast::While(expr, stmt) => {
                let loop_start = chunk.len_code();

                self.codegen(expr, chunk);

                chunk.push_op(OpCode::JumpIfFalse as u8);
                let done_jump = chunk.push_op_u16(0);
                chunk.push_op(OpCode::PopU8 as u8);

                self.codegen(stmt, chunk);

                chunk.push_op(OpCode::Jump as u8);
                chunk.push_op_u16(loop_start as u16);

                chunk.backpatch_jump(done_jump);
                chunk.push_op(OpCode::PopU8 as u8);
            }
            Ast::ExprStatement(expr, t) => {
                self.codegen(expr, chunk);
                match t.unwrap() {
                    AstType::Bool => chunk.push_op(OpCode::PopU8 as u8),
                    AstType::Float => chunk.push_op(OpCode::PopU64 as u8),
                    _ => todo!(),
                };
            }
            Ast::Float(n) => {
                let i = chunk.add_const_f64(*n);
                chunk.push_op(OpCode::ConstantF64 as u8);
                chunk.push_op_u16(i);
            }
            Ast::Bool(a) => {
                match a {
                    true => chunk.push_op(OpCode::True as u8),
                    false => chunk.push_op(OpCode::False as u8),
                };
            }
            Ast::Nil => {
                chunk.push_op(OpCode::Nil as u8);
            }
            Ast::Negate(n) => {
                self.codegen(n, chunk);
                chunk.push_op(OpCode::NegateF64 as u8);
            }
            Ast::Not(n) => {
                self.codegen(n, chunk);
                chunk.push_op(OpCode::Not as u8);
            }
            Ast::Multiply(l, r, _) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                chunk.push_op(OpCode::MultiplyF64 as u8);
            }
            Ast::Divide(l, r, _) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                chunk.push_op(OpCode::DivideF64 as u8);
            }
            Ast::Add(l, r, _) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                chunk.push_op(OpCode::AddF64 as u8);
            }
            Ast::Sub(l, r, _) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                chunk.push_op(OpCode::SubF64 as u8);
            }
            Ast::Equal(l, r, t) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                match t.unwrap() {
                    AstType::Nil | AstType::Bool => chunk.push_op(OpCode::EqualU8 as u8),
                    AstType::Float => chunk.push_op(OpCode::EqualU64 as u8),
                };
            }
            Ast::NotEqual(l, r, t) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                match t.unwrap() {
                    AstType::Nil | AstType::Bool => chunk.push_op(OpCode::EqualU8 as u8),
                    AstType::Float => chunk.push_op(OpCode::EqualU64 as u8),
                };
                chunk.push_op(OpCode::Not as u8);
            }
            Ast::Greater(l, r, t) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                match t.unwrap() {
                    AstType::Float => chunk.push_op(OpCode::GreaterF64 as u8),
                    _ => panic!(),
                };
            }
            Ast::GreaterEqual(l, r, t) => {
                self.codegen(r, chunk);
                self.codegen(l, chunk);
                match t.unwrap() {
                    AstType::Float => chunk.push_op(OpCode::LesserF64 as u8),
                    _ => panic!(),
                };
                chunk.push_op(OpCode::Not as u8);
            }
            Ast::Lesser(l, r, t) => {
                self.codegen(l, chunk);
                self.codegen(r, chunk);
                match t.unwrap() {
                    AstType::Float => chunk.push_op(OpCode::LesserF64 as u8),
                    _ => panic!(),
                };
            }
            Ast::LesserEqual(l, r, t) => {
                self.codegen(r, chunk);
                self.codegen(l, chunk);
                match t.unwrap() {
                    AstType::Float => chunk.push_op(OpCode::GreaterF64 as u8),
                    _ => panic!(),
                };
                chunk.push_op(OpCode::Not as u8);
            }
            Ast::And(l, r) => {
                self.codegen(l, chunk);

                chunk.push_op(OpCode::JumpIfFalse as u8);
                let false_jump = chunk.push_op_u16(0);

                chunk.push_op(OpCode::PopU8 as u8);

                self.codegen(r, chunk);

                chunk.backpatch_jump(false_jump);
            }
            Ast::Or(l, r) => {
                self.codegen(l, chunk);

                chunk.push_op(OpCode::JumpIfFalse as u8);
                let false_jump = chunk.push_op_u16(0);

                chunk.push_op(OpCode::Jump as u8);
                let true_jump = chunk.push_op_u16(0);

                chunk.backpatch_jump(false_jump);

                chunk.push_op(OpCode::PopU8 as u8);

                self.codegen(r, chunk);

                chunk.backpatch_jump(true_jump);
            }
        }
    }
}
