#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

extern crate sdl2;

use crate::clock::Clocked;
use std::f64::consts::PI;
use rand::seq::index::sample;
use sdl2::AudioSubsystem;
use sdl2::audio::{AudioQueue, AudioFormatNum};
use self::sdl2::audio::{AudioSpec, AudioSpecDesired};
use crate::audio::Audio;
use std::sync::{Arc, Mutex};
use ndarray::{Array2, Array, Array1, array};
use lazy_static::lazy_static;
use crate::data_bus::DataBus;
use std::cell::RefCell;
use std::rc::Rc;

const STEP_ONE_CYCLE: u16 = 3729;
const STEP_TWO_CYCLE: u16 = 7457;
const STEP_THREE_CYCLE: u16 = 11186;
const STEP_FOUR_CYCLE: u16 = 14914;
const STEP_FOUR_CYCLE_PLUS_ONE: u16 = 14915;
const STEP_FIVE_CYCLE: u16 = 18641;
const CPU_TICK_TIME: f64 = 1.0 / 1789773.0;
const CPU_FREQ: f64 = 1789773.0;
const AUDIO_HERTZ: u16 = 44100;
const HARMONICS: u8 = 100;

const BUFFER_SIZE: u16 = 2048;

fn approxSin(time: f64) -> f64{
	let mut j: f64 = time * 0.15915;
	j = j - j as u64 as f64;
	return 20.785 * j * (j - 0.5) * (j - 1.0);
}

struct PulseOsc {
	freq: f64,
	duty: f64,
	amp: f64,
}

lazy_static! {

	static ref SQUARE_SEQUENCE_TABLE: Array2<u8> = array![
		[0, 1, 0, 0, 0, 0, 0, 0],
		[0, 1, 1, 0, 0, 0, 0, 0],
		[0, 1, 1, 1, 1, 0, 0, 0],
		[1, 0, 0, 1, 1, 1, 1, 1],
	];

/*
15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0
 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15
*/

	static ref TRIANGLE_SEQUENCE_TABLE: Array1<u8> = array![
		15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
 		0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15
	];

	static ref NOISE_TIMER_TABLE: Array1<u16> = array![
		4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068
	];

	static ref DMC_RATE_TABLE: Array1<u16> = array![
		428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54
	];
}



impl PulseOsc {
	fn sample(&mut self, time: f64) -> f32 {

		let mut a: f64 = 0.0;
		let mut b: f64 = 0.0;
		let mut period = self.duty * 2.0 * PI;

		for i in 1..HARMONICS {
			let c = i as f64 * self.freq * 2.0 * PI * time;
			a += -approxSin(c) / i as f64;
			b += -approxSin(c - period * i as f64) / i as f64;
		}

		return ((self.amp * 2.0 * PI) * (a - b)) as f32;
	}
}

struct Pulse {
	enabled: bool,
	sequence: u8,
	duty: u8,
	dutyMode: u8,
	output: u8,
	lengthHalt: bool,

	constVolume: bool,
	volume: u8,
	envVolume: u8,
	envValue: u8,
	envPeriod: u8,
	envEnabled: bool,
	envLoop: bool,
	envStart: bool,

	sweepEnabled: bool,
	sweepReload: bool,
	sweepPeriod: u8,
	sweepValue: u8,
	negate: bool,
	shift: u8,

	timerPeriod: u16,
	timer: u16,
	lengthCounter: u8,

	osc: PulseOsc,
	sample: u8,
}

impl Pulse {

	fn new() -> Self {
		Pulse {
			enabled: false,
			sequence: 0,
			duty: 0,
			dutyMode: 0,
			output: 0,
			lengthHalt: false,
			constVolume: false,
			volume: 0,
			envVolume: 0,
			envValue: 0,
			envPeriod: 0,
			envEnabled: false,
			envLoop: false,
			envStart: false,
			sweepEnabled: false,
			sweepReload: false,
			sweepPeriod: 0,
			sweepValue: 0,
			negate: false,
			shift: 0,
			timerPeriod: 0,
			timer: 0,
			lengthCounter: 0,
			osc: PulseOsc {
				freq: 0.0,
				duty: 0.0,
				amp: 0.0
			},
			sample: 0
		}
	}

