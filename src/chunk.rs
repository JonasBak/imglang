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
    True,
    False,
    PushU16,
    PopU8,
    PopU16,
    PopU64,
    Not,
    EqualU8,
    EqualU64,
    GreaterF64,
    LesserF64,
    PrintF64,
    PrintBool,
    VariableU8,
    VariableU16,
    VariableU64,
    AssignU8,
    AssignU16,
    AssignU64,
    JumpIfFalse,
    Jump,
    Function,
    Call,
);

#[derive(Debug)]
pub struct Chunk {
    code: Vec<u8>,
    data: Vec<u8>,
}
impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            data: Vec::new(),
        }
    }
    pub fn len_code(&self) -> usize {
        self.code.len()
    }
    pub fn len_data(&self) -> usize {
        self.data.len()
    }

    pub fn push_op(&mut self, op: u8) -> usize {
        self.code.push(op);
        self.code.len() - 1
    }
    pub fn push_op_u16(&mut self, op: u16) -> usize {
        self.code.extend_from_slice(&op.to_le_bytes());
        self.code.len() - 2
    }
    pub fn push_op_u32(&mut self, op: u32) -> usize {
        self.code.extend_from_slice(&op.to_le_bytes());
        self.code.len() - 4
    }

    pub fn get_op(&self, ip: usize) -> u8 {
        self.code[ip]
    }
    pub fn get_op_u16(&self, ip: usize) -> u16 {
        u16::from_le_bytes(self.code[ip..ip + 2].try_into().unwrap())
    }
    pub fn get_op_u32(&self, ip: usize) -> u32 {
        u32::from_le_bytes(self.code[ip..ip + 4].try_into().unwrap())
    }

    pub fn backpatch_jump(&mut self, offset: usize) {
        let top = self.code.len() as u16;
        self.code[offset..offset + 2].copy_from_slice(&top.to_le_bytes());
    }

    pub fn add_const_f64(&mut self, data: f64) -> u16 {
        self.data.extend_from_slice(&data.to_le_bytes());
        (self.data.len() - 8) as u16
    }

    pub fn get_const_f64(&self, i: u16) -> f64 {
        let i = i as usize;
        f64::from_le_bytes(self.data[i..i + 8].try_into().unwrap())
    }
    pub fn get_const_u64(&self, i: u16) -> u64 {
        let i = i as usize;
        u64::from_le_bytes(self.data[i..i + 8].try_into().unwrap())
    }
}
