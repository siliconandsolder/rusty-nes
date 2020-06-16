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
	CombinedLogger::init(
		vec![
			WriteLogger::new(LevelFilter::Info, Config::default(), File::create("rusty_logs.txt").unwrap())
		]
	).unwrap();

	let path = Path::new("./donkey_kong.nes");
	let mut console = Console::new(path);
	console.cycle();

    // let sdl = sdl2::init().unwrap();
    // let vid = sdl.video().unwrap();
    //
    // let window = vid
    //     .window("Hello!", 768, 720)
    //     .resizable()
    //     .build()
    //     .unwrap();
    //
    // let mut rng = rand::thread_rng();
	// let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    // canvas.set_draw_color(Color::RGB(0,0,0));
    // canvas.present();
    //
    // let mut vRectangles: Vec<Rect> = vec![];
    //
    // let width: u32 = 768 / 256;
    // let height: u32  = 720 / 240;
    //
    // for x in 0..256 {
    //     for y in 0..240 {
    //         vRectangles.push(Rect::new((x * width) as i32, (y * width) as i32, width, height));
    //     }
    // }
    //
    //
    // let mut event_pump = sdl.event_pump().unwrap();
    // 'main: loop {
    //
    //     canvas.clear();
    //
    //     for event in event_pump.poll_iter() {
    //         match event {
    //             sdl2::event::Event::Quit {..} => break 'main,
    //             sdl2::event::Event::KeyDown {..} => break 'main,
    //             _ => {},
    //         }
    //     }
    //
    //     for x in 0..256 {
    //         for y in 0..240 {
    //             let red: u16 = rng.gen_range(0, 256);
    //             let green: u16 = rng.gen_range(0, 256);
    //             let blue: u16 = rng.gen_range(0, 256);
    //             canvas.set_draw_color(Color::RGB(red as u8,green as u8,blue as u8));
    //             canvas.fill_rect(vRectangles[x + (256 * y)]).unwrap();
    //         }
    //     }
    //
    //     canvas.present();
    //
    //     //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    //     // render window contents here
    // }
}
