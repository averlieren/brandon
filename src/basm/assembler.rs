#![allow(dead_code)]
use super::tokenizer::{Token, TokenType};

pub struct Assembler<'a> {
    tokens: &'a [Token],
    index: usize,
    addr: u32
}

impl<'a> Assembler<'a> {
    pub fn load(tokens: &'a [Token]) -> Assembler<'a> {
        Assembler {
            tokens: tokens,
            index: 0,
            addr: 0
        }
    }

    fn cur(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn peak(&self) -> Option<&Token> {
        if self.index + 1 < self.tokens.len() {
            Some(&self.tokens[self.index + 1])
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<&Token> {
        if self.index + 1 < self.tokens.len() {
            self.index += 1;
            Some(&self.cur())
        } else {
            None
        }
    }

    pub fn assemble(&mut self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::with_capacity(self.tokens.len());
        let mut passes = 0;

        while self.index < self.tokens.len() {
            match self.cur().r#type {
                TokenType::DIRECTIVE => {},
                TokenType::STRING => {},
                TokenType::NUMBER => {},
                TokenType::ADDRESS => {},
                TokenType::REGISTER => {},
                TokenType::WORD => {}
            }
        }

        buf
    }
}

fn is_valid_instruction(string: &str) -> bool {
    let instructions: &[&str] = &[
        "mov", "swp", "jmp", "jsr", "ret", "cmpeq", "cmpge", "cmple", "cmpgt",
        "cmplt", "cmpeqz", "cmpgez", "cmplez", "cmpgtz", "cmpltz", "and", "add",
        "sub", "mul", "div", "fadd", "fsub", "fmul", "fdiv", "not", "cal", "flx"
    ];

    instructions.contains(&&string.to_lowercase().as_str())
}

fn is_valid_call(string: &str) -> bool {
    let calls: &[&str] = &[
        "hlt", "pnt"
    ];

    calls.contains(&&string.to_lowercase().as_str())
}