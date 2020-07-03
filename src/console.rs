#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

extern crate sdl2;

use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::cartridge::Cartridge;
use crate::data_bus::DataBus;
use crate::clock::Clocked;
use std::time::{SystemTime, Duration};
use std::thread::sleep;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use sdl2::EventPump;
use self::sdl2::pixels::{Color, PixelFormatEnum};
use crate::ppu_bus::PpuBus;
use crate::controller::Controller;

const MASTER_CLOCK_NANO: u8 = 47; // should be about 46.56, but the std::thread functions don't allow decimals

pub struct Console<'a> {
	cpu: Rc<RefCell<Cpu<'a>>>,
	ppu: Rc<RefCell<Ppu<'a>>>,
	bus: Rc<RefCell<DataBus<'a>>>,
	cartridge: Rc<RefCell<Cartridge>>,
}

impl<'a> Console<'a> {
	pub fn new(game: &Path) -> Self {

		// sdl setup
		let sdl = sdl2::init().unwrap();
		let vid = sdl.video().unwrap();

		let window = vid
			.window("RustyNES", 768, 720)
			.resizable()
			.build()
			.unwrap();

		// will eventually pass this to the controller
		let eventPump = Rc::new(RefCell::new(sdl.event_pump().unwrap()));

		let mut canvas = window.into_canvas().present_vsync().build().unwrap();
		canvas.clear();
		canvas.set_draw_color(Color::RGB(0,0,0));
		canvas.present();

		let controller1 = Rc::new(RefCell::new(Controller::new(eventPump)));
		let cartridge = Rc::new(RefCell::new(Cartridge::new(game)));
		let bus = Rc::new(RefCell::new(DataBus::new()));
		bus.borrow_mut().attachController1(controller1);
		bus.borrow_mut().attachCartridge(cartridge.clone());
		let cpu = Rc::new(RefCell::new(Cpu::new(bus.clone())));
		bus.borrow_mut().attachCpu(cpu.clone());
		let ppuBus = PpuBus::new(cartridge.clone());
		let ppu = Rc::new(RefCell::new(Ppu::new(bus.clone(), Rc::new(RefCell::new(canvas)), ppuBus)));
		bus.borrow_mut().attachPpu(ppu.clone());
		
		Console {
			cpu,
			ppu,
			bus,
			cartridge,
		}
	}
}

impl<'a> Clocked for Console<'a> {

	#[inline]
	fn cycle(&mut self) {
		let mut fps: u8 = 0;
		let mut now = SystemTime::now();
		'game: loop {

			// for i in 1..=12 {
			// 	if i == 1 {
			// 		self.cpu.borrow_mut().cycle();
			// 	}
			// 	if i % 4 == 0 {
			// 		self.ppu.borrow_mut().cycle();
			// 	}

			// }


			// one frame (approximately)
			for i in 0..29781 {
				self.cpu.borrow_mut().cycle();
				self.ppu.borrow_mut().cycle();
				self.ppu.borrow_mut().cycle();
				self.ppu.borrow_mut().cycle();
				//println!("Nanoseconds: {}", now.elapsed().unwrap().as_nanos());
			}



			// for event in self.eventPump.borrow_mut().poll_iter() {
			// 	match event {
			// 		sdl2::event::Event::Quit {..} => break 'game,
			// 		sdl2::event::Event::KeyDown {..} => break 'game,
			// 		_ => {},
			// 	}
			// }
			self.bus.borrow_mut().getControllerInput();
			// fps += 1;
			//
			// if now.elapsed().unwrap().as_secs() >= 1 {
			// 	println!("FPS: {}", fps);
			// 	fps = 0;
			// 	now = SystemTime::now();
			// }

			// wait if we were too fast
			// let timeDiff: i128 = (MASTER_CLOCK_NANO as u128 - now.elapsed().unwrap().as_nanos()) as i128;
			// if timeDiff > 0 {
			// 	sleep(Duration::new(0, timeDiff as u32))
			// }
		}
	}
}