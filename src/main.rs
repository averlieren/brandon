extern crate bvm;

use bvm::vm::VM;

fn main() {
    // Initialize virtual machine
    let mut bvm = VM::new();

    bvm.mem.write(0x00000000, 0x13_00_00_09);
    bvm.mem.write(0x00000001, 0x00_68_00_65);
    bvm.mem.write(0x00000002, 0x00_6C_00_6C);
    bvm.mem.write(0x00000003, 0x00_6F_00_5F);
    bvm.mem.write(0x00000004, 0x00_77_00_6F);
    bvm.mem.write(0x00000005, 0x00_72_00_6C);
    bvm.mem.write(0x00000006, 0x00_64_00_2E);
    bvm.mem.write(0x00000007, 0x00_62_00_69);
    bvm.mem.write(0x00000008, 0x00_6E_00_00);
    bvm.mem.write(0x00000009, 0x15_00_00_00);
    bvm.mem.write(0x0000000A, 0x0B_00_00_01);

    bvm.run();
}