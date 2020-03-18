#[path = "instructions.rs"]
pub mod instructions;

#[path = "externals.rs"]
pub mod externals;

#[path = "memory.rs"]
pub mod memory;

#[path = "registers.rs"]
pub mod registers;

use registers::Registers;
use memory::Memory;

// If there are no instrucions for this long, then halt.
const TIMEOUT: u32 = 128;

pub struct VM {
    pub mem: Memory,
    pub reg: Registers,
    pub addr: u32,
    pub running: bool
}

impl VM {
    pub fn new() -> VM {
        VM {
            mem: Memory::new(),
            reg: Registers::new(),
            addr: 0,
            running: false
        }
    }

    pub fn run(&mut self) {
        self.addr = 0;
        self.running = true;
        let mut nop = 0;

        loop {
            let memory = self.mem.read(0);

            match memory {
                None => nop += 1,
                Some(_) => {
                    nop = 0;
                }
            }

            if nop >= TIMEOUT {
                break;
            }

            self.addr += 1;
        }
    }
}