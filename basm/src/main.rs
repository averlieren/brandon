extern crate bvm;

use std::u32;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::slice;
use std::mem;
use bvm::instructions::{Call, Opcode};

struct Token {
    r#type: TokenType,
    val: String
}

struct Tokenizer<'a> {
    tokens: RefCell<Vec<Token>>,
    data: &'a str,
    head: usize
}

struct Assembler<'a> {
    tokens: &'a Vec<Token>,
    addr: u32
}

#[derive(PartialEq)]
enum TokenType {
    DIRECTIVE,
    STRING,
    NUMBER,
    WORD
}

impl<'a> Tokenizer<'a> {
    fn new(data: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            tokens: RefCell::new(Vec::with_capacity(128)),
            data: data,
            head: 0
        }
    }

    fn cur(&self) -> &'a str {
        &self.data.get(self.head..self.head + 1).unwrap()
    }

    fn peak(&self) -> &'a str {
        if self.head + 2 < self.data.len() {
            &self.data.get(self.head + 1 .. self.head + 2).unwrap()
        } else {
            ""
        }
    }

    fn next(&mut self) -> &'a str {
        self.incr();
        self.cur()
    }

    fn incr(&mut self) {
        self.head += 1;
    }

    fn tokenize(&mut self) {
        while self.head < self.data.len() {
            match self.cur() {
                "#" => {
                    let mut directive = String::new();
                    
                    loop {
                        let next = self.peak();
                        if next != " " && next != "\n" && next != "\t" {
                            directive += self.next();
                        } else {
                            break;
                        }
                    }

                    &self.tokens.borrow_mut().push(
                        Token {
                            r#type: TokenType::DIRECTIVE,
                            val: directive
                        }
                    );
                },
                "0" | "R" => {
                    let mut number = String::new();
                    let mut radix: u32 = 10;

                    match self.peak() {
                        "x" => radix = 16,
                        "o" => radix = 8,
                        "b" => radix = 2,
                        _ if self.peak().parse::<u32>().is_ok() => {
                            self.head -= 1;
                        },
                        _ if self.cur() == "0" => {
                            &self.tokens.borrow_mut().push(
                                Token {
                                    r#type: TokenType::NUMBER,
                                    val: "0".to_owned()
                                }
                            );

                            self.incr();
                            continue;
                        }
                        _ => {
                            self.incr();
                            continue;
                        }
                    }

                    self.incr();

                    loop {
                        if self.peak().parse::<u32>().is_ok() {
                            number += self.next();
                        } else {
                            break;
                        }
                    }

                    let number = u32::from_str_radix(&number, radix).unwrap();

                    &self.tokens.borrow_mut().push(
                        Token {
                            r#type: TokenType::NUMBER,
                            val: number.to_string()
                        }
                    );
                },
                "\"" => {
                    let mut string = String::new();

                    loop {
                        let next = self.next();

                        if next != "\"" {
                            string += next;
                        } else {
                            break;
                        }
                    }

                    &self.tokens.borrow_mut().push(
                        Token {
                            r#type: TokenType::STRING,
                            val: string
                        }
                    );
                },
                " " | "\n" | "\t" => {},
                _ => {
                    let mut word = String:: new();

                    word += self.cur();

                    loop {
                        let next = self.peak();

                        if next != " " && next != "\n" && next != "\t" {
                            word += self.next();
                        } else {
                            break;
                        }
                    }

                    &self.tokens.borrow_mut().push(
                        Token {
                            r#type: TokenType::WORD,
                            val: word
                        }
                    );
                }
            }

            self.incr();
        }
    }
}