	fn clockTimer(&mut self) -> () {
		if self.enabled {

			self.timer = self.timer.wrapping_sub(1);
			if self.timer == 0xFF {
				self.timer = self.timerPeriod;
				//self.sequence = ((self.sequence & 1) << 7 | (self.sequence & 0b11111110) >> 1);
				self.duty = (self.duty + 1) & 7;
				//self.output = self.sequence & 1;
			}
		}
		//self.output = 0;
	}

	fn clockLengthCounter(&mut self) -> () {
		if !self.lengthHalt && self.lengthCounter > 0 {
			self.lengthCounter -= 1;
		}
	}

	fn clockEnvelope(&mut self) -> () {
		if self.envStart {
			self.envVolume = 15;
			self.envValue = self.envPeriod;
			self.envStart = false;
		}
		else if self.envValue > 0 {
			self.envValue -= 1;
		}
		else {
			self.envValue = self.envPeriod;

			if self.envLoop && self.envVolume == 0 {
				self.envVolume = 15;
			}
			else if self.envVolume > 0 {
				self.envVolume -= 1;
			}
		}
	}

	fn clockSweep(&mut self) -> () {
		if self.sweepValue == 0 {

			if self.sweepEnabled && self.sweepValue == 0 {
				self.sweep();
			}

			if self.sweepReload {
				self.sweepValue = self.sweepPeriod;
				self.sweepReload = false;
			}
		}
		else if self.sweepValue > 0 {
			self.sweepValue -= 1;
		}
	}

	fn sweep(&mut self) -> () {
		let delta = self.timerPeriod >> self.shift;
		if self.negate {
			self.timerPeriod -= delta;
			self.timerPeriod -= 1;
		}
		else {
			self.timerPeriod += delta;
		}
	}

	fn output(&self) -> u8 {
		return if !self.enabled ||
			self.lengthCounter == 0 ||
			self.timerPeriod < 8 ||
			self.timerPeriod > 0x7FF ||
			SQUARE_SEQUENCE_TABLE[[self.dutyMode as usize, self.duty as usize]] == 0 {
			0
		} else if self.envEnabled {
			self.envVolume
		} else {
			self.volume
		}
	}

}

struct Triangle {
	enabled: bool,
	lengthCounterEnabled: bool,
	lengthCounterValue: u8,
	linearCounterControl: bool,
	linearCounterReload: bool,
	linearCounterValue: u8,
	linearCounterPeriod: u8,
	dutyValue: u8,
	timer: u16,
	timerStart: u16,
}

impl Triangle {
	fn new() -> Self {
		Triangle {
			enabled: false,
			lengthCounterEnabled: false,
			lengthCounterValue: 0,
			linearCounterControl: false,
			linearCounterReload: false,
			linearCounterValue: 0,
			linearCounterPeriod: 0,
			dutyValue: 0,
			timer: 0,
			timerStart: 0
		}
	}

	fn clockTimer(&mut self) -> () {

		self.timer = self.timer.wrapping_sub(1);

		if self.timer == 0xFFFF {
			self.timer = self.timerStart;
			if self.lengthCounterValue > 0 && self.linearCounterValue > 0 {
				self.dutyValue = (self.dutyValue + 1) & 31;
			}
		}
	}

	fn clockLinearCounter(&mut self) -> () {
		if self.linearCounterReload {
			self.linearCounterValue = self.linearCounterPeriod;
		}
		else if self.linearCounterValue > 0 {
			self.linearCounterValue -= 1;
		}

		if !self.linearCounterControl {
			self.linearCounterReload = false;
		}
	}

	fn clockLengthCounter(&mut self) -> () {
		if self.lengthCounterEnabled && self.lengthCounterValue > 0 {
			self.lengthCounterValue -= 1;
		}
	}

	fn output(&mut self) -> u8 {
		if !self.enabled ||
			self.lengthCounterValue == 0 ||
			self.linearCounterValue == 0 {
			return 0;
		}

		return TRIANGLE_SEQUENCE_TABLE[self.dutyValue as usize];
	}
}

struct Noise {
	enabled: bool,
	mode: bool,
	output: u8,
	lengthHalt: bool,

