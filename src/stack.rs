macro_rules! impl_from {
    ($t:ident, $variant:ident) => {
        impl From<Value> for $t {
            fn from(val: Value) -> Self {
                match val {
                    Value::$variant(v) => v,
                    _ => panic!(),
                }
            }
        }
        impl From<&Value> for $t {
            fn from(val: &Value) -> Self {
                match val {
                    Value::$variant(v) => *v,
                    _ => panic!(),
                }
            }
        }
        impl From<$t> for Value {
            fn from(v: $t) -> Self {
                Value::$variant(v)
            }
        }
    };
}

pub type StackAdr = u16;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Value {
    Float(f64),
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    EnumFloat(u8, f64),
}

#[derive(Debug)]
pub struct Stack(pub Vec<Value>);

impl Stack {
    pub fn new() -> Stack {
        Stack(Vec::new())
    }
    pub fn push<T: Into<Value>>(&mut self, v: T) -> StackAdr {
        self.0.push(v.into());
        self.0.len() as StackAdr - 1
    }
    pub fn pop(&mut self) -> Value {
        self.0.pop().unwrap()
    }
    pub fn get(&self, i: StackAdr) -> &Value {
        &self.0[i as usize]
    }
    pub fn set(&mut self, val: Value, i: StackAdr) {
        self.0[i as usize] = val;
    }
    pub fn len(&self) -> StackAdr {
        self.0.len() as StackAdr
    }
}

impl_from!(f64, Float);
impl_from!(bool, Bool);
impl_from!(u8, U8);
impl_from!(u16, U16);
impl_from!(u32, U32);
impl_from!(u64, U64);
