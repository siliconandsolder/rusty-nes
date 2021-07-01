#[macro_use]
extern crate clap;

use clap::App;
use rustynes::console::Console;
use std::path::Path;
use rustynes::clock::Clocked;
use std::fs::File;


fn main() {
    let yaml = load_yaml!("./config/clap_args.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let rom = matches.value_of("ROM");
    let mut console = Console::new(rom);
    console.cycle();
}
