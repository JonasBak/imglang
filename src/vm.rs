use super::*;
use std::io::Write;

macro_rules! expr {
    ($e:expr) => {
        $e
    };
}
macro_rules! binary_op_f64{
    ($vm:ident, $op:tt) => {{
        let r: f64 = $vm.stack.pop();
        let l: f64 = $vm.stack.pop();
        $vm.stack.push(expr!(l $op r));
    }}
}

pub type ChunkAdr = u16;

struct CallFrame {
    parent_ip: CodeAdr,
    parent_chunk: ChunkAdr,
    parent_frame_offset: StackAdr,
    args_width: u8,
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
                        args_width,
                    } = self.call_frames.pop().unwrap();

                    if return_value {
                        self.stack.truncate(frame_offset + args_width as StackAdr);
                        let i = frame_offset as usize + args_width as usize;
                        self.stack
                            .0
                            .copy_within(i.., self.stack.1 - args_width as usize);
                    } else {
                        self.stack.truncate(frame_offset);
                    }

                    ip = parent_ip;
                    current_chunk = parent_chunk;
                    frame_offset = parent_frame_offset;
                }
                OpCode::PrintF64 => {
                    let a: f64 = self.stack.pop();
                    writeln!(out, "{:?}", a).unwrap();
                }
                OpCode::PrintBool => {
                    let a: bool = self.stack.pop();
                    writeln!(out, "{:?}", a).unwrap();
                }
                OpCode::PrintString => {
                    let a: u32 = self.stack.pop();
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
                    let a: f64 = self.stack.pop();
                    self.stack.push(-a);
                }
                OpCode::Not => {
                    let a: bool = self.stack.pop();
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
                OpCode::PushU8 { data } => {
                    self.stack.push(data);
                }
                OpCode::PushU16 { data } => {
                    self.stack.push(data);
                }
                OpCode::Pop { width } => {
                    let new_top = self.stack.1 - width as usize;
                    self.stack.truncate(new_top as StackAdr);
                }
                OpCode::Equal => {
                    todo!();
                    let r: f64 = self.stack.pop();
                    let l: f64 = self.stack.pop();
                    self.stack.push(l == r);
                }
                OpCode::GreaterF64 => {
                    let r: f64 = self.stack.pop();
                    let l: f64 = self.stack.pop();
                    self.stack.push(l > r);
                }
                OpCode::LesserF64 => {
                    let r: f64 = self.stack.pop();
                    let l: f64 = self.stack.pop();
                    self.stack.push(l < r);
                }
                OpCode::Variable { stack_i, width } => {
                    let top = self.stack.1;
                    let i = stack_i as usize + frame_offset as usize;
                    self.stack.reserved(width as usize);
                    self.stack.0.copy_within(i..i + width as usize, top);
                    self.stack.1 = top + width as usize;
                }
                OpCode::Assign { stack_i } => {
                    todo!();
                    let top = self.stack.len() - 1;
                    let v: u64 = self.stack.get(top);
                    self.stack.set(v, stack_i + frame_offset);
                }
                OpCode::AssignObj { stack_i } => {
                    let top = self.stack.len() - 1;
                    let new_val: HeapAdr = self.stack.get(top);
                    let old_val: HeapAdr = self.stack.get(stack_i + frame_offset);
                    self.heap.increase_rc(new_val);
                    self.heap.decrease_rc(old_val);
                    self.stack.set(new_val, stack_i + frame_offset);
                }
                OpCode::AssignHeapified { stack_i } => {
                    todo!();
                    // let adr: HeapAdr = self.stack.get(stack_i + frame_offset);
                    // let top = self.stack.len() - 1;
                    // let v = self.stack.get(top);
                    // self.heap.set_object(adr, Obj::Heapified(*v));
                }
                OpCode::JumpIfFalse { ip: jmp_ip } => {
                    let top = self.stack.len() - 1;
                    let v: bool = self.stack.get(top);
                    if !v {
                        ip = jmp_ip;
                    }
                }
                OpCode::Jump { ip: jmp_ip } => ip = jmp_ip,
                OpCode::SwitchJump { ip: jmp_ip } => {
                    todo!();
                    let compare: u64 = self.stack.pop();
                    let top = self.stack.len() - 1;
                    let switch_value: u64 = self.stack.get(top);
                    if compare == switch_value {
                        ip = jmp_ip;
                    }
                }
                OpCode::Function { chunk_i } => {
                    self.stack.push(chunk_i);
                }
                OpCode::Call { args_width } => {
                    let chunk_i: ChunkAdr = self.stack.pop();

                    self.call_frames.push(CallFrame {
                        parent_ip: ip,
                        parent_chunk: current_chunk,
                        parent_frame_offset: frame_offset,
                        args_width,
                    });
                    current_chunk = chunk_i;
                    ip = 0;
                    frame_offset = self.stack.len() - args_width as StackAdr;
                }
                OpCode::CallClosure { args_width } => {
                    let closure_adr: HeapAdr = self.stack.pop();

                    let closure = self.heap.get_closure_ref(closure_adr).unwrap();
                    let args_width = args_width + closure.captured.len() as u8;

                    for var in closure.captured.iter() {
                        self.stack.push(*var);
                    }

                    self.call_frames.push(CallFrame {
                        parent_ip: ip,
                        parent_chunk: current_chunk,
                        parent_frame_offset: frame_offset,
                        args_width,
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
                    let func_i: ExternalAdr = self.stack.pop();

                    let offset = self.stack.len() - args_width as StackAdr;
                    let ret = self
                        .externals
                        .unwrap()
                        .dispatch(func_i, &mut self.stack, offset);
                    self.stack.truncate(offset);
                    match ret {
                        ExternalArg::Float(f) => {
                            self.stack.push(f);
                        }
                        ExternalArg::Nil => {}
                        _ => todo!(),
                    };
                }
                OpCode::EnumVariant => {
                    todo!();
                    // let variant: u8 = self.stack.pop();
                    // let value = self.stack.pop();
                    // self.stack.push(match value {
                    //     Value::Float(value) => Value::EnumFloat(variant, value),
                    //     _ => todo!(),
                    // });
                }
                OpCode::IncreaseRC => {
                    let top = self.stack.len() - 1;
                    let v: HeapAdr = self.stack.get(top);
                    self.heap.increase_rc(v);
                }
                OpCode::DecreaseRC => {
                    let top = self.stack.len() - 1;
                    let v: HeapAdr = self.stack.get(top);
                    self.heap.decrease_rc(v);
                }
                OpCode::Heapify => {
                    todo!();
                    // let adr = self.heap.add_object(Obj::Heapified(self.stack.pop()));
                    // self.stack.push(adr);
                }
                OpCode::Closure {
                    chunk_i,
                    capture_len,
                } => {
                    let mut captured = Vec::new();
                    for _ in 0..capture_len {
                        captured.push(self.stack.pop());
                    }
                    captured = captured.into_iter().rev().collect();

                    let adr = self.heap.add_object(Obj::Closure(Closure {
                        function: chunk_i,
                        captured,
                    }));
                    self.stack.push(adr);
                }
                OpCode::FromHeap { stack_i } => {
                    todo!();
                    // let adr: HeapAdr = self.stack.get(stack_i + frame_offset);
                    // let v = self.heap.get_value(adr);
                    // self.stack.push(v.unwrap());
                }
            }
        }
    }
}
