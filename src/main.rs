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

fn argread(mem: &mut [u32], addr: &mut u32) -> u32 {
    let inst: Instruction = iread(mem, *addr + 1);
    
    if inst.get_op() != Some(Opcode::ARG) {
        panic!("At {}, expected ARG instruction, got {} instead.", *addr, inst.get_opcode());
    }

    *addr += 1;
    inst.mask(0xFFFFFF)
}

fn run() {
    // initialize virtual machine

    // no u24 type; 2^24 = 1677216
    // use vec to put in heap rather than stack, stackoverflow otherwise
    let mut mem = vec![0; 16777216];
    let mut reg = vec![0; 32];

    let program: Vec<u32> = vec![
        // Move into R29, address 0x002931
        // 0x002929 | 0b00010_000000000000000000011100 0x0200001C - MRX R28
        0x0200001C,
        // 0x00292A | 0b01011_000000000010100100110011 0x0B002933 - ARG 0x002933
        0x0B002933,
        
        // Then move data from R29 -> R00
        // 0x00292B | 0b00000_100000000000000000011100 0x0080001C - MOV R00 R28
        0x0080001C,
        
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

    let mut running: bool = true;

    while running {
        let mut incr = true; // choose whether or not to increment RPC

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

                // DST/SRC values provided by ARGs
                let DST: usize = argread(&mut mem, &mut reg[RPC]) as usize;
                let SRC: usize = argread(&mut mem, &mut reg[RPC]) as usize;

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

                // MRX argument is provided by ARG
                reg[DST] = argread(&mut mem, &mut reg[RPC]);
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

                // MMX argument is provided by ARG
                let DST: usize = argread(&mut mem, &mut reg[RPC]) as usize;

                mem[DST] = reg[SRC];
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
                let DST: usize = inst.mask(0x1F) as usize;
                // LFX argument provided by ARG
                let SRC: usize = argread(&mut mem, &mut reg[RPC]) as usize;

                reg[DST] = mem[SRC];
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

                let ADDR1: usize = argread(&mut mem, &mut reg[RPC]) as usize;
                let ADDR2: usize = argread(&mut mem, &mut reg[RPC]) as usize;
                
                let SWP = mem[ADDR1];

                mem[ADDR1] = mem[ADDR2];
                mem[ADDR2] = SWP;
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

                let SRC: u32 = inst.mask(0x1F);
                reg[RPC] = SRC;
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
                
                let ADDR: u32 = inst.mask(0xFFFFFF);

                // Do not want to execute the JSR instruction upon RET
                // so increment RPC before storing.
                reg[LNK] = reg[RPC] + 1; 
                reg[RPC] = ADDR;
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
                let FLAG: u32 = inst.smask(21, 0x7);
                
                // Get the registers to perform the comparison on
                let CMP1: usize = inst.smask(12, 0x1F) as usize;
                let CMP2: usize = inst.mask(0x1F) as usize;

                // Get values to reduce the amount of typing required
                let mut CMP1: u32 = reg[CMP1];
                let mut CMP2: u32 = reg[CMP2];

                if inst.get_op() == Some(Opcode::CMZ) {
                    CMP1 = CMP2;
                    CMP2 = 0;
                }

                let passed;

                match FLAG {
                    0b001 => passed = CMP1 == CMP2, // CEQ | CEZ
                    0b010 => passed = CMP1 <= CMP2, // CEL | CNZ
                    0b011 => passed = CMP1 >= CMP2, // CEG | CPZ
                    0b100 => passed = CMP1 < CMP2,  // CLT | CLZ
                    0b101 => passed = CMP1 > CMP2,  // CGT | CGZ
                    _ => panic!("At {} found an unknown flag passed to CMP/CMD instruction.", reg[RPC])
                }

                if passed {
                    reg[RPC] += 1;
                } else {
                    reg[RPC] += 2;
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
                panic!("At {} found ARG instruction without accompanying command.", reg[RPC]);
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
                let DST: usize = inst.smask(16, 0x1F) as usize;
                let A: usize = inst.smask(8, 0x1F) as usize;
                let B: usize = inst.mask(0x1F) as usize;

                reg[DST] = reg[A] + reg[B];
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
                let DST: usize = inst.smask(16, 0x1F) as usize;
                let A: usize = inst.smask(8, 0x1F) as usize;
                let B: usize = inst.mask(0x1F) as usize;

                reg[DST] = reg[A] - reg[B];
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
                let DST: usize = inst.smask(16, 0x1F) as usize;
                let A: usize = inst.smask(8, 0x1F) as usize;
                let B: usize = inst.mask(0x1F) as usize;

                reg[DST] = reg[A] * reg[B];
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
                let DST: usize = inst.smask(16, 0x1F) as usize;
                let A: usize = inst.smask(8, 0x1F) as usize;
                let B: usize = inst.mask(0x1F) as usize;

                reg[DST] = reg[A] / reg[B];
                reg[RMD] = reg[A] % reg[B];
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
                let DST: usize = inst.smask(16, 0x1F) as usize;
                let A: usize = inst.smask(8, 0x1F) as usize;
                let B: usize = inst.mask(0x1F) as usize;

                reg[DST] = reg[A] & reg[B];
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
                let DST: usize = inst.smask(16, 0x1F) as usize;
                let A: usize = inst.mask(0x1F) as usize;

                reg[DST] = !reg[A];
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
                                // Break if all 32 bits are zero, or if
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
                /*
                INSTRUCTION:
                JPX <IMM24>
                DECOMP:
                10011 xxxxxxxxxxxxxxxxxxxxxxxx
                 \_ opcode  \_ addr
                
                NOTES:
                Jumps directly to address provided
                */
                incr = false;

                let ADDR: u32 = inst.mask(0xFFFFFF);
                reg[RPC] = ADDR;
            },
            _ => {
                panic!("At {} found unknown instruction with opcode {}", reg[RPC], inst.get_opcode())
            }
        }

        if incr {
            reg[RPC] += 1;
        }
    }
}

fn main() {
    run();
}