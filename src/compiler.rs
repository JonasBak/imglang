use super::*;
use std::collections::HashMap;
use std::mem;

#[derive(Debug, Clone)]
struct LocalVariable {
    name: String,
    depth: u16,
    offset: StackAdr,
    t: AstType,
}
#[derive(Debug, Clone)]
enum GlobalVariable {
    Function(ChunkAdr),
}
enum Variable {
    Local(LocalVariable),
    Global(GlobalVariable),
}
pub struct Compiler {
    variables: Vec<LocalVariable>,
    globals: HashMap<String, GlobalVariable>,
    current_scope_depth: u16,
    chunks: Vec<Chunk>,
    current_chunk: ChunkAdr,
    is_root: bool,
}

impl Compiler {
    pub fn compile(ast: &Ast) -> Vec<Chunk> {
        let mut compiler = Compiler {
            variables: vec![],
            globals: HashMap::new(),
            current_scope_depth: 0,
            chunks: vec![Chunk::new()],
            current_chunk: 0,
            is_root: true,
        };
        compiler.codegen(ast);
        compiler.chunks
    }
    fn chunk(&mut self) -> &mut Chunk {
        &mut self.chunks[self.current_chunk as usize]
    }
    fn declare_variable(&mut self, name: &String, t: AstType) {
        self.variables.push(LocalVariable {
            name: name.clone(),
            depth: self.current_scope_depth,
            offset: self
                .variables
                .iter()
                .last()
                .map(|v| v.offset + v.t.size())
                .unwrap_or(0),
            t,
        });
    }
    fn resolve_variable(&mut self, name: &String) -> Option<Variable> {
        let local = self
            .variables
            .iter_mut()
            .rev()
            .find(|v| &v.name == name)
            .cloned()
            .map(|v| Variable::Local(v));
        if local.is_some() {
            return local;
        }
        self.globals.get(name).map(|v| Variable::Global(v.clone()))
    }
    fn pop_variables(&mut self) {
        while self.variables.last().map(|v| v.depth).unwrap_or(0) > self.current_scope_depth {
            match self.variables.pop().unwrap().t {
                AstType::Bool => self.chunk().push_op(OpCode::PopU8 as u8),
                AstType::Function(..) => self.chunk().push_op(OpCode::PopU16 as u8),
                AstType::Float => self.chunk().push_op(OpCode::PopU64 as u8),
                AstType::String => {
                    self.chunk().push_op(OpCode::DecreaseRC as u8);
                    self.chunk().push_op(OpCode::PopU32 as u8)
                }
                AstType::Nil => panic!(),
            };
        }
    }
    fn codegen(&mut self, ast: &Ast) {
        match ast {
            Ast::Program(ps) => {
                for p in ps.iter() {
                    self.codegen(p);
                }
                self.chunk().push_op(OpCode::Return as u8);
            }
            Ast::Block(ps) => {
                self.current_scope_depth += 1;
                for p in ps.iter() {
                    self.codegen(p);
                }
                self.current_scope_depth -= 1;
                self.pop_variables();
            }
            Ast::Print(expr, t) => {
                self.codegen(expr);
                match t.as_ref().unwrap() {
                    AstType::Float => self.chunk().push_op(OpCode::PrintF64 as u8),
                    AstType::Bool => self.chunk().push_op(OpCode::PrintBool as u8),
                    AstType::Function(..) => todo!(),
                    AstType::Nil => todo!(),
                    AstType::String => self.chunk().push_op(OpCode::PrintString as u8),
                };
            }
            Ast::Return(expr) => {
                if let Some(expr) = expr {
                    self.codegen(expr);
                }
                self.chunk().push_op(OpCode::Return as u8);
            }
            Ast::Declaration(name, expr, t) => {
                self.codegen(expr);
                match t {
                    Some(AstType::String) => {
                        self.chunk().push_op(OpCode::IncreaseRC as u8);
                    }
                    _ => {}
                }
                self.declare_variable(name, t.clone().unwrap());
            }
            Ast::FuncDeclaration(name, func, _, _) => {
                self.globals.insert(
                    name.clone(),
                    GlobalVariable::Function(self.chunks.len() as ChunkAdr),
                );
                self.codegen(func);
                self.chunk().push_op(OpCode::PopU16 as u8);
            }
            Ast::Variable(name, t) => {
                let v = self
                    .resolve_variable(name)
                    .expect("TODO could not resolve variable");
                match v {
                    Variable::Local(v) => {
                        match t.as_ref().unwrap() {
                            AstType::Bool => self.chunk().push_op(OpCode::VariableU8 as u8),
                            AstType::Function { .. } => {
                                self.chunk().push_op(OpCode::VariableU16 as u8)
                            }
                            AstType::Float => self.chunk().push_op(OpCode::VariableU64 as u8),
                            AstType::String => self.chunk().push_op(OpCode::VariableU32 as u8),
                            AstType::Nil => panic!(),
                        };
                        self.chunk().push_op_u16(v.offset);
                    }
                    Variable::Global(GlobalVariable::Function(chunk_i)) => {
                        self.chunk().push_op(OpCode::PushU16 as u8);
                        self.chunk().push_op_u16(chunk_i);
                    }
                }
            }
            Ast::Assign(name, expr, t) => {
                self.codegen(expr);
                let v = self
                    .resolve_variable(name)
                    .expect("TODO could not resolve variable");
                match v {
                    Variable::Local(v) => {
                        match t.as_ref().unwrap() {
                            AstType::Bool => self.chunk().push_op(OpCode::AssignU8 as u8),
                            AstType::Function(..) => self.chunk().push_op(OpCode::AssignU16 as u8),
                            AstType::Float => self.chunk().push_op(OpCode::AssignU64 as u8),
                            AstType::String => todo!("handle rc when assigning"),
                            AstType::Nil => panic!(),
                        };
                        self.chunk().push_op_u16(v.offset);
                    }
                    _ => panic!(),
                }
            }
            Ast::If(expr, stmt, else_stmt) => {
                self.codegen(expr);

                self.chunk().push_op(OpCode::JumpIfFalse as u8);
                let else_jump = self.chunk().push_op_u16(0);
                self.chunk().push_op(OpCode::PopU8 as u8);

                self.codegen(stmt);

                self.chunk().push_op(OpCode::Jump as u8);
                let if_jump = self.chunk().push_op_u16(0);

                self.chunk().backpatch_jump(else_jump);
                self.chunk().push_op(OpCode::PopU8 as u8);

                if let Some(else_stmt) = else_stmt {
                    self.codegen(else_stmt);
                }

                self.chunk().backpatch_jump(if_jump);
            }
            Ast::While(expr, stmt) => {
                let loop_start = self.chunk().len_code();

                self.codegen(expr);

                self.chunk().push_op(OpCode::JumpIfFalse as u8);
                let done_jump = self.chunk().push_op_u16(0);
                self.chunk().push_op(OpCode::PopU8 as u8);

                self.codegen(stmt);

                self.chunk().push_op(OpCode::Jump as u8);
                self.chunk().push_op_u16(loop_start);

                self.chunk().backpatch_jump(done_jump);
                self.chunk().push_op(OpCode::PopU8 as u8);
            }
            Ast::ExprStatement(expr, t) => {
                self.codegen(expr);
                match t.as_ref().unwrap() {
                    AstType::Bool => {
                        self.chunk().push_op(OpCode::PopU8 as u8);
                    }
                    AstType::Float => {
                        self.chunk().push_op(OpCode::PopU64 as u8);
                    }
                    AstType::Nil => (),
                    _ => todo!(),
                };
            }
            Ast::Function { body, args, .. } => {
                let prev_chunk = self.current_chunk;
                self.chunks.push(Chunk::new());
                self.current_chunk = self.chunks.len() as ChunkAdr - 1;

                let old_variables = mem::replace(&mut self.variables, vec![]);
                let old_depth = self.current_scope_depth;
                let old_is_root = self.is_root;
                self.current_scope_depth = 0;
                self.is_root = false;

                for arg in args.iter() {
                    self.declare_variable(&arg.0, arg.1.clone());
                }

                self.codegen(body);
                self.chunk().push_op(OpCode::Return as u8);

                mem::replace(&mut self.variables, old_variables);
                self.current_scope_depth = old_depth;
                self.is_root = old_is_root;

                let c = self.current_chunk;

                self.current_chunk = prev_chunk;

                self.chunk().push_op(OpCode::Function as u8);
                self.chunk().push_op_u16(c);
            }
            Ast::Call(ident, args, args_width) => {
                for arg in args.iter() {
                    self.codegen(arg);
                }

                self.codegen(ident);

                let args_width = args_width.unwrap();

                self.chunk().push_op(OpCode::Call as u8);
                self.chunk().push_op(args_width);
            }
            Ast::Float(n) => {
                let i = self.chunk().add_const_f64(*n);
                self.chunk().push_op(OpCode::ConstantF64 as u8);
                self.chunk().push_op_u16(i);
            }
            Ast::Bool(a) => {
                match a {
                    true => self.chunk().push_op(OpCode::True as u8),
                    false => self.chunk().push_op(OpCode::False as u8),
                };
            }
            Ast::String(s) => {
                let i = self.chunk().add_const_string(s);
                self.chunk().push_op(OpCode::ConstantString as u8);
                self.chunk().push_op_u16(i);
            }
            Ast::Negate(n) => {
                self.codegen(n);
                self.chunk().push_op(OpCode::NegateF64 as u8);
            }
            Ast::Not(n) => {
                self.codegen(n);
                self.chunk().push_op(OpCode::Not as u8);
            }
            Ast::Multiply(l, r, _) => {
                self.codegen(l);
                self.codegen(r);
                self.chunk().push_op(OpCode::MultiplyF64 as u8);
            }
            Ast::Divide(l, r, _) => {
                self.codegen(l);
                self.codegen(r);
                self.chunk().push_op(OpCode::DivideF64 as u8);
            }
            Ast::Add(l, r, _) => {
                self.codegen(l);
                self.codegen(r);
                self.chunk().push_op(OpCode::AddF64 as u8);
            }
            Ast::Sub(l, r, _) => {
                self.codegen(l);
                self.codegen(r);
                self.chunk().push_op(OpCode::SubF64 as u8);
            }
            Ast::Equal(l, r, t) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Bool => self.chunk().push_op(OpCode::EqualU8 as u8),
                    AstType::Float => self.chunk().push_op(OpCode::EqualU64 as u8),
                    _ => todo!(),
                };
            }
            Ast::NotEqual(l, r, t) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Bool => self.chunk().push_op(OpCode::EqualU8 as u8),
                    AstType::Float => self.chunk().push_op(OpCode::EqualU64 as u8),
                    _ => todo!(),
                };
                self.chunk().push_op(OpCode::Not as u8);
            }
            Ast::Greater(l, r, t) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Float => self.chunk().push_op(OpCode::GreaterF64 as u8),
                    _ => panic!(),
                };
            }
            Ast::GreaterEqual(l, r, t) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Float => self.chunk().push_op(OpCode::LesserF64 as u8),
                    _ => panic!(),
                };
                self.chunk().push_op(OpCode::Not as u8);
            }
            Ast::Lesser(l, r, t) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Float => self.chunk().push_op(OpCode::LesserF64 as u8),
                    _ => panic!(),
                };
            }
            Ast::LesserEqual(l, r, t) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Float => self.chunk().push_op(OpCode::GreaterF64 as u8),
                    _ => panic!(),
                };
                self.chunk().push_op(OpCode::Not as u8);
            }
            Ast::And(l, r) => {
                self.codegen(l);

                self.chunk().push_op(OpCode::JumpIfFalse as u8);
                let false_jump = self.chunk().push_op_u16(0);

                self.chunk().push_op(OpCode::PopU8 as u8);

                self.codegen(r);

                self.chunk().backpatch_jump(false_jump);
            }
            Ast::Or(l, r) => {
                self.codegen(l);

                self.chunk().push_op(OpCode::JumpIfFalse as u8);
                let false_jump = self.chunk().push_op_u16(0);

                self.chunk().push_op(OpCode::Jump as u8);
                let true_jump = self.chunk().push_op_u16(0);

                self.chunk().backpatch_jump(false_jump);

                self.chunk().push_op(OpCode::PopU8 as u8);

                self.codegen(r);

                self.chunk().backpatch_jump(true_jump);
            }
        }
    }
}
