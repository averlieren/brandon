#![allow(dead_code)]

pub struct Assembler<'a> {
    tokens: &'a [Token],
    addr: u32
}

impl<'a> Assembler<'a> {}