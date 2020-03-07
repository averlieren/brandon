extern crate bvm;
extern crate regex;

use std::u32;
use bvm::instructions::{Call, Instruction, Opcode};
use regex::Regex;

fn match_opcode(token: &str) -> Opcode {
    match token {
        "MOV" | "MEX" | "MRX" | "MMX" | "MIX"=> Opcode::MOV,
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

fn match_call(token: &str) -> Call {
    match token {
        "INP" => Call::INP,
        "OUT" => Call::OUT,
        "PNT" => Call::PNT,
        "HLT" => Call::HLT,
        _ => Call::INVALID
    }
}

fn decode(line: &str) -> u32 {
    let comment = Regex::new(
        r";.*"
    ).unwrap();

    let line = comment.replace_all(line, "");
    let tokens: Vec<&str> = line.split(" ").collect();
    let mut instruction = 0;

    if match_opcode(tokens[0]) != Opcode::INVALID {
        instruction = (match_opcode(tokens[0]) as u32) << 24;
    } else if match_call(tokens[0]) != Call::INVALID {
        instruction = (Opcode::CAL as u32) << 24
            | (match_call(tokens[0]) as u32);
    } else {
        // TODO
    }

    instruction
}

fn main() {
    let asm: Vec<&str> = vec![
        "#LFH 0x002929 ; load file at address 0x2929",
        "MRX R00 ; move the data supplied by arg to R00",
        "ARG STR ; the data is the start address of a string",
        "PNT ; print string starting at address stored in R00",
        "HLT ; halt program",
        "STR: #STR \"hello world\\n\" ; the string",
        "#END ; end of file"
    ];
}

#[test]
fn test_decode_call() {
    assert_eq!(decode("HLT"), 0x1200009D);
}

#[test]
fn test_decode_opcode() {
    assert_eq!(decode("MOV R00 R01") >> 24, 0);
    assert_eq!(decode("ARG 123456789") >> 24, 0x0B);
}