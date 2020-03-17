#[path = "bvm/vm.rs"]
pub mod bvm;

use bvm::memory::Memory;

fn main() {
    let mem = Memory::new();
    mem.write(2929, 2929);

    assert_eq!(mem.read(29), None);
    assert_ne!(mem.read(2929), None);
    assert_eq!(mem.read(2929).unwrap(), 2929);

    mem.write(2929, 2930);

    assert_eq!(mem.read(2929).unwrap(), 2930);

}