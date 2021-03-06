use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OpCode {
    Return { width: u8 },
    ConstantF64 { data_i: DataAdr },
    ConstantString { data_i: DataAdr },
    NegateF64,
    MultiplyF64,
    DivideF64,
    AddF64,
    SubF64,
    True,
    False,
    PushU8 { data: u8 },
    PushU16 { data: u16 },
    PushPadding { width: u8 },
    Pop { width: u8 },
    Not,
    Equal { width: u8 },
    GreaterF64,
    LesserF64,
    PrintF64,
    PrintBool,
    PrintString,
    Variable { stack_i: StackAdr, width: u8 },
    Assign { stack_i: StackAdr, width: u8 },
    AssignObj { stack_i: StackAdr },
    AssignHeapified { stack_i: StackAdr },
    JumpIfFalse { ip: CodeAdr },
    Jump { ip: CodeAdr },
    SwitchJump { ip: CodeAdr },
    Function { chunk_i: ChunkAdr },
    Call { args_width: u8 },
    CallClosure { args_width: u8 },
    CallExternal { args_width: u8 },
    IncreaseRC,
    DecreaseRC,
    Heapify { width: u8 },
    Closure { chunk_i: ChunkAdr, capture_len: u8 },
    FromHeap { stack_i: StackAdr },
}

pub type CodeAdr = u16;
pub type DataAdr = u16;

pub struct Data {
    floats: Vec<f64>,
    strings: Vec<String>,
}

impl Data {
    pub fn new() -> Data {
        Data {
            floats: Vec::new(),
            strings: Vec::new(),
        }
    }
}

pub struct Chunk {
    code: Vec<OpCode>,
    data: Data,
}
impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            data: Data::new(),
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
            OpCode::SwitchJump { ref mut ip } => {
                *ip = top;
            }
            _ => panic!(),
        }
    }

    pub fn add_const_f64(&mut self, data: f64) -> DataAdr {
        self.data.floats.push(data);
        self.data.floats.len() as DataAdr - 1
    }
    pub fn add_const_string(&mut self, data: &String) -> DataAdr {
        self.data.strings.push(data.clone());
        self.data.strings.len() as DataAdr - 1
    }

    pub fn get_const_f64(&self, i: DataAdr) -> f64 {
        self.data.floats[i as usize]
    }
    pub fn get_const_string(&self, i: u16) -> String {
        self.data.strings[i as usize].clone()
    }
}
