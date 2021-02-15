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
use sdl2::gfx::framerate::FPSManager;
use crate::ppu_bus::PpuBus;
use crate::controller::Controller;
use crate::apu::Apu;
use self::sdl2::audio::AudioSpecDesired;

const MASTER_CLOCK_NANO: u8 = 47; // should be about 46.56, but the std::thread functions don't allow decimals
const CPU_HERTZ_PER_CYCLE: f64 = 1.0 / 1789773.0;
const CPU_HERTZ: f64 = 1789773.0;
const AUDIO_HERTZ: u16 = 44100;
const AUDIO_HERTZ_PER_SAMPLE: f64 = 1.0 / 44100.0;

const NANO_PER_FRAME: u128 = ((1.0 / 60.0) * 1000.0 * 1000000.0) as u128;

pub struct Console<'a> {
	cpu: Rc<RefCell<Cpu<'a>>>,
	ppu: Rc<RefCell<Ppu<'a>>>,
	apu: Rc<RefCell<Apu<'a>>>,
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
			.position_centered()
			.opengl()
			.build()
			.unwrap();

		let audio = sdl.audio().unwrap();


		// will eventually pass this to the controller
		let eventPump = Rc::new(RefCell::new(sdl.event_pump().unwrap()));

		let mut canvas = window.into_canvas().build().unwrap();
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
		let apu = Rc::new(RefCell::new(Apu::new(bus.clone())));
		bus.borrow_mut().attachApu(apu.clone());
		let ppuBus = PpuBus::new(cartridge.clone());
		let ppu = Rc::new(RefCell::new(Ppu::new(bus.clone(), Rc::new(RefCell::new(canvas)), ppuBus)));
		bus.borrow_mut().attachPpu(ppu.clone());
		
		Console {
			cpu,
			ppu,
			apu,
			bus,
			cartridge,
		}
	}
}

impl<'a> Clocked for Console<'a> {

	#[inline]
	fn cycle(&mut self) {
		let mut fps = FPSManager::new();
		fps.set_framerate(60);
		let mut cycleApu: bool = true;
		let mut audioTime: f64 = 0.0;
		'game: loop {

			// one frame (approximately)
			for i in 0..29781 {
				self.ppu.borrow_mut().cycle();
				self.ppu.borrow_mut().cycle();
				self.ppu.borrow_mut().cycle();

				self.cpu.borrow_mut().cycle();
				self.apu.borrow_mut().cycle();

				audioTime += CPU_HERTZ_PER_CYCLE;
				if audioTime >= AUDIO_HERTZ_PER_SAMPLE {
					// output audio here
					audioTime = 0.0;
					self.apu.borrow_mut().addSampleToBuffer();
				}

				//counter += 1;
				//println!("Cycle: {}", counter);
				//println!("Nanoseconds: {}", now.elapsed().unwrap().as_nanos());
			}

			self.bus.borrow_mut().getControllerInput();
			fps.delay();

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