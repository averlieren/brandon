#![allow(dead_code)]
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

pub fn read(path: &str) -> Vec<u8>{
    fs::read(path)
        .expect(
            &format!("Cannot open {}", path)
        )
}

pub fn u64_to_u8arr(int: u64) -> [u8; 8] {
    [
        (int >> 56 & 0xFF) as u8,
        (int >> 48 & 0xFF) as u8,
        (int >> 40 & 0xFF) as u8,
        (int >> 32 & 0xFF) as u8,
        (int >> 24 & 0xFF) as u8,
        (int >> 16 & 0xFF) as u8,
        (int >> 08 & 0xFF) as u8,
        (int       & 0xFF) as u8
    ]
}

pub fn u8arr_to_u32(bytes: &[u8]) -> u32 {
    // Converts a u8 slice (len <= 4) to a u32 int
    let mut num: u32 = 0;

    for i in (0..=3).rev() {
        let mut byte: u8 = 0;
        let offset = 4 - bytes.len(); // use offset to preserve byte order

        // Offset must <= i so we don't overflow with subtraction
        if offset <= i && (i - offset) < bytes.len() {
            byte = bytes[i - offset];
        }

        num |= (byte as u32) << 24 - (8 * i);
    }

    num
}

pub fn u8arr_to_u64(bytes: &[u8]) -> u64 {
    // Converts a u8 slice (len <= 8) to a u64 int
    if bytes.len() <= 4 {
        u8arr_to_u32(bytes) as u64
    } else {
        let mut num1: Vec<u8> = Vec::with_capacity(4);
        let mut num2: Vec<u8> = Vec::with_capacity(4);

        for i in 0..=3 {
            if i < bytes.len() {
                num1.push(bytes[i]);
            }
        }
        for i in 4..=7 {
            if i < bytes.len() {
                num2.push(bytes[i]);
            }
        }

        let shift = 8 * (bytes.len() - 4);
        (u8arr_to_u32(&num1) as u64) << shift | u8arr_to_u32(&num2) as u64
    }
}

#[test]
fn test_u8arr_to_u32() {
    assert_eq!(
        u8arr_to_u32(&[0, 0, 41, 41]),
        0x2929
    );

    assert_eq!(
        u8arr_to_u32(&[0x29, 0x29]),
        0x2929
    )
}

#[test]
fn test_u8arr_to_u64() {
    assert_eq!(
        u8arr_to_u64(&[9, 10, 41, 41, 0, 0, 41, 41]),
        0x090A292900002929
    );

    assert_eq!(
        u8arr_to_u64(&[41, 10, 0, 0, 41, 41]),
        0x290A00002929
    );

    assert_eq!(
        u8arr_to_u64(&[0, 0, 41, 41]),
        0x2929
    );

    assert_eq!(
        u8arr_to_u64(&[0x29, 0x29]),
        0x2929
    )
}

#[test]
fn test_u64_to_u8arr() {
    let num: u64 = 0xF0E1D2C3B4A59687;
    let expected: [u8; 8] = [
        0xF0, 0xE1, 0xD2, 0xC3, 0xB4, 0xA5, 0x96, 0x87
    ];

    assert_eq!(u64_to_u8arr(num), expected);
}