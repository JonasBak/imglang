use super::*;

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

pub type CodeAdr = u16;
pub type DataAdr = u16;

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
    pub fn len_code(&self) -> CodeAdr {
        self.code.len() as CodeAdr
    }

    pub fn push_op(&mut self, op: u8) -> CodeAdr {
        self.code.push_u8(op) as CodeAdr
    }
    pub fn push_op_u16(&mut self, op: u16) -> CodeAdr {
        self.code.push_u16(op) as CodeAdr
    }
    pub fn push_op_u32(&mut self, op: u32) -> CodeAdr {
        self.code.push_u32(op) as CodeAdr
    }

    pub fn get_op(&self, ip: CodeAdr) -> u8 {
        self.code.get_u8(ip as Adr)
    }
    pub fn get_op_u16(&self, ip: CodeAdr) -> u16 {
        self.code.get_u16(ip as Adr)
    }
    pub fn get_op_u32(&self, ip: CodeAdr) -> u32 {
        self.code.get_u32(ip as Adr)
    }

    pub fn backpatch_jump(&mut self, offset: CodeAdr) {
        let top = self.code.len() as CodeAdr;
        self.code.0[offset as usize..offset as usize + 2].copy_from_slice(&top.to_le_bytes());
    }

    pub fn add_const_f64(&mut self, data: f64) -> DataAdr {
        self.data.push_f64(data) as DataAdr
    }

    pub fn add_const_string(&mut self, data: &String) -> DataAdr {
        self.data.push_string(data) as DataAdr
    }

    pub fn get_const_f64(&self, i: DataAdr) -> f64 {
        self.data.get_f64(i as Adr)
    }
    pub fn get_const_u64(&self, i: DataAdr) -> u64 {
        self.data.get_u64(i as Adr)
    }
    pub fn get_const_string(&self, i: u16) -> String {
        self.data.get_string(i as Adr)
    }
}