impl<'a> Assembler<'a> {
    fn new(tokens: &'a Vec<Token>) -> Assembler<'a> {
        Assembler {
            tokens: tokens,
            addr: 0
        }
    }

    fn assemble(&mut self) -> HashMap<u32, u32> {
        let mut buf: HashMap<u32, u32> = HashMap::with_capacity(128);
        let mut labels: HashMap<String, Vec<u32>> = HashMap::with_capacity(32);
        let mut tokens = self.tokens.iter();

        loop {
            let token = tokens.next();

            if token.is_none() {    
                break;
            }

            let token = token.unwrap();

            match token.r#type {
                TokenType::DIRECTIVE => {
                    match token.val.as_str() {
                        "LFH" => {
                            self.addr = tokens.next().unwrap().val.parse::<u32>().unwrap();
                        },
                        "STR" => {
                            let string = &tokens.next().unwrap().val;
                            let string: Vec<u16> = string.encode_utf16().collect();

                            for i in (0..string.len()).step_by(2) {
                                let mut mem = (*string.get(i).unwrap() as u32) << 16;

                                if i + 1 < string.len() {
                                    mem |= *string.get(i + 1).unwrap() as u32;
                                }

                                if !&buf.contains_key(&self.addr) {
                                    &buf.insert(self.addr, mem);
                                }

                                self.addr += 1;
                            }
                        },
                        _ => {}
                    }
                },
                TokenType::WORD => {
                    if match_opcode(&token.val) != Opcode::INVALID {
                        let mut instruction: u32 = (match_opcode(&token.val) as u32) << 24;

                        match match_opcode(&token.val) {
                            Opcode::MOV => {
                                instruction |= 1 << 23;

                                match token.val.as_str() {
                                    "MOV" => {
                                        let dst = tokens.next().unwrap().val.parse::<u32>().unwrap();
                                        let src = tokens.next().unwrap().val.parse::<u32>().unwrap();

                                        instruction |= 0b000001 << 17;
                                        instruction |= dst << 12;
                                        instruction |= src;
                                    },
                                    "MEX" => instruction |=0b000010 << 17,
                                    "MRX" => {
                                        let dst = tokens.next().unwrap().val.parse::<u32>().unwrap();

                                        instruction |= 0b000011 << 17;
                                        instruction |= dst;
                                    },
                                    "MMX" => {
                                        let src = tokens.next().unwrap().val.parse::<u32>().unwrap();

                                        instruction |= 0b000100 << 17;
                                        instruction |= src;
                                    },
                                    "MIX" => {
                                        let data = tokens.next().unwrap().val.parse::<u32>().unwrap();
                                        
                                        instruction |= 0b000101 << 17;
                                        instruction |= data;
                                    },
                                    "MFX" => {
                                        let dst = tokens.next().unwrap().val.parse::<u32>().unwrap();

                                        instruction |= 0b000110 << 17;
                                        instruction |= dst;
                                    },
                                    _ => {}
                                }
                            },
                            Opcode::SWX => {},
                            Opcode::JMP => {
                                if &token.val == "RET" {
                                    instruction |= 0b11101;
                                }
                            },
                            Opcode::JSR => {
                                let next = tokens.next().unwrap();

                                if next.r#type == TokenType::NUMBER {
                                    instruction |= next.val.parse::<u32>().unwrap();
                                }
                            },
                            Opcode::CMP => {
                                let mut flag = 0;

                                match token.val.as_str() {
                                    "CEQ" => flag = 0b001,
                                    "CEL" => flag = 0b010,
                                    "CEG" => flag = 0b011,
                                    "CLT" => flag = 0b100,
                                    "CGT" => flag = 0b101,
                                    _ => {}
                                }

                                instruction |= flag << 21;

                                let cmp1 = tokens.next().unwrap();
                                let cmp2 = tokens.next().unwrap();

                                instruction |= cmp1.val.parse::<u32>().unwrap() << 12;
                                instruction |= cmp2.val.parse::<u32>().unwrap();
                            },
                            Opcode::CMZ => {
                                match token.val.as_str() {
                                    "CEZ" => {},
                                    "CNZ" => {},
                                    "CPZ" => {},
                                    "CLZ" => {},
                                    "CGZ" => {},
                                    _ => {}
                                }
                            },
                            Opcode::ARG => {
                                let next = tokens.next().unwrap();

                                if next.r#type == TokenType::NUMBER {
                                    instruction |= next.val.parse::<u32>().unwrap();
                                } else if next.r#type == TokenType::WORD {
                                    if !labels.contains_key(&next.val) {
                                        labels.insert(next.val.to_owned(), Vec::new());
                                    }

                                    labels.get_mut(&next.val).unwrap().push(self.addr);
                                }
                            },
                            Opcode::ADD | Opcode::SUB | Opcode::MUL |
                            Opcode::DIV | Opcode::AND => {
                                let dst = tokens.next().unwrap().val.parse::<u32>().unwrap();
                                let rega = tokens.next().unwrap().val.parse::<u32>().unwrap();
                                let regb = tokens.next().unwrap().val.parse::<u32>().unwrap();

                                instruction |= dst << 16;
                                instruction |= rega << 8;
                                instruction |= regb;
                            },
                            Opcode::NOT => {
                                let dst = tokens.next().unwrap().val.parse::<u32>().unwrap();
                                let rega = tokens.next().unwrap().val.parse::<u32>().unwrap();

                                instruction |= dst << 16;
                                instruction |= rega;
                            },
                            Opcode::CAL => {
                                let vec = tokens.next().unwrap().val.parse::<u32>().unwrap();

                                instruction |= vec;
                            },
                            Opcode::JPA => {
                                let addr = tokens.next().unwrap().val.parse::<u32>().unwrap();

                                instruction |= addr;
                            },
                            Opcode::FLX => {},
                            Opcode::ILX => {}
                            _ => {}
                        }

                        if !&buf.contains_key(&self.addr) {
                            &buf.insert(self.addr, instruction);
                        }

                        self.addr += 1;
                    } else if match_call(&token.val) != Call::INVALID {
                        let mut instruction: u32 = (Opcode::CAL as u32) << 24;
                        instruction |= match_call(&token.val) as u32;

                        if !&buf.contains_key(&self.addr) {
                            &buf.insert(self.addr, instruction);
                        }

                        self.addr += 1;
                    } else {
                        for item in &labels {
                            let label = item.0;

                            if label == &token.val {
                                let addrs = item.1;

                                for addr in addrs {
                                    let mut instruction = buf.get(addr).unwrap().to_owned();
                                    instruction |= self.addr;
                                    buf.remove(addr);
                                    buf.insert(*addr, instruction);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        buf
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

fn main() -> std::io::Result<()> {
    let asm = "#LFH 0x002929\nMRX R28\nARG STR\nMOV R00 R28\nMEX\nARG 0x002938\nARG 0x002939\nMMX R01\nARG 0x002939\nPNT\nHLT\nSTR #STR \"hello worljjd\n\"\n";

    let mut tokenizer = Tokenizer::new(&asm);
    tokenizer.tokenize();

    let tokens = tokenizer.tokens.borrow();
    let mut assembler = Assembler::new(&tokens);
    let buf = assembler.assemble();

    let mut addresses: Vec<&u32> = buf.keys().collect();
    let mut instructions: Vec<u32> = Vec::with_capacity(addresses.len());
    addresses.sort();
    
    for addr in &addresses {
        instructions.push(buf.get(addr).unwrap().to_owned());
    }

    let instructions: &[u32] = &*instructions;

    let instructions: &[u8] = unsafe {
        slice::from_raw_parts(instructions.as_ptr() as *const u8, instructions.len() * mem::size_of::<u32>())
    };

    let mut bin = File::create("out.bin")?;

    // LFH
    let lfh: [u8; 3] = [
        (*addresses[0] >> 16) as u8 & 0xFF,
        (*addresses[0] >> 8) as u8 & 0xFF,
        (*addresses[0] & 0xFF) as u8
    ];

    bin.write_all(&lfh)?;
    bin.write_all(&instructions)?;

    Ok(())
}