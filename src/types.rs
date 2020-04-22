use super::*;
use std::collections::HashMap;
use std::mem;

#[derive(Debug, Clone, PartialEq)]
pub enum AstType {
    Function(Vec<AstType>, Box<AstType>),

    Float,
    Bool,
    Nil,
}
impl AstType {
    pub fn size(&self) -> u16 {
        match self {
            AstType::Bool | AstType::Nil => 1,
            AstType::Float => 8,
            AstType::Function { .. } => 2,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    NotAllowed(AstType),
    Mismatch(AstType, AstType),
    NotDefined(String),
    NotCallable(String),
    NotAssignable(String),
    BadCallSignature(String),
    Other(String),
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
                for p in ps.iter_mut() {
                    self.annotate_type(p)?;
                }
                AstType::Nil
            }
            Ast::Block(ps) => {
                self.current_scope_depth += 1;
                for p in ps.iter_mut() {
                    self.annotate_type(p)?;
                }
                self.current_scope_depth -= 1;
                while self.variables.last().map(|v| v.depth).unwrap_or(0) > self.current_scope_depth
                {
                    self.variables.pop();
                }
                AstType::Nil
            }
            Ast::Print(expr, t) => {
                let expr_t = match self.annotate_type(expr)? {
                    t @ AstType::Bool | t @ AstType::Float => t,
                    t @ _ => return Err(TypeError::NotAllowed(t)),
                };
                t.replace(expr_t);
                AstType::Nil
            }
            Ast::Return(expr) => {
                // TODO don't allow if root
                let expr_t = self.annotate_type(expr)?;
                self.return_values.push(expr_t);
                AstType::Nil
            }
            Ast::Declaration(name, expr, t) => {
                let expr_t = self.annotate_type(expr)?;
                t.replace(expr_t.clone());
                self.declare_variable(name, expr_t);
                AstType::Nil
            }
            Ast::FuncDeclaration(name, func, args_t, t) => {
                if self.is_root && self.current_scope_depth == 0 {
                    self.globals.insert(
                        name.clone(),
                        AstType::Function(args_t.clone(), Box::new(t.clone())),
                    );
                } else {
                    return Err(TypeError::Other(
                        "global function declarations are only allowed at the top level"
                            .to_string(),
                    ));
                }
                self.annotate_type(func)?;
                AstType::Nil
            }
            Ast::Variable(name, t) => {
                let v = self
                    .resolve_variable(name)
                    .ok_or(TypeError::NotDefined(name.clone()))?;
                match v {
                    Variable::Local(local) => {
                        t.replace(local.t.clone());
                        local.t.clone()
                    }
                    Variable::Global(global) => {
                        t.replace(global.clone());
                        global.clone()
                    }
                }
            }
            Ast::Assign(name, expr, t) => {
                let v_t = match self
                    .resolve_variable(name)
                    .ok_or(TypeError::NotDefined(name.clone()))?
                {
                    Variable::Local(local) => local.t.clone(),
                    Variable::Global(_) => return Err(TypeError::NotAssignable(name.clone())),
                };
                let expr_t = self.annotate_type(expr)?;
                if v_t != expr_t {
                    return Err(TypeError::Mismatch(v_t, expr_t));
                }
                t.replace(expr_t);
                v_t
            }
            Ast::If(expr, stmt, else_stmt) => {
                let expr_t = self.annotate_type(expr)?;
                if expr_t != AstType::Bool {
                    return Err(TypeError::NotAllowed(expr_t));
                }
                self.annotate_type(stmt)?;
                if let Some(else_stmt) = else_stmt {
                    self.annotate_type(else_stmt)?;
                }
                AstType::Nil
            }
            Ast::While(expr, stmt) => {
                let expr_t = self.annotate_type(expr)?;
                if expr_t != AstType::Bool {
                    return Err(TypeError::NotAllowed(expr_t));
                }
                self.annotate_type(stmt)?;
                AstType::Nil
            }
            Ast::ExprStatement(expr, t) => {
                let expr_t = self.annotate_type(expr)?;
                t.replace(expr_t);
                AstType::Nil
            }
            Ast::Function { body, args, ret_t } => {
                let old_variables = mem::replace(&mut self.variables, vec![]);
                let old_return_values = mem::replace(&mut self.return_values, vec![]);
                let old_depth = self.current_scope_depth;
                let old_is_root = self.is_root;
                self.current_scope_depth = 0;
                self.is_root = false;

                for arg in args.iter() {
                    self.declare_variable(&arg.0, arg.1.clone());
                }

                let body_t = self.annotate_type(body)?;

                // TODO check for divergens and potential "leftouts" that default to nil
                // TODO check if all self.return_values is ret_t
                // if *ret_t != body_t {
                //     return Err(TypeError::Mismatch(ret_t.clone(), body_t));
                // }

                mem::replace(&mut self.variables, old_variables);
                mem::replace(&mut self.return_values, old_return_values);
                self.current_scope_depth = old_depth;
                self.is_root = old_is_root;

                AstType::Function(
                    args.iter().map(|t| t.1.clone()).collect(),
                    Box::new(ret_t.clone()),
                )
            }
            Ast::Call(name, args) => {
                let mut args_t = vec![];
                for arg in args.iter_mut() {
                    args_t.push(self.annotate_type(arg)?);
                }
                let (func_args_t, ret_t) = match self
                    .resolve_variable(name)
                    .ok_or(TypeError::NotDefined(name.clone()))?
                {
                    Variable::Local(LocalVariable {
                        t: AstType::Function(a, r),
                        ..
                    }) => (a, r),
                    Variable::Global(AstType::Function(a, r)) => (a, r),
                    _ => return Err(TypeError::NotCallable(name.clone())),
                };
                if args_t != func_args_t {
                    return Err(TypeError::BadCallSignature(name.clone()));
                }
                *ret_t.clone()
            }
            Ast::Float(_) => AstType::Float,
            Ast::Bool(_) => AstType::Bool,
            Ast::Nil => AstType::Nil,
            Ast::Negate(a) => self.annotate_type(a)?,
            Ast::Not(a) => {
                self.annotate_type(a)?;
                AstType::Bool
            }
            Ast::Multiply(l, r, t)
            | Ast::Divide(l, r, t)
            | Ast::Add(l, r, t)
            | Ast::Sub(l, r, t) => {
                let t_l = self.annotate_type(l)?;
                let t_r = self.annotate_type(r)?;
                if t_l != t_r {
                    return Err(TypeError::Mismatch(t_l, t_r));
                }
                t.replace(t_r.clone());
                t_r
            }
            Ast::Equal(l, r, t)
            | Ast::NotEqual(l, r, t)
            | Ast::Greater(l, r, t)
            | Ast::GreaterEqual(l, r, t)
            | Ast::Lesser(l, r, t)
            | Ast::LesserEqual(l, r, t) => {
                let t_l = self.annotate_type(l)?;
                let t_r = self.annotate_type(r)?;
                if t_l != t_r {
                    return Err(TypeError::Mismatch(t_l, t_r));
                }
                t.replace(t_r);
                AstType::Bool
            }
            Ast::And(l, r) | Ast::Or(l, r) => {
                let t_l = self.annotate_type(l)?;
                let t_r = self.annotate_type(r)?;
                if t_l != AstType::Bool {
                    return Err(TypeError::NotAllowed(t_l));
                }
                if t_r != AstType::Bool {
                    return Err(TypeError::NotAllowed(t_r));
                }
                AstType::Bool
            }
        };
        Ok(t)
    }
}
