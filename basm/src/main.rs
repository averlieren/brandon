extern crate bvm;
extern crate regex;

use std::u32;
use bvm::instructions::{Instruction, Opcode};
use regex::Regex;

fn match_opcode(opcode: &str) -> Opcode {
    match opcode {
        "MOV" => Opcode::MOV,
        "MEX" => Opcode::MEX,
        "MRX" => Opcode::MRX,
        "MMX" => Opcode::MMX,
        "NIL" => Opcode::NIL,
        "LFX" => Opcode::LFX,
        "SWX" => Opcode::SWX,
        "JMP" => Opcode::JMP,
        "JSR" => Opcode::JSR,
        "CMP" => Opcode::CMP,
        "CMZ" => Opcode::CMZ,
        "ARG" => Opcode::ARG,
        "ADD" => Opcode::ADD,
        "SUB" => Opcode::SUB,
        "MUL" => Opcode::MUL,
        "DIV" => Opcode::DIV,
        "AND" => Opcode::AND,
        "NOT" => Opcode::NOT,
        "CAL" => Opcode::CAL,
        "JPA" => Opcode::JPA,
        "FLX" => Opcode::FLX,
        "ILX" => Opcode::ILX,
        _ => panic!(
            format!("Unknown opcode {}", opcode)
        )
    }
}

fn main() {
    let mut addr: u32 = 0;
    let asm: Vec<&str> = vec![
        "#LFH 0x002929 ; load file at address 0x2929",
        "MRX R00 ; move the data supplied by arg to R00",
        "ARG STR ; the data is the start address of a string",
        "PNT ; print string starting at address stored in R00",
        "HLT ; halt program",
        "STR: #STR \"hello world\\n\" ; the string",
        "#END ; end of file"
    ];

    let comment = Regex::new(
        r";.*"
    ).unwrap();



    for line in asm {
        let line = comment.replace_all(line, "");
        let split: Vec<&str> = line.split(" ").collect();
        match split[0] {
            "#LFH" => {
                let lfh = split[1];
                let lfh = lfh.trim_start_matches("0x");
                let lfh = u32::from_str_radix(lfh, 16).unwrap();

                addr = lfh;
                println!("LFH: {}", addr);
            },
            "#END" => {},
            "PNT" | "HLT" => {},
            _ => {
                let opcode = match_opcode(split[0]);
                println!("{}", opcode as u8);
            }
        }
    }
}