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

pub type StackAdr = u16;
pub type ChunkAdr = u16;

struct CallFrame {
    parent_ip: CodeAdr,
    parent_chunk: ChunkAdr,
    parent_frame_offset: StackAdr,
    args_width: StackAdr,
}

pub struct VM {
    stack: ByteVector,
    heap: Heap,
    chunks: Vec<Chunk>,
    call_frames: Vec<CallFrame>,
}

impl VM {
    pub fn new(chunks: Vec<Chunk>) -> VM {
        VM {
            stack: ByteVector::new(),
            heap: Heap::new(),
            chunks,
            call_frames: vec![],
        }
    }
    pub fn len_stack(&self) -> StackAdr {
        self.stack.len() as StackAdr
    }
    pub fn stack_ptr(&self) -> &ByteVector {
        &self.stack
    }
    pub fn heap_ptr(&self) -> &Heap {
        &self.heap
    }
    pub fn run(&mut self, out: &mut dyn Write) {
        let mut ip: CodeAdr = 0;
        let mut current_chunk: ChunkAdr = 0;
        let mut frame_offset: StackAdr = 0;
        loop {
            let chunk = &self.chunks[current_chunk as usize];
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
                    let return_width = chunk.get_op(ip) as StackAdr;
                    if self.call_frames.len() == 0 {
                        return;
                    }

                    let CallFrame {
                        parent_ip,
                        parent_chunk,
                        parent_frame_offset,
                        ..
                    } = self.call_frames.pop().unwrap();

                    let return_pos = self.len_stack() - return_width;
                    self.stack
                        .0
                        .copy_within(return_pos as usize.., frame_offset as usize);
                    self.stack
                        .0
                        .truncate((frame_offset + return_width) as usize);

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
                    let string = self.heap.get_string_ref(a).unwrap();
                    writeln!(out, "{}", string).unwrap();
                    self.heap.decrease_rc(a);
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
                    let string = self.heap.add_object(Obj::String(string_data));
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
                OpCode::True => {
                    self.stack.push_bool(true);
                }
                OpCode::False => {
                    self.stack.push_bool(false);
                }
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
                    let v = self.stack.get_u8((i + frame_offset) as Adr);
                    self.stack.push_u8(v);
                }
                OpCode::VariableU16 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = self.stack.get_u16((i + frame_offset) as Adr);
                    self.stack.push_u16(v);
                }
                OpCode::VariableU32 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = self.stack.get_u32((i + frame_offset) as Adr);
                    self.stack.push_u32(v);
                }
                OpCode::VariableU64 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let v = self.stack.get_u64((i + frame_offset) as Adr);
                    self.stack.push_u64(v);
                }
                OpCode::AssignU8 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 1;
                    let v = self.stack.get_u8(top as Adr);
                    self.stack.set_u8(v, (i + frame_offset) as Adr);
                }
                OpCode::AssignU16 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 2;
                    let v = self.stack.get_u16(top as Adr);
                    self.stack.set_u16(v, (i + frame_offset) as Adr);
                }
                OpCode::AssignU32 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 4;
                    let v = self.stack.get_u32(top as Adr);
                    self.stack.set_u32(v, (i + frame_offset) as Adr);
                }
                OpCode::AssignU64 => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 8;
                    let v = self.stack.get_u64(top as Adr);
                    self.stack.set_u64(v, (i + frame_offset) as Adr);
                }
                OpCode::AssignObj => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 4;
                    let new_val = self.stack.get_u32(top as Adr);
                    let old_val = self.stack.get_u32((i + frame_offset) as Adr);
                    self.heap.increase_rc(new_val);
                    self.heap.decrease_rc(old_val);
                    self.stack.set_u32(new_val, (i + frame_offset) as Adr);
                }
                OpCode::AssignHeapFloat => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let adr = self.stack.get_u32((i + frame_offset) as Adr);
                    let top = self.len_stack() - 8;
                    let v = self.stack.get_f64(top as Adr);
                    self.heap.set_object(adr, Obj::Float(v));
                }
                OpCode::JumpIfFalse => {
                    let offset = chunk.get_op_u16(ip);
                    ip += 2;
                    let top = self.len_stack() - 1;
                    let v = self.stack.get_bool(top as Adr);
                    if !v {
                        ip = offset;
                    }
                }
                OpCode::Jump => {
                    ip = chunk.get_op_u16(ip);
                }
                OpCode::Function => {
                    let chunk_i = chunk.get_op_u16(ip);
                    ip += 2;
                    self.stack.push_u16(chunk_i);
                }
                OpCode::Call => {
                    let args_width = chunk.get_op(ip) as StackAdr;
                    ip += 1;
                    let chunk_i = self.stack.pop_u16();

                    self.call_frames.push(CallFrame {
                        parent_ip: ip,
                        parent_chunk: current_chunk,
                        parent_frame_offset: frame_offset,
                        args_width,
                    });
                    current_chunk = chunk_i;
                    ip = 0;
                    frame_offset = self.len_stack() - args_width;
                }
                OpCode::CallClosure => {
                    let mut args_width = chunk.get_op(ip) as StackAdr;
                    ip += 1;
                    let closure_adr = self.stack.pop_u32();

                    let closure = self.heap.get_closure_ref(closure_adr).unwrap();
                    args_width += closure.captured.len() as StackAdr * 4;

                    for var in closure.captured.iter() {
                        self.stack.push_u32(*var);
                    }

                    self.call_frames.push(CallFrame {
                        parent_ip: ip,
                        parent_chunk: current_chunk,
                        parent_frame_offset: frame_offset,
                        args_width,
                    });
                    current_chunk = closure.function;
                    ip = 0;
                    frame_offset = self.len_stack() - args_width;

                    let captured = closure.captured.clone();
                    self.heap.decrease_rc(closure_adr);
                    for var in captured.iter() {
                        self.heap.increase_rc(*var);
                    }
                }
                OpCode::IncreaseRC => {
                    let top = self.len_stack() - 4;
                    let v = self.stack.get_u32(top as Adr);
                    self.heap.increase_rc(v);
                }
                OpCode::DecreaseRC => {
                    let top = self.len_stack() - 4;
                    let v = self.stack.get_u32(top as Adr);
                    self.heap.decrease_rc(v);
                }
                OpCode::HeapifyFloat => {
                    let n = self.stack.pop_f64();
                    let adr = self.heap.add_object(Obj::Float(n));
                    self.stack.push_u32(adr);
                }
                OpCode::Closure => {
                    let function = chunk.get_op_u16(ip);
                    let n_vars = chunk.get_op(ip + 2);
                    ip += 3;

                    let mut captured = Vec::new();
                    for _ in 0..n_vars {
                        captured.push(self.stack.pop_u32());
                    }
                    captured = captured.into_iter().rev().collect();

                    let adr = self
                        .heap
                        .add_object(Obj::Closure(Closure { function, captured }));
                    self.stack.push_u32(adr);
                }
                OpCode::HeapFloat => {
                    let i = chunk.get_op_u16(ip);
                    ip += 2;
                    let adr = self.stack.get_u32((i + frame_offset) as Adr);
                    let v = self.heap.get_float(adr);
                    self.stack.push_f64(v.unwrap());
                }
            }
        }
    }
}
