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
                            let size = Instruction::get_size(op, self.next_byte(i)) as usize;
                            let rbytes = self.mem.read_bytes(self.addr, (i + size) as u32);
                            let mut bytes: Vec<u8> = Vec::new();
                            bytes.extend_from_slice(&rbytes[i..i + size]);

                            if i == 7 {
                                bytes.extend_from_slice(&[self.next_byte(i)]);
                            }

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

    fn next_byte(&self, i: usize) -> u8 {
        // Get the next byte in memory starting from byte index i
        let i = i + 1;

        if i < 7 {
            let memory = self.mem.read(self.addr);

            u64_to_u8arr(memory.unwrap())[i]
        } else {
            let memory = self.mem.read(self.addr + 1);

            if memory != None {
                (memory.unwrap() >> 56) as u8
            } else {
                panic!("Unexpected empty address {:#010X}", self.addr + 1);
            }
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
            Opcode::CMP_GT_REG_IMM => self.execute_comparison(inst),
            // TODO: we should use signed integers rather than unsigned
            // TODO: add sign extend function to support this the above
            Opcode::AND |
            Opcode::ADD |
            Opcode::SUB |
            Opcode::MUL |
            Opcode::DIV => self.execute_arithmetic(inst),
            Opcode::FADD |
            Opcode::FSUB |
            Opcode::FMUL |
            Opcode::FDIV => self.execute_fp_arithmetic(inst),
            Opcode::NOT => self.execute_not(inst),
            Opcode::CAL => self.execute_call(inst),
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
                let dst = u8arr_to_u32(&mov.bytes[2..d]);
                let src = u8arr_to_u32(&mov.bytes[d..]);

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
                let dst = u8arr_to_u32(&mov.bytes[2..d]);
                let src = u8arr_to_u64(&mov.bytes[d..]);

                self.mem.write(dst, src);
            },
            _ => panic!("Non mov instruction found.")
        }
    }

    fn execute_jump(&mut self, jmp: Instruction) {
        match jmp.opcode {
            Opcode::JMP_IMM |
            Opcode::JSR => {
                let addr = u8arr_to_u32(&jmp.bytes[2..]);

                if jmp.opcode == Opcode::JSR {
                    // Store incremented address in register 255
                    // upon RET (JMP R255), jump back to this stored addr.
                    self.reg.set(255, (self.addr + 1) as u64);
                }

                // Because addr is incremented after execution,
                // we must subtract.
                // NOTE: maybe fix this?
                self.addr = addr - 1;
            },
            Opcode::JMP_REG => {
                let addr = self.reg.get(&jmp.bytes[1]) as u32;
                self.addr = addr - 1; // see above as to why we subtract
            },
            _ => panic!("Non jmp instruction found.")
        }
    }

    fn execute_comparison(&mut self, comp: Instruction) {
        match comp.opcode {
            // we increment addr by 1 instead of 2 because the address is
            // incremented again after the execution of this function
            // NOTE: fix this?
            Opcode::CMP_EQ_REG_REG =>
                if !(self.reg.get(&comp.bytes[1]) == self.reg.get(&comp.bytes[2])) {self.addr += 1; },
            Opcode::CMP_LE_REG_REG =>
                if !(self.reg.get(&comp.bytes[1]) <= self.reg.get(&comp.bytes[2])) {self.addr += 1; },
            Opcode::CMP_GE_REG_REG =>
                if !(self.reg.get(&comp.bytes[1]) >= self.reg.get(&comp.bytes[2])) {self.addr += 1; },
            Opcode::CMP_LT_REG_REG =>
                if !(self.reg.get(&comp.bytes[1]) < self.reg.get(&comp.bytes[2])) {self.addr += 1; },
            Opcode::CMP_GT_REG_REG =>
                if !(self.reg.get(&comp.bytes[1]) > self.reg.get(&comp.bytes[2])) {self.addr += 1; },
            Opcode::CMP_EQ_REG_IMM =>
                if !(self.reg.get(&comp.bytes[2]) == u8arr_to_u64(&comp.bytes[3..])) { self.addr += 1 },
            Opcode::CMP_LE_REG_IMM =>
                if !(self.reg.get(&comp.bytes[2]) <= u8arr_to_u64(&comp.bytes[3..])) { self.addr += 1 },
            Opcode::CMP_GE_REG_IMM =>
                if !(self.reg.get(&comp.bytes[2]) >= u8arr_to_u64(&comp.bytes[3..])) { self.addr += 1 },
            Opcode::CMP_LT_REG_IMM =>
                if !(self.reg.get(&comp.bytes[2]) < u8arr_to_u64(&comp.bytes[3..])) { self.addr += 1 },
            Opcode::CMP_GT_REG_IMM =>
                if !(self.reg.get(&comp.bytes[2]) > u8arr_to_u64(&comp.bytes[3..])) { self.addr += 1 },
            _ => panic!("Non jmp instruction found.")
        }
    }

    fn execute_arithmetic(&self, arith: Instruction) {
        let dst = arith.bytes[2];

        let mut src1: u64 = 0;
        let mut src2: u64 = 0;

        match arith.bytes[1] >> 6 {
            0 => {
                src1 = self.reg.get(&arith.bytes[3]);
                src2 = self.reg.get(&arith.bytes[4]);
            },
            1 => {
                src1 = self.reg.get(&arith.bytes[3]);
                src2 = u8arr_to_u64(&arith.bytes[4..]);
            },
            2 => {
                let d = 2 + (arith.bytes[1] & 0xF) as usize;
                src1 = u8arr_to_u64(&arith.bytes[2..d]);
                src2 = u8arr_to_u64(&arith.bytes[d..]);
            },
            _ => {}
        }

        match arith.opcode {
            Opcode::ADD => self.reg.set(dst, src1 + src2),
            Opcode::SUB => self.reg.set(dst, src1 - src2),
            Opcode::MUL => self.reg.set(dst, src1 * src2),
            Opcode::DIV => self.reg.set(dst, src1 / src2),
            _ => {}
        }
    }

    fn execute_fp_arithmetic(&self, arith: Instruction) {
        let dst = arith.bytes[2];

        let mut src1: u64 = 0;
        let mut src2: u64 = 0;

        match arith.bytes[1] >> 6 {
            0 => {
                src1 = self.reg.get(&arith.bytes[3]);
                src2 = self.reg.get(&arith.bytes[4]);
            },
            1 => {
                src1 = self.reg.get(&arith.bytes[3]);
                src2 = u8arr_to_u64(&arith.bytes[4..]);
            },
            2 => {
                let d = 2 + (arith.bytes[1] & 0xF) as usize;
                src1 = u8arr_to_u64(&arith.bytes[2..d]);
                src2 = u8arr_to_u64(&arith.bytes[d..]);
            },
            _ => {}
        }

        let src1 = src1 as f64;
        let src2 = src2 as f64;

        match arith.opcode {
            Opcode::FADD => self.reg.set(dst, (src1 + src2) as u64),
            Opcode::FSUB => self.reg.set(dst, (src1 - src2) as u64),
            Opcode::FMUL => self.reg.set(dst, (src1 * src2) as u64),
            Opcode::FDIV => self.reg.set(dst, (src1 / src2) as u64),
            _ => {}
        }
    }

    fn execute_not(&self, inst: Instruction) {
        let dst = inst.bytes[2];

        match inst.bytes[1] >> 6 {
            0 => self.reg.set(dst, !self.reg.get(&inst.bytes[3])),
            1 => self.reg.set(dst, !u8arr_to_u64(&inst.bytes[3..])),
            _ => {}
        }
    }

    fn execute_call(&mut self, inst: Instruction) {
        match inst.bytes[1] {
            // TODO: add more calls
            0x9D => self.running = false, // HLT
            _ => {}
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
        // JMP [0x3]
        &[Opcode::JMP_IMM as u8, 1, 3]
    );
    vm.mem.write_bytes(1,
        // MOV [0x92CA] 0xAABBCCDDEE
        &[Opcode::MOV_MEM_IMM as u8, 0b0010_0101, 0x92, 0xCA, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE]
    );
    vm.mem.write_bytes(3,
        // MOV [0x93CA] 0xAABBFFDDEE
        &[Opcode::MOV_MEM_IMM as u8, 0b0010_0101, 0x93, 0xCA, 0xAA, 0xBB, 0xFF, 0xDD, 0xEE]
    );

    vm.reg.set(29, 8); // MOV R29 8
    vm.mem.write_bytes(5,
        // JMP R29
        &[Opcode::JMP_REG as u8, 29]
    );
    vm.mem.write_bytes(6,
        // MOV [0x92CA] 0xAABBCCDDEE
        &[Opcode::MOV_MEM_IMM as u8, 0b0010_0101, 0x92, 0xCA, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE]
    );
    vm.mem.write_bytes(8,
        // MOV [0x94CA] 0xAABBCCFFEE
        &[Opcode::MOV_MEM_IMM as u8, 0b0010_0101, 0x94, 0xCA, 0xAA, 0xBB, 0xCC, 0xFF, 0xEE]
    );

    vm.mem.write_bytes(10,
        // JSR 0xEE
        &[Opcode::JSR as u8, 1, 0xEE]
    );

    vm.mem.write_bytes(0xEE,
        // MOV [0xEE] 0x29
        &[Opcode::MOV_MEM_IMM as u8, 0b0001_0001, 0xEE, 0x29]
    );
    vm.mem.write_bytes(0xEF,
        // RET (JMP R255)
        &[Opcode::JMP_REG as u8, 0xFF]
    );

    vm.mem.write_bytes(11,
        // MOV [0x95CA] 0xAABBCCDDFF
        &[Opcode::MOV_MEM_IMM as u8, 0b0010_0101, 0x95, 0xCA, 0xAA, 0xBB, 0xCC, 0xDD, 0xFF]
    );

    vm.run();

    assert_eq!(vm.mem.read(0x92CA), None);
    assert_eq!(vm.mem.read(0x93CA).unwrap(), 0xAABBFFDDEE);
    assert_eq!(vm.mem.read(0x94CA).unwrap(), 0xAABBCCFFEE);
    assert_eq!(vm.mem.read(0x95CA).unwrap(), 0xAABBCCDDFF);
    assert_eq!(vm.mem.read(0xEE).unwrap(), 0x29);
}

