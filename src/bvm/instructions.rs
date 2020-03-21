use std::cell::RefCell;
use std::fmt;

#[allow(non_camel_case_types)]
#[derive(PartialEq, FromPrimitive)]
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

pub struct Instruction {
    opcode: Opcode,
    bytes: RefCell<Vec<u8>>
}

impl Instruction {
    pub fn new() -> Instruction {
        // Create arbitrary new instruction
        Instruction {
            opcode: Opcode::INVALID,
            bytes: RefCell::new(Vec::with_capacity(0))
        }
    }

    pub fn with_data(opcode: Opcode, bytes: Vec<u8>) -> Instruction {
        // Create instruction with data
        Instruction {
            opcode: opcode,
            bytes: RefCell::new(bytes)
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut space = "";

        for byte in self.bytes.borrow().iter() {
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
        vec![0, 0x68, 0x69, 0x21]
    );

    assert_eq!(instruction.to_string(), "00 68 69 21")
}

#[test]
fn test_opcode_from_u8() {
    let valid_opcode = Opcode::from_u8(0);
    let invalid_opcode = Opcode::from_u8(123);

    assert!(valid_opcode.unwrap() == Opcode::MOV_REG_REG);
    assert!(invalid_opcode == None);
}