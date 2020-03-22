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
use externals::{u64_to_u8arr, u8arr_to_u32, u8arr_to_u64};

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
                            let size = Instruction::get_size(op, (i + 1) as u8) as usize;
                            let bytes = self.mem.read_bytes(self.addr, (i + size) as u32);
                            let bytes = &bytes[i .. i+size];

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

    fn execute(&mut self, inst: Instruction) {
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
            Opcode::JSR => self.execute_jump(inst),
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
            Opcode::FDIV => {},
            Opcode::AND |
            Opcode::NOT => {},
            Opcode::CAL => {},
            Opcode::FILE_LOAD => {},
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
                let src = u8arr_to_u32(&mov.bytes[2..=5]);

                if self.mem.exists(&src) {
                    self.reg.set(dst, self.mem.read(src).unwrap());
                } else {
                    panic!("Memory address, {:#010X}, does not exist!", src);
                }
            },
            Opcode::MOV_MEM_REG => {
                let dst = u8arr_to_u32(&mov.bytes[1..=4]);
                let src = mov.bytes[5];

                self.mem.write(dst, self.reg.get(&src));
            },
            Opcode::MOV_MEM_MEM => {
                let d = 2 + (mov.bytes[1] >> 4) as usize;
                let s = d + (mov.bytes[1] & 0xF) as usize;
                let dst = u8arr_to_u32(&mov.bytes[2..d]);
                let src = u8arr_to_u32(&mov.bytes[d..s]);

                if self.mem.exists(&src) {
                    self.mem.write(dst, self.mem.read(src).unwrap());
                } else {
                    panic!("Memory address, {:#010X}, does not exist!", src);
                }
            },
            Opcode::MOV_REG_IMM => {
                let dst = mov.bytes[2];
                let src = u8arr_to_u64(&mov.bytes[3..]);

                self.reg.set(dst, src);
            },
            Opcode::MOV_MEM_IMM => {
                let d = 2 + (mov.bytes[1] >> 4) as usize;
                let s = d + (mov.bytes[1] & 0xF) as usize;
                let dst = u8arr_to_u32(&mov.bytes[2..d]);
                let src = u8arr_to_u64(&mov.bytes[d..s]);

                self.mem.write(dst, src);
            },
            _ => panic!("Non mov instruction found.")
        }
    }

    fn execute_jump(&mut self, jmp: Instruction) {
        match jmp.opcode {
            Opcode::JMP_IMM |
            Opcode::JSR => {
                let d = 2 + (jmp.bytes[1] >> 4) as usize;
                let addr = u8arr_to_u32(&jmp.bytes[2..d]);

                if jmp.opcode == Opcode::JSR {
                    // Store incremented address in register 255
                    // upon RET (JMP R255), jump back to this stored addr.
                    self.reg.set(255, (self.addr + 1) as u64);
                }

                self.addr = addr;
            },
            Opcode::JMP_REG => {
                let addr = self.mem.read(jmp.bytes[1] as u32).unwrap() as u32;
                self.addr = addr;
            },
            _ => panic!("Non jmp instruction found.")
        }
    }
}

#[test]
fn test_mov() {
    let vm = VM::new();

    vm.reg.set(29, 12345); // <=> MOV R29 12345
    vm.execute_mov(
        // MOV R4 R29
        Instruction::with_data(
            Opcode::MOV_REG_REG,
            &[Opcode::MOV_REG_REG as u8, 4, 29]
        )
    );
    assert_eq!(vm.reg.get(&4), 12345);

    vm.mem.write(0x2929, 54321); // <=> MOV [0x2929] 54321
    vm.execute_mov(
        // MOV R0 [0x2929]
        Instruction::with_data(
            Opcode::MOV_REG_MEM,
            &[Opcode::MOV_REG_MEM as u8, 0, 0, 0, 0x29, 0x29]
        )
    );
    assert_eq!(vm.reg.get(&0), 54321);

    vm.execute_mov(
        // MOV [0x27] [0x2929]
        Instruction::with_data(
            Opcode::MOV_MEM_MEM,
            &[Opcode::MOV_MEM_MEM as u8, 0b0001_0010, 0x27, 0x29, 0x29]
        )
    );

    assert_eq!(vm.mem.read(0x27).unwrap(), 54321);

    vm.execute_mov(
        // MOV R59 0x5923242526272829
        Instruction::with_data(
            Opcode::MOV_REG_IMM,
            &[Opcode::MOV_REG_IMM as u8, 7, 0x59, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29]
        )
    );

    assert_eq!(vm.reg.get(&0x59), 0x23242526272829);

    vm.execute_mov(
        // MOV [0x92CA] 0xAABBCCDDEE
        Instruction::with_data(
            Opcode::MOV_MEM_IMM,
            &[Opcode::MOV_MEM_IMM as u8, 0b0010_0101, 0x92, 0xCA, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE]
        )
    );

    assert_eq!(vm.mem.read(0x92CA).unwrap(), 0xAABBCCDDEE);
}

#[test]
fn test_jmp() {
    let mut vm = VM::new();

    vm.mem.write_bytes(0,
        &[Opcode::JMP_IMM as u8, 1, 3]
    );
    vm.mem.write_bytes(1,
        &[Opcode::MOV_MEM_IMM as u8, 0b0010_0101, 0x92, 0xCA, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE]
    );
    vm.mem.write_bytes(3,
        &[Opcode::MOV_MEM_IMM as u8, 0b0010_0101, 0x93, 0xCA, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE]
    );

    vm.run();

    assert_eq!(vm.mem.read(0x92CA), None);
    assert_eq!(vm.mem.read(0x93CA).unwrap(), 0xAABBCCDDEE);
}