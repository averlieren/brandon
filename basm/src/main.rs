extern crate bvm;
extern crate regex;

use std::u32;
use std::collections::HashSet;
use bvm::instructions::{Call, Instruction, Opcode};
use regex::Regex;

struct Label {
    name: String,
    addr: u32
}

fn match_opcode(opcode: &str) -> Opcode {
    match opcode {
        "MOV" => Opcode::MOV,
        "MEX" => Opcode::MOV,
        "MRX" => Opcode::MOV,
        "MMX" => Opcode::MOV,
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
        _ => Opcode::INVALID
    }
}


fn main() {
    let mut buf: Vec<u32> = Vec::new();
    //let mut labels = HashSet::new();
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

    let label = Regex::new(
        r"([^\s]*?):"
    ).unwrap();

    for line in asm {
        let line = comment.replace_all(line, "");
        let split: Vec<&str> = line.split(" ").collect();

        // check if label
        let lbl = label.captures(split[0]);
        if lbl.is_some() {
            let lbl = lbl.unwrap().get(1).map_or("", |m| m.as_str());
            println!("{}", lbl);
        } else {
            match split[0] {
                "#LFH" => {
                    let lfh = split[1];
                    let lfh = lfh.trim_start_matches("0x");
                    let lfh = u32::from_str_radix(lfh, 16).unwrap();
    
                    addr = lfh;
                    buf.push(lfh)
                },
                "#END" => {},
                "PNT" => {
                    buf.push(
                        0x12_00_00_00 | Call::PNT as u32
                    )
                },
                "HLT" => {
                    buf.push(
                        0x12_00_00_00 | Call::HLT as u32
                    )
                },
                _ => {
                    let opcode = match_opcode(split[0]);
                    println!("{}", opcode as u8);
                }
            }
        }
    }
}