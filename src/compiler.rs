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
    External(ExternalAdr),
}
enum Variable {
    Local(LocalVariable),
    Global(GlobalVariable),
}
pub struct Compiler<'a> {
    variables: Vec<LocalVariable>,
    globals: HashMap<String, GlobalVariable>,
    externals: Option<&'a Externals>,
    current_scope_depth: u16,
    chunks: Vec<Chunk>,
    current_chunk: ChunkAdr,
    is_root: bool,
}

impl<'a> Compiler<'a> {
    pub fn compile(ast: &Ast, externals: Option<&'a Externals>) -> Vec<Chunk> {
        let mut compiler = Compiler {
            variables: vec![],
            globals: HashMap::new(),
            externals,
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
        let global = self.globals.get(name).map(|v| Variable::Global(v.clone()));
        if global.is_some() {
            return global;
        }
        self.externals
            .map(|ext| ext.lookup_function(name))
            .flatten()
            .map(|t| Variable::Global(GlobalVariable::External(t)))
    }
    fn pop_type(&mut self, t: &AstType) {
        match t {
            AstType::Bool => {
                self.chunk().push_op(OpCode::PopU8);
            }
            AstType::Function(..) => {
                self.chunk().push_op(OpCode::PopU16);
            }
            AstType::Float => {
                self.chunk().push_op(OpCode::PopU64);
            }
            AstType::Closure(..) | AstType::HeapAllocated(_) | AstType::String => {
                self.chunk().push_op(OpCode::DecreaseRC);
                self.chunk().push_op(OpCode::PopU32);
            }
            AstType::ExternalFunction(..) => {
                self.chunk().push_op(OpCode::PopU16);
            }
            AstType::Nil => {}
        };
    }
    fn push_return(&mut self, expr: &Option<Box<Ast>>, t: &AstType) {
        if let Some(expr) = expr {
            self.codegen(expr);
        }
        let objects: Vec<StackAdr> = self
            .variables
            .iter_mut()
            .filter(|v| v.t.is_obj())
            .map(|v| v.offset)
            .collect();
        for obj_var in objects.into_iter() {
            // TODO change to OpCode::DecreaseRCAt or something to get fewer ops
            self.chunk()
                .push_op(OpCode::VariableU32 { stack_i: obj_var });
            self.chunk().push_op(OpCode::DecreaseRC);
            self.chunk().push_op(OpCode::PopU32);
        }
        self.chunk().push_op(OpCode::Return {
            ret_width: t.size() as u8,
        });
    }
    fn pop_variables(&mut self) {
        while self.variables.last().map(|v| v.depth).unwrap_or(0) > self.current_scope_depth {
            let t = self.variables.pop().unwrap().t;
            self.pop_type(&t);
        }
    }
    fn codegen(&mut self, ast: &Ast) {
        match ast {
            Ast::Program(ps) => {
                for p in ps.iter() {
                    self.codegen(p);
                }
                self.chunk().push_op(OpCode::Return { ret_width: 0 });
            }
            Ast::Block { cont, .. } => {
                self.current_scope_depth += 1;
                for p in cont.iter() {
                    self.codegen(p);
                }
                self.current_scope_depth -= 1;
                self.pop_variables();
            }
            Ast::Print { expr, t, .. } => {
                self.codegen(expr);
                match t.as_ref().unwrap() {
                    AstType::Float => {
                        self.chunk().push_op(OpCode::PrintF64);
                    }
                    AstType::Bool => {
                        self.chunk().push_op(OpCode::PrintBool);
                    }
                    AstType::ExternalFunction(..)
                    | AstType::Closure(..)
                    | AstType::HeapAllocated(_)
                    | AstType::Function(..)
                    | AstType::Nil => todo!(),
                    AstType::String => {
                        self.chunk().push_op(OpCode::PrintString);
                    }
                };
            }
            Ast::Return { expr, t, .. } => {
                self.push_return(expr, t.as_ref().unwrap());
            }
            Ast::Declaration { name, expr, t, .. } => {
                self.codegen(expr);
                self.declare_variable(name, t.clone().unwrap());
            }
            Ast::FuncDeclaration { name, func, .. } => {
                self.globals.insert(
                    name.clone(),
                    GlobalVariable::Function(self.chunks.len() as ChunkAdr),
                );
                self.codegen(func);
                self.chunk().push_op(OpCode::PopU16);
            }
            Ast::Variable { name, t, .. } => {
                let v = self.resolve_variable(name).unwrap();
                match v {
                    Variable::Local(v) => {
                        let is_rc = match t.as_ref().unwrap() {
                            AstType::Bool => {
                                self.chunk()
                                    .push_op(OpCode::VariableU8 { stack_i: v.offset });
                                false
                            }
                            AstType::Function { .. } => {
                                self.chunk()
                                    .push_op(OpCode::VariableU16 { stack_i: v.offset });
                                false
                            }
                            AstType::Float => {
                                self.chunk()
                                    .push_op(OpCode::VariableU64 { stack_i: v.offset });
                                false
                            }
                            AstType::HeapAllocated(inner_t) => {
                                match **inner_t {
                                    AstType::Float => self
                                        .chunk()
                                        .push_op(OpCode::HeapFloat { stack_i: v.offset }),
                                    AstType::Bool => {
                                        self.chunk().push_op(OpCode::HeapBool { stack_i: v.offset })
                                    }
                                    _ => todo!(),
                                };
                                false
                            }
                            AstType::Closure(..) | AstType::String => {
                                self.chunk()
                                    .push_op(OpCode::VariableU32 { stack_i: v.offset });
                                true
                            }
                            AstType::ExternalFunction(..) | AstType::Nil => panic!(),
                        };
                        if is_rc {
                            self.chunk().push_op(OpCode::IncreaseRC);
                        }
                    }
                    Variable::Global(GlobalVariable::Function(chunk_i)) => {
                        self.chunk().push_op(OpCode::PushU16 { data: chunk_i });
                    }
                    Variable::Global(GlobalVariable::External(func_i)) => {
                        self.chunk().push_op(OpCode::PushU16 { data: func_i });
                    }
                }
            }
            Ast::Assign {
                name,
                expr,
                t,
                move_to_heap,
                ..
            } => {
                self.codegen(expr);
                let v = self.resolve_variable(name).unwrap();
                match v {
                    Variable::Local(v) => {
                        match t.as_ref().unwrap() {
                            AstType::Bool => {
                                self.chunk().push_op(OpCode::AssignU8 { stack_i: v.offset })
                            }
                            AstType::Function(..) => self
                                .chunk()
                                .push_op(OpCode::AssignU16 { stack_i: v.offset }),
                            AstType::Float => self
                                .chunk()
                                .push_op(OpCode::AssignU64 { stack_i: v.offset }),
                            AstType::HeapAllocated(inner_t) => {
                                if move_to_heap.unwrap() {
                                    match **inner_t {
                                        AstType::Float => self
                                            .chunk()
                                            .push_op(OpCode::AssignHeapFloat { stack_i: v.offset }),
                                        AstType::Bool => self
                                            .chunk()
                                            .push_op(OpCode::AssignHeapBool { stack_i: v.offset }),
                                        _ => todo!(),
                                    }
                                } else {
                                    self.chunk()
                                        .push_op(OpCode::AssignObj { stack_i: v.offset })
                                }
                            }
                            AstType::Closure(..) | AstType::String => self
                                .chunk()
                                .push_op(OpCode::AssignObj { stack_i: v.offset }),
                            AstType::ExternalFunction(..) | AstType::Nil => panic!(),
                        };
                    }
                    _ => panic!(),
                }
            }
            Ast::If {
                condition,
                body,
                else_body,
                ..
            } => {
                self.codegen(condition);

                let else_jump = self.chunk().push_op(OpCode::JumpIfFalse { ip: 0 });
                self.chunk().push_op(OpCode::PopU8);

                self.codegen(body);

                let if_jump = self.chunk().push_op(OpCode::Jump { ip: 0 });

                self.chunk().backpatch_jump(else_jump);
                self.chunk().push_op(OpCode::PopU8);

                if let Some(else_stmt) = else_body {
                    self.codegen(else_stmt);
                }

                self.chunk().backpatch_jump(if_jump);
            }
            Ast::While {
                condition, body, ..
            } => {
                let loop_start = self.chunk().len_code();

                self.codegen(condition);

                let done_jump = self.chunk().push_op(OpCode::JumpIfFalse { ip: 0 });
                self.chunk().push_op(OpCode::PopU8);

                self.codegen(body);

                self.chunk().push_op(OpCode::Jump { ip: loop_start });

                self.chunk().backpatch_jump(done_jump);
                self.chunk().push_op(OpCode::PopU8);
            }
            Ast::ExprStatement { expr, t, .. } => {
                self.codegen(expr);
                self.pop_type(t.as_ref().unwrap());
            }
            Ast::Function {
                body,
                args,
                ret_t,
                captured,
                pos,
            } => {
                let prev_chunk = self.current_chunk;
                self.chunks.push(Chunk::new());
                self.current_chunk = self.chunks.len() as ChunkAdr - 1;

                let old_variables = mem::replace(&mut self.variables, vec![]);
                let old_depth = mem::replace(&mut self.current_scope_depth, 0);
                let old_is_root = mem::replace(&mut self.is_root, false);

                for arg in args.iter() {
                    self.declare_variable(&arg.0, arg.1.clone());
                }
                for var in captured.iter() {
                    self.declare_variable(
                        &var.0,
                        AstType::HeapAllocated(Box::new(var.1.clone().unwrap())),
                    );
                }

                self.codegen(body);
                if *ret_t == AstType::Nil {
                    self.push_return(&None, &AstType::Nil);
                }

                mem::replace(&mut self.variables, old_variables);
                mem::replace(&mut self.current_scope_depth, old_depth);
                mem::replace(&mut self.is_root, old_is_root);

                let c = mem::replace(&mut self.current_chunk, prev_chunk);

                if captured.len() == 0 {
                    self.chunk().push_op(OpCode::Function { chunk_i: c });
                } else {
                    for var in captured.iter() {
                        self.codegen(&Ast::Variable {
                            name: var.0.clone(),
                            t: var.1.clone(),
                            pos: *pos,
                        });
                        match var.1.as_ref().unwrap() {
                            AstType::Float => self.chunk().push_op(OpCode::HeapifyFloat),
                            AstType::Bool => self.chunk().push_op(OpCode::HeapifyBool),
                            _ => todo!(),
                        };
                    }
                    self.chunk().push_op(OpCode::Closure {
                        chunk_i: c,
                        capture_len: captured.len() as u8,
                    });
                }
            }
            Ast::Call {
                ident,
                args,
                args_width,
                call_t,
                ..
            } => {
                for arg in args.iter() {
                    self.codegen(arg);
                }

                self.codegen(ident);

                let args_width = args_width.unwrap();

                match call_t.as_ref().unwrap() {
                    CallType::Function => self.chunk().push_op(OpCode::Call { args_width }),
                    CallType::Closure => self.chunk().push_op(OpCode::CallClosure { args_width }),
                    CallType::External => self.chunk().push_op(OpCode::CallExternal { args_width }),
                };
            }
            Ast::Float(n, _) => {
                let i = self.chunk().add_const_f64(*n);
                self.chunk().push_op(OpCode::ConstantF64 { data_i: i });
            }
            Ast::Bool(a, _) => {
                match a {
                    true => self.chunk().push_op(OpCode::True),
                    false => self.chunk().push_op(OpCode::False),
                };
            }
            Ast::String(s, _) => {
                let i = self.chunk().add_const_string(s);
                self.chunk().push_op(OpCode::ConstantString { data_i: i });
            }
            Ast::Negate(n, _) => {
                self.codegen(n);
                self.chunk().push_op(OpCode::NegateF64);
            }
            Ast::Not(n, _) => {
                self.codegen(n);
                self.chunk().push_op(OpCode::Not);
            }
            Ast::Multiply(l, r, _, _) => {
                self.codegen(l);
                self.codegen(r);
                self.chunk().push_op(OpCode::MultiplyF64);
            }
            Ast::Divide(l, r, _, _) => {
                self.codegen(l);
                self.codegen(r);
                self.chunk().push_op(OpCode::DivideF64);
            }
            Ast::Add(l, r, _, _) => {
                self.codegen(l);
                self.codegen(r);
                self.chunk().push_op(OpCode::AddF64);
            }
            Ast::Sub(l, r, _, _) => {
                self.codegen(l);
                self.codegen(r);
                self.chunk().push_op(OpCode::SubF64);
            }
            Ast::Equal(l, r, t, _) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Bool => self.chunk().push_op(OpCode::EqualU8),
                    AstType::Float => self.chunk().push_op(OpCode::EqualU64),
                    _ => todo!(),
                };
            }
            Ast::NotEqual(l, r, t, _) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Bool => self.chunk().push_op(OpCode::EqualU8),
                    AstType::Float => self.chunk().push_op(OpCode::EqualU64),
                    _ => todo!(),
                };
                self.chunk().push_op(OpCode::Not);
            }
            Ast::Greater(l, r, t, _) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Float => self.chunk().push_op(OpCode::GreaterF64),
                    _ => panic!(),
                };
            }
            Ast::GreaterEqual(l, r, t, _) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Float => self.chunk().push_op(OpCode::LesserF64),
                    _ => panic!(),
                };
                self.chunk().push_op(OpCode::Not);
            }
            Ast::Lesser(l, r, t, _) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Float => self.chunk().push_op(OpCode::LesserF64),
                    _ => panic!(),
                };
            }
            Ast::LesserEqual(l, r, t, _) => {
                self.codegen(l);
                self.codegen(r);
                match t.as_ref().unwrap() {
                    AstType::Float => self.chunk().push_op(OpCode::GreaterF64),
                    _ => panic!(),
                };
                self.chunk().push_op(OpCode::Not);
            }
            Ast::And(l, r, _) => {
                self.codegen(l);

                let false_jump = self.chunk().push_op(OpCode::JumpIfFalse { ip: 0 });

                self.chunk().push_op(OpCode::PopU8);

                self.codegen(r);

                self.chunk().backpatch_jump(false_jump);
            }
            Ast::Or(l, r, _) => {
                self.codegen(l);

                let false_jump = self.chunk().push_op(OpCode::JumpIfFalse { ip: 0 });

                let true_jump = self.chunk().push_op(OpCode::Jump { ip: 0 });

                self.chunk().backpatch_jump(false_jump);

                self.chunk().push_op(OpCode::PopU8);

                self.codegen(r);

                self.chunk().backpatch_jump(true_jump);
            }
        }
    }
}
