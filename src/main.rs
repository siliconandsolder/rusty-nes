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

	let path = Path::new("./super_mario_bros.nes");
	let mut console = Console::new(path);
	console.cycle();
}
