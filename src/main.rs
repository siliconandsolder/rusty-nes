use rustynes::console::Console;
use std::path::Path;
use rustynes::clock::Clocked;
use std::fs::File;

#[macro_use]
extern crate clap;
use clap::App;

fn main() {
    let yaml = load_yaml!("./config/clap_args.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let rom = matches.value_of("ROM").unwrap();
    let path = Path::new(rom);
    let mut console = Console::new(path);
    console.cycle();
}
