#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use super::utils::TRIANGLE_SEQUENCE_TABLE;

pub struct Triangle {
	pub enabled: bool,
	pub lengthCounterEnabled: bool,
	pub lengthCounterValue: u8,
	pub linearCounterControl: bool,
	pub linearCounterReload: bool,
	pub linearCounterValue: u8,
	pub linearCounterPeriod: u8,
	pub dutyValue: u8,
	pub timer: u16,
	pub timerPeriod: u16,
}

impl Triangle {
	pub fn new() -> Self {
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
			timerPeriod: 0
		}
	}

	pub fn clockTimer(&mut self) -> () {

		if self.timer == 0 {
			self.timer = self.timerPeriod;
			if self.lengthCounterValue > 0 && self.linearCounterValue > 0 {
				self.dutyValue = (self.dutyValue + 1) % 32;
			}
		}
		else {
			self.timer -= 1;
		}
	}

	pub fn clockLinearCounter(&mut self) -> () {
		if self.linearCounterReload {
			self.linearCounterValue = self.linearCounterPeriod;
		}
		else if self.linearCounterValue > 0 {
			self.linearCounterValue -= 1;
		}

		if self.lengthCounterEnabled {
			self.linearCounterReload = false;
		}
	}

	pub fn clockLengthCounter(&mut self) -> () {
		if self.lengthCounterEnabled && self.lengthCounterValue > 0 {
			self.lengthCounterValue -= 1;
		}
	}

	pub fn output(&mut self) -> u8 {
		if !self.enabled ||
			self.lengthCounterValue == 0 ||
			self.linearCounterValue == 0 {
			return 0;
		}

		return TRIANGLE_SEQUENCE_TABLE[self.dutyValue as usize];
	}
}
