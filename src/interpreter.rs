use super::*;
use std::mem;

type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug)]
pub enum RuntimeError {
    MismatchedTypes(Value, Value),
    IllegalOperation(Value, Value),
}

#[derive(Debug)]
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

impl Ast {
    pub fn eval(self) -> RuntimeResult<Value> {
        let val = match self {
            Ast::Number(n) => Value::Number(n),
            Ast::String(s) => Value::String(s),
            Ast::False => Value::Bool(false),
            Ast::True => Value::Bool(true),
            Ast::Nil => Value::Nil,
            Ast::Bang(a) => Value::Bool(!a.eval()?.truthy()),
            Ast::Negated(a) => match a.eval()? {
                Value::Number(n) => Value::Number(-n),
                _ => panic!("create error for both unary and binary illegal operation"),
            },
            Ast::Mul(a, b) => match (a.eval()?, b.eval()?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Div(a, b) => match (a.eval()?, b.eval()?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Add(a, b) => match (a.eval()?, b.eval()?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                (Value::String(a), Value::String(b)) => Value::String(a + &b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Sub(a, b) => match (a.eval()?, b.eval()?) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::G(a, b) => match (a.eval()?, b.eval()?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a > b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::GE(a, b) => match (a.eval()?, b.eval()?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a >= b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::L(a, b) => match (a.eval()?, b.eval()?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a < b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::LE(a, b) => match (a.eval()?, b.eval()?) {
                (Value::Number(a), Value::Number(b)) => Value::Bool(a <= b),
                tup @ _ => return Err(handle_binary_error(tup.0, tup.1)),
            },
            Ast::Eq(a, b) => match (a.eval()?, b.eval()?) {
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
