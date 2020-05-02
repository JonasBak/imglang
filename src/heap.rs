use super::*;
use std::mem;

pub type HeapAdr = u32;

pub struct Closure {
    pub function: ChunkAdr,
    pub captured: Vec<HeapAdr>,
}

pub enum Obj {
    String(String),
    Float(f64),
    Bool(bool),

    Closure(Closure),

    Free,
}

pub struct Heap {
    objects: Vec<(u16, Obj)>,
    gaps: Vec<HeapAdr>,
}

impl Heap {
    pub fn new() -> Heap {
        Heap {
            objects: Vec::with_capacity(64),
            gaps: Vec::with_capacity(64),
        }
    }

    pub fn count_objects(&self) -> HeapAdr {
        self.objects
            .iter()
            .filter(|o| match o.1 {
                Obj::Free => false,
                _ => true,
            })
            .count() as HeapAdr
    }

    pub fn add_object(&mut self, obj: Obj) -> HeapAdr {
        if let Some(i) = self.gaps.pop() {
            #[cfg(feature = "debug_heap")]
            eprintln!("new object {} filled gap", i);

            self.objects[i as usize] = (1, obj);
            i
        } else {
            #[cfg(feature = "debug_heap")]
            eprintln!("new object {}", self.objects.len());

            self.objects.push((1, obj));
            self.objects.len() as HeapAdr - 1
        }
    }

    pub fn set_object(&mut self, i: HeapAdr, obj: Obj) {
        self.objects[i as usize].1 = obj;
    }

    pub fn increase_rc(&mut self, i: HeapAdr) {
        let mut entry = &mut self.objects[i as usize];
        entry.0 += 1;

        #[cfg(feature = "debug_heap")]
        println!("increased rc of {} to {}", i, entry.0);
    }

    pub fn decrease_rc(&mut self, i: HeapAdr) {
        let mut entry = &mut self.objects[i as usize];
        entry.0 -= 1;

        #[cfg(feature = "debug_heap")]
        println!("decreased rc of {} to {}", i, entry.0);

        if entry.0 == 0 {
            #[cfg(feature = "debug_heap")]
            println!("freed {}", i);

            match mem::replace(&mut self.objects[i as usize], (0, Obj::Free)) {
                (_, Obj::Closure(c)) => {
                    for var in c.captured.iter() {
                        self.decrease_rc(*var);
                    }
                }
                _ => {}
            }

            self.gaps.push(i);
        }
    }

    pub fn get_object_ref(&self, i: HeapAdr) -> Option<&Obj> {
        self.objects.get(i as usize).map(|obj| &obj.1)
    }

    pub fn get_string_ref(&self, i: HeapAdr) -> Option<&String> {
        self.get_object_ref(i).map(|obj| match obj {
            Obj::String(s) => s,
            _ => todo!(),
        })
    }

    pub fn get_closure_ref(&self, i: HeapAdr) -> Option<&Closure> {
        self.get_object_ref(i).map(|obj| match obj {
            Obj::Closure(c) => c,
            _ => todo!(),
        })
    }

    pub fn get_float(&self, i: HeapAdr) -> Option<f64> {
        self.get_object_ref(i).map(|obj| match obj {
            Obj::Float(f) => *f,
            _ => todo!(),
        })
    }

    pub fn get_bool(&self, i: HeapAdr) -> Option<bool> {
        self.get_object_ref(i).map(|obj| match obj {
            Obj::Bool(b) => *b,
            _ => todo!(),
        })
    }
}
