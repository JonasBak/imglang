use super::*;
use std::collections::HashMap;
use std::mem;

#[derive(Debug, Clone, PartialEq)]
pub enum AstType {
    Function(Vec<AstType>, Box<AstType>),
    Closure(Vec<AstType>, Box<AstType>),

    Float,
    Bool,
    Nil,

    String,

    HeapAllocated(Box<AstType>),
}
impl AstType {
    pub fn size(&self) -> StackAdr {
        let n = match self {
            AstType::Bool => 1,
            AstType::Float => 8,
            AstType::Function(..) => mem::size_of::<ChunkAdr>(),
            AstType::Closure(..) | AstType::HeapAllocated(_) | AstType::String => {
                mem::size_of::<HeapAdr>()
            }
            AstType::Nil => 0,
        };
        n as StackAdr
    }
    pub fn is_obj(&self) -> bool {
        match self {
            AstType::HeapAllocated(_) | AstType::Closure(_, _) | AstType::String => true,
            _ => false,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    Error(String, usize),

    BlockErrors(Vec<TypeError>),
}

#[derive(Debug, Clone)]
struct LocalVariable {
    name: String,
    depth: u16,
    t: AstType,
}
enum Variable {
    Local(LocalVariable),
    Global(AstType),
}
pub struct TypeChecker {
    variables: Vec<LocalVariable>,
    globals: HashMap<String, AstType>,
    current_scope_depth: u16,
    is_root: bool,
    return_values: Vec<AstType>,
}

impl TypeChecker {
    pub fn annotate_types(ast: &mut Ast) -> Result<(), TypeError> {
        let mut type_checker = TypeChecker {
            variables: vec![],
            globals: HashMap::new(),
            current_scope_depth: 0,
            is_root: true,
            return_values: vec![],
        };
        type_checker.annotate_type(ast)?;
        Ok(())
    }
    fn declare_variable(&mut self, name: &String, t: AstType) {
        self.variables.push(LocalVariable {
            name: name.clone(),
            depth: self.current_scope_depth,
            t,
        });
    }
    fn resolve_variable(&mut self, name: &String) -> Option<Variable> {
        let local = self
            .variables
            .iter()
            .rev()
            .find(|v| &v.name == name)
            .map(|v| Variable::Local(v.clone()));
        if local.is_some() {
            return local;
        }
        self.globals
            .get(name)
            .cloned()
            .map(|var| Variable::Global(var))
    }
    fn annotate_type(&mut self, ast: &mut Ast) -> Result<AstType, TypeError> {
        let t = match ast {
            Ast::Program(ps) => {
                let mut errors = Vec::new();
                for p in ps.iter_mut() {
                    match self.annotate_type(p) {
                        Ok(_) => {}
                        Err(error) => errors.push(error),
                    }
                }
                if errors.len() > 0 {
                    return Err(TypeError::BlockErrors(errors));
                }
                AstType::Nil
            }
            Ast::Block { cont, .. } => {
                let mut errors = Vec::new();
                self.current_scope_depth += 1;
                for p in cont.iter_mut() {
                    match self.annotate_type(p) {
                        Ok(_) => {}
                        Err(error) => errors.push(error),
                    }
                }
                self.current_scope_depth -= 1;
                while self.variables.last().map(|v| v.depth).unwrap_or(0) > self.current_scope_depth
                {
                    self.variables.pop();
                }
                if errors.len() > 0 {
                    return Err(TypeError::BlockErrors(errors));
                }
                AstType::Nil
            }
            Ast::Print { expr, t, pos } => {
                let expr_t = match self.annotate_type(expr)? {
                    t @ AstType::Bool | t @ AstType::Float | t @ AstType::String => t,
                    t @ _ => {
                        return Err(TypeError::Error(format!("cannot print type {:?}", t), *pos))
                    }
                };
                t.replace(expr_t);
                AstType::Nil
            }
            Ast::Return { expr, t, pos } => {
                if self.is_root {
                    return Err(TypeError::Error(
                        "can't return from root function".to_string(),
                        *pos,
                    ));
                }
                let expr_t = if let Some(expr) = expr {
                    self.annotate_type(expr)?
                } else {
                    AstType::Nil
                };
                t.replace(expr_t.clone());
                self.return_values.push(expr_t);
                AstType::Nil
            }
            Ast::Declaration { name, expr, t, .. } => {
                let expr_t = self.annotate_type(expr)?;
                t.replace(expr_t.clone());
                self.declare_variable(name, expr_t);
                AstType::Nil
            }
            Ast::FuncDeclaration {
                name,
                func,
                args_t,
                ret_t,
                pos,
            } => {
                if self.is_root && self.current_scope_depth == 0 {
                    self.globals.insert(
                        name.clone(),
                        AstType::Function(args_t.clone(), Box::new(ret_t.clone())),
                    );
                } else {
                    return Err(TypeError::Error(
                        "global function declarations are only allowed at the top level"
                            .to_string(),
                        *pos,
                    ));
                }
                self.annotate_type(func)?;
                AstType::Nil
            }
            Ast::Variable { name, t, pos } => {
                let v = self.resolve_variable(name).ok_or(TypeError::Error(
                    format!("variable {} is not defined", name),
                    *pos,
                ))?;
                match v {
                    Variable::Local(local) => {
                        t.replace(local.t.clone());
                        if let AstType::HeapAllocated(t) = local.t {
                            *t.clone()
                        } else {
                            local.t.clone()
                        }
                    }
                    Variable::Global(global) => {
                        t.replace(global.clone());
                        global.clone()
                    }
                }
            }
            Ast::Assign {
                name,
                expr,
                t,
                move_to_heap,
                pos,
            } => {
                let v_t = match self.resolve_variable(name).ok_or(TypeError::Error(
                    format!("variable {} is not defined", name),
                    *pos,
                ))? {
                    Variable::Local(local) => local.t.clone(),
                    Variable::Global(_) => {
                        return Err(TypeError::Error(
                            format!("can't assign to global variable {}", name),
                            *pos,
                        ))
                    }
                };
                let expr_t = self.annotate_type(expr)?;
                match (&v_t, &expr_t) {
                    (a, b) if a == b => {
                        move_to_heap.replace(false);
                    }
                    (AstType::HeapAllocated(a), b) if **a == *b => {
                        move_to_heap.replace(true);
                    }
                    _ => {
                        return Err(TypeError::Error(
                            format!(
                                "cannot assign value of type {:?} to variable with type {:?}",
                                expr_t, v_t
                            ),
                            *pos,
                        ));
                    }
                }
                t.replace(v_t);
                expr_t
            }
            Ast::If {
                condition,
                body,
                else_body,
                pos,
            } => {
                let condition_t = self.annotate_type(condition)?;
                if condition_t != AstType::Bool {
                    return Err(TypeError::Error(
                        "condition must be a bool".to_string(),
                        *pos,
                    ));
                }
                self.annotate_type(body)?;
                if let Some(else_body) = else_body {
                    self.annotate_type(else_body)?;
                }
                AstType::Nil
            }
            Ast::While {
                condition,
                body,
                pos,
            } => {
                let condition_t = self.annotate_type(condition)?;
                if condition_t != AstType::Bool {
                    return Err(TypeError::Error(
                        "condition must be a bool".to_string(),
                        *pos,
                    ));
                }
                self.annotate_type(body)?;
                AstType::Nil
            }
            Ast::ExprStatement { expr, t, .. } => {
                let expr_t = self.annotate_type(expr)?;
                t.replace(expr_t);
                AstType::Nil
            }
            Ast::Function {
                body,
                args,
                captured,
                ret_t,
                pos,
            } => {
                captured
                    .iter_mut()
                    .map(|(name, var_t)| match self.resolve_variable(name) {
                        Some(Variable::Local(LocalVariable { t, .. }))
                        | Some(Variable::Global(t)) => {
                            var_t.replace(t);
                            Ok(())
                        }
                        None => Err(TypeError::Error(
                            format!("variable {} is not defined", name),
                            *pos,
                        )),
                    })
                    .collect::<Result<Vec<_>, TypeError>>()?;

                let old_variables = mem::replace(&mut self.variables, vec![]);
                let old_return_values = mem::replace(&mut self.return_values, vec![]);
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

                let result = self.annotate_type(body);

                mem::replace(&mut self.variables, old_variables);
                let return_values = mem::replace(&mut self.return_values, old_return_values);
                mem::replace(&mut self.current_scope_depth, old_depth);
                mem::replace(&mut self.is_root, old_is_root);

                result?;

                // TODO check for divergence and potential "leftouts" that default to nil
                if let Some(t) = return_values.iter().filter(|t| *t != ret_t).next() {
                    return Err(TypeError::Error(
                        format!("return type {:?} doesn't match signature {:?}", t, ret_t),
                        *pos,
                    ));
                } else if return_values.len() == 0 && *ret_t != AstType::Nil {
                    return Err(TypeError::Error(
                        format!(
                            "function with return type {:?} needs explicit return statement",
                            ret_t
                        ),
                        *pos,
                    ));
                }

                if captured.len() == 0 {
                    AstType::Function(
                        args.iter().map(|t| t.1.clone()).collect(),
                        Box::new(ret_t.clone()),
                    )
                } else {
                    AstType::Closure(
                        args.iter().map(|t| t.1.clone()).collect(),
                        Box::new(ret_t.clone()),
                    )
                }
            }
            Ast::Call {
                ident,
                args,
                args_width,
                is_closure,
                pos,
            } => {
                let ident_t = self.annotate_type(ident)?;
                let mut args_t = vec![];
                for arg in args.iter_mut() {
                    args_t.push(self.annotate_type(arg)?);
                }
                let (func_args_t, ret_t) = match ident_t {
                    AstType::Closure(a, r) => {
                        is_closure.replace(true);
                        (a, r)
                    }
                    AstType::Function(a, r) => {
                        is_closure.replace(false);
                        (a, r)
                    }
                    t @ _ => {
                        return Err(TypeError::Error(format!("cannot call type {:?}", t), *pos))
                    }
                };
                if args_t != func_args_t {
                    return Err(TypeError::Error(
                        format!(
                            "arguments doesn't match, requires {:?}, got {:?}",
                            func_args_t, args_t
                        ),
                        *pos,
                    ));
                }
                args_width.replace(args_t.iter().map(|t| t.size()).fold(0, |a, b| a + b) as u8);
                *ret_t.clone()
            }
            Ast::Float(_, _) => AstType::Float,
            Ast::Bool(_, _) => AstType::Bool,
            Ast::String(_, _) => AstType::String,
            Ast::Negate(a, pos) => {
                let t = self.annotate_type(a)?;
                if match t {
                    AstType::Float => false,
                    _ => true,
                } {
                    return Err(TypeError::Error(
                        format!("operation can't be preformed on type {:?}", t),
                        *pos,
                    ));
                }
                t
            }
            Ast::Not(a, pos) => {
                let t = self.annotate_type(a)?;
                if t != AstType::Bool {
                    return Err(TypeError::Error(
                        "not (!) operation requires a bool".to_string(),
                        *pos,
                    ));
                }
                AstType::Bool
            }
            Ast::Multiply(l, r, t, pos)
            | Ast::Divide(l, r, t, pos)
            | Ast::Add(l, r, t, pos)
            | Ast::Sub(l, r, t, pos) => {
                let t_l = self.annotate_type(l)?;
                let t_r = self.annotate_type(r)?;
                if t_l != t_r {
                    return Err(TypeError::Error(
                        format!(
                            "type of left operand ({:?}) doesn't match type of right ({:?})",
                            t_l, t_r
                        ),
                        *pos,
                    ));
                }
                if match t_l {
                    AstType::Float => false,
                    _ => true,
                } {
                    return Err(TypeError::Error(
                        format!("operation can't be preformed on type {:?}", t_l),
                        *pos,
                    ));
                }
                t.replace(t_r.clone());
                t_r
            }
            Ast::Equal(l, r, t, pos)
            | Ast::NotEqual(l, r, t, pos)
            | Ast::Greater(l, r, t, pos)
            | Ast::GreaterEqual(l, r, t, pos)
            | Ast::Lesser(l, r, t, pos)
            | Ast::LesserEqual(l, r, t, pos) => {
                let t_l = self.annotate_type(l)?;
                let t_r = self.annotate_type(r)?;
                if t_l != t_r {
                    return Err(TypeError::Error(
                        format!(
                            "type of left operand ({:?}) doesn't match type of right ({:?})",
                            t_l, t_r
                        ),
                        *pos,
                    ));
                }
                if match t_l {
                    AstType::Float => false,
                    _ => true,
                } {
                    return Err(TypeError::Error(
                        format!("operation can't be preformed on type {:?}", t_l),
                        *pos,
                    ));
                }
                t.replace(t_r);
                AstType::Bool
            }
            Ast::And(l, r, pos) | Ast::Or(l, r, pos) => {
                let t_l = self.annotate_type(l)?;
                let t_r = self.annotate_type(r)?;
                if t_l != AstType::Bool || t_r != AstType::Bool {
                    return Err(TypeError::Error(
                        "operation requires both operands to be bool".to_string(),
                        *pos,
                    ));
                }
                AstType::Bool
            }
        };
        Ok(t)
    }
}
