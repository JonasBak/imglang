use super::*;
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
    ConstantString,
    NegateF64,
    MultiplyF64,
    DivideF64,
    AddF64,
    SubF64,
    True,
    False,
    PushU16,
    PopU8,
    PopU16,
    PopU32,
    PopU64,
    Not,
    EqualU8,
    EqualU64,
    GreaterF64,
    LesserF64,
    PrintF64,
    PrintBool,
    PrintString,
    VariableU8,
    VariableU16,
    VariableU32,
    VariableU64,
    AssignU8,
    AssignU16,
    AssignU64,
    JumpIfFalse,
    Jump,
    Function,
    Call,
);

pub struct Chunk {
    code: ByteVector,
    data: ByteVector,
}
impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: ByteVector::new(),
            data: ByteVector::new(),
        }
    }
    pub fn len_code(&self) -> usize {
        self.code.len()
    }
    pub fn len_data(&self) -> usize {
        self.data.len()
    }

    pub fn push_op(&mut self, op: u8) -> usize {
        self.code.push_u8(op);
        self.code.len() - 1
    }
    pub fn push_op_u16(&mut self, op: u16) -> usize {
        self.code.push_u16(op);
        self.code.len() - 2
    }
    pub fn push_op_u32(&mut self, op: u32) -> usize {
        self.code.push_u32(op);
        self.code.len() - 4
    }

    pub fn get_op(&self, ip: usize) -> u8 {
        self.code.get_u8(ip)
    }
    pub fn get_op_u16(&self, ip: usize) -> u16 {
        self.code.get_u16(ip)
    }
    pub fn get_op_u32(&self, ip: usize) -> u32 {
        self.code.get_u32(ip)
    }

    pub fn backpatch_jump(&mut self, offset: usize) {
        let top = self.code.len() as u16;
        self.code.0[offset..offset + 2].copy_from_slice(&top.to_le_bytes());
    }

    pub fn add_const_f64(&mut self, data: f64) -> u16 {
        self.data.push_f64(data);
        (self.data.len() - 8) as u16
    }

    pub fn add_const_string(&mut self, data: &String) -> u16 {
        self.data.push_string(data) as u16
    }

    pub fn get_const_f64(&self, i: u16) -> f64 {
        self.data.get_f64(i as usize)
    }
    pub fn get_const_u64(&self, i: u16) -> u64 {
        self.data.get_u64(i as usize)
    }
    pub fn get_const_string(&self, i: u16) -> String {
        self.data.get_string(i as u32)
    }
}
