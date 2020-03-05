#[path = "instructions.rs"]
pub mod instructions;

#[path = "externals.rs"]
pub mod externals;

use std::cell::RefCell;
use self::instructions::{Call, Instruction, Opcode};
use self::externals::{get_char, read};

pub const LNK: u32 = 29;
pub const RMD: u32 = 30;
pub const RPC: u32 = 31;

pub struct Memory {
    mem: RefCell<Vec<u32>>
}

pub struct Registers {
    reg: RefCell<Vec<u32>>
}

pub struct VM {
    pub mem: Memory,
    pub reg: Registers,
    pub running: bool
}

impl Memory {
    fn new() -> Memory {
        Memory {
            mem: RefCell::new(
                vec![0; 0xFFFFFF]
            )
        }
    }

    pub fn read(&self, addr: u32) -> u32 {
        let addr = addr as usize;

        self.mem.borrow()[addr]
    }

    pub fn write_string(&self, addr: u32, string: &str) {
        // Writes a UTF-16BE string beginning at addr.
        let mut buf: Vec<u8> = Vec::new();
        let string: Vec<u16> = string.encode_utf16().collect();

        for chr in string {
            buf.push((chr >> 8) as u8);
            buf.push((chr & 0xFF) as u8);
        }

        self.load(&mut buf, addr);
    }

    pub fn read_string(&self, addr: u32) -> String{
        // Reads a UTF-16BE string beginning at addr, until null byte.
        let addr = addr as usize;
        let mut buf: Vec<u16> = Vec::new();

        for data in &self.mem.borrow()[addr ..] {
            // Get chars stored in 16 most and 16 least significant bits
            let mchr = (data >> 16) as u16;
            let lchr = (data & 0xFFFF) as u16;

            if mchr == 0 {
                // Encountered null byte first, break.
                break;
            } else if lchr == 0 {
                // Encountered string byte first, then null byte.
                // Push first 16 bits to buffer, then break.
                buf.push(mchr);
                break;
            }

            buf.push(mchr);
            buf.push(lchr);
        }

        String::from_utf16(&buf).unwrap()
    }

    pub fn load(&self, buf: &mut Vec<u8>, addr: u32) {
        let len: usize = buf.len();
        
        if len % 4 != 0 {
            // Pad data so that it fits
            for _ in 0..len % 4 {
                buf.push(0);
            }
        }

        for i in (0..len).step_by(4) {
            let head = addr + (i as u32 / 4);
            let data: u32 =
                (buf[i] as u32)       << 24u32
                | (buf[i + 1] as u32) << 16u32
                | (buf[i + 2] as u32) <<  8u32
                | buf[i + 3] as u32;

            self.write(head, data);
        }
    }

    pub fn write(&self, addr: u32, data: u32) {
        let addr = addr as usize;
        self.mem.borrow_mut()[addr] = data;
    }
}

impl Registers {
    fn new() -> Registers {
        Registers {
            reg: RefCell::new(
                vec![0; 0x20]
            )
        }
    }

    fn get(&self, register: u32) -> u32 {
        // Get value of register
        self.reg.borrow()[register as usize]
    }

    fn set(&self, register: u32, value: u32) {
        // Set value of register
        self.reg.borrow_mut()[register as usize] = value;
    }
}

impl VM {
    pub fn new() -> VM {
        VM {
            mem: Memory::new(),
            reg: Registers::new(),
            running: false
        }
    }

    fn get_head(&self) -> u32 {
        self.reg.get(RPC)
    }

    fn get_head_hex(&self) -> String {
        format!("{:#010X}", self.get_head())
    }

    fn set_head(&self, value: u32) {
        self.reg.set(RPC, value);
    }

    fn incr_head(&self) {
        self.reg.set(RPC, self.reg.get(RPC) + 1);
    }

    fn read_instruction(&self, addr: u32) -> Instruction {
        // Return the instruction found in memory at addr
        Instruction(
            self.mem.read(addr)
        )
    }