	constVolume: u8,
	volume: u8,
	envVolume: u8,
	envValue: u8,
	envPeriod: u8,
	envEnabled: bool,
	envLoop: bool,
	envStart: bool,

	shift: u16,

	timerPeriod: u16,
	timer: u16,
	lengthCounter: u8,
}

impl Noise {
	fn new() -> Self {
		Noise {
			enabled: false,
			mode: false,
			output: 0,
			lengthHalt: false,
			constVolume: 0,
			volume: 0,
			envVolume: 0,
			envValue: 0,
			envPeriod: 0,
			envEnabled: false,
			envLoop: false,
			envStart: false,
			shift: 0,
			timerPeriod: 0,
			timer: 0,
			lengthCounter: 0
		}
	}

	fn clockTimer(&mut self) -> () {
		self.timer = self.timer.wrapping_sub(1);

		if self.timer == 0xFFFF {
			let shiftBit: u16 = if self.mode {6} else {1};
			let feedBack: u16 = (self.shift & 1) ^ ((self.shift >> shiftBit) & 1);
			self.shift >>= 1;
			self.shift |= (feedBack << 14);
		}
		else {
			self.timer -= 1;
		}
	}

	fn clockEnvelope(&mut self) -> () {
		if self.envStart {
			self.envVolume = 15;
			self.envValue = self.envPeriod;
			self.envStart = false;
		}
		else if self.envValue > 0 {
			self.envValue -= 1;
		}
		else {
			self.envValue = self.envPeriod;

			if self.envLoop && self.envVolume == 0 {
				self.envVolume = 15;
			}
			else if self.envVolume > 0 {
				self.envVolume -= 1;
			}
		}
	}

	fn clockLength(&mut self) -> () {
		if !self.lengthHalt && self.lengthCounter > 0 {
			self.lengthCounter -= 1;
		}
	}

	fn output(&mut self) -> u8 {
		if !self.enabled ||
			self.lengthCounter == 0 ||
			self.shift & 1 == 1 {
			return 0;
		}

		return if self.envEnabled {
			self.envVolume
		} else {
			self.constVolume
		}
	}
}

struct DMC<'a> {
	enabled: bool,
	irqEnabled: bool,
	loopEnabled: bool,
	ratePeriod: u16,
	rateValue: u16,
	directLoad: u8,
	bitCounter: u8,
	freq: u8,
	loadCounter: u8,
	sampleAddr: u16,
	curSampleAddr: u16,
	sampleLength: u16,
	curSampleLength: u16,
	shift: u8,
	dataBus: Rc<RefCell<DataBus<'a>>>,

}

impl<'a> DMC<'a> {
	fn new(dataBus: Rc<RefCell<DataBus<'a>>>) -> Self {
		DMC {
			enabled: false,
			irqEnabled: false,
			loopEnabled: false,
			ratePeriod: 0,
			rateValue: 0,
			directLoad: 0,
			bitCounter: 0,
			freq: 0,
			loadCounter: 0,
			sampleAddr: 0,
			curSampleAddr: 0,
			sampleLength: 0,
			curSampleLength: 0,
			shift: 0,
			dataBus: dataBus
		}
	}

	fn clockRate(&mut self) -> () {
		self.clockReader();

		self.rateValue = self.rateValue.wrapping_sub(1);
		if self.rateValue == 0xFFFF {

			self.rateValue = self.ratePeriod;

			if self.shift & 1 == 1 {
				if self.directLoad <= 125 {
					self.directLoad += 2;
				}
			}
			else {
				if self.directLoad >= 2 {
					self.directLoad -= 2;
				}
			}

			self.shift >>= 1;
			self.bitCounter -= 1;
		}
	}

	fn clockReader(&mut self) -> () {
		if self.curSampleLength > 0 && self.bitCounter == 0 {
			self.dataBus.borrow_mut().setDmcCpuStall();
			self.shift = self.dataBus.borrow().readCpuMem(self.curSampleAddr);

			self.curSampleAddr =  self.curSampleAddr.wrapping_add(1);
			if self.curSampleAddr == 0 {
				self.curSampleAddr = 0x8000;
			}

			self.bitCounter = 8;

			self.curSampleLength -= 1;
			if self.curSampleLength == 0 && self.loopEnabled {
				self.reset();
			}
		}
	}

