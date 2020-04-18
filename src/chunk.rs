use std::convert::TryInto;

pub const OP_RETURN: u8 = 0;
pub const OP_CONSTANT_I64: u8 = 1;
pub const OP_CONSTANT_F64: u8 = 2;
pub const OP_ADD_I64: u8 = 3;
pub const OP_ADD_F64: u8 = 4;
pub const OP_SUB_I64: u8 = 5;
pub const OP_SUB_F64: u8 = 6;

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
pub fn add_op(chunk: &mut Chunk, op: [u8; 4]) -> usize {
    chunk.code.extend_from_slice(&op);
    chunk.code.len() - 4
}

// get opcode pointed at by ip
// called by vm
pub fn get_op(chunk: &Chunk, ip: usize) -> [u8; 4] {
    chunk.code[ip..ip + 4].try_into().unwrap()
}

// add value to constants array, returns index used
// to load value with constant opcodes
// called by compiler

pub fn add_f64(chunk: &mut Chunk, data: f64) -> u8 {
    chunk.data.extend_from_slice(&data.to_le_bytes());
    (chunk.data.len() - 8) as u8
}

// get value from index in constants array
// called by vm

pub fn get_f64(chunk: &Chunk, i: u8) -> f64 {
    let i = i as usize;
    f64::from_le_bytes(chunk.data[i..i + 8].try_into().unwrap())
}

pub fn get_u64(chunk: &Chunk, i: u8) -> u64 {
    let i = i as usize;
    u64::from_le_bytes(chunk.data[i..i + 8].try_into().unwrap())
}

// push/pop value to/from stack
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
