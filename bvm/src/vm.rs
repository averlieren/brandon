#[path = "instructions.rs"]
pub mod instructions;

#[path = "externals.rs"]
pub mod externals;

use std::cell::RefCell;
use std::collections::HashMap;
use self::instructions::{Call, Instruction, Opcode};
use self::externals::{get_char, read};

pub const LNK: u32 = 29;
pub const RMD: u32 = 30;
pub const RPC: u32 = 31;

pub struct Memory(RefCell<HashMap<u32, u32>>);
pub struct Registers(RefCell<HashMap<u32, u32>>);

pub struct VM {
    pub mem: Memory, // RAM for VM
    pub reg: Registers, // VM registers
    pub status: u8, // Status of VM
    pub running: bool
}

impl Memory {
    fn new() -> Memory {
        Memory(
            // Initialize HashMap with at least 65536 addresses before reallocating
            RefCell::new(HashMap::with_capacity(65536))
        )
    }

    pub fn exists(&self, addr: u32) -> bool {
        self.0.borrow().contains_key(&addr)
    }

    pub fn write(&self, addr: u32, content: u32) {
        *self.0.borrow_mut().entry(addr).or_insert(0) = content;
    }

    pub fn read(&self, addr: u32) -> u32{
        if self.exists(addr) {
            self.0.borrow()[&addr]
        } else {
            0
        }
    }

    pub fn read_string(&self, start: u32) -> String {
        // Reads a UTF-16BE string beginning at start, terminating at a null byte.
        let mut addr = start;
        let mut buf: Vec<u16> = Vec::new();

        loop {
            let data = self.read(addr);
            let mchr = (data >> 16) as u16;
            let lchr = (data & 0xFFFF) as u16;
            
            if mchr == 0 {
                // Encountered null byte, break.
                break;
            } else if lchr == 0 {
                // Got null byte in last 16 bits, push first 16.
                buf.push(mchr);
                break;
            }

            buf.push(mchr);
            buf.push(lchr);

            addr += 1;
        }

        String::from_utf16(&buf).unwrap()
    }

    pub fn write_string(&self, addr: u32, string: &str) {
        // Writes a UTF-16BE string starting at addr.
        let string: Vec<u16> = string.encode_utf16().collect();
        let mut buf: Vec<u8> = Vec::with_capacity(string.len() * 2);

        for chr in string {
            buf.push((chr >> 8) as u8);
            buf.push((chr & 0xFF) as u8);
        }

        self.load(&mut buf, addr);
    }

    pub fn load(&self, buf: &mut Vec<u8>, start: u32) {
        let len: usize = buf.len();

        if len % 4 != 0 {
            // Pad data to be a multiple of 4 (32 bit chunks)
            for _ in 0..len % 4 {
                buf.push(0);
            }
        }

        for i in (0..len).step_by(4) {
            let addr = start + (i as u32 / 4);
            let data: u32 =
                (buf[i] as u32) << 24u32
                | (buf[i + 1] as u32) << 16u32
                | (buf[i + 2] as u32) << 8u32
                | buf[i + 3] as u32;

            self.write(addr, data);
        }
    }
}

impl Registers {
    fn new() -> Registers {
        Registers(
            // Initialize with at least 32 registers.
            RefCell::new(HashMap::with_capacity(32))
        )
    }

    pub fn exists(&self, register: u32) -> bool {
        self.0.borrow().contains_key(&register)
    }

    pub fn get(&self, register: u32) -> u32 {
        if self.exists(register) {
            self.0.borrow()[&register]
        } else {
            0
        }
    }

    pub fn set(&self, register: u32, data: u32) {
        *self.0.borrow_mut().entry(register).or_insert(0) = data;
    }
}

impl VM {
    pub fn new() -> VM {
        VM {
            mem: Memory::new(),
            reg: Registers::new(),
            status: 0,
            running: false
        }
    }

    fn get_addr(&self) -> u32 {
        self.reg.get(RPC)
    }

    fn get_addr_hex(&self) -> String {
        format!("{:#010X}", self.get_addr())
    }

    fn set_addr(&self, value: u32) {
        self.reg.set(RPC, value);
    }

    fn incr_addr(&self) {
        self.reg.set(RPC, self.get_addr() + 1);
    }

    fn read_instruction(&self, addr: u32) -> Instruction {
        Instruction(
            self.mem.read(addr)
        )
    }

