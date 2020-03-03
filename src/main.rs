#![allow(dead_code,non_snake_case)]
extern crate num;
#[macro_use]
extern crate num_derive;
struct Instruction(u32);

impl Instruction {
    fn get_opcode(&self) -> u8 {
        (&self.0 >> 24) as u8
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
    LFM,
    STM,
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
    JPX
}

const LNK: usize = 29;
const RMD: usize = 30;
const RPC: usize = 31;

fn mread(mem: &mut [u32], addr: u32) -> u32{
    let addr: usize = addr as usize;
    if addr == 0x1FFFFFFF {
        let buf = [0; 1];
        
        if buf[0] == 0 {
            mem[0x1FFFFFFF] = 0;
        } else {
            mem[0x1FFFFFFF] = buf[0];
        }
    }

    mem[addr]
}

fn iread(mem: &mut [u32], addr: u32) -> Instruction {
    let data: u32 = mread(mem, addr);
    Instruction(data)
}

fn run() {
    // initialize virtual machine

    // no u24 type; 2^24 = 1677216
    // use vec to put in heap rather than stack, stackoverflow otherwise
    let mut mem = vec![0; 16777216];
    let mut reg = vec![0; 32];

    let program: Vec<u32> = vec![
        // Move into R29, address 0x002931
        // 0x002929 | 0b00010_000000000000000000011101 0x0200001D - MRX R29
        0x0200001D,
        // 0x00292A | 0b01011_000000000010100100110011 0x0B002933 - ARG 0x002933
        0x0B002933,
        
        // Then move data from R29 -> R00
        // 0x00292B | 0b00000_100000000000000000011101 0x0080001D - MOV R00 R29
        0x0080001D,
        
        // 0x002939 is supposed to be in 0x002938!
        // 0x00292C | 0b00001_000000000000000000000000 0x01000000 - MEX
        0x01000000,
        // 0x00292D | 0b01011_000000000010100100111000 0x0B002938 - ARG 0x002938
        0x0B002938,
        // 0x00292E | 0b01011_000000000010100100111001 0x0B002939 - ARG 0x002939
        0x0B002939,

        // We must clear 0x002939!
        // 0x00292F | 0b00011_000000000000000000000001 0x03000001 - MMX R01
        0x03000001,
        // 0x002930 | 0b01011_000000000010100100111001 0x0B002939 - ARG 0x002939
        0x0B002939,
        // 0x002931 | 0b10010_000000000000000010011010 0x1200009A - PNT
        0x1200009A,
        // 0x002932 | 0b10010_000000000000000010011101 0x1200009D - HLT
        0x1200009D,
        // begin string
        // 0x002933
        0x00680065, // h e
        // 0x002934
        0x006C006C, // l l
        // 0x002935
        0x006F0020, // o  
        // 0x002936
        0x0077006F, // w o
        // 0x002937
        0x0072006C, // r l
        // 0x002938
        0x29292929,
        // 0x002939
        0x0064000A, // d \n
    ];

    for (addr, instruction) in program.iter().enumerate() {
        mem[(0x002929 + addr) as usize] = *instruction;
    }


    // set program counter to 0x2929
    reg[RPC] = 0x002929;

    let mut running: bool = true;

    while running {
        let inst = iread(&mut mem, reg[RPC]);
        let op = inst.get_op();

        match op {
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
                    let DST: usize = inst.smask(12, 0x1F) as usize;
                    let SRC: usize = inst.mask(0x1F) as usize;
                    reg[DST] = reg[SRC];
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

                // interpret next, and over next instructions
                let next: Instruction = iread(&mut mem, reg[RPC] + 1);
                let over: Instruction = iread(&mut mem, reg[RPC] + 2);

                if next.get_op() != Some(Opcode::ARG) || over.get_op() != Some(Opcode::ARG) {
                    // the next two instructions are not ARG
                    running = false;
                    println!("Halted execution, MEX missing ARG");
                }
                
                let DST: usize = next.mask(0xFFFFFF) as usize;
                let SRC: usize = over.mask(0xFFFFFF) as usize;

                reg[RPC] += 2; // increment RPC
                mem[DST] = mem[SRC];
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

                let DST: usize = inst.mask(0x1F) as usize;

                // MRX argument is supplied by ARG
                reg[RPC] += 1; // increment program counter
                
                // interpret next instruction
                let next: Instruction = iread(&mut mem, reg[RPC]);
                
                if next.get_op() != Some(Opcode::ARG) {
                    // Next instruction is not ARG
                    running = false;
                    println!("Halted execution, MRX missing ARG");
                }

                reg[DST] = next.mask(0xFFFFFF);
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

                let SRC: usize = inst.mask(0x1F) as usize;

                // MMX argument is supplied by ARG
                reg[RPC] += 1; // increment program counter
                
                // interpret next instruction
                let next: Instruction = iread(&mut mem, reg[RPC]);
                
                if next.get_op() != Some(Opcode::ARG) {
                    // Next instruction is not ARG
                    running = false;
                    println!("Halted execution, MRX missing ARG");
                }

                let DST: usize  = next.mask(0xFFFFFF) as usize;

                mem[DST] = reg[SRC];
            },
            Some(Opcode::NIL) => {
                // NIL
            },
            Some(Opcode::LFM) => {
                // LFX
            },
            Some(Opcode::STM) => {
                // STM
            },
            Some(Opcode::JMP) => {
                // JMP (RET)
            },
            Some(Opcode::JSR) => {
                // JSR
            },
            Some(Opcode::CMP) => {
                // CMP
            },
            Some(Opcode::CMZ) => {
                // CMZ
            },
            Some(Opcode::ARG) => {
                // ARG
            },
            Some(Opcode::ADD) => {
                // ADD
            },
            Some(Opcode::SUB) => {
                // SUB
            },
            Some(Opcode::MUL) => {
                // MUL
            },
            Some(Opcode::DIV) => {
                // DIV
            },
            Some(Opcode::AND) => {
                // AND
            },
            Some(Opcode::NOT) => {
                // NOT
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
                    0x9A => {
                        // PNT ; CAL 0x9A
                        // Equivalent to LC-3 PUTS trap.
                        // BVM uses UTF-16 encoding for strings.

                        // Goto memory address stored in R00, loop through
                        // memory addresses until stopped.

                        let mut string: Vec<u16> = Vec::new();

                        for c in &mem[reg[0] as usize ..] {
                            // char stored in first 16 bits
                            let bchr: u16 = (c >> 16) as u16;

                            // char stored in last 16 bits
                            let schr: u16 = (c & 0xFFFF) as u16; 

                            if c == &0x00000000 || bchr == 0x0000u16 {
                                // Break if memory is filled with zeros, or if
                                // the first 16 bits are zero.
                                break;
                            } else if schr == 0x0000u16 {
                                // If the last 16 bits are zero, and not the
                                // first 16, then push the first 16 into string,
                                // then break.

                                string.push(bchr);
                                break;
                            }

                            string.push(bchr);
                            string.push(schr);
                        }
                        print!("{}", String::from_utf16(&string).unwrap());
                    },
                    0x9D => {
                        //println!("HLT ; CAL 0x9D");
                        running = false;
                    }
                    _ => {}
                }
            },
            Some(Opcode::JPX) => {
                // JPX
            },
            _ => {
                // invalid opcode
            }
        }

        reg[RPC] += 1;
    }
}

fn main() {
    run();
}