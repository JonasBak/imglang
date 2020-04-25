use super::*;

pub type HeapAdr = u32;

pub enum Obj {
    String(String),

    Free,
}

pub struct Heap {
    objects: Vec<(u16, Obj)>,
    gaps: Vec<HeapAdr>,
}

impl Heap {
    pub fn new() -> Heap {
        Heap {
            objects: Vec::new(),
            gaps: Vec::new(),
        }
    }

    pub fn count_objects(&self) -> HeapAdr {
        self.objects
            .iter()
            .filter(|o| match o.1 {
                Obj::Free => true,
                _ => false,
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

            self.objects[i as usize] = (0, Obj::Free);
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
}
