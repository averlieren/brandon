extern crate bvm;

use std::env;
use bvm::externals::read;
use bvm::vm::VM;

fn main() {
    if env::args().len() == 2 {
        let file_name = &env::args().collect::<Vec<String>>()[1];
        let mut data: Vec<u8> = read(file_name);
        let lfh: u32 =
            (data.remove(0) as u32) << 16 |
            (data.remove(0) as u32) << 8 |
            data.remove(0) as u32;

        let mut bvm = VM::new();
        bvm.mem.load(&mut data, lfh);
        bvm.run();
    }
}