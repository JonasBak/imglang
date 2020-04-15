use super::*;
use std::collections::HashMap;
use std::mem;

type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug)]
pub enum RuntimeError {
    MismatchedTypes(Value, Value),
    IllegalOperation(Value, Value),
    UndefinedVariable(String),
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl Value {
    fn t(&self) -> mem::Discriminant<Value> {
        mem::discriminant(self)
    }
    pub fn truthy(&self) -> bool {
        match self {
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

pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    map: HashMap<String, Value>,
}

impl Scope<'_> {
    pub fn new() -> Scope<'static> {
        Scope {
            parent: None,
            map: HashMap::new(),
        }
    }
    fn child(&mut self) -> Scope {
        Scope {
            parent: Some(self),
            map: HashMap::new(),
        }
    }
    fn get(&self, identifier: &String) -> Option<Value> {
        if let Some(val) = self.map.get(identifier) {
            return Some(val.clone());
        }
        self.parent
            .as_ref()
            .map(|scope| scope.get(identifier))
            .flatten()
    }
    fn set(&mut self, identifier: &String, value: Value) -> RuntimeResult<()> {
        self.map.insert(identifier.clone(), value);
        Ok(())
    }
    pub fn eval(&mut self, node: Ast) -> RuntimeResult<Value> {
        let val = match node {
            Ast::Program(prog) => {
                for stat in prog.into_iter() {
                    println!("{:?}", self.eval(*stat)?);
                }
                Value::Nil
            }
            Ast::Decl(identifier, expr) => {
                let value = self.eval(*expr)?;
                self.set(&identifier, value)?;
                Value::Nil
            }
            Ast::Print(expr) => {
                let value = self.eval(*expr)?;
                println!("< {:?}", value);
                Value::Nil
            }
            Ast::Block(exprs) => {
                let mut block_scope = self.child();
                for stat in exprs.into_iter() {
                    println!("{:?}", block_scope.eval(*stat)?);
                }
                Value::Nil
            }
            Ast::Number(n) => Value::Number(n),
            Ast::String(s) => Value::String(s),
            Ast::Bool(b) => Value::Bool(b),
            Ast::Nil => Value::Nil,
            Ast::Identifier(identifier) => self
                .get(&identifier)
                .ok_or(RuntimeError::UndefinedVariable(identifier))?,
            Ast::Bang(a) => Value::Bool(!self.eval(*a)?.truthy()),
            Ast::Negated(a) => match self.eval(*a)? {
                Value::Number(n) => Value::Number(-n),
                _ => panic!("create error for both unary and binary illegal operation"),
            },
            Ast::Mul(a, b) => match (self.eval(*a)?, self.eval(*b)?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Div(a, b) => match (self.eval(*a)?, self.eval(*b)?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Add(a, b) => match (self.eval(*a)?, self.eval(*b)?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                (Value::String(a), Value::String(b)) => Value::String(a + &b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Sub(a, b) => match (self.eval(*a)?, self.eval(*b)?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::G(a, b) => match (self.eval(*a)?, self.eval(*b)?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a > b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::GE(a, b) => match (self.eval(*a)?, self.eval(*b)?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a >= b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::L(a, b) => match (self.eval(*a)?, self.eval(*b)?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a < b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::LE(a, b) => match (self.eval(*a)?, self.eval(*b)?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a <= b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Eq(a, b) => match (self.eval(*a)?, self.eval(*b)?) {
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