	fn reset(&mut self) -> () {
		self.curSampleAddr = self.sampleAddr;
		self.curSampleLength = self.sampleLength;
	}

	fn output(&self) -> u8 {
		return self.directLoad;
	}
}

/*
 |  0   1   2   3   4   5   6   7    8   9   A   B   C   D   E   F
-----+----------------------------------------------------------------
00-0F  10,254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
10-1F  12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
*/

pub struct Apu<'a> {
	frame: u16,
	globalTime: f64,
	pulse1: Pulse,
	pulse2: Pulse,
	triangle: Triangle,
	dmc: DMC<'a>,
	dataBus: Rc<RefCell<DataBus<'a>>>,
	noise: Noise,
	fiveStep: bool,
	frameInterrupt: bool,
	inhibitInterrupt: bool,
	audio: Audio,

	lengthTable: Vec<u8>,
	pulseTable: Vec<f32>,
	tndTable: Vec<f32>
}

impl<'a> Apu<'a> {
	pub fn new(audioSystem: AudioSubsystem, dataBus: Rc<RefCell<DataBus<'a>>>) -> Self {

		let lengthTable: [u8; 32] = [
			10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
			12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
		];

		let mut pulseTable: Vec<f32> = vec![0.0; 31];
		for i in 0..31 {
			pulseTable[i as usize] = 95.52 / (8128.0/i as f32 + 100 as f32);
		}

		let mut tndTable: Vec<f32> = vec![0.0; 203];
		for i in 0..203 {
			tndTable[i as usize] = 163.67 / (24329.0/i as f32 + 100 as f32);
		}

		Apu {
			frame: 0,
			globalTime: 0.0,
			pulse1: Pulse::new(),
			pulse2: Pulse::new(),
			triangle: Triangle::new(),
			dmc: DMC::new(dataBus.clone()),
			dataBus: dataBus.clone(),
			noise: Noise::new(),
			fiveStep: false,
			frameInterrupt: false,
			inhibitInterrupt: false,
			audio: Audio::new(audioSystem),
			lengthTable: Vec::from(lengthTable),
			pulseTable: pulseTable,
			tndTable: tndTable,
		}
	}



