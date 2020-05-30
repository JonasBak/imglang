use std::mem;

pub type StackAdr = u16;

#[derive(Debug)]
pub struct Stack(pub Box<[u8]>, pub usize);

impl Stack {
    pub fn new() -> Stack {
        Stack(Box::new([0; 1 << 8]), 0)
    }
    pub fn reserved(&mut self, width: usize) {
        if self.1 + width >= self.0.len() {
            let mut new = vec![0; self.0.len() << 1];
            new[..self.0.len()].copy_from_slice(&self.0[..]);
            self.0 = new.into_boxed_slice();
        }
    }
    pub fn truncate(&mut self, new_size: StackAdr) {
        self.1 = new_size as usize;
    }
    pub fn push<T: ByteCodec>(&mut self, v: T) -> StackAdr {
        self.reserved(T::width());
        let l = self.1;
        self.1 += T::width();
        v.set(&mut self.0[l] as *mut u8);
        l as StackAdr
    }
    pub fn pop<T: ByteCodec>(&mut self) -> T {
        self.1 -= T::width();
        T::get(&self.0[self.1] as *const u8)
    }
    pub fn get<T: ByteCodec>(&self, i: StackAdr) -> T {
        T::get(&self.0[i as usize] as *const u8)
    }
    pub fn set<T: ByteCodec>(&mut self, val: T, i: StackAdr) {
        val.set(&mut self.0[i as usize] as *mut u8);
    }
    pub fn len(&self) -> StackAdr {
        self.1 as StackAdr
    }
}

pub trait ByteCodec {
    fn width() -> usize;
    fn set(self, ptr: *mut u8);
    fn get(ptr: *const u8) -> Self;
}

macro_rules! impl_codec_default {
    ($t:ident) => {
        impl ByteCodec for $t {
            fn width() -> usize {
                mem::size_of::<$t>()
            }
            fn set(self, ptr: *mut u8) {
                let ptr: *mut $t = ptr.cast();
                unsafe {
                    *ptr = self;
                }
            }
            fn get(ptr: *const u8) -> Self {
                let ptr: *const $t = ptr.cast();
                unsafe { *ptr }
            }
        }
    };
}

impl_codec_default!(f64);
impl_codec_default!(bool);
impl_codec_default!(u8);
impl_codec_default!(u16);
impl_codec_default!(u32);
impl_codec_default!(u64);
