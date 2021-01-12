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

	// TODO: Fix SHY, SHX
	//let path = Path::new("./tests/ppu_sprite_hit/rom_singles/06-right_edge.nes");
	let path = Path::new("./super_mario_bros.nes");
	let mut console = Console::new(path);
	console.cycle();
}
