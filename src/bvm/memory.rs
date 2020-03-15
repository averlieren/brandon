use std::cell::RefCell;
use std::mem::transmute;
use std::collections::HashMap;

pub struct Memory(RefCell<HashMap<u32, u64>>);

impl Memory {
    pub fn new() -> Memory {
        Memory (
            // Initialize with 2^16 memory locations
            RefCell::new(HashMap::with_capacity(65536))
        )
    }

    pub fn exists(&self, addr: u32) -> bool {
        self.0.borrow().contains_key(&addr)
    }

    pub fn write(&self, addr: u32, content: u64) {
        *self.0.borrow_mut().entry(addr).or_insert(0) = content;
    }

    pub fn read(&self, addr: u32) -> u64 {
        if self.exists(addr) {
            self.0.borrow()[&addr]
        } else {
            0
        }
    }

    pub fn read_bytes(&self, start: u32) -> Vec<u8> {
        // Reads bytes into buffer until empty address.
        let mut addr: u32 = start;
        let mut buf: Vec<u8> = Vec::with_capacity(64);

        loop {
            let data: u64 =  self.read(addr);

            if data == 0 {
                break;
            }

            let bytes: [u8; 8] = unsafe {
                transmute(data.to_be())
            };

            for byte in &bytes {
                buf.push(*byte);
            }

            addr += 1;
        }

        buf
    }
}

#[test]
fn test_read_bytes() {
    let mem = Memory::new();
    let correct: Vec<u8> = vec![0, 104, 0, 101, 0, 108, 0, 108, 0, 111, 0, 32, 0, 119, 0, 111, 0, 114, 0, 108, 0, 100, 0, 0];

    mem.write(0, 0x00680065006C006C);
    mem.write(1, 0x006F00200077006F);
    mem.write(2, 0x0072006C00640000);

    assert_eq!(correct, mem.read_bytes(0));
}