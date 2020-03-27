#![allow(dead_code)]

struct Token {
    r#type: TokenType,
    val: String
}

struct Tokenizer<'a> {
    tokens: &'a [Token],
    data: &'a str,
    addr: usize
}

#[derive(PartialEq)]
enum TokenType {
    DIRECTIVE,
    STRING,
    NUMBER,
    ADDRESS,
    REGISTER,
    WORD
}

impl<'a> Tokenizer<'a> {}