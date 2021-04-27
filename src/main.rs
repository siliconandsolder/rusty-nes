use nes::console::Console;
use std::path::Path;
use nes::clock::Clocked;
use std::fs::File;

fn main() {
    // TODO: Fix SHY, SHX
    let path = Path::new("./ff.nes");
    let mut console = Console::new(path);
    console.cycle();
}
