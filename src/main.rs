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
    
    // instructions
    //0b00010_000000000000000000000000 0x02000000 - MRX R00
    mem[0x002929usize] = 0x02000000; 
    //0b01011_000000000010100100101001 0x0B002929 - ARG 0x00292D
    mem[0x00292Ausize] = 0x0B00292D;
    //0b10010_000000000000000010011010 0x1200009A - PNT
    mem[0x00292Busize] = 0x1200009A;
    //0b10010_000000000000000010011101 0x1200009D - HLT
    mem[0x00292Cusize] = 0x1200009D;

    // begin string
    mem[0x00292Dusize] = 0x00680065; // h e
    mem[0x00292Eusize] = 0x006C006C; // l l
    mem[0x00292Fusize] = 0x006F0020; // o  
    mem[0x002930usize] = 0x0077006F; // w o
    mem[0x002931usize] = 0x0072006C; // r l
    mem[0x002932usize] = 0x0064000A; // d \n

    // set program counter to 0x2929
    reg[RPC] = 0x002929;

    let mut running: bool = true;

    while running {
        let inst = iread(&mut mem, reg[RPC]);
        let op = inst.get_op();

        match op {
            Some(Opcode::MOV) => {
                // MOV
            },
            Some(Opcode::MEX) => {
                // MEX
            },
            Some(Opcode::MRX) => {
                /*
                INSTRUCTION:
                MRX <REGISTER | DST> (argument)
                
                DECOMP:
                00011 0000000000000000000 xxxxx
                 \_ opcode  \_ filler      \_ register

                NOTES:
                The DST register is encoded in the last 5 bits, mask: 0x1F. The
                ARG instruction is used to provide the argument for the MRX
                instruction; therefore, MRX must be followed by ARG.
                */

                let DST: usize = inst.mask(0x1F) as usize;

                // MRX argument is supplied by ARG
                reg[RPC] += 1; // increment program counter
                
                // interpret next instruction
                let next: Instruction = Instruction(mread(&mut mem, reg[RPC]));
                
                if next.get_op() != Some(Opcode::ARG) {
                    // Next instruction is not ARG
                    running = false;
                    println!("Halted execution, MRX missing ARG");
                } else {
                    reg[DST] = next.mask(0xFFFFFF);
                }
            },
            Some(Opcode::MMX) => {
                // MMX
            },
            Some(Opcode::NIL) => {
                // NIL
            },
            Some(Opcode::LFM) => {
                // LFM
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
                let arg: u32 = inst.mask(0xFFFFFF);
                println!("ARG {}", arg);
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
                // CAL - based on LC-3 traps
                
                match inst.mask(0xFF) {
                    0x9A => {
                        // PNT ; CAL 0x9A
                        // Equivalent to LC-3 PUTS trap.
                        // BVM uses UTF-16 encoding for strings.

                        // Goto memory address stored in R00, loop through
                        // memory addresses until stopped.

                        let mut string: Vec<u16> = Vec::new();

                        for c in &mem[reg[0] as usize ..] {
                            let bchr: u16 = (c >> 16) as u16; // char stored in first 16 bits
                            let schr: u16 = (c & 0xFFFF) as u16; // char stored in last 16 bits

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
                        println!("HLT ; CAL 0x9D");
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
    let inst: Instruction = Instruction(0b00000_000000010101_000000011111);
    println!("{} {} {}", 
        inst.get_opcode(),
        inst.smask(12, 0x1F),
        inst.mask(0x1F),
    );

    run();
}