    fn read_arg(&self) -> u32 {
        let inst = self.read_instruction(self.get_head() + 1);

        if inst.get_op() != Some(Opcode::ARG) {
            panic!(
                "At {}, expected ARG instruction, got {} instead.",
                self.get_head_hex(),
                inst.get_opcode()
            )
        }

        self.incr_head();
        inst.mask(0xFFFFFF)
    }

    pub fn run(&mut self) {
        self.set_head(0);
        self.running = true;

        while self.running {
            let mut incr = true;
            let inst = self.read_instruction(self.get_head());

            match inst.get_op() {
                Some(Opcode::MOV) => {
                    /*
                    INSTRUCTION:
                    MOV <REGISTER | DST> <REGISTER | SRC>
                    MOV (argument | [addr | imm24]) (argument | [addr | imm24])
    
                    DECOMP:
                    00000 1 xxxxxx xxxxx 0000000xxxxx
                    |     |  \_ mode  \_ dst     \_ src
                    |     \_ flag
                    \_ opcode
    
                    NOTES:
                    Transfers data to and from registers and memory locations.
                    Bit 6 is tied to 1 to ensure that 0x00000000 will not be
                    confused as an instruction (MOV R00 R00)
                    */

                    // Check to see if bit 6 is equal to 1
                    if inst.smask(23, 0x001) == 1 {
                        // TODO: Update descriptions.

                        // It's not required to use smask because the opcode is
                        // 0b00000, but it's good practice anyways.
                        let mode = inst.smask(17, 0x3F);

                        match mode {
                            0x1 => {
                                /*
                                INSTRUCTION:
                                MOV <REGISTER | DST> <REGISTER | SRC>
                                DECOMP:
                                00000 1 000000 xxxxx 0000000xxxxx
                                |     |  \_ mode  \_ dst     \_ src
                                |     \_ flag
                                \_ opcode
                                */
                                // Move between registers
                                let dst = inst.smask(12, 0x1F);
                                let src = inst.mask(0x1F);

                                self.reg.set(dst, self.reg.get(src));
                            },
                            0x2 => {
                                /*
                                INSTRUCTION:
                                MEX (argument | dst) (argument | src)
                                DECOMP:
                                00001 000000000000000000000000
                                \_ opcode  \_ filler
                                */
                                // Move data from memory address (arg2) to
                                // register (arg1)
                                let dst = self.read_arg();
                                let src = self.read_arg();

                                self.mem.write(dst, self.mem.read(src));
                            },
                            0x3 => {
                                /*
                                INSTRUCTION:
                                MRX <REGISTER | DST> (argument)

                                DECOMP:
                                00010 0000000000000000000 xxxxx
                                \_ opcode  \_ filler      \_ dst
                                */
                                // Moves immediate value (arg1) into register
                                let dst = inst.mask(0x1F);
                                let val = self.read_arg();

                                self.reg.set(dst, val);
                            },
                            0x4 => {
                                /*
                                INSTRUCTION:
                                MMX <REGISTER | SRC> (argument | dst)
                                DECOMP:
                                00011 0000000000000000000 xxxxx
                                \_ opcode  \_ filler      \_ src

                                NOTES:
                                */
                                // Moves data from register (inst | 0x1F)
                                // into memory address (arg1)
                                let src = inst.mask(0x1F);
                                let addr = self.read_arg();

                                self.mem.write(addr, src);
                            },
                            0x05 => {
                                let addr = inst.mask(0xFFFFFF);
                                let val = self.read_arg();
                                
                                self.mem.write(addr, val);
                            }
                            _ => {
                                panic!("At {} found an unknown mode, {} , passed to MOV instruction.",
                                self.get_head_hex(),
                                format!("{:#06X}", mode)
                                )
                            }
                        }
                    }
                },
                Some(Opcode::LFX) => {
                    /*
                    INSTRUCTION:
                    LFX <REGISTER | DST> (argument | src)
                    DECOMP:
                    00101 0000000000000000000 xxxxx
                    \_ opcode  \_ filler      \_ dst

                    NOTES:
                    Transfers data from memory into dst register.
                    */

                    let dst = inst.mask(0x1F);
                    // LFX argument provided by ARG
                    let src = self.read_arg();
                    self.reg.set(dst, self.reg.get(src));
                },
                Some(Opcode::SWX) => {
                    /*
                    INSTRUCTION:
                    SWX (argument | addr1)  (argument | addr2)
                    DECOMP:
                    00110 000000000000000000000000
                    \_ opcode  \_ filler

                    NOTES:
                    Swaps the data in addr1 and addr2.
                    Normally this would have been done by putting the value of addr1
                    into a register, then replacing the addr1 with addr2, then
                    setting the value of addr2 to the value stored in that register.
                    However, this is a virtual machine, and we have the luxury of
                    not needing to do this.
                    */

                    let addr1 = self.read_arg();
                    let addr2 = self.read_arg();

                    let swp = self.mem.read(addr1);

                    self.mem.write(addr1, self.mem.read(addr2));
                    self.mem.write(addr2, swp);
                },
                Some(Opcode::JMP) => {
                    /*
                    INSTRUCTION:
                    JMP <REGISTER | SRC>
                    DECOMP:
                    00111 0000000000000000000 xxxxx
                    \_ opcode  \_ filler      \_ src

                    INSTRUCTION:
                    RET
                    DECOMP:
                    00111 0000000000000000000 11101
                    \_ opcode  \_ filler      \_ src

                    NOTES:
                    JMP jumps to an address stored in a register
                    RET jumps to the address stored in the LNK register (R29)
                    Do not increment RPC at end of cycle.
                    */

                    incr = false;
                    let addr = inst.mask(0x1F);
                    
                    self.set_head(self.reg.get(addr));
                },
                Some(Opcode::JSR) => {
                    // JSR
                    /*
                    INSTRUCTION:
                    JSR <IMM24>
                    DECOMP:
                    01000 xxxxxxxxxxxxxxxxxxxxxxxx
                    \_ opcode  \_ address

                    NOTES:
                    Stores RPC into LNK, then jumps to address.
                    */
                    incr = false;

                    let addr = inst.mask(0xFFFFFF);

                    // Do not want to execute the JSR instruction upon RET
                    // so increment RPC before storing.
                    self.reg.set(LNK, self.get_head() + 1);
                    self.set_head(addr);
                },
                Some(Opcode::CMP) | Some(Opcode::CMZ) => {
                    /*
                    INSTRUCTION:
                    CMP <REGISTER | CMP1> <REGISTER | CMP2>
                    DECOMP:
                    01001 000 0000xxxxx 0000000xxxxx
                    |     |   \_ cmp1   \_ cmp2
                    |     \_ flag
                    \_ opcode

                    NOTES:
                    If comparison tests are true, increment RPC by 1, else increment
                    RPC by 2. Typically a jump instruction would follow after this.

                    INSTRUCTION:
                    CMZ <REGISTER | CMP1>
                    DECOMP:
                    01010 000 000000000 0000000xxxxx
                    |     \_ flag       \_ cmp1
                    \_ opcode

                    NOTES:
                    Compares cmp1 to 0, if true then increment RPC by 1, else
                    increment RPC by 2. Typically a jump instruction would follow
                    after this.
                    */
                    // We will handle RPC incrementation manually here
                    incr = false;

                    // Get flag to see what type of comparison to do
                    let flag = inst.smask(21, 0x7);
    
                    // Get the registers to perform the comparison on
                    let cmp1 = inst.smask(12, 0x1F);
                    let cmp2 = inst.mask(0x1F);

                    // Get values to reduce the amount of typing required
                    let mut cmp1 = self.reg.get(cmp1);
                    let mut cmp2 = self.reg.get(cmp2);
    
                    if inst.get_op() == Some(Opcode::CMZ) {
                        cmp1 = cmp2;
                        cmp2 = 0;
                    }

                    let passed;

                    match flag {
                        0b001 => passed = cmp1 == cmp2, // CEQ | CEZ
                        0b010 => passed = cmp1 <= cmp2, // CEL | CNZ
                        0b011 => passed = cmp1 >= cmp2, // CEG | CPZ
                        0b100 => passed = cmp1 < cmp2,  // CLT | CLZ
                        0b101 => passed = cmp1 > cmp2,  // CGT | CGZ
                        _ => panic!(
                            "At {} found an unknown flag passed to CMP/CMD instruction.",
                            self.get_head_hex()
                        )
                    }

                    if passed {
                        self.incr_head();
                    } else {
                        self.set_head(self.get_head() + 2);
                    }
                },
                Some(Opcode::ARG) => {
                    /*
                    INSTRUCTION:
                    ARG <IMM24>
                    DECOMP:
                    01011 xxxxxxxxxxxxxxxxxxxxxxxx
                    \_ opcode  \_ imm24

                    NOTES:
                    Provides an imm24 value as an argument to another, preceeding
                    instruction.
                    */

                    panic!(
                        "At {} found ARG instruction without accompanying command.",
                        self.get_head_hex()
                    );
                },
                Some(Opcode::ADD) => {
                    /*
                    INSTRUCTION:
                    ADD <REGISTER | DST> <REGISTER | A> <REGISTER |B>
                    DECOMP:
                    01100 000xxxxx 000xxxxx 000xxxxx
                    \_ opcode  \_ dst   \_ a     \_ b

                    NOTES:
                    Sums values of registers a and b, stores it in dst
                    */
                    let dst = inst.smask(16, 0x1F);
                    let a = inst.smask(8, 0x1F);
                    let b = inst.mask(0x1F);

                    self.reg.set(
                        dst,
                        self.reg.get(a) + self.reg.get(b)
                    );
                },
                Some(Opcode::SUB) => {
                    /*
                    INSTRUCTION:
                    SUB <REGISTER | DST> <REGISTER | A> <REGISTER |B>
                    DECOMP:
                    01101 000xxxxx 000xxxxx 000xxxxx
                    \_ opcode  \_ dst   \_ a     \_ b

                    NOTES:
                    Subtracts values of registers a and b, stores it in dst
                    */
                    let dst = inst.smask(16, 0x1F);
                    let a = inst.smask(8, 0x1F);
                    let b = inst.mask(0x1F);

                    self.reg.set(
                        dst,
                        self.reg.get(a) - self.reg.get(b)
                    );
                },
                Some(Opcode::MUL) => {
                    /*
                    INSTRUCTION:
                    MUL <REGISTER | DST> <REGISTER | A> <REGISTER |B>
                    DECOMP:
                    01110 000xxxxx 000xxxxx 000xxxxx
                    \_ opcode  \_ dst   \_ a     \_ b

                    NOTES:
                    Multiplies values of registers a and b, stores it in dst
                    */
                    let dst = inst.smask(16, 0x1F);
                    let a = inst.smask(8, 0x1F);
                    let b = inst.mask(0x1F);

                    self.reg.set(
                        dst,
                        self.reg.get(a) + self.reg.get(b)
                    );
                },
                Some(Opcode::DIV) => {
                    /*
                    INSTRUCTION:
                    DIV <REGISTER | DST> <REGISTER | A> <REGISTER |B>
                    DECOMP:
                    01111 000xxxxx 000xxxxx 000xxxxx
                    \_ opcode  \_ dst   \_ a     \_ b

                    NOTES:
                    Integer division of a by b, stores remainder in RMD register
                    */
                    let dst = inst.smask(16, 0x1F);
                    let a = self.reg.get(inst.smask(8, 0x1F));
                    let b = self.reg.get(inst.mask(0x1F));

                    self.reg.set(
                        dst,
                        a / b
                    );
                    self.reg.set(RMD, a % b);
                },
                Some(Opcode::AND) => {
                    /*
                    INSTRUCTION:
                    AND <REGISTER | DST> <REGISTER | A> <REGISTER |B>
                    DECOMP:
                    10000 000xxxxx 000xxxxx 000xxxxx
                    \_ opcode  \_ dst   \_ a     \_ b

                    NOTES:
                    Computes the bitwise and of a and b
                    */
                    let dst = inst.smask(16, 0x1F);
                    let a = inst.smask(8, 0x1F);
                    let b = inst.mask(0x1F);

                    self.reg.set(
                        dst,
                        self.reg.get(a) & self.reg.get(b)
                    );
                },
                Some(Opcode::NOT) => {
                    /*
                    INSTRUCTION:
                    NOT <REGISTER | DST> <REGISTER | A>
                    DECOMP:
                    10001 000xxxxx 00000000000xxxxx
                    \_ opcode  \_ dst  \_ a

                    NOTES:
                    Bitwise NOT of the value of register a, store in dst register
                    */
                    let dst = inst.smask(16, 0x1F);
                    let a = inst.mask(0x1F);

                    self.reg.set(dst, !self.reg.get(a));
                },
                Some(Opcode::CAL) => {
                    /*
                    INSTRUCTION:
                    CAL <VECTOR>

                    DECOMP:
                    10010 0000000000000000 xxxxxxxx
                    \ opcode   \_ filler   \_ vector

                    NOTES:
                    The CAL instruction is similar to LC-3 traps.
                    Implemented here instead of in assembly code on the virtual
                    machine to improve efficiency.
                    */

                    let call = num::FromPrimitive::from_u32(inst.mask(0xFF));

                    match call {
                        Some(Call::INP) => {
                            // INP ; CAL 0x98
                            self.reg.set(0, get_char() as u32);
                        }
                        Some(Call::OUT) => {
                            // OUT ; CAL 0x99
                            // Prints out the 16 least significant bits in R00
                            let chr = (self.reg.get(0) & 0xFFFF) as u16;
                            print!("{}", String::from_utf16(&[chr]).unwrap());
                        },
                        Some(Call::PNT) => {
                            // PNT ; CAL 0x9A
                            // Equivalent to LC-3 PUTS trap.
                            // BVM uses UTF-16BE encoding for strings.

                            // Read address in R00, goto address, print string.
                            print!("{}", self.mem.read_string(self.reg.get(0)));
                        },
                        Some(Call::HLT) => {
                            // HLT ; CAL 0x9D
                            self.running = false;
                        },
                        _ => {}
                    }
                },
                Some(Opcode::JPA) => {
                    /*
                    INSTRUCTION:
                    JPA <IMM24>
                    DECOMP:
                    10011 xxxxxxxxxxxxxxxxxxxxxxxx
                    \_ opcode  \_ addr

                    NOTES:
                    Jumps directly to address provided
                    */
                    incr = false;

                    let addr = inst.mask(0xFFFFFF);
                    self.set_head(addr);
                },
                Some(Opcode::FLX) => {
                    /*
                    INSTRUCTION:
                    FLX (argument | path_addr) (argument | load_addr)
                    DECOMP:
                    10100 000000000000000000000000
                    \_ opcode  \_ filler

                    NOTES: Loads file from path string stored at path_addr to load_addr
                    */
                    let path_addr = self.read_arg();
                    let load_addr = self.read_arg();

                    // decode path and open file
                    let path = self.mem.read_string(path_addr);
                    let mut buf: Vec<u8> = read(&path);

                    self.mem.load(&mut buf, load_addr);
                },
                Some(Opcode::ILX) => {
                    /*
                    INSTRUCTION:
                    ILX (argument | path_addr)
                    DECOMP:
                    10101 000000000000000000000000
                    \_ opcode  \_ filler

                    NOTES: Loads program file from path string stored at path_addr to LFH header in program binary.
                    */
                    let path_addr = self.read_arg();

                    // decode path and open file
                    let path = self.mem.read_string(path_addr);
                    let mut buf: Vec<u8> = read(&path);
                    let lfh: u32 =
                        (buf.remove(0) as u32) << 16u32
                        | (buf.remove(0) as u32) << 8u32
                        | (buf.remove(0) as u32);

                    self.mem.load(&mut buf, lfh);
                }
                _ => {
                    panic!(
                        "At {} found unknown instruction with value {}, opcode {}",
                        self.get_head_hex(),
                        inst.as_hex(),
                        inst.get_ophex()
                    );
                }
            }
            
            if incr {
                self.incr_head();
            }
        }
    }
}