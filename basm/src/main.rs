extern crate bvm;
extern crate regex;

use std::u32;
use std::cell::RefCell;
use std::collections::HashMap;
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

enum TokenType {
    DIRECTIVE,
    STRING,
    NUMBER,
    WORD
}

impl<'a> Tokenizer<'a> {
    fn new() -> Tokenizer<'a> {
        Tokenizer {
            tokens: RefCell::new(Vec::with_capacity(128)),
            data: "",
            head: 0
        }
    }

    fn load(&mut self, data: &'a str) {
        self.data = data;
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
            println!("{} {}", self.head, self.data.len());
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

fn main() {
    let asm = "#LFH 0x002929\nMRX R29\nARG STR\nMOV R00 R28\nMEX\nARG 0x002938\nARG 0x002939\nMMX R01\nARG 0x002939\nPNT\nHLT\nSTR #STR \"hello world\\n\"\n";

    let mut tokenizer = Tokenizer::new();
    tokenizer.load(&asm);
    tokenizer.tokenize();

    for token in tokenizer.tokens.borrow().iter() {
        println!("{}", token.val);
    }
}