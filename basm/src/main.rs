extern crate bvm;
extern crate regex;

use std::u32;
use std::cell::RefCell;
use bvm::instructions::{Call, Opcode};
use regex::Regex;

struct Token(String);
struct Tokenizer<'a> {
    data: RefCell<Vec<&'a str>>,
    tokens: RefCell<Vec<Token>>,
    addr: u32
}

impl Token {
    fn new(data: String) -> Token {
        Token(data)
    }

    fn get_type(&self) -> TokenType {
        if self.0.starts_with("#") {
            TokenType::DIRECTIVE
        } else if match_opcode(&self.0) != Opcode::INVALID {
            TokenType::OPCODE
        } else if match_call(&self.0) != Call::INVALID {
            TokenType::CALL
        } else  {
            TokenType::LABEL
        }
    }

    fn get(&self) -> &str {
        &self.0
    }
}

enum TokenType {
    DIRECTIVE = 0,
    OPCODE,
    CALL,
    ARGUMENT,
    LABEL
}

impl<'a> Tokenizer<'a> {
    fn new() -> Tokenizer<'a> {
        Tokenizer {
            data: RefCell::new(Vec::new()),
            tokens: RefCell::new(Vec::new()),
            addr: 0
        }
    }

    fn load(&self, data: Vec<String>) {
        let comment = Regex::new(
            r";.*"
        ).unwrap();

        for line in data {
            let line = comment.replace_all(&line, "");
            let split: Vec<&str> = line.split(" ").collect();

            for t in split {
                self.tokens.borrow_mut().push(Token::new(t.to_owned()));
            }
        }
    }

    fn next(&self) -> Token {
        self.tokens.borrow_mut().remove(0)
    }

    fn decode(&self) -> u32 {
        let mut instruction: u32 = 0;
        let token = self.next();

        match token.get_type() {
            TokenType::DIRECTIVE => {
                match token.get() {
                    "#LFH" => {},
                    "#END" => {}
                    _ => {}
                }
            },
            TokenType::OPCODE => {
                match match_opcode(token.get()) {
                    Opcode::MOV => {
                        instruction |= 1 << 23;
    
                        match token.get() {
                            "MOV" => {
                                instruction |= 0b000001 << 17;
                            },
                            "MEX" => instruction |= 0b000010 << 17,
                            "MRX" => instruction |= 0b000011 << 17,
                            "MMX" => instruction |= 0b000100 << 17,
                            "MIX" => instruction |= 0b000101 << 17,
                            "MFX" => instruction |= 0b000110 << 17,
                            _ => {}
                        }
                    },
                    Opcode::SWX => {},
                    Opcode::JMP => {
                        if token.get() == "RET" {
                            instruction |= 0b11101;
                        }
                    },
                    Opcode::JSR => {},
                    Opcode::CMP => {
                        match token.get() {
                            "CEQ" => {},
                            "CEL" => {},
                            "CEG" => {},
                            "CLT" => {},
                            "CGT" => {},
                            _ => {}
                        }
                    },
                    Opcode::CMZ => {
                        match token.get() {
                            "CEZ" => {},
                            "CNZ" => {},
                            "CPZ" => {},
                            "CLZ" => {},
                            "CGZ" => {},
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
            },
            TokenType::CALL => {
                instruction = (Opcode::CAL as u32) << 24;
                instruction |= match_call(token.get()) as u32;
            },
            TokenType::ARGUMENT => {},
            _ => {}
        }

        instruction
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
    let asm: Vec<String> = vec![
        "#LFH 0x002929   ; load file at 0x002929".to_owned(),
        "MRX R28         ; load into R28".to_owned(),           // 00 86 00 1C
        "ARG STR         ; the address of STR".to_owned(),      // 0B 00 29 33
        "MOV R00 R28     ; move data of R28 to R00".to_owned(), // 00 82 00 1C
        "MEX             ; move data in memory".to_owned(),     // 00 84 00 00
        "ARG 0x002938    ; to 0x002938".to_owned(),             // 0B 00 29 38
        "ARG 0x002939    ; from 0x002939".to_owned(),           // 0B 00 29 39
        "MMX R01         ; move data from R01".to_owned(),      // 00 88 00 01
        "ARG 0x002939    ; to memory at 0x002939".to_owned(),   // 0B 00 29 39
        "PNT             ; print starting at stored address in R00".to_owned(),
                                                                // 12 00 00 9A
        "HLT             ; halt program".to_owned(),            // 12 00 00 9D
        "STR: #STR \"hello world\\n\"".to_owned(),
        "#END            ; end of file".to_owned()
    ];

    let tokenizer = Tokenizer::new();
    tokenizer.load(asm);
    println!("{}", tokenizer.decode());
}

/*
#[test]
fn test_decode_call() {
    assert_eq!(decode("HLT"), 0x1200009D);
}

#[test]
fn test_decode_opcode() {
    assert_eq!(decode("MEX R00 R01") >> 24, 0);
    assert_eq!(decode("ARG 123456789") >> 24, 0x0B);
}
*/