#[test]
fn test_comparison() {
    let mut vm = VM::new();

    vm.reg.set(1, 1);
    vm.reg.set(2, 2);
    let instructions: Vec<&[u8]> = vec![
        &[Opcode::CMP_EQ_REG_REG as u8, 0, 3], // CMPeq R00, R03
        &[Opcode::JMP_IMM as u8, 1, 0x3], // JMP [0x3]
        &[Opcode::MOV_MEM_IMM as u8, 0b0001_0001, 0x2, 0xFF], // MOV [0x2] 0xFF
        &[Opcode::CMP_GE_REG_REG as u8, 1, 0], // CMPge R01 R00
        &[Opcode::JMP_IMM as u8, 1, 0x6], // JMP [0x6]
        &[Opcode::MOV_MEM_IMM as u8, 0b0001_0001, 0x2, 0xFF], // MOV [0x2] 0xFF
        &[Opcode::CMP_LE_REG_REG as u8, 1, 2], // CMPle R01 R02
        &[Opcode::JMP_IMM as u8, 1, 0x9], // JMP [0x9]
        &[Opcode::MOV_MEM_IMM as u8, 0b0001_0001, 0x2, 0xFF], // MOV [0x2] 0xFF
        &[Opcode::CMP_GE_REG_REG as u8, 0, 3], // CMPge R00 R03
        &[Opcode::JMP_IMM as u8, 1, 0xC], // JMP [0xC]
        &[Opcode::MOV_MEM_IMM as u8, 0b0001_0001, 0x2, 0xFF], // MOV [0x2] 0xFF
        &[Opcode::CMP_LE_REG_REG as u8, 0, 3], // CMPle R00 R03
        &[Opcode::JMP_IMM as u8, 1, 0xF], // JMP [0xF]
        &[Opcode::MOV_MEM_IMM as u8, 0b0001_0001, 0x2, 0xFF], // MOV [0x2] 0xFF
        &[Opcode::CMP_GT_REG_REG as u8, 2, 1], // CMPgt R02 R01
        &[Opcode::JMP_IMM as u8, 1, 0x12], // JMP [0x12]
        &[Opcode::MOV_MEM_IMM as u8, 0b0001_0001, 0x2, 0xFF], // MOV [0x2] 0xFF
        &[Opcode::CMP_LT_REG_REG as u8, 0, 2], // CMPlt R00 R02
        &[Opcode::JMP_IMM as u8, 1, 0x15], // JMP [0x15]
        &[Opcode::MOV_MEM_IMM as u8, 0b0001_0001, 0x2, 0xFF], // MOV [0x2] 0xFF
    ];

    for (i, inst) in instructions.iter().enumerate() {
        vm.mem.write_bytes(i as u32, inst);
    }

    vm.run();

    assert_ne!(vm.mem.read(0x2).unwrap(), 0xFF);
}