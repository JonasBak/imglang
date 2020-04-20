use std::convert::TryInto;

macro_rules! generate_opcodes {
    ($($types:ident,)+) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum OpCode { $($types,)+ }
        const OPCODE_LOOKUP: &[OpCode] = &[
            $(OpCode::$types,)+
        ];
        impl From<u8> for OpCode {
	    fn from(op: u8) -> Self {
                OPCODE_LOOKUP[op as usize]
	    }
	}
    };
}

generate_opcodes!(
    Return,
    ConstantF64,
    NegateF64,
    MultiplyF64,
    DivideF64,
    AddF64,
    SubF64,
    Nil,
    True,
    False,
    PopU8,
    PopU64,
    Not,
    EqualU8,
    EqualU64,
    GreaterF64,
    LesserF64,
    PrintF64,
    PrintBool,
    VariableU8,
    VariableU64,
);

pub struct Chunk {
    code: Vec<u8>,
    stack: Vec<u8>,
    data: Vec<u8>,
}
impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            stack: Vec::new(),
            data: Vec::new(),
        }
    }
    pub fn len_code(&self) -> usize {
        self.code.len()
    }
    pub fn len_stack(&self) -> usize {
        self.stack.len()
    }
    pub fn len_data(&self) -> usize {
        self.data.len()
    }
}

// add opcode, returns index useful for jumps
// called by compiler
pub fn push_op(chunk: &mut Chunk, op: u8) -> usize {
    chunk.code.push(op);
    chunk.code.len() - 1
}
pub fn push_op_u16(chunk: &mut Chunk, op: u16) -> usize {
    chunk.code.extend_from_slice(&op.to_le_bytes());
    chunk.code.len() - 2
}
pub fn push_op_u32(chunk: &mut Chunk, op: u32) -> usize {
    chunk.code.extend_from_slice(&op.to_le_bytes());
    chunk.code.len() - 4
}

// get opcode pointed at by ip
// called by vm
pub fn get_op(chunk: &Chunk, ip: usize) -> u8 {
    chunk.code[ip]
}
pub fn get_op_u16(chunk: &Chunk, ip: usize) -> u16 {
    u16::from_le_bytes(chunk.code[ip..ip + 2].try_into().unwrap())
}
pub fn get_op_u32(chunk: &Chunk, ip: usize) -> u32 {
    u32::from_le_bytes(chunk.code[ip..ip + 4].try_into().unwrap())
}

// add value to constants array, returns index used
// to load value with constant opcodes
// called by compiler

pub fn add_const_f64(chunk: &mut Chunk, data: f64) -> u16 {
    chunk.data.extend_from_slice(&data.to_le_bytes());
    (chunk.data.len() - 8) as u16
}

// get value from index in constants array
// called by vm

pub fn get_f64(chunk: &Chunk, i: u16) -> f64 {
    let i = i as usize;
    f64::from_le_bytes(chunk.data[i..i + 8].try_into().unwrap())
}
pub fn get_u64(chunk: &Chunk, i: u16) -> u64 {
    let i = i as usize;
    u64::from_le_bytes(chunk.data[i..i + 8].try_into().unwrap())
}

// peek/push/pop value to/from stack
// called by vm

pub fn push_f64(chunk: &mut Chunk, data: f64) {
    chunk.stack.extend_from_slice(&data.to_le_bytes());
}
pub fn pop_f64(chunk: &mut Chunk) -> f64 {
    let l = chunk.stack.len() - 8;
    let v = f64::from_le_bytes(chunk.stack[l..].try_into().unwrap());
    chunk.stack.truncate(l);
    v
}

pub fn peek_u8(chunk: &mut Chunk, i: usize) -> u8 {
    chunk.stack[i]
}
pub fn push_u8(chunk: &mut Chunk, data: u8) {
    chunk.stack.push(data);
}
pub fn pop_u8(chunk: &mut Chunk) -> u8 {
    chunk.stack.pop().unwrap()
}

pub fn peek_u64(chunk: &mut Chunk, i: usize) -> u64 {
    u64::from_le_bytes(chunk.stack[i..i + 8].try_into().unwrap())
}
pub fn push_u64(chunk: &mut Chunk, data: u64) {
    chunk.stack.extend_from_slice(&data.to_le_bytes());
}
pub fn pop_u64(chunk: &mut Chunk) -> u64 {
    let l = chunk.stack.len() - 8;
    let v = u64::from_le_bytes(chunk.stack[l..].try_into().unwrap());
    chunk.stack.truncate(l);
    v
}

pub fn push_bool(chunk: &mut Chunk, data: bool) {
    chunk.stack.push(data as u8);
}
pub fn pop_bool(chunk: &mut Chunk) -> bool {
    chunk.stack.pop().unwrap() != 0
}

pub fn push_nil(chunk: &mut Chunk) {
    chunk.stack.push(0);
}
pub fn pop_nil(chunk: &mut Chunk) {
    chunk.stack.pop().unwrap();
}
