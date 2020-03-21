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
use instructions::{Instruction, Opcode};
use externals::u64_to_u8arr;

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
        let mut skip_bytes = 0;

        loop {
            let memory = self.mem.read(self.addr);

            match memory {
                None => nop += 1,
                Some(_) => {
                    nop = 0;
                    let bytes: [u8; 8] = u64_to_u8arr(memory.unwrap());

                    for i in 0..8 {
                        if skip_bytes != 0 {
                            skip_bytes -= 1;
                            continue;
                        }
                        let op = Opcode::from_u8(bytes[i]);

                        if op != None {
                            let op = op.unwrap();
                            let size = Instruction::get_size(op, bytes[i + 1]) as usize;
                            let bytes = self.mem.read_bytes(self.addr, (i + size) as u32);

                            skip_bytes += size - 1;

                            self.execute(Instruction::with_data(op, &bytes));
                        }
                    }
                }
            }

            if nop >= TIMEOUT {
                break;
            }

            self.addr += 1;
        }
    }

    fn execute(&self, inst: Instruction) {
        match inst.opcode {
            Opcode::MOV_REG_REG |
            Opcode::MOV_REG_MEM |
            Opcode::MOV_MEM_REG |
            Opcode::MOV_MEM_MEM |
            Opcode::MOV_REG_IMM |
            Opcode::MOV_MEM_IMM |
            Opcode::SWP => self.execute_mov(inst),
            Opcode::JMP_IMM |
            Opcode::JMP_REG |
            Opcode::JSR => {},
            Opcode::CMP_EQ_REG_REG |
            Opcode::CMP_LE_REG_REG |
            Opcode::CMP_GE_REG_REG |
            Opcode::CMP_LT_REG_REG |
            Opcode::CMP_GT_REG_REG |
            Opcode::CMP_EQ_REG_IMM |
            Opcode::CMP_LE_REG_IMM |
            Opcode::CMP_GE_REG_IMM |
            Opcode::CMP_LT_REG_IMM |
            Opcode::CMP_GT_REG_IMM => {},
            Opcode::ADD |
            Opcode::FADD |
            Opcode::SUB |
            Opcode::FSUB |
            Opcode::MUL |
            Opcode::FMUL |
            Opcode::DIV |
            Opcode::FDIV |
            Opcode::AND |
            Opcode::NOT |
            Opcode::CAL |
            Opcode::FILE_LOAD => {}
            _ => {}
        }
    }

    fn execute_mov(&self, mov: Instruction) {
        match mov.opcode {
            Opcode::MOV_REG_REG => {
                let dst = mov.bytes[1];
                let src = mov.bytes[2];

                self.reg.set(dst, self.reg.get(&src));
            },
            Opcode::MOV_REG_MEM => {
                let dst = mov.bytes[1];
                let src: u32 =
                    (mov.bytes[2] as u32) << 24 |
                    (mov.bytes[3] as u32) << 16 |
                    (mov.bytes[4] as u32) <<  8 |
                    mov.bytes[5] as u32;

                if self.mem.exists(&src) {
                    self.reg.set(dst, self.mem.read(src).unwrap());
                } else {
                    panic!("Memory address does not exist!");
                }
            },
            Opcode::MOV_MEM_REG => {
                let dst: u32 =
                    (mov.bytes[1] as u32) << 24 |
                    (mov.bytes[2] as u32) << 16 |
                    (mov.bytes[3] as u32) <<  8 |
                    mov.bytes[4] as u32;
                let src = mov.bytes[5];

                self.mem.write(dst, self.reg.get(&src));
            },
            Opcode::MOV_MEM_MEM => {},
            Opcode::MOV_REG_IMM => {},
            Opcode::MOV_MEM_IMM => {},
            _ => panic!("Non mov instruction found.")
        }
    }
}