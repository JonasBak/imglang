use super::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

type RuntimeResult<T> = Result<T, RuntimeControl>;

#[derive(Debug, PartialEq)]
pub enum RuntimeControl {
    Return(Value),

    // Errors
    MismatchedTypes(Value, Value),
    IllegalOperation(Value, Value),
    UndefinedVariable(String),
    NotCallable(String),
    WrongArity(usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,

    Function(Vec<String>, Box<Ast>, Scope),
    ExternFunction(fn(Vec<Value>) -> Value),
}

impl Value {
    fn t(&self) -> mem::Discriminant<Value> {
        mem::discriminant(self)
    }
    fn truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => false,
        }
    }
}

fn handle_binary_error(a: Value, b: Value) -> RuntimeControl {
    if a.t() == b.t() {
        return RuntimeControl::IllegalOperation(a, b);
    } else {
        return RuntimeControl::MismatchedTypes(a, b);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    map: HashMap<String, Value>,
}
type Scope = Rc<RefCell<Environment>>;

impl Environment {
    pub fn new() -> Scope {
        Rc::new(RefCell::new(Environment {
            parent: None,
            map: HashMap::new(),
        }))
    }
}

pub fn get_value(scope: &Scope, identifier: &String) -> Option<Value> {
    let s = scope.borrow();
    if let Some(value) = s.map.get(identifier) {
        Some(value.clone())
    } else if let Some(s) = &s.parent {
        get_value(&s, identifier)
    } else {
        None
    }
}
fn child(scope: &Scope) -> Scope {
    Rc::new(RefCell::new(Environment {
        parent: Some(scope.clone()),
        map: HashMap::new(),
    }))
}
pub fn declare(scope: &Scope, identifier: &String, value: Value) -> RuntimeResult<()> {
    scope.borrow_mut().map.insert(identifier.clone(), value);
    Ok(())
}
fn assign(scope: &Scope, identifier: &String, value: Value) -> RuntimeResult<()> {
    let mut s = scope.borrow_mut();
    if s.map.get(identifier).is_some() {
        s.map.insert(identifier.clone(), value);
        return Ok(());
    }
    if let Some(s) = &s.parent {
        assign(&s, identifier, value)
    } else {
        Err(RuntimeControl::UndefinedVariable(identifier.clone()))
    }
}

impl Ast {
    pub fn eval(&self, scope: &Scope) -> RuntimeResult<Value> {
        let val = match self {
            Ast::Program(prog) => {
                for stat in prog.into_iter() {
                    println!("{:?}", stat.eval(scope)?);
                }
                Value::Nil
            }
            Ast::Decl(identifier, expr) => {
                let value = expr.eval(scope)?;
                declare(scope, &identifier, value)?;
                Value::Nil
            }
            Ast::Assign(identifier, expr) => {
                let value = expr.eval(scope)?;
                assign(scope, &identifier, value)?;
                Value::Nil
            }
            Ast::Print(expr) => {
                let value = expr.eval(scope)?;
                println!("< {:?}", value);
                Value::Nil
            }
            Ast::Block(exprs) => {
                let child_scope = child(scope);
                for stat in exprs.into_iter() {
                    stat.eval(&child_scope)?;
                }
                Value::Nil
            }
            Ast::Call(func, args) => {
                let mut args_values = vec![];
                for stat in args.into_iter() {
                    args_values.push(stat.eval(scope)?);
                }
                match get_value(scope, func) {
                    Some(Value::ExternFunction(f)) => f(args_values),
                    Some(Value::Function(args_names, block, func_scope)) => {
                        if args_names.len() != args_values.len() {
                            return Err(RuntimeControl::WrongArity(
                                args_names.len(),
                                args_values.len(),
                            ));
                        }
                        let child_scope = child(&func_scope);
                        for (i, arg) in args_values.into_iter().enumerate() {
                            declare(&child_scope, &args_names[i], arg)?;
                        }
                        match block.eval(&child_scope) {
                            Ok(v) => v,
                            Err(RuntimeControl::Return(v)) => v,
                            error @ _ => return error,
                        }
                    }
                    _ => return Err(RuntimeControl::NotCallable(func.clone())),
                }
            }
            Ast::Function(func, args, block) => {
                declare(
                    scope,
                    func,
                    Value::Function(args.clone(), block.clone(), scope.clone()),
                )?;
                Value::Nil
            }
            Ast::Return(expr) => {
                if let Some(expr) = expr {
                    return Err(RuntimeControl::Return(expr.eval(scope)?));
                }
                return Err(RuntimeControl::Return(Value::Nil));
            }
            Ast::While { condition, body } => {
                while condition.eval(scope)?.truthy() {
                    body.eval(scope)?;
                }
                Value::Nil
            }
            Ast::If {
                condition,
                if_true,
                if_false,
            } => {
                if condition.eval(scope)?.truthy() {
                    if_true.eval(scope)?;
                } else if let Some(if_false) = if_false {
                    if_false.eval(scope)?;
                }
                Value::Nil
            }
            Ast::Number(n) => Value::Number(*n),
            Ast::String(s) => Value::String(s.clone()),
            Ast::Bool(b) => Value::Bool(*b),
            Ast::Nil => Value::Nil,
            Ast::Identifier(identifier) => get_value(scope, &identifier)
                .ok_or(RuntimeControl::UndefinedVariable(identifier.clone()))?,
            Ast::Bang(a) => Value::Bool(!a.eval(scope)?.truthy()),
            Ast::Negated(a) => match a.eval(scope)? {
                Value::Number(n) => Value::Number(-n),
                _ => panic!("create error for both unary and binary illegal operation"),
            },
            Ast::Mul(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Div(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Add(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                (Value::String(a), Value::String(b)) => Value::String(a + &b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Sub(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::G(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a > b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::GE(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a >= b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::L(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a < b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::LE(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a <= b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Eq(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a == b),
                (Value::String(a), Value::String(b)) => Value::Bool(a == b),
                (Value::Bool(a), Value::Bool(b)) => Value::Bool(a == b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::NotEq(a, b) => match (a.eval(scope)?, b.eval(scope)?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a != b),
                (Value::String(a), Value::String(b)) => Value::Bool(a != b),
                (Value::Bool(a), Value::Bool(b)) => Value::Bool(a != b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::And(a, b) => Value::Bool(a.eval(scope)?.truthy() && b.eval(scope)?.truthy()),
            Ast::Or(a, b) => Value::Bool(a.eval(scope)?.truthy() || b.eval(scope)?.truthy()),
        };
        Ok(val)
    }
}
