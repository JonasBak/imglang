use super::*;
use std::convert::TryInto;

macro_rules! expr {
    ($e:expr) => {
        $e
    };
}
macro_rules! binary_op_f64{
    ($vm:ident, $op:tt) => {{
        let r = $vm.pop_f64();
        let l = $vm.pop_f64();
        $vm.push_f64(expr!(l $op r));
    }}
}

struct CallFrame {
    parent_ip: usize,
    parent_chunk: usize,
    parent_frame_offset: usize,
    args_width: u8,
}

pub struct VM {
    stack: Vec<u8>,
    chunks: Vec<Chunk>,
    call_frames: Vec<CallFrame>,
}

impl VM {
    pub fn new(chunks: Vec<Chunk>) -> VM {
        VM {
            stack: vec![],
            chunks,
            call_frames: vec![],
        }
    }
    pub fn len_stack(&self) -> usize {
        self.stack.len()
    }
    pub fn run(&mut self) {
        let mut ip = 0;
        let mut current_chunk = 0;
        let mut frame_offset = 0;
        loop {
            let chunk = &self.chunks[current_chunk];
            // print!(
            //     "{:0>4}\tchunk:{: >3} stack:{: >4} nested:{: >2}\t",
            //     ip,
            //     current_chunk,
            //     self.len_stack(),
            //     self.call_frames.len(),
            // );
            // disassemble(&chunk, ip);
            ip = ip + 1;
            match OpCode::from(chunk.get_op(ip - 1)) {
                OpCode::Return => {
                    if self.call_frames.len() == 0 {
                        return;
                    }

                    let CallFrame {
                        parent_ip,
                        parent_chunk,
                        parent_frame_offset,
                        args_width,
                    } = self.call_frames.pop().unwrap();

                    let return_width = self.stack.len() - frame_offset - args_width as usize;
                    self.stack
                        .copy_within(frame_offset + args_width as usize.., frame_offset);
                    self.stack.truncate(frame_offset + return_width);

                    ip = parent_ip;
                    current_chunk = parent_chunk;
                    frame_offset = parent_frame_offset;
                }
                OpCode::PrintF64 => {
                    let a = self.pop_f64();
                    println!("< {}", a);
                }
                OpCode::PrintBool => {
                    let a = self.pop_bool();
                    println!("< {}", a);
                }
                OpCode::ConstantF64 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = chunk.get_const_f64(i);
                    self.push_f64(v);
                }
                OpCode::NegateF64 => {
                    let a = self.pop_f64();
                    self.push_f64(-a);
                }
                OpCode::Not => {
                    let a = self.pop_bool();
                    self.push_bool(!a);
                }
                OpCode::MultiplyF64 => binary_op_f64!(self, *),
                OpCode::DivideF64 => binary_op_f64!(self, /),
                OpCode::AddF64 => binary_op_f64!(self, +),
                OpCode::SubF64 => binary_op_f64!(self, -),
                OpCode::Nil => self.push_nil(),
                OpCode::True => self.push_bool(true),
                OpCode::False => self.push_bool(false),
                OpCode::PushU16 => {
                    let data = chunk.get_op_u16(ip);
                    ip += 2;
                    self.push_u16(data);
                }
                OpCode::PopU8 => {
                    self.pop_u8();
                }
                OpCode::PopU16 => {
                    self.pop_u16();
                }
                OpCode::PopU64 => {
                    self.pop_f64();
                }
                OpCode::EqualU8 => {
                    let r = self.pop_u8();
                    let l = self.pop_u8();
                    self.push_bool(l == r);
                }
                OpCode::EqualU64 => {
                    let r = self.pop_u64();
                    let l = self.pop_u64();
                    self.push_bool(l == r);
                }
                OpCode::GreaterF64 => {
                    let r = self.pop_f64();
                    let l = self.pop_f64();
                    self.push_bool(l > r);
                }
                OpCode::LesserF64 => {
                    let r = self.pop_f64();
                    let l = self.pop_f64();
                    self.push_bool(l < r);
                }
                OpCode::VariableU8 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = self.peek_u8(i as usize + frame_offset);
                    self.push_u8(v);
                }
                OpCode::VariableU16 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = self.peek_u16(i as usize + frame_offset);
                    self.push_u16(v);
                }
                OpCode::VariableU64 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = self.peek_u64(i as usize + frame_offset);
                    self.push_u64(v);
                }
                OpCode::AssignU8 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 1;
                    let v = self.peek_u8(top);
                    self.set_u8(v, i as usize + frame_offset);
                }
                OpCode::AssignU64 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 8;
                    let v = self.peek_u64(top);
                    self.set_u64(v, i as usize + frame_offset);
                }
                OpCode::JumpIfFalse => {
                    let offset = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 1;
                    let v = self.peek_bool(top);
                    if !v {
                        ip = offset as usize;
                    }
                }
                OpCode::Jump => {
                    ip = chunk.get_op_u16(ip) as usize;
                }
                OpCode::Function => {
                    let chunk = chunk.get_op_u16(ip);
                    ip += 2;
                    self.push_u16(chunk);
                }
                OpCode::Call => {
                    let args_width = chunk.get_op(ip);
                    ip += 1;
                    let chunk_i = self.pop_u16();

                    self.call_frames.push(CallFrame {
                        parent_ip: ip,
                        parent_chunk: current_chunk,
                        parent_frame_offset: frame_offset,
                        args_width,
                    });
                    current_chunk = chunk_i as usize;
                    ip = 0;
                    frame_offset = self.len_stack() - args_width as usize;
                }
            }
        }
    }
    pub fn push_f64(&mut self, data: f64) {
        self.stack.extend_from_slice(&data.to_le_bytes());
    }
    pub fn pop_f64(&mut self) -> f64 {
        let l = self.stack.len() - 8;
        let v = f64::from_le_bytes(self.stack[l..].try_into().unwrap());
        self.stack.truncate(l);
        v
    }

    pub fn peek_u8(&mut self, i: usize) -> u8 {
        self.stack[i]
    }
    pub fn set_u8(&mut self, data: u8, i: usize) {
        self.stack[i] = data;
    }
    pub fn push_u8(&mut self, data: u8) {
        self.stack.push(data);
    }
    pub fn pop_u8(&mut self) -> u8 {
        self.stack.pop().unwrap()
    }

    pub fn peek_u16(&mut self, i: usize) -> u16 {
        u16::from_le_bytes(self.stack[i..i + 2].try_into().unwrap())
    }
    pub fn push_u16(&mut self, data: u16) {
        self.stack.extend_from_slice(&data.to_le_bytes());
    }
    pub fn pop_u16(&mut self) -> u16 {
        let l = self.stack.len() - 2;
        let v = u16::from_le_bytes(self.stack[l..].try_into().unwrap());
        self.stack.truncate(l);
        v
    }

    pub fn peek_u64(&mut self, i: usize) -> u64 {
        u64::from_le_bytes(self.stack[i..i + 8].try_into().unwrap())
    }
    pub fn set_u64(&mut self, data: u64, i: usize) {
        self.stack[i..i + 8].copy_from_slice(&data.to_le_bytes());
    }
    pub fn push_u64(&mut self, data: u64) {
        self.stack.extend_from_slice(&data.to_le_bytes());
    }
    pub fn pop_u64(&mut self) -> u64 {
        let l = self.stack.len() - 8;
        let v = u64::from_le_bytes(self.stack[l..].try_into().unwrap());
        self.stack.truncate(l);
        v
    }

    pub fn peek_bool(&mut self, i: usize) -> bool {
        self.stack[i] != 0
    }
    pub fn push_bool(&mut self, data: bool) {
        self.stack.push(data as u8);
    }
    pub fn pop_bool(&mut self) -> bool {
        self.stack.pop().unwrap() != 0
    }

    pub fn push_nil(&mut self) {
        self.stack.push(0);
    }
    pub fn pop_nil(&mut self) {
        self.stack.pop().unwrap();
    }
}
