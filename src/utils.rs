use std::convert::TryInto;

pub struct ByteVector(pub Vec<u8>);

impl ByteVector {
    pub fn new() -> ByteVector {
        ByteVector(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_f64(&self, i: usize) -> f64 {
        f64::from_le_bytes(self.0[i..i + 8].try_into().unwrap())
    }
    pub fn push_f64(&mut self, data: f64) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }
    pub fn pop_f64(&mut self) -> f64 {
        let l = self.0.len() - 8;
        let v = f64::from_le_bytes(self.0[l..].try_into().unwrap());
        self.0.truncate(l);
        v
    }

    pub fn get_u8(&self, i: usize) -> u8 {
        self.0[i]
    }
    pub fn set_u8(&mut self, data: u8, i: usize) {
        self.0[i] = data;
    }
    pub fn push_u8(&mut self, data: u8) {
        self.0.push(data);
    }
    pub fn pop_u8(&mut self) -> u8 {
        self.0.pop().unwrap()
    }

    pub fn get_u16(&self, i: usize) -> u16 {
        u16::from_le_bytes(self.0[i..i + 2].try_into().unwrap())
    }
    pub fn set_u16(&mut self, data: u16, i: usize) {
        self.0[i..i + 2].copy_from_slice(&data.to_le_bytes());
    }
    pub fn push_u16(&mut self, data: u16) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }
    pub fn pop_u16(&mut self) -> u16 {
        let l = self.0.len() - 2;
        let v = u16::from_le_bytes(self.0[l..].try_into().unwrap());
        self.0.truncate(l);
        v
    }

    pub fn get_u32(&self, i: usize) -> u32 {
        u32::from_le_bytes(self.0[i..i + 4].try_into().unwrap())
    }
    pub fn set_u32(&mut self, data: u32, i: usize) {
        self.0[i..i + 4].copy_from_slice(&data.to_le_bytes());
    }
    pub fn push_u32(&mut self, data: u32) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }
    pub fn pop_u32(&mut self) -> u32 {
        let l = self.0.len() - 4;
        let v = u32::from_le_bytes(self.0[l..].try_into().unwrap());
        self.0.truncate(l);
        v
    }

    pub fn get_u64(&self, i: usize) -> u64 {
        u64::from_le_bytes(self.0[i..i + 8].try_into().unwrap())
    }
    pub fn set_u64(&mut self, data: u64, i: usize) {
        self.0[i..i + 8].copy_from_slice(&data.to_le_bytes());
    }
    pub fn push_u64(&mut self, data: u64) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }
    pub fn pop_u64(&mut self) -> u64 {
        let l = self.0.len() - 8;
        let v = u64::from_le_bytes(self.0[l..].try_into().unwrap());
        self.0.truncate(l);
        v
    }

    pub fn get_bool(&self, i: usize) -> bool {
        self.0[i] != 0
    }
    pub fn push_bool(&mut self, data: bool) {
        self.0.push(data as u8);
    }
    pub fn pop_bool(&mut self) -> bool {
        self.0.pop().unwrap() != 0
    }

    pub fn push_string(&mut self, data: &String) -> u32 {
        let i = self.0.len() as u32;
        let len = data.len() as u16;
        self.0.extend_from_slice(&len.to_le_bytes());
        self.0.extend_from_slice(&data.as_bytes());
        i
    }
    pub fn get_string(&self, i: u32) -> String {
        let i = i as usize;
        let len = u16::from_le_bytes(self.0[i..i + 2].try_into().unwrap());
        String::from_utf8_lossy(&self.0[i + 2..i + 2 + len as usize]).into_owned()
    }
}
