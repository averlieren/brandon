struct Token {
    r#type: TokenType,
    val: String
}

struct Tokenizer<'a> {
    tokens: &'a [Token],
    data: &'a str,
    addr: usize
}

struct Assembler<'a> {
    tokens: &'a [Token],
    addr: u32
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
impl<'a> Assembler<'a> {}