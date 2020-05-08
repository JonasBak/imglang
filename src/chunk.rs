use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OpCode {
    Return { return_value: bool },
    ConstantF64 { data_i: DataAdr },
    ConstantString { data_i: DataAdr },
    NegateF64,
    MultiplyF64,
    DivideF64,
    AddF64,
    SubF64,
    True,
    False,
    PushU16 { data: u16 },
    Pop,
    Not,
    EqualU8,
    EqualU64,
    GreaterF64,
    LesserF64,
    PrintF64,
    PrintBool,
    PrintString,
    Variable { stack_i: StackAdr },
    Assign { stack_i: StackAdr },
    AssignObj { stack_i: StackAdr },
    AssignHeapFloat { stack_i: StackAdr },
    AssignHeapBool { stack_i: StackAdr },
    JumpIfFalse { ip: CodeAdr },
    Jump { ip: CodeAdr },
    Function { chunk_i: ChunkAdr },
    Call { args_width: u8 },
    CallClosure { args_width: u8 },
    CallExternal { args_width: u8 },
    IncreaseRC,
    DecreaseRC,
    HeapifyFloat,
    HeapifyBool,
    Closure { chunk_i: ChunkAdr, capture_len: u8 },
    HeapFloat { stack_i: StackAdr },
    HeapBool { stack_i: StackAdr },
}

pub type CodeAdr = u16;
pub type DataAdr = u16;

pub struct Chunk {
    code: Vec<OpCode>,
    data: ByteVector,
}
impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            data: ByteVector::new(),
        }
    }
    pub fn len_code(&self) -> CodeAdr {
        self.code.len() as CodeAdr
    }

    pub fn push_op(&mut self, op: OpCode) -> CodeAdr {
        self.code.push(op);
        (self.code.len() - 1) as CodeAdr
    }

    pub fn get_op(&self, ip: CodeAdr) -> OpCode {
        self.code[ip as usize]
    }

    pub fn backpatch_jump(&mut self, ip: CodeAdr) {
        let top = self.code.len() as CodeAdr;
        match &mut self.code[ip as usize] {
            OpCode::Jump { ref mut ip } => {
                *ip = top;
            }
            OpCode::JumpIfFalse { ref mut ip } => {
                *ip = top;
            }
            _ => panic!(),
        }
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