	pub fn write(&mut self, ref addr: u16, data: u8) -> () {
		match *addr {
			0x4000 => {
				match (data & 0b11000000) >> 6 {
					0x0 => {
						//self.pulse1.duty = 0b00000001;
						self.pulse1.dutyMode = 0;
						self.pulse1.osc.duty = 0.125;
					},
					0x1 => {
						//self.pulse1.duty = 0b00000011;
						self.pulse1.dutyMode = 1;
						self.pulse1.osc.duty = 0.250;
					},
					0x2 => {
						//self.pulse1.duty = 0b00001111;
						self.pulse1.dutyMode = 2;
						self.pulse1.osc.duty = 0.500;
					},
					0x3 => {
						//self.pulse1.duty = 0b11111100;
						self.pulse1.dutyMode = 3;
						self.pulse1.osc.duty = 0.750;
					},
					_ => {},
				}
				self.pulse1.lengthHalt = (data & 0b00100000) == 0b00100000;
				self.pulse1.envLoop = self.pulse1.lengthHalt;
				self.pulse1.constVolume = (data & 0b00010000) == 0b00010000;
				self.pulse1.envEnabled = !self.pulse1.constVolume;
				self.pulse1.envPeriod = data & 0b00001111;
			},
			0x4001 => {
				self.pulse1.sweepEnabled = (data & 0b10000000) == 0b10000000;
				self.pulse1.sweepPeriod = (data & 0b01110000) >> 4;
				self.pulse1.negate = (data & 0b00001000) == 0b00001000;
			},
			0x4002 => {
				self.pulse1.timer = self.pulse1.timer & 0xFF00 | data as u16;
				self.pulse1.timerPeriod = self.pulse1.timer;
			},
			0x4003 => {
				self.pulse1.timer = (self.pulse1.timer & 0x00FF) | ((data & 0b00000111) << 4) as u16;
				self.pulse1.timerPeriod = self.pulse1.timer;
				self.pulse1.lengthCounter = self.lengthTable[(data >> 3) as usize];
				self.pulse1.envStart = true;
				self.pulse1.sequence = 0;
			},
			0x4004 => {
				match (data & 0b11000000) >> 6 {
					0x0 => {
						//self.pulse1.duty = 0b00000001;
						self.pulse2.dutyMode = 0;
						self.pulse2.osc.duty = 0.125;
					},
					0x1 => {
						//self.pulse1.duty = 0b00000011;
						self.pulse2.dutyMode = 1;
						self.pulse2.osc.duty = 0.250;
					},
					0x2 => {
						//self.pulse1.duty = 0b00001111;
						self.pulse2.dutyMode = 2;
						self.pulse2.osc.duty = 0.500;
					},
					0x3 => {
						//self.pulse1.duty = 0b11111100;
						self.pulse2.dutyMode = 3;
						self.pulse2.osc.duty = 0.750;
					},
					_ => {},
				}
				self.pulse2.lengthHalt = (data & 0b00100000) == 0b00100000;
				self.pulse2.envLoop = self.pulse1.lengthHalt;
				self.pulse2.constVolume = (data & 0b00010000) == 0b00010000;
				self.pulse2.envEnabled = !self.pulse1.constVolume;
				self.pulse2.envPeriod = data & 0b00001111;
			},
			0x4005 => {
				self.pulse2.sweepEnabled = (data & 0b10000000) == 0b10000000;
				self.pulse2.sweepPeriod = (data & 0b01110000) >> 4;
				self.pulse2.negate = (data & 0b00001000) == 0b00001000;
			},
			0x4006 => {
				self.pulse2.timerPeriod = self.pulse1.timerPeriod & 0xFF00 | data as u16;
			},
			0x4007 => {
				self.pulse2.timerPeriod = (self.pulse1.timerPeriod & 0x00FF) | ((data & 0b00000111) << 4) as u16;
				self.pulse2.timer = self.pulse1.timerPeriod;
				self.pulse2.lengthCounter = self.lengthTable[(data >> 3) as usize];
				self.pulse2.envStart = true;
				self.pulse2.sequence = 0;
			},
			0x4008 => {
				self.triangle.linearCounterControl = (data & 128) == 128;
				self.triangle.lengthCounterEnabled = !self.triangle.linearCounterControl;
				self.triangle.linearCounterPeriod = data & 0b01111111;
			},
			0x400A => {
				self.triangle.timerStart = (self.triangle.timerStart & 0xFF00) | data as u16;
			},
			0x400B => {
				self.triangle.timerStart = (self.triangle.timerStart & 0x00FF) as u16 | ((data & 0b00000111) << 4) as u16;
				self.triangle.timer = self.triangle.timerStart;
				self.triangle.lengthCounterValue = self.lengthTable[(data >> 3) as usize];
				self.triangle.linearCounterReload = true;
			},
			0x400C => {
				self.noise.lengthHalt = (data & 0b00100000) == 0b00100000;
				self.noise.envLoop = !self.pulse1.lengthHalt;
				self.noise.envEnabled = !((data & 0b00010000) == 0b00010000);
				self.noise.envPeriod = data & 0b00001111;
				self.noise.constVolume = self.noise.envPeriod;
			},
			0x400E => {
				self.noise.mode = (data & 128) == 128;
				self.noise.timerPeriod = NOISE_TIMER_TABLE[(data & 15) as usize]
			},
			0x400F => {
				self.noise.lengthCounter = data >> 3;
				self.noise.envStart = true;
			},
			0x4010 => {
				self.dmc.irqEnabled = (data & 128) == 128;
				self.dmc.loopEnabled = (data & 64) == 64;
				self.dmc.ratePeriod = DMC_RATE_TABLE[(data & 15) as usize];
			},
			0x4011 => {
				self.dmc.bitCounter = data & 0b01111111;
			},
			0x4012 => {
				self.dmc.sampleAddr = (0xC000 + (data as u16 * 64 as u16));
			},
			0x4013 => {
				self.dmc.sampleLength = ((data as u16) << 4) | 1;
			},
			/*
		 7654 3210  APUFLAGS ($4015)
			| ||||
   			| |||+- Square 1 (0: disable; 1: enable)
   			| ||+-- Square 2
   			| |+--- Triangle
   			| +---- Noise
   			+------ DMC
			*/
			0x4015 => {
				self.pulse1.enabled = (data & 1) == 1;
				if !self.pulse1.enabled { self.pulse1.lengthCounter = 0; }

				self.pulse2.enabled = (data & 2) == 2;
				if !self.pulse2.enabled { self.pulse2.lengthCounter = 0; }

				self.triangle.enabled = (data & 4) == 4;
				if !self.triangle.enabled { self.triangle.lengthCounterValue = 0; }

				self.noise.enabled = (data & 8) == 8;
				if !self.noise.enabled { self.noise.lengthCounter = 0; }

				self.dmc.enabled = (data & 16) == 16;
				if !self.dmc.enabled {
					self.dmc.curSampleLength = 0;
				}
				else {
					if self.dmc.curSampleLength == 0 {
						self.dmc.reset();
					}
				}
			},
			0x4017 => {
				self.fiveStep = (data & 128) == 128;
				self.inhibitInterrupt = (data & 64) == 64;

				if self.inhibitInterrupt {
					self.frameInterrupt = false;
				}

				if self.fiveStep {
					self.quarterStep();
					self.halfStep();
				}
			},
			_ => {}
		}
	}

