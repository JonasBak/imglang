use super::*;
use std::io::Write;

macro_rules! expr {
    ($e:expr) => {
        $e
    };
}
macro_rules! binary_op_f64{
    ($vm:ident, $op:tt) => {{
        let r: f64 = $vm.stack.pop().into();
        let l: f64 = $vm.stack.pop().into();
        $vm.stack.push(expr!(l $op r));
    }}
}

pub type ChunkAdr = u16;

struct CallFrame {
    parent_ip: CodeAdr,
    parent_chunk: ChunkAdr,
    parent_frame_offset: StackAdr,
}

pub struct VM<'a> {
    stack: Stack,
    heap: Heap,
    chunks: Vec<Chunk>,
    call_frames: Vec<CallFrame>,
    externals: Option<&'a Externals>,
}

impl<'a> VM<'a> {
    pub fn new(chunks: Vec<Chunk>, externals: Option<&'a Externals>) -> VM {
        VM {
            stack: Stack::new(),
            heap: Heap::new(),
            chunks,
            call_frames: vec![],
            externals,
        }
    }
    #[allow(dead_code)]
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
                eprintln!(
                    "{:0>4}\tchunk:{: >3} stack:{: >4} nested:{: >2}\t{:?}",
                    ip,
                    current_chunk,
                    self.stack.len(),
                    self.call_frames.len(),
                    chunk.get_op(ip)
                );
            }
            ip = ip + 1;
            match chunk.get_op(ip - 1) {
                OpCode::Return { return_value } => {
                    if self.call_frames.len() == 0 {
                        return;
                    }

                    let CallFrame {
                        parent_ip,
                        parent_chunk,
                        parent_frame_offset,
                        ..
                    } = self.call_frames.pop().unwrap();

                    if return_value {
                        let ret_val = self.stack.pop();
                        self.stack.0.truncate(frame_offset as usize);
                        self.stack.push(ret_val);
                    } else {
                        self.stack.0.truncate(frame_offset as usize);
                    }

                    ip = parent_ip;
                    current_chunk = parent_chunk;
                    frame_offset = parent_frame_offset;
                }
                OpCode::PrintF64 => {
                    let a: f64 = self.stack.pop().into();
                    writeln!(out, "{:?}", a).unwrap();
                }
                OpCode::PrintBool => {
                    let a: bool = self.stack.pop().into();
                    writeln!(out, "{:?}", a).unwrap();
                }
                OpCode::PrintString => {
                    let a = self.stack.pop().into();
                    let string = self.heap.get_string_ref(a).unwrap();
                    writeln!(out, "{}", string).unwrap();
                    self.heap.decrease_rc(a);
                }
                OpCode::ConstantF64 { data_i } => {
                    let v = chunk.get_const_f64(data_i);
                    self.stack.push(v);
                }
                OpCode::ConstantString { data_i } => {
                    let string_data = chunk.get_const_string(data_i);
                    let adr = self.heap.add_object(Obj::String(string_data));
                    self.stack.push(adr);
                }
                OpCode::NegateF64 => {
                    let a: f64 = self.stack.pop().into();
                    self.stack.push(-a);
                }
                OpCode::Not => {
                    let a: bool = self.stack.pop().into();
                    self.stack.push(!a);
                }
                OpCode::MultiplyF64 => binary_op_f64!(self, *),
                OpCode::DivideF64 => binary_op_f64!(self, /),
                OpCode::AddF64 => binary_op_f64!(self, +),
                OpCode::SubF64 => binary_op_f64!(self, -),
                OpCode::True => {
                    self.stack.push(true);
                }
                OpCode::False => {
                    self.stack.push(false);
                }
                OpCode::PushU16 { data } => {
                    self.stack.push(data);
                }
                OpCode::Pop => {
                    self.stack.0.pop();
                }
                OpCode::EqualU8 => {
                    let r = self.stack.pop();
                    let l = self.stack.pop();
                    self.stack.push(l == r);
                }
                OpCode::EqualU64 => {
                    let r = self.stack.pop();
                    let l = self.stack.pop();
                    self.stack.push(l == r);
                }
                OpCode::GreaterF64 => {
                    let r: f64 = self.stack.pop().into();
                    let l: f64 = self.stack.pop().into();
                    self.stack.push(l > r);
                }
                OpCode::LesserF64 => {
                    let r: f64 = self.stack.pop().into();
                    let l: f64 = self.stack.pop().into();
                    self.stack.push(l < r);
                }
                OpCode::Variable { stack_i } => {
                    let v = *self.stack.get(stack_i + frame_offset);
                    self.stack.push(v);
                }
                OpCode::Assign { stack_i } => {
                    let top = self.stack.len() - 1;
                    let v = *self.stack.get(top);
                    self.stack.set(v, stack_i + frame_offset);
                }
                OpCode::AssignObj { stack_i } => {
                    let top = self.stack.len() - 1;
                    let new_val: HeapAdr = self.stack.get(top).into();
                    let old_val: HeapAdr = self.stack.get(stack_i + frame_offset).into();
                    self.heap.increase_rc(new_val);
                    self.heap.decrease_rc(old_val);
                    self.stack.set(Value::U32(new_val), stack_i + frame_offset);
                }
                OpCode::AssignHeapFloat { stack_i } => {
                    let adr: HeapAdr = self.stack.get(stack_i + frame_offset).into();
                    let top = self.stack.len() - 1;
                    let v = self.stack.get(top).into();
                    self.heap.set_object(adr, Obj::Float(v));
                }
                OpCode::AssignHeapBool { stack_i } => {
                    let adr: HeapAdr = self.stack.get(stack_i + frame_offset).into();
                    let top = self.stack.len() - 1;
                    let v = self.stack.get(top).into();
                    self.heap.set_object(adr, Obj::Bool(v));
                }
                OpCode::JumpIfFalse { ip: jmp_ip } => {
                    let top = self.stack.len() - 1;
                    let v: bool = self.stack.get(top).into();
                    if !v {
                        ip = jmp_ip;
                    }
                }
                OpCode::Jump { ip: jmp_ip } => ip = jmp_ip,
                OpCode::Function { chunk_i } => {
                    self.stack.push(chunk_i);
                }
                OpCode::Call { args_width } => {
                    let chunk_i: ChunkAdr = self.stack.pop().into();

                    self.call_frames.push(CallFrame {
                        parent_ip: ip,
                        parent_chunk: current_chunk,
                        parent_frame_offset: frame_offset,
                    });
                    current_chunk = chunk_i;
                    ip = 0;
                    frame_offset = self.stack.len() - args_width as StackAdr;
                }
                OpCode::CallClosure { args_width } => {
                    let closure_adr: HeapAdr = self.stack.pop().into();

                    let closure = self.heap.get_closure_ref(closure_adr).unwrap();
                    let args_width = args_width + closure.captured.len() as u8;

                    for var in closure.captured.iter() {
                        self.stack.push(*var);
                    }

                    self.call_frames.push(CallFrame {
                        parent_ip: ip,
                        parent_chunk: current_chunk,
                        parent_frame_offset: frame_offset,
                    });
                    current_chunk = closure.function;
                    ip = 0;
                    frame_offset = self.stack.len() - args_width as StackAdr;

                    let captured = closure.captured.clone();
                    self.heap.decrease_rc(closure_adr);
                    for var in captured.iter() {
                        self.heap.increase_rc(*var);
                    }
                }
                OpCode::CallExternal { args_width } => {
                    let func_i: ExternalAdr = self.stack.pop().into();

                    let offset = self.stack.len() - args_width as StackAdr;
                    let ret = self
                        .externals
                        .unwrap()
                        .dispatch(func_i, &mut self.stack, offset);
                    self.stack.0.truncate(offset as usize);
                    match ret {
                        ExternalArg::Float(f) => {
                            self.stack.push(f);
                        }
                        ExternalArg::Nil => {}
                        _ => todo!(),
                    };
                }
                OpCode::IncreaseRC => {
                    let top = self.stack.len() - 1;
                    let v: HeapAdr = self.stack.get(top).into();
                    self.heap.increase_rc(v);
                }
                OpCode::DecreaseRC => {
                    let top = self.stack.len() - 1;
                    let v: HeapAdr = self.stack.get(top).into();
                    self.heap.decrease_rc(v);
                }
                OpCode::HeapifyFloat => {
                    let n = self.stack.pop().into();
                    let adr = self.heap.add_object(Obj::Float(n));
                    self.stack.push(adr);
                }
                OpCode::HeapifyBool => {
                    let b = self.stack.pop().into();
                    let adr = self.heap.add_object(Obj::Bool(b));
                    self.stack.push(adr);
                }
                OpCode::Closure {
                    chunk_i,
                    capture_len,
                } => {
                    let mut captured = Vec::new();
                    for _ in 0..capture_len {
                        captured.push(self.stack.pop().into());
                    }
                    captured = captured.into_iter().rev().collect();

                    let adr = self.heap.add_object(Obj::Closure(Closure {
                        function: chunk_i,
                        captured,
                    }));
                    self.stack.push(adr);
                }
                OpCode::HeapFloat { stack_i } => {
                    let adr: HeapAdr = self.stack.get(stack_i + frame_offset).into();
                    let v = self.heap.get_float(adr);
                    self.stack.push(v.unwrap());
                }
                OpCode::HeapBool { stack_i } => {
                    let adr: HeapAdr = self.stack.get(stack_i + frame_offset).into();
                    let v = self.heap.get_bool(adr);
                    self.stack.push(v.unwrap());
                }
            }
        }
    }
}