    fn read_arg(&self) -> u32 {
        let inst = self.read_instruction(self.get_addr() + 1);

        if inst.get_op() != Some(Opcode::ARG) {
            panic!(
                "At {}, expected ARG instruction, got {} instead.",
                self.get_addr_hex(),
                inst.get_opcode()
            )
        }

        self.incr_addr();
        inst.mask(0xFFFFFF)
    }

    pub fn run(&mut self) {
        self.set_addr(0);
        self.running = true;
        self.status = 1;

        while self.running && self.status != 0 {
            let mut incr = true;
            let inst = self.read_instruction(self.get_addr());

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
                                00000  000000000000000000000000
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
                                00000 0000000000000000000 xxxxx
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
                                00000 0000000000000000000 xxxxx
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
                            },
                            0x06 => {
                                /*
                                INSTRUCTION:
                                MFX <REGISTER | DST> (argument | src)
                                DECOMP:
                                00000 0000000000000000000 xxxxx
                                \_ opcode  \_ filler      \_ dst

                                NOTES:
                                Transfers data from memory into dst register.
                                */

                                let dst = inst.mask(0x1F);
                                // LFX argument provided by ARG
                                let src = self.read_arg();
                                self.reg.set(dst, self.reg.get(src));
                            }
                            _ => {
                                panic!("At {} found an unknown mode, {} , passed to MOV instruction.",
                                self.get_addr_hex(),
                                format!("{:#06X}", mode)
                                )
                            }
                        }
                    }
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

                    self.set_addr(self.reg.get(addr));
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
                    self.reg.set(LNK, self.get_addr() + 1);
                    self.set_addr(addr);
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
                            self.get_addr_hex()
                        )
                    }

                    if passed {
                        self.incr_addr();
                    } else {
                        self.set_addr(self.get_addr() + 2);
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
                        self.get_addr_hex()
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
                            self.status = 0;
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
                    self.set_addr(addr);
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
                        self.get_addr_hex(),
                        inst.as_hex(),
                        inst.get_ophex()
                    );
                }
            }

            if incr {
                self.incr_addr();
            }
            if self.status == 2 {
                self.running = false;
            }
        }
    }
}

#[test]
fn test_mem_write() {
    let mem = Memory::new();

    mem.write(0x2929, 0x89ABCDEF);
    assert_eq!(0x89ABCDEF, mem.read(0x2929));
}

#[test]
fn test_mem_mul_write() {
    let mem = Memory::new();

    mem.write(0x2929, 0x89ABCDEF);
    mem.write(0x2929, 0xFEDCBA98);

    assert_eq!(0xFEDCBA98, mem.read(0x2929));
}

#[test]
fn test_mem_load() {
    let mut buf: Vec<u8> = vec![
        0x00,0x68,0x00,0x65, // 0x2929
        0x00,0x6c,0x00,0x6c, // 0x292A
        0x00,0x6f,0x00,0x20, // 0x292B
        0x00,0x77,0x00,0x6f, // 0x292C
        0x00,0x72,0x00,0x6c, // 0x292D
        0x00,0x64 // 0x292E
    ];
    let mem = Memory::new();
    
    mem.load(&mut buf, 0x2929);

    assert_eq!(0x00680065, mem.read(0x2929));
    assert_eq!(0x006C006C, mem.read(0x292A));
    assert_eq!(0x006F0020, mem.read(0x292B));
    assert_eq!(0x0077006F, mem.read(0x292C));
    assert_eq!(0x0072006C, mem.read(0x292D));
    assert_eq!(0x00640000, mem.read(0x292E));
}

#[test]
fn test_mem_write_string() {
    let mem = Memory::new();

    mem.write_string(0x2929, "hello world");

    assert_eq!(0x00680065, mem.read(0x2929));
    assert_eq!(0x006C006C, mem.read(0x292A));
    assert_eq!(0x006F0020, mem.read(0x292B));
    assert_eq!(0x0077006F, mem.read(0x292C));
    assert_eq!(0x0072006C, mem.read(0x292D));
    assert_eq!(0x00640000, mem.read(0x292E));
}

#[test]
fn test_mem_read_string() {
    let mem = Memory::new();

    mem.write_string(0x2929, "hello world");

    assert_eq!(mem.read_string(0x2929), "hello world");
}