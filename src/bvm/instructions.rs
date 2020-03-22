use std::fmt;
use std::mem::size_of;

const REG: u8 = size_of::<u8>() as u8;
const MEM: u8 = size_of::<u32>() as u8;
const OPCODE: u8 = 1;
const OPTION: u8 = 1;

#[allow(non_camel_case_types)]
#[derive(PartialEq, FromPrimitive, Copy, Clone)]
pub enum Opcode {
    MOV_REG_REG = 0,
    MOV_REG_MEM,
    MOV_MEM_REG,
    MOV_MEM_MEM,
    MOV_REG_IMM,
    MOV_MEM_IMM,
    SWP,
    JMP_IMM,
    JMP_REG,
    JSR,
    CMP_EQ_REG_REG,
    CMP_LE_REG_REG,
    CMP_GE_REG_REG,
    CMP_LT_REG_REG,
    CMP_GT_REG_REG,
    CMP_EQ_REG_IMM,
    CMP_LE_REG_IMM,
    CMP_GE_REG_IMM,
    CMP_LT_REG_IMM,
    CMP_GT_REG_IMM,
    ADD,
    FADD,
    SUB,
    FSUB,
    MUL,
    FMUL,
    DIV,
    FDIV,
    AND,
    NOT,
    CAL,
    FILE_LOAD,
    INVALID
}

impl Opcode {
    pub fn from_u8(num: u8) -> Option<Opcode> {
        // Convert u8 int to corresponding enum constant
        num::FromPrimitive::from_u8(num)
    }
}

#[derive(Copy, Clone)]
pub struct Instruction<'a> {
    pub opcode: Opcode,
    pub bytes: &'a [u8]
}

impl<'a> Instruction<'a> {
    pub fn new() -> Instruction<'a> {
        // Create arbitrary new instruction
        Instruction {
            opcode: Opcode::INVALID,
            bytes: &[]
        }
    }

    pub fn with_data(opcode: Opcode, bytes: &'a [u8]) -> Instruction<'a> {
        // Create instruction with data
        Instruction {
            opcode: opcode,
            bytes: bytes
        }
    }

    pub fn get_size(opcode: Opcode, byte: u8) -> u8 {
        // Get the size of an instruction in bytes
        // Ask for opcode and following byte, as some instructions
        // may have flags set in the next byte
        match opcode {
            Opcode::MOV_REG_REG => OPCODE + REG + REG,
            Opcode::MOV_REG_MEM | Opcode::MOV_MEM_REG => OPCODE + REG + MEM,
            Opcode::MOV_REG_IMM => OPCODE + OPTION + REG + (byte >> 4),
            Opcode::MOV_MEM_MEM | // Memory addresses dont always take up 32bits
            Opcode::MOV_MEM_IMM => OPCODE + OPTION + (byte >> 4) + (byte & 0xF),
            Opcode::SWP => OPCODE + OPTION + (byte >> 4) + (byte & 0xF),
            Opcode::JMP_IMM => OPCODE + OPTION + (byte >> 4),
            Opcode::JMP_REG => OPCODE + REG,
            Opcode::JSR => OPCODE + OPTION + (byte >> 4),
            Opcode::CMP_EQ_REG_REG |
            Opcode::CMP_LE_REG_REG |
            Opcode::CMP_GE_REG_REG |
            Opcode::CMP_LT_REG_REG |
            Opcode::CMP_GT_REG_REG => OPCODE + REG + REG,
            Opcode::CMP_EQ_REG_IMM |
            Opcode::CMP_LE_REG_IMM |
            Opcode::CMP_GE_REG_IMM |
            Opcode::CMP_LT_REG_IMM |
            Opcode::CMP_GT_REG_IMM => OPCODE + OPTION + REG + (byte >> 4),
            Opcode::AND |
            Opcode::ADD |
            Opcode::SUB |
            Opcode::MUL |
            Opcode::DIV |
            Opcode::FADD |
            Opcode::FSUB |
            Opcode::FMUL |
            Opcode::FDIV => {
                match byte >> 6 {
                    0b00 => OPCODE + OPTION + REG + REG + REG,
                    0b01 => OPCODE + OPTION + REG + REG + (byte & 0xF),
                    0b10 => OPCODE + OPTION + REG + 2 * (byte & 0xF),
                    _ => panic!("Unexpected option passed")
                }
            },
            Opcode::NOT => {
                match byte >> 6 {
                    0b00 => OPCODE + OPTION  + REG + REG,
                    0b01 => OPCODE + OPTION  + REG + (byte & 0xF),
                    _ => panic!("Unexpecte option passed")
                }
            },
            Opcode::CAL => OPCODE + 1,
            Opcode::FILE_LOAD => OPCODE + byte,
            _ => OPCODE
        }
    }
}

impl<'a> fmt::Display for Instruction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut space = "";

        for byte in self.bytes.iter() {
            f.write_str(space)?;
            f.write_str(&format!("{:02X}", byte))?;
            space = " ";
        }

        Ok(())
    }
}

#[test]
fn test_instruction_tostring() {
    let instruction = Instruction::with_data(
        Opcode::MOV_REG_REG,
        &[0, 0x68, 0x69, 0x21]
    );

    assert_eq!(instruction.to_string(), "00 68 69 21")
}

#[test]
fn test_instruction_get_size() {
    let opcodes: [Opcode; 17] = [
        Opcode::MOV_REG_REG,
        Opcode::MOV_REG_MEM,
        Opcode::MOV_REG_IMM,
        Opcode::MOV_MEM_MEM,
        Opcode::SWP,
        Opcode::JMP_IMM,
        Opcode::JMP_REG,
        Opcode::JSR,
        Opcode::CMP_EQ_REG_REG,
        Opcode::CMP_EQ_REG_IMM,
        Opcode::ADD,
        Opcode::ADD,
        Opcode::ADD,
        Opcode::NOT,
        Opcode::NOT,
        Opcode::CAL,
        Opcode::FILE_LOAD
    ];

    let bytes: [u8; 17] = [
        0, // Values with 0 are not expecting a byte
        0,
        0b01000000, // MOV_REG_IMM, byte encodes how large IMM is
        0b00110100, // MOV_MEM_MEM, byte encodes how large mem addrs are
        0b00110011, // SWP
        0b00110000, // JMP_IMM, byte encodes how large IMM is
        0,
        0b01000000, // JSR, byte encodes how large mem addr is
        0,
        0b00110000, // CMP_*_REG_IMM, byte encodes how large IMM is

        // ADD, byte encodes option (2 most significant bits)
        // and then the size of IMM if applicable
        0b00_000000,
        0b01_00_0100,
        0b10_00_0011,
        // NOT, encodes options (2 most signficant bits),
        // and then the size of IMM if applicable
        0b00_000000,
        0b01_000011,
        0,
        0b00000100 // FILE_LOAD, byte encodes mem addr
    ];

    let expected: [u8; 17] = [
        OPCODE + REG + REG, // 3
        OPCODE + REG + MEM, // 6
        OPCODE + OPTION + REG + 4, // 6
        OPCODE + OPTION + 3 + 4, // 8
        OPCODE + OPTION + 6, // 7
        OPCODE + OPTION + 3, // 4
        OPCODE + REG, // 2
        OPCODE + OPTION + 4, // 5
        OPCODE + REG + REG, // 3
        OPCODE + OPTION + REG + 3, // 5
        OPCODE + OPTION + REG + REG + REG, // 4
        OPCODE + OPTION + REG + REG + 4, // 7
        OPCODE + OPTION + REG + 2 * (3), // 8
        OPCODE + OPTION + REG + REG, // 3
        OPCODE + OPTION + REG + 3, // 5
        OPCODE + 1, // 2
        OPCODE + 4 // 5
    ];

    assert_eq!(Instruction::get_size(opcodes[0], bytes[0]), expected[0]);
    assert_eq!(Instruction::get_size(opcodes[1], bytes[1]), expected[1]);
    assert_eq!(Instruction::get_size(opcodes[2], bytes[2]), expected[2]);
    assert_eq!(Instruction::get_size(opcodes[3], bytes[3]), expected[3]);
    assert_eq!(Instruction::get_size(opcodes[4], bytes[4]), expected[4]);
    assert_eq!(Instruction::get_size(opcodes[5], bytes[5]), expected[5]);
    assert_eq!(Instruction::get_size(opcodes[6], bytes[6]), expected[6]);
    assert_eq!(Instruction::get_size(opcodes[7], bytes[7]), expected[7]);
    assert_eq!(Instruction::get_size(opcodes[8], bytes[8]), expected[8]);
    assert_eq!(Instruction::get_size(opcodes[9], bytes[9]), expected[9]);
    assert_eq!(Instruction::get_size(opcodes[10], bytes[10]), expected[10]);
    assert_eq!(Instruction::get_size(opcodes[11], bytes[11]), expected[11]);
    assert_eq!(Instruction::get_size(opcodes[12], bytes[12]), expected[12]);
    assert_eq!(Instruction::get_size(opcodes[13], bytes[13]), expected[13]);
    assert_eq!(Instruction::get_size(opcodes[14], bytes[14]), expected[14]);
    assert_eq!(Instruction::get_size(opcodes[15], bytes[15]), expected[15]);
}

#[test]
fn test_opcode_from_u8() {
    let valid_opcode = Opcode::from_u8(0);
    let invalid_opcode = Opcode::from_u8(123);

    assert!(valid_opcode.unwrap() == Opcode::MOV_REG_REG);
    assert!(invalid_opcode == None);
}