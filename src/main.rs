//extern crate sdl2;

// use sdl2::pixels::Color;
// use sdl2::rect::Rect;
// use rand::Rng;
// use std::time::Duration;

use simplelog::*;
use nes::console::Console;
use std::path::Path;
use nes::clock::Clocked;
use std::fs::File;

fn main() {
	// CombinedLogger::init(
	// 	vec![
	// 		WriteLogger::new(LevelFilter::Info, Config::default(), File::create("rusty_logs.txt").unwrap())
	// 	]
	// ).unwrap();

	let path = Path::new("./tests/instr_test-v5/rom_singles/01-basics.nes");
	//let path = Path::new("./loz.nes");
	let mut console = Console::new(path);
	console.cycle();
}
