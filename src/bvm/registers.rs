use std::cell::RefCell;
use std::collections::HashMap;

pub struct Registers(RefCell<HashMap<u8, u64>>);

impl Registers {
    pub fn new() -> Registers {
        Registers(
            // Initialize with space for 32 registers
            RefCell::new(HashMap::with_capacity(32))
        )
    }

    pub fn exists(&self, register: &u8) -> bool {
        // Check to see if register exists
        self.0.borrow().contains_key(register)
    }

    pub fn get(&self, register: &u8) -> Option<u64> {
        // Get the value stored in register, if exists
        if self.exists(register) {
            Some(self.0.borrow()[register])
        } else {
            None
        }
    }

    pub fn set(&self, register: u8, data: u64) {
        // Set the value of a register
        &self.0.borrow_mut().insert(register, data);
    }
}

#[test]
fn test_exists() {
    let reg = Registers::new();
    assert!(!reg.exists(&29));

    reg.0.borrow_mut().insert(29, 29);
    assert!(reg.exists(&29));
}

#[test]
fn test_get() {
    let reg = Registers::new();

    assert_eq!(reg.get(&29), None);

    reg.0.borrow_mut().insert(29, 29);
    assert_eq!(reg.get(&29).unwrap(), 29);
}