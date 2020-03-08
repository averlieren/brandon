extern crate bvm;
extern crate regex;

use std::u32;
use std::cell::RefCell;
use bvm::instructions::{Call, Opcode};
use regex::Regex;

struct Tokenizer<'a>(RefCell<Vec<&'a str>>);
struct Token<'a>(&'a str);

impl<'a> Token<'a> {
    fn is_register(&self) -> bool {
        self.0.starts_with("R")
    }
}

impl<'a> Tokenizer<'a> {
    fn new() -> Tokenizer<'a> {
        Tokenizer(RefCell::new(Vec::new()))
    }
}

fn match_opcode(token: &str) -> Opcode {
    match token {
        "MOV" | "MEX" | "MRX" | "MMX" | "MIX" | "LFX" => Opcode::MOV,
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
    let mut instruction: u32 = 0;

    if match_opcode(&tokens[0]) != Opcode::INVALID {
        instruction = (match_opcode(&tokens[0]) as u32) << 24;

        match match_opcode(&tokens[0]) {
            Opcode::MOV => {
                instruction |= 1 << 23;

                match &tokens[0] {
                    &"MOV" => {
                        instruction |= 0b000001 << 17;
                    },
                    &"MEX" => instruction |= 0b000010 << 17,
                    &"MRX" => instruction |= 0b000011 << 17,
                    &"MMX" => instruction |= 0b000100 << 17,
                    &"MIX" => instruction |= 0b000101 << 17,
                    &"MFX" => instruction |= 0b000110 << 17,
                    _ => {}
                }
            },
            Opcode::SWX => {},
            Opcode::JMP => {
                if &tokens[0] == &"RET" {
                    instruction |= 0b11101;
                }
            },
            Opcode::JSR => {},
            Opcode::CMP => {
                match &tokens[0] {
                    &"CEQ" => {},
                    &"CEL" => {},
                    &"CEG" => {},
                    &"CLT" => {},
                    &"CGT" => {},
                    _ => {}
                }
            },
            Opcode::CMZ => {
                match &tokens[0] {
                    &"CEZ" => {},
                    &"CNZ" => {},
                    &"CPZ" => {},
                    &"CLZ" => {},
                    &"CGZ" => {},
                    _ => {}
                }
            },
            Opcode::ARG => {},
            Opcode::ADD => {},
            Opcode::SUB => {},
            Opcode::MUL => {},
            Opcode::DIV => {},
            Opcode::AND => {},
            Opcode::NOT => {},
            Opcode::CAL => {},
            Opcode::JPA => {},
            Opcode::FLX => {},
            Opcode::ILX => {}
            _ => {}
        }
    } else if match_call(&tokens[0]) != Call::INVALID {
        instruction = (Opcode::CAL as u32) << 24;
        instruction |= match_call(&tokens[0]) as u32;
    } else {
        // TODO
    }

    instruction
}

fn main() {
    /*
    00 68 00 65
    00 6C 00 6C
    00 6F 00 20
    00 77 00 6F
    00 72 00 6C
    29 29 29 29
    00 64 00 0A
    */
    let asm: Vec<&str> = vec![
        "#LFH 0x002929   ; load file at 0x002929",
        "MRX R28         ; load into R28",              // 00 86 00 1C
        "ARG STR         ; the address of STR",         // 0B 00 29 33
        "MOV R00 R28     ; move data of R28 to R00",    // 00 82 00 1C
        "MEX             ; move data in memory",        // 00 84 00 00
        "ARG 0x002938    ; to 0x002938",                // 0B 00 29 38
        "ARG 0x002939    ; from 0x002939",              // 0B 00 29 39
        "MMX R01         ; move data from R01",         // 00 88 00 01
        "ARG 0x002939    ; to memory at 0x002939",      // 0B 00 29 39
        "PNT             ; print starting at stored address in R00",
                                                        // 12 00 00 9A
        "HLT             ; halt program",               // 12 00 00 9D
        "STR: #STR \"hello world\\n\"",
        "#END            ; end of file"
    ];

    for line in &asm {
        println!("{}", format!("{:#010X}", decode(&line)));
    }
}

#[test]
fn test_decode_call() {
    assert_eq!(decode("HLT"), 0x1200009D);
}

#[test]
fn test_decode_opcode() {
    assert_eq!(decode("MEX R00 R01") >> 24, 0);
    assert_eq!(decode("ARG 123456789") >> 24, 0x0B);
}