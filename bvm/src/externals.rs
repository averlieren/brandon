extern crate libc;

use std::fs;
use libc::c_int;

extern "C" {
    fn getchar() -> c_int;
}

pub fn get_char() -> i32 {
    unsafe {
        getchar()
    }
}

pub fn read(file: &str) -> Vec<u8>{
    fs::read(file)
        .expect(
            &format!("Cannot open {}", file)
        )
}