use super::*;
use std::collections::HashMap;
use std::mem;

type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
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

    Function(Vec<String>, Box<Ast>),
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

fn handle_binary_error(a: Value, b: Value) -> RuntimeError {
    if a.t() == b.t() {
        return RuntimeError::IllegalOperation(a, b);
    } else {
        return RuntimeError::MismatchedTypes(a, b);
    }
}

pub struct Scope {
    maps: Vec<HashMap<String, Value>>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            maps: vec![HashMap::new()],
        }
    }
    fn push(&mut self) {
        self.maps.push(HashMap::new());
    }
    fn pop(&mut self) {
        self.maps.pop();
    }
    pub fn get(&self, identifier: &String) -> Option<Value> {
        self.maps
            .iter()
            .rev()
            .find_map(|map| map.get(identifier))
            .map(|value| value.clone())
    }
    pub fn declare(&mut self, identifier: &String, value: Value) -> RuntimeResult<()> {
        self.maps
            .last_mut()
            .unwrap()
            .insert(identifier.clone(), value);
        Ok(())
    }
    fn assign(&mut self, identifier: &String, value: Value) -> RuntimeResult<()> {
        match self
            .maps
            .iter_mut()
            .rev()
            .find(|map| map.get(identifier).is_some())
        {
            Some(scope) => {
                scope.insert(identifier.clone(), value);
                Ok(())
            }
            None => Err(RuntimeError::UndefinedVariable(identifier.clone())),
        }
    }
}

impl Ast {
    pub fn eval(&self, scope: &mut Scope) -> RuntimeResult<Value> {
        let val = match self {
            Ast::Program(prog) => {
                for stat in prog.into_iter() {
                    println!("{:?}", stat.eval(scope)?);
                }
                Value::Nil
            }
            Ast::Decl(identifier, expr) => {
                let value = expr.eval(scope)?;
                scope.declare(&identifier, value)?;
                Value::Nil
            }
            Ast::Assign(identifier, expr) => {
                let value = expr.eval(scope)?;
                scope.assign(&identifier, value)?;
                Value::Nil
            }
            Ast::Print(expr) => {
                let value = expr.eval(scope)?;
                println!("< {:?}", value);
                Value::Nil
            }
            Ast::Block(exprs) => {
                scope.push();
                for stat in exprs.into_iter() {
                    if let Err(error) = stat.eval(scope) {
                        scope.pop();
                        return Err(error);
                    }
                }
                scope.pop();
                Value::Nil
            }
            Ast::Call(func, args) => {
                let mut args_values = vec![];
                for stat in args.into_iter() {
                    args_values.push(stat.eval(scope)?);
                }
                match scope.get(func) {
                    Some(Value::ExternFunction(f)) => f(args_values),
                    Some(Value::Function(args_names, block)) => {
                        if args_names.len() != args_values.len() {
                            return Err(RuntimeError::WrongArity(
                                args_names.len(),
                                args_values.len(),
                            ));
                        }
                        scope.push();
                        for (i, arg) in args_values.into_iter().enumerate() {
                            if let Err(error) = scope.declare(&args_names[i], arg) {
                                scope.pop();
                                return Err(error);
                            }
                        }
                        let v = block.eval(scope);
                        scope.pop();
                        v?
                    }
                    _ => return Err(RuntimeError::NotCallable(func.clone())),
                }
            }
            Ast::Function(func, args, block) => {
                scope.declare(func, Value::Function(args.clone(), block.clone()))?;
                Value::Nil
            }
            Ast::While { condition, body } => {
                while condition.eval(scope)?.truthy() {
                    body.eval(scope)?;
                }
                Value::Nil
            }
            Ast::Number(n) => Value::Number(*n),
            Ast::String(s) => Value::String(s.clone()),
            Ast::Bool(b) => Value::Bool(*b),
            Ast::Nil => Value::Nil,
            Ast::Identifier(identifier) => scope
                .get(&identifier)
                .ok_or(RuntimeError::UndefinedVariable(identifier.clone()))?,
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
            _ => panic!("not implemented"),
        };
        Ok(val)
    }
}
