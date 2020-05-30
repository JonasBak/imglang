use super::*;
use std::collections::HashMap;
use std::mem;

#[derive(Debug, Clone, PartialEq)]
pub enum CallType {
    Function,
    Closure,
    External,
    Enum,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstType {
    Function(Vec<AstType>, Box<AstType>),
    Closure(Vec<AstType>, Box<AstType>),
    ExternalFunction(Vec<AstType>, Box<AstType>),

    Enum(String),
    EnumVariant { enum_type: String, t: Box<AstType> },

    Float,
    Bool,
    Nil,

    String,

    HeapAllocated(Box<AstType>),
}
impl AstType {
    pub fn is_obj(&self) -> bool {
        match self {
            AstType::HeapAllocated(_) | AstType::Closure(_, _) | AstType::String => true,
            _ => false,
        }
    }
    pub fn width(&self) -> usize {
        match self {
            AstType::Bool => bool::width(),
            AstType::Function(..) => ChunkAdr::width(),
            AstType::Float => f64::width(),
            AstType::Enum(..) => u8::width(),
            AstType::ExternalFunction(..) => ExternalAdr::width(),
            AstType::Closure(..) | AstType::HeapAllocated(_) | AstType::String => HeapAdr::width(),
            AstType::Nil | AstType::EnumVariant { .. } => panic!(),
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

pub struct TypeChecker<'a> {
    variables: Vec<LocalVariable>,
    globals: HashMap<String, AstType>,
    externals: Option<&'a Externals>,
    current_scope_depth: u16,
    is_root: bool,
    return_values: Vec<AstType>,
}

impl<'a> TypeChecker<'a> {
    pub fn annotate_types(
        ast: &mut Ast,
        externals: Option<&'a Externals>,
    ) -> Result<(), TypeError> {
        let mut type_checker = TypeChecker {
            variables: vec![],
            globals: HashMap::new(),
            externals,
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
        let global = self
            .globals
            .get(name)
            .cloned()
            .map(|var| Variable::Global(var));
        if global.is_some() {
            return global;
        }
        self.externals
            .map(|ext| ext.lookup_type(name))
            .flatten()
            .map(|t| Variable::Global(t))
    }
    fn annotate_type(&mut self, ast: &mut Ast) -> Result<(AstType, bool), TypeError> {
        let (t, diverges) = match ast {
            Ast::Program(ps) => {
                let mut errors = Vec::new();
                let mut diverges = false;
                for p in ps.iter_mut() {
                    match self.annotate_type(p) {
                        Ok((_, d)) => {
                            diverges = diverges || d;
                        }
                        Err(error) => errors.push(error),
                    }
                }
                if errors.len() > 0 {
                    return Err(TypeError::BlockErrors(errors));
                }
                (AstType::Nil, diverges)
            }
            Ast::Block { cont, .. } => {
                let mut errors = Vec::new();
                self.current_scope_depth += 1;
                let mut diverges = false;
                for p in cont.iter_mut() {
                    match self.annotate_type(p) {
                        Ok((_, d)) => {
                            diverges = d || diverges;
                        }
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
                (AstType::Nil, diverges)
            }
            Ast::Print { expr, t, pos } => {
                let expr_t = match self.annotate_type(expr)?.0 {
                    t @ AstType::Bool | t @ AstType::Float | t @ AstType::String => t,
                    t @ _ => {
                        return Err(TypeError::Error(format!("cannot print type {:?}", t), *pos))
                    }
                };
                t.replace(expr_t);
                (AstType::Nil, false)
            }
            Ast::Return { expr, t, pos } => {
                if self.is_root {
                    return Err(TypeError::Error(
                        "can't return from root function".to_string(),
                        *pos,
                    ));
                }
                let expr_t = if let Some(expr) = expr {
                    self.annotate_type(expr)?.0
                } else {
                    AstType::Nil
                };
                t.replace(expr_t.clone());
                self.return_values.push(expr_t);
                (AstType::Nil, true)
            }
            Ast::Declaration { name, expr, t, .. } => {
                let expr_t = self.annotate_type(expr)?.0;
                t.replace(expr_t.clone());
                self.declare_variable(name, expr_t);
                (AstType::Nil, false)
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
                self.annotate_type(func)?.0;
                (AstType::Nil, false)
            }
            Ast::EnumDeclaration {
                name,
                variants,
                pos,
            } => {
                if !(self.is_root && self.current_scope_depth == 0) {
                    return Err(TypeError::Error(
                        "enum declarations are only allowed at the top level".to_string(),
                        *pos,
                    ));
                }
                for var in variants.iter() {
                    if self
                        .globals
                        .insert(
                            var.0.clone(),
                            AstType::EnumVariant {
                                enum_type: name.clone(),
                                t: Box::new(var.1.clone()),
                            },
                        )
                        .is_some()
                    {
                        return Err(TypeError::Error(
                            format!("name {} already in use", var.0),
                            *pos,
                        ));
                    }
                }
                (AstType::Nil, false)
            }
            Ast::Variable { name, t, pos } => {
                let v = self.resolve_variable(name).ok_or(TypeError::Error(
                    format!("variable {} is not defined", name),
                    *pos,
                ))?;
                (
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
                            match global {
                                AstType::EnumVariant { enum_type, t } if *t == AstType::Nil => {
                                    AstType::Enum(enum_type)
                                }
                                _ => global.clone(),
                            }
                        }
                    },
                    false,
                )
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
                let expr_t = self.annotate_type(expr)?.0;
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
                (expr_t, false)
            }
            Ast::Switch {
                head,
                cases,
                default,
                pos,
            } => {
                let switch_t = self.annotate_type(head)?.0;
                if switch_t == AstType::Nil {
                    return Err(TypeError::Error(
                        "switch value cannot be nil".to_string(),
                        *pos,
                    ));
                }
                let mut diverges = if let Some(default) = default {
                    self.annotate_type(default)?.1
                } else {
                    false
                };
                for (case, body) in cases.iter_mut() {
                    let t = self.annotate_type(case)?.0;
                    if t != switch_t {
                        return Err(TypeError::Error(
                            format!(
                                "expected switch case to be type {:?}, but found type {:?}",
                                switch_t, t
                            ),
                            *pos,
                        ));
                    }
                    diverges = self.annotate_type(body)?.1 && diverges;
                }
                (AstType::Nil, diverges)
            }
            Ast::If {
                condition,
                body,
                else_body,
                pos,
            } => {
                if self.annotate_type(condition)?.0 != AstType::Bool {
                    return Err(TypeError::Error(
                        "condition must be a bool".to_string(),
                        *pos,
                    ));
                }
                let mut diverges = self.annotate_type(body)?.1;
                if let Some(else_body) = else_body {
                    diverges = self.annotate_type(else_body)?.1 && diverges;
                } else {
                    diverges = false;
                }
                (AstType::Nil, diverges)
            }
            Ast::While {
                condition,
                body,
                pos,
            } => {
                if self.annotate_type(condition)?.0 != AstType::Bool {
                    return Err(TypeError::Error(
                        "condition must be a bool".to_string(),
                        *pos,
                    ));
                }
                let diverges = self.annotate_type(body)?.1;
                (AstType::Nil, diverges)
            }
            Ast::ExprStatement { expr, t, .. } => {
                let expr_t = self.annotate_type(expr)?.0;
                t.replace(expr_t);
                (AstType::Nil, false)
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

                let diverges = result?.1;

                if let Some(t) = return_values.iter().filter(|t| *t != ret_t).next() {
                    return Err(TypeError::Error(
                        format!("return type {:?} doesn't match signature {:?}", t, ret_t),
                        *pos,
                    ));
                }
                if return_values.len() > 0 && !diverges {
                    return Err(TypeError::Error(
                        format!("all possible brances of function body needs to return",),
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

                (
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
                    },
                    false,
                )
            }
            Ast::Call {
                ident,
                args,
                args_width,
                call_t,
                pos,
            } => {
                let ident_t = self.annotate_type(ident)?.0;
                let mut args_t = vec![];
                for arg in args.iter_mut() {
                    args_t.push(self.annotate_type(arg)?.0);
                }
                let (func_args_t, ret_t) = match ident_t {
                    AstType::Closure(a, r) => {
                        call_t.replace(CallType::Closure);
                        (a, r)
                    }
                    AstType::Function(a, r) => {
                        call_t.replace(CallType::Function);
                        (a, r)
                    }
                    AstType::ExternalFunction(a, r) => {
                        call_t.replace(CallType::External);
                        (a, r)
                    }
                    AstType::EnumVariant { enum_type, t } => {
                        call_t.replace(CallType::Enum);
                        (vec![*t], Box::new(AstType::Enum(enum_type.clone())))
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
                args_width.replace(args_t.iter().map(|t| t.width()).sum::<usize>() as u8);
                (*ret_t.clone(), false)
            }
            Ast::Float(_, _) => (AstType::Float, false),
            Ast::Bool(_, _) => (AstType::Bool, false),
            Ast::String(_, _) => (AstType::String, false),
            Ast::Negate(a, pos) => {
                let t = self.annotate_type(a)?.0;
                if match t {
                    AstType::Float => false,
                    _ => true,
                } {
                    return Err(TypeError::Error(
                        format!("operation can't be preformed on type {:?}", t),
                        *pos,
                    ));
                }
                (t, false)
            }
            Ast::Not(a, pos) => {
                let t = self.annotate_type(a)?.0;
                if t != AstType::Bool {
                    return Err(TypeError::Error(
                        "not (!) operation requires a bool".to_string(),
                        *pos,
                    ));
                }
                (AstType::Bool, false)
            }
            Ast::Multiply(l, r, t, pos)
            | Ast::Divide(l, r, t, pos)
            | Ast::Add(l, r, t, pos)
            | Ast::Sub(l, r, t, pos) => {
                let t_l = self.annotate_type(l)?.0;
                let t_r = self.annotate_type(r)?.0;
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
                (t_r, false)
            }
            Ast::Equal(l, r, t, pos) | Ast::NotEqual(l, r, t, pos) => {
                let t_l = self.annotate_type(l)?.0;
                let t_r = self.annotate_type(r)?.0;
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
                    AstType::Enum(..) | AstType::Bool | AstType::Float => false,
                    _ => true,
                } {
                    return Err(TypeError::Error(
                        format!("operation can't be preformed on type {:?}", t_l),
                        *pos,
                    ));
                }
                t.replace(t_r);
                (AstType::Bool, false)
            }
            Ast::Greater(l, r, t, pos)
            | Ast::GreaterEqual(l, r, t, pos)
            | Ast::Lesser(l, r, t, pos)
            | Ast::LesserEqual(l, r, t, pos) => {
                let t_l = self.annotate_type(l)?.0;
                let t_r = self.annotate_type(r)?.0;
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
                (AstType::Bool, false)
            }
            Ast::And(l, r, pos) | Ast::Or(l, r, pos) => {
                let t_l = self.annotate_type(l)?.0;
                let t_r = self.annotate_type(r)?.0;
                if t_l != AstType::Bool || t_r != AstType::Bool {
                    return Err(TypeError::Error(
                        "operation requires both operands to be bool".to_string(),
                        *pos,
                    ));
                }
                (AstType::Bool, false)
            }
        };
        Ok((t, diverges))
    }
}