	pub fn read(&mut self, ref addr: u16) -> u8 {
		let mut status: u8 = 0;

		if self.pulse1.lengthCounter > 0 {
			status |= 1;
		}

		if self.pulse2.lengthCounter > 0 {
			status |= 2;
		}

		if self.triangle.lengthCounterValue > 0 {
			status |= 4;
		}

		if self.noise.lengthCounter > 0 {
			status |= 8;
		}

		if self.dmc.curSampleLength > 0 {
			status |= 16;
		}

		if self.frameInterrupt {
			status |= 64;
			self.frameInterrupt = false;
		}

		if self.dmc.irqEnabled {
			status |= 128;
		}

		return status;
	}

	pub fn addSampleToBuffer(&mut self) -> () {
		let pulseOut: f32 = self.pulseTable[(self.pulse1.output() + self.pulse2.output()) as usize];
		let tndOut: f32 = self.tndTable[
			(3 * self.triangle.output()) as usize +
			(2 * self.noise.output()) as usize +
			(self.dmc.output()) as usize
		];
		self.audio.pushSample(pulseOut + tndOut);
	}

	fn quarterStep(&mut self) -> () {
		self.pulse1.clockEnvelope();
		self.pulse2.clockEnvelope();
		self.triangle.clockLinearCounter();
		self.noise.clockEnvelope();
	}

	fn halfStep(&mut self) -> () {
		self.pulse1.clockSweep();
		self.pulse2.clockSweep();
		self.triangle.clockLengthCounter();
		self.noise.clockLength();
	}
}

impl<'a> Clocked for Apu<'a> {
	fn cycle(&mut self) {


		match self.frame {
			STEP_ONE_CYCLE => {
				self.quarterStep();
			},
			STEP_TWO_CYCLE => {
				self.halfStep();
				self.quarterStep();
			},
			STEP_THREE_CYCLE => {
				self.quarterStep();
			},
			STEP_FOUR_CYCLE => {
				if !self.fiveStep {
					self.halfStep();
					self.quarterStep();
					if !self.inhibitInterrupt {
						self.dataBus.borrow_mut().triggerCpuIRQ();
					}
					self.frame = 0;
				}
			},
			// STEP_FOUR_CYCLE_PLUS_ONE => {
			// 	if !self.fiveStep {
			// 		self.dataBus.borrow_mut().triggerCpuIRQ();
			// 	}
			// }
			STEP_FIVE_CYCLE => {
				if self.fiveStep {
					self.quarterStep();
					self.halfStep();
					self.frame = 0;
				}
			},
			_ => {}
		}

		self.frame += 1;

		if (self.frame & 2) == 2 {
			self.pulse1.clockTimer();
			self.pulse2.clockTimer();
			self.noise.clockTimer();
			self.dmc.clockRate();
		}
		self.triangle.clockTimer();

	}
}
