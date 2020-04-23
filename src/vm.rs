use super::*;
use std::io::Write;

macro_rules! expr {
    ($e:expr) => {
        $e
    };
}
macro_rules! binary_op_f64{
    ($vm:ident, $op:tt) => {{
        let r = $vm.stack.pop_f64();
        let l = $vm.stack.pop_f64();
        $vm.stack.push_f64(expr!(l $op r));
    }}
}

struct CallFrame {
    parent_ip: usize,
    parent_chunk: usize,
    parent_frame_offset: usize,
    args_width: u8,
}

pub struct VM {
    stack: ByteVector,
    heap: ByteVector,
    chunks: Vec<Chunk>,
    call_frames: Vec<CallFrame>,
}

impl VM {
    pub fn new(chunks: Vec<Chunk>) -> VM {
        VM {
            stack: ByteVector::new(),
            heap: ByteVector::new(),
            chunks,
            call_frames: vec![],
        }
    }
    pub fn len_stack(&self) -> usize {
        self.stack.len()
    }
    pub fn run(&mut self, out: &mut dyn Write) {
        let mut ip = 0;
        let mut current_chunk = 0;
        let mut frame_offset = 0;
        loop {
            let chunk = &self.chunks[current_chunk];
            #[cfg(feature = "debug_runtime")]
            {
                eprint!(
                    "{:0>4}\tchunk:{: >3} stack:{: >4} nested:{: >2}\t",
                    ip,
                    current_chunk,
                    self.len_stack(),
                    self.call_frames.len(),
                );
                disassemble(&chunk, ip);
            }
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
                        .0
                        .copy_within(frame_offset + args_width as usize.., frame_offset);
                    self.stack.0.truncate(frame_offset + return_width);

                    ip = parent_ip;
                    current_chunk = parent_chunk;
                    frame_offset = parent_frame_offset;
                }
                OpCode::PrintF64 => {
                    let a = self.stack.pop_f64();
                    writeln!(out, "{}", a).unwrap();
                }
                OpCode::PrintBool => {
                    let a = self.stack.pop_bool();
                    writeln!(out, "{}", a).unwrap();
                }
                OpCode::PrintString => {
                    let a = self.stack.pop_u32();
                    let string = self.heap.get_string(a);
                    writeln!(out, "{}", string).unwrap();
                }
                OpCode::ConstantF64 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = chunk.get_const_f64(i);
                    self.stack.push_f64(v);
                }
                OpCode::ConstantString => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let string_data = chunk.get_const_string(i);
                    let string = self.heap.push_string(&string_data);
                    self.stack.push_u32(string);
                }
                OpCode::NegateF64 => {
                    let a = self.stack.pop_f64();
                    self.stack.push_f64(-a);
                }
                OpCode::Not => {
                    let a = self.stack.pop_bool();
                    self.stack.push_bool(!a);
                }
                OpCode::MultiplyF64 => binary_op_f64!(self, *),
                OpCode::DivideF64 => binary_op_f64!(self, /),
                OpCode::AddF64 => binary_op_f64!(self, +),
                OpCode::SubF64 => binary_op_f64!(self, -),
                OpCode::True => self.stack.push_bool(true),
                OpCode::False => self.stack.push_bool(false),
                OpCode::PushU16 => {
                    let data = chunk.get_op_u16(ip);
                    ip += 2;
                    self.stack.push_u16(data);
                }
                OpCode::PopU8 => {
                    self.stack.pop_u8();
                }
                OpCode::PopU16 => {
                    self.stack.pop_u16();
                }
                OpCode::PopU32 => {
                    self.stack.pop_u32();
                }
                OpCode::PopU64 => {
                    self.stack.pop_f64();
                }
                OpCode::EqualU8 => {
                    let r = self.stack.pop_u8();
                    let l = self.stack.pop_u8();
                    self.stack.push_bool(l == r);
                }
                OpCode::EqualU64 => {
                    let r = self.stack.pop_u64();
                    let l = self.stack.pop_u64();
                    self.stack.push_bool(l == r);
                }
                OpCode::GreaterF64 => {
                    let r = self.stack.pop_f64();
                    let l = self.stack.pop_f64();
                    self.stack.push_bool(l > r);
                }
                OpCode::LesserF64 => {
                    let r = self.stack.pop_f64();
                    let l = self.stack.pop_f64();
                    self.stack.push_bool(l < r);
                }
                OpCode::VariableU8 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = self.stack.get_u8(i as usize + frame_offset);
                    self.stack.push_u8(v);
                }
                OpCode::VariableU16 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = self.stack.get_u16(i as usize + frame_offset);
                    self.stack.push_u16(v);
                }
                OpCode::VariableU32 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = self.stack.get_u32(i as usize + frame_offset);
                    self.stack.push_u32(v);
                }
                OpCode::VariableU64 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = self.stack.get_u64(i as usize + frame_offset);
                    self.stack.push_u64(v);
                }
                OpCode::AssignU8 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 1;
                    let v = self.stack.get_u8(top);
                    self.stack.set_u8(v, i as usize + frame_offset);
                }
                OpCode::AssignU16 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 2;
                    let v = self.stack.get_u16(top);
                    self.stack.set_u16(v, i as usize + frame_offset);
                }
                OpCode::AssignU64 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 8;
                    let v = self.stack.get_u64(top);
                    self.stack.set_u64(v, i as usize + frame_offset);
                }
                OpCode::JumpIfFalse => {
                    let offset = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 1;
                    let v = self.stack.get_bool(top);
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
                    self.stack.push_u16(chunk);
                }
                OpCode::Call => {
                    let args_width = chunk.get_op(ip);
                    ip += 1;
                    let chunk_i = self.stack.pop_u16();

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
}
