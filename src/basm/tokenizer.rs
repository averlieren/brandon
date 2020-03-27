#![allow(dead_code)]

pub struct Token {
    pub r#type: TokenType,
    pub val: String
}

pub struct Tokenizer<'a> {
    tokens: &'a [Token],
    data: &'a str,
    pos: usize
}

#[derive(PartialEq, Debug)]
pub enum TokenType {
    DIRECTIVE,
    STRING,
    NUMBER,
    ADDRESS,
    REGISTER,
    WORD
}


impl<'a> Tokenizer<'a> {
    pub fn load(data: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            tokens: &[],
            data: data,
            pos: 0
        }
    }

    fn cur(&self) -> char {
        self.data.chars().nth(self.pos).unwrap()
    }

    fn peak(&self) -> char {
        self.data.chars().nth(self.pos + 1).unwrap()
    }

    fn next(&mut self) -> char {
        self.pos += 1;
        self.cur()
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::with_capacity(128);

        while self.pos < self.data.len() {
            match self.cur() {
                '#' => {
                    tokens.push(Token {
                        r#type: TokenType::DIRECTIVE,
                        val: self.match_until_whitespace()
                    });
                },
                '[' => {
                    tokens.push(Token {
                        r#type: TokenType::ADDRESS,
                        val: self.match_until(']')
                    });
                },
                'R' if self.peak().is_numeric() => {
                    self.pos += 1;

                    tokens.push(Token {
                        r#type: TokenType::REGISTER,
                        val: self.match_until_whitespace()
                    });
                },
                '0' if ['x', 'o', 'b'].contains(&self.peak()) => {
                    tokens.push(Token {
                        r#type: TokenType::NUMBER,
                        val: self.match_until_whitespace()
                    });
                },
                '"' => {
                    self.pos += 1;

                    tokens.push(Token {
                        r#type: TokenType::STRING,
                        val: self.match_until('"')
                    });
                },
                ';' => {
                    self.match_until('\n');
                },
                _ if self.cur().is_numeric() => {
                    tokens.push(Token {
                        r#type: TokenType::NUMBER,
                        val: self.match_until_whitespace()
                    })
                },
                _ if self.cur().is_whitespace() => {
                    self.next();
                }
                _ => {
                    tokens.push(Token {
                        r#type: TokenType::WORD,
                        val: self.match_until_whitespace()
                    })
                }
            }

            self.pos += 1;
        }

        tokens
    }

    fn match_until_whitespace(&mut self) -> String {
        let mut string = String::new();

        while !self.cur().is_whitespace() {
            if self.cur() == ';' {
                self.match_until('\n');
                break;
            }

            string.push(self.cur());
            self.pos += 1;
        }

        string
    }

    fn match_until(&mut self, end: char) -> String {
        let mut string = String::new();

        while self.cur() != end {
            string.push(self.cur());
            self.pos += 1;
        }

        string
    }
}

#[test]
fn test_tokenizer() {
    let data = "#LFH [0x2929]; this is a directive\nJMP [0x2929]\nLABEL MOV R00 0x292929\nSTRING #STR \"hello world\n\"";
    let mut tokenizer = Tokenizer::load(data);
    let tokens = tokenizer.tokenize();

    assert_eq!(tokens.len(), 11);
    assert_eq!(tokens[0].r#type, TokenType::DIRECTIVE);
    assert_eq!(tokens[1].r#type, TokenType::ADDRESS);
    assert_eq!(tokens[2].r#type, TokenType::WORD);
    assert_eq!(tokens[3].r#type, TokenType::ADDRESS);
    assert_eq!(tokens[4].r#type, TokenType::WORD);
    assert_eq!(tokens[5].r#type, TokenType::WORD);
    assert_eq!(tokens[6].r#type, TokenType::REGISTER);
    assert_eq!(tokens[7].r#type, TokenType::NUMBER);
    assert_eq!(tokens[8].r#type, TokenType::WORD);
    assert_eq!(tokens[9].r#type, TokenType::DIRECTIVE);
    assert_eq!(tokens[10].r#type, TokenType::STRING);
}