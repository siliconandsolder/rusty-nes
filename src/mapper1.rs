#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use crate::mapper::{Mapper, MIRROR};
use sdl2::gfx::imagefilter::add;

#[derive(Eq, PartialEq)]
enum ChrMode {
	FourKB,
	EightKB
}

#[derive(Eq, PartialEq)]
enum PrgMode {
	Switch32,
	FixFirstBank,
	FixLastBank,
}

pub struct Mapper1 {
	shiftReg: u8,
	shiftCount: u8,
	ctrlReg: u8,
	chrBank0: u8,
	chrBank1: u8,
	prgBank0: u8,
	prgBank1: u8,
	numPrgBanks: u8,
	numChrBanks: u8,
	vPrgRam: Vec<u8>,

	mirrorMode: MIRROR,
	chrMode: ChrMode,
	prgMode: PrgMode
}

impl Mapper1 {
	pub fn new(numPrgBanks: u8, numChrBanks: u8) -> Self {
		Mapper1 {
			shiftReg: 0,
			shiftCount: 0,
			ctrlReg: 0b11111,
			chrBank0: 0,
			chrBank1: 0,
			prgBank0: 0,
			prgBank1: 0,
			numPrgBanks,
			numChrBanks,
			vPrgRam: vec![0; 0x8000],
			mirrorMode: MIRROR::ONESCREEN_LO,
			chrMode: ChrMode::FourKB,
			prgMode: PrgMode::Switch32
		}
	}

	fn resetShiftRegister(&mut self) -> () {
		self.shiftReg = 0;
		self.shiftCount = 0;
	}

	fn setCtrlRegister(&mut self) -> () {

		self.prgMode = match (self.shiftReg >> 2) & 3 {
			0 | 1 => { PrgMode::Switch32 }
			2 => { PrgMode::FixFirstBank }
			3 => { PrgMode::FixLastBank }
			_ => { PrgMode::Switch32 }
		};

		self.chrMode = match (self.shiftReg >> 4) & 1 {
			1 => { ChrMode::EightKB }
			_ => { ChrMode::FourKB }
		};

		self.mirrorMode = match self.shiftReg & 3 {
			0 => { MIRROR::ONESCREEN_LO }
			1 => { MIRROR::ONESCREEN_HI }
			2 => { MIRROR::VERTICAL }
			3 => { MIRROR::HORIZONTAL }
			_=> { MIRROR::ONESCREEN_LO }
		};


		self.ctrlReg = self.shiftReg;
	}

	// fn setOffsets(&mut self) -> () {
	// 	match self.chrMode {
	// 		ChrMode::FourKB => {
	// 			self
	// 		}
	//
	// 		ChrMode::EightKB => {
	//
	// 		}
	// 	}
	// }
}

impl Mapper for Mapper1 {
	fn cpuMapRead(&mut self, ref addr: u16) -> Option<u32> {
		if *addr >= 0x6000 && *addr <= 0x7FFF {
			return Some(self.vPrgRam[(*addr & 0x1FFF) as usize] as u32);
		}
		else if *addr >= 0x8000 {
			return match self.prgMode {
				PrgMode::Switch32 => {
					Some((self.prgBank0 as u32) * 0x8000 + (*addr as u32 & 0x7FFF))
				}
				PrgMode::FixFirstBank => {
					if *addr < 0xC000 {
						// first bank is fixed to the start
						Some(self.prgBank0 as u32 * 0x4000 + (*addr as u32 & 0x3FFF))
					}
				}
				PrgMode::FixLastBank => {
					if *addr >= 0xC000 {
						// first bank is fixed to the start
						Some(self.prgBank1 as u32 * 0x4000 + (*addr as u32 & 0x3FFF))
					}
				}
			}
		}

		return None;
	}

	fn cpuMapWrite(&mut self, ref addr: u16, ref val: u8) -> Option<u32> {
		if *addr >= 0x6000 && *addr <= 0x7FFF {
			self.vPrgRam[(*addr & 0x1FFF) as usize] = *val;
			return None;
		}
		else if *addr >= 0x8000 && *addr <= 0xFFFF {
			if val & 0x80 == 0x80 {
				self.resetShiftRegister();
				self.ctrlReg |= 0xC0;
			}
			else {
				self.shiftReg >>= 1;
				self.shiftReg |= ((*val & 1) << 4);
				self.shiftCount += 1;

				// on the fifth CPU write...
				if self.shiftCount == 5 {
					// copy to internal register
					let register = (*addr & 0x6000) >> 13; // get bits 13 and 14
					match register {
						0 => {
							self.setCtrlRegister();
						}
						1 => {
							match self.chrMode {
								ChrMode::FourKB => {
									self.chrBank0 = self.ctrlReg & 0b1_1111;
								}
								ChrMode::EightKB => {
									self.chrBank0 = self.ctrlReg & 0b1_1110;
								}
							}
						}
						2 => {
							if self.chrMode == ChrMode::FourKB {
								self.chrBank1 = self.shiftReg;
							}
						}
						3 => {
							match self.prgMode {
								PrgMode::Switch32 => {
									self.prgBank0 = (self.ctrlReg & 0b1110) >> 1;
								}
								PrgMode::FixFirstBank => {
									self.prgBank0 = 0;
									self.prgBank1 = self.ctrlReg & 0b1111;
								}
								PrgMode::FixLastBank => {
									self.prgBank0 = self.ctrlReg & 0b1111;
									self.prgBank1 = self.numPrgBanks - 1;
								}
							}
							self.prgBank0 = self.shiftReg & 0b11110;
						}
						_ => {}
					}
					// update
					self.resetShiftRegister();
				}
			}
		}
		return None;
	}

	fn ppuMapRead(&mut self, ref addr: u16) -> Option<u32> {
		if *addr < 0x2000 {
			if self.numChrBanks == 0 {
				return Some(*addr as u32);
			}
			else {
				return match self.chrMode {
					ChrMode::FourKB => { // 4k mode
						if *addr < 0x1000 {
							Some(self.chrBank0 as u32 * 0x1000 + (*addr as u32 & 0x0FFF))
						}
						else {
							Some(self.chrBank1 as u32 * 0x1000 + (*addr as u32 & 0x0FFF))
						}
					},
					ChrMode::EightKB => { // 8k mode
						Some((self.chrBank0) as u32 * 0x2000 + (*addr as u32 & 0x1FFF))
					}
				}
			}
		}

		return None;
	}

	fn ppuMapWrite(&mut self, ref addr: u16, ref val: u8) -> Option<u32> {
		if *addr < 0x2000 {
			if self.numChrBanks == 0 {
				return Some(*addr as u32);	// Carts with CHR Ram only have 8KB
			}
		}
		return None;
	}

	fn getMirrorType(&self) -> MIRROR {
		return self.mirrorMode;
	}
}