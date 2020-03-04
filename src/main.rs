extern crate num;
extern crate libc;

#[macro_use]
extern crate num_derive;

use std::fs;
use std::cell::RefCell;
use libc::c_int;

extern "C" {
    fn getchar() -> c_int;
}

struct Instruction(u32);

const LNK: u32 = 29;
const RMD: u32 = 30;
const RPC: u32 = 31;

// TODO: Implement keyboard input
#[allow(dead_code)] const KBS: usize = 0xFFFFFA;
#[allow(dead_code)] const KBD: usize = 0xFFFFFD;

#[allow(dead_code)]
impl Instruction {
    fn as_u32(&self) -> u32 {
        *&self.0
    }

    fn as_hex(&self) -> String {
        format!("{:#06X}", *&self.0)
    }

    fn as_bin(&self) -> String {
        format!("{:#29b}", *&self.0)
    }

    fn get_opcode(&self) -> u8 {
        (&self.0 >> 24) as u8
    }

    fn get_ophex(&self) -> String {
        format!("{:#02X}", self.get_opcode())
    }

    fn get_op(&self) -> Option<Opcode> {
        num::FromPrimitive::from_u8(self.get_opcode())
    }

    fn get_bit(&self, pos: u8) -> bool {
        ((&self.0 >> pos) & 1) == 1
    }

    fn mask(&self, mask: u32) -> u32 {
        &self.0 & mask
    }

    fn smask(&self, shift: i8, mask: u32) -> u32 {
        self.shift(shift) & mask
    }

    fn shift(&self, shift: i8) -> u32 {
        if shift < 0 {
            (&self.0 << (shift.abs() as u32))
        } else {
            &self.0 >> shift as u32
        }
    }
}
#[derive(FromPrimitive,PartialEq)]
enum Opcode {
    MOV = 0,
    MEX,
    MRX,
    MMX,
    NIL,
    LFX,
    SWX,
    JMP,
    JSR,
    CMP,
    CMZ,
    ARG,
    ADD,
    SUB,
    MUL,
    DIV,
    AND,
    NOT,
    CAL,
    JPA,
    FLX,
    ILX
}

fn get_char() -> i32 {
    unsafe {
        getchar()
    }
}

fn read(file: &str) -> Vec<u8>{
    fs::read(file)
        .expect(
            &format!("Cannot open {}", file)
        )
}

struct Memory {
    mem: RefCell<Vec<u32>>
}

struct Registers {
    reg: RefCell<Vec<u32>>
}

struct VM {
    mem: Memory,
    reg: Registers,
    running: bool
}

impl Memory {
    fn new() -> Memory {
        Memory {
            mem: RefCell::new(
                vec![0; 0xFFFFFF]
            )
        }
    }

    fn read(&self, addr: u32) -> u32 {
        let addr = addr as usize;

        self.mem.borrow()[addr]
    }

    fn write_string(&self, addr: u32, string: &str) {
        // Writes a UTF-16BE string beginning at addr.
        let mut buf: Vec<u8> = Vec::new();
        let string: Vec<u16> = string.encode_utf16().collect();

        for chr in string {
            buf.push((chr >> 8) as u8);
            buf.push((chr & 0xFF) as u8);
        }

        self.load(&mut buf, addr);
    }

    fn read_string(&self, addr: u32) -> String{
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

    fn load(&self, buf: &mut Vec<u8>, addr: u32) {
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

    fn write(&self, addr: u32, data: u32) {
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
    fn new() -> VM {
        VM {
            mem: Memory::new(),
            reg: Registers::new(),
            running: false
        }
    }

    fn get_head(&self) -> u32 {
        self.reg.get(RPC)
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
                format!("{:#08x}", self.get_head()),
                inst.get_opcode()
            )
        }

        self.incr_head();
        inst.mask(0xFFFFFF)
    }

    fn run(&mut self) {
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
    
                    DECOMP:
                    00000 1 000000xxxxx 0000000xxxxx
                     |    \_ flag   \_ dst  \_ src
                     \_ opcode
    
                    NOTES:
                    Transfers the data from SRC register to the DST register.
                    Bit 6 is tied to 1 to ensure that that the memory contents of
                    0x00000000 will not be confused with instruction MOV R00 R00
                    */
    
                    // Check to see if bit 6 is equal to 1
                    if inst.smask(23, 0x001) == 1 {
                        // It's not required to use smask because the opcode is
                        // 0b00000, but it's good practice anyways.
                        let dst = inst.smask(12, 0x1F);
                        let src = inst.mask(0x1F);

                        self.reg.set(dst, self.reg.get(src));
                    }
                },
                Some(Opcode::MEX) => {
                    /*
                    INSTRUCTION:
                    MEX (argument | dst) (argument | src)
                    DECOMP:
                    00001 000000000000000000000000
                     \_ opcode  \_ filler
    
                    NOTES:
                    Transfers data from the src memory address to the dst memory
                    address. The dst and src memory addresses are provided by
                    additional ARG instructions.
                    */

                    // DST/SRC values provided by ARGs
                    let dst = self.read_arg();
                    let src = self.read_arg();
                    
                    self.mem.write(dst, self.mem.read(src));
                },
                Some(Opcode::MRX) => {
                    /*
                    INSTRUCTION:
                    MRX <REGISTER | DST> (argument)

                    DECOMP:
                    00010 0000000000000000000 xxxxx
                     \_ opcode  \_ filler      \_ dst
    
                    NOTES:
                    The DST register is encoded in the last 5 bits, mask: 0x1F. The
                    ARG instruction is used to provide the argument for the MRX
                    instruction; therefore, MRX must be followed by ARG.
                    */
                    let dst = inst.mask(0x1F);

                    // MRX argument is provided by ARG
                    self.reg.set(dst, self.read_arg());
                },
                Some(Opcode::MMX) => {
                    /*
                    INSTRUCTION:
                    MMX <REGISTER | SRC> (argument | dst)
                    DECOMP:
                    00011 0000000000000000000 xxxxx
                     \_ opcode  \_ filler      \_ src

                    NOTES:
                    Transfers data from the src register into memory address dst.
                    The dst memory address is supplied by an ARG instruction.
                    */
                    let src = inst.mask(0x1F);
    
                    // MMX argument is provided by ARG
                    self.mem.write(self.read_arg(), self.reg.get(src));
                },
                Some(Opcode::NIL) => {
                    // NIL
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
                        _ => panic!("At {} found an unknown flag passed to CMP/CMD instruction.", format!("{:#08X}", self.get_head()))
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
                    panic!("At {} found ARG instruction without accompanying command.", format!("{:#08X}", self.get_head()));
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

                    TABLE:
                     - 0x9A: PNT
                     - 0x9D: HLT
                    */

                    match inst.mask(0xFF) {
                        0x98 => {
                            // INP ; CAL 0x98
                            self.reg.set(0, get_char() as u32);
                        },
                        0x99 => {
                            // OUT ; CAL 0x99
                            // Prints out the 16 least significant bits in R00
                            let chr = (self.reg.get(0) & 0xFFFF) as u16;
                            print!("{}", String::from_utf16(&[chr]).unwrap());
                        },
                        0x9A => {
                            // PNT ; CAL 0x9A
                            // Equivalent to LC-3 PUTS trap.
                            // BVM uses UTF-16BE encoding for strings.

                            // Read address in R00, goto address, print string.
                            print!("{}", self.mem.read_string(self.reg.get(0)));
                        },
                        0x9D => {
                            //println!("HLT ; CAL 0x9D");
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
                        self.get_head(), inst.as_hex(), inst.get_ophex());
                }
            }
            
            if incr {
                self.incr_head();
            }
        }
    }
}
fn main() {
    // Initialize virtual machine
    let mut bvm = VM::new();

    bvm.mem.write(0x00000000, 0x13_00_00_09);
    bvm.mem.write(0x00000001, 0x00_68_00_65);
    bvm.mem.write(0x00000002, 0x00_6C_00_6C);
    bvm.mem.write(0x00000003, 0x00_6F_00_5F);
    bvm.mem.write(0x00000004, 0x00_77_00_6F);
    bvm.mem.write(0x00000005, 0x00_72_00_6C);
    bvm.mem.write(0x00000006, 0x00_64_00_2E);
    bvm.mem.write(0x00000007, 0x00_62_00_69);
    bvm.mem.write(0x00000008, 0x00_6E_00_00);
    bvm.mem.write(0x00000009, 0x15_00_00_00);
    bvm.mem.write(0x0000000A, 0x0B_00_00_01);

    bvm.run();
}