#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use crate::mapper::{Mapper, MIRROR};

pub struct Mapper1 {
	shiftReg: u8,
	shiftCount: u8,
	ctrlReg: u8,
	chrBank0: u8,
	chrBank1: u8,
	prgBank: u8,
	numPrgBanks: u8,
	numChrBanks: u8,
	vPrgRam: Vec<u8>
}

impl Mapper1 {
	pub fn new(numPrgBanks: u8, numChrBanks: u8) -> Self {
		Mapper1 {
			shiftReg: 0,
			shiftCount: 0,
			ctrlReg: 0b11111,
			chrBank0: 0,
			chrBank1: 0,
			prgBank: 0,
			numPrgBanks,
			numChrBanks,
			vPrgRam: vec![0; 0x8000]
		}
	}

	fn resetShiftRegister(&mut self) -> () {
		self.shiftReg = 0;
		self.shiftCount = 0;
	}
}

impl Mapper for Mapper1 {
	fn cpuMapRead(&mut self, ref addr: u16) -> Option<u32> {
		if *addr >= 0x6000 && *addr <= 0x7FFF {
			return Some(self.vPrgRam[(*addr & 0x1FFF) as usize] as u32);
		}
		else if *addr >= 0x8000 {
			let prgBankMode = (self.ctrlReg & 0b01100) >> 2;
			match prgBankMode {
				0 | 1 => {
					return Some((self.prgBank as u32 & 0b1110) * 0x8000 + (*addr as u32 & 0x7FFF));
				}
				2 => {
					if *addr < 0xC000 {
						// first bank is fixed to the start
						return Some(*addr as u32 & 0x3FFF);
					}
					else {
						return Some(self.prgBank as u32 * 0x4000 + (*addr as u32 & 0x3FFF));
					}
				}
				3 => {
					if *addr >= 0xC000 {
						// first bank is fixed to the start
						return Some((self.numPrgBanks as u32 - 1) * 0x4000 + (*addr as u32 & 0x3FFF));
					}
					else {
						return Some(self.prgBank as u32 * 0x4000 + (*addr as u32 & 0x3FFF));
					}
				}
				_ => { panic!("Should not reach here!"); }
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
							self.ctrlReg = self.shiftReg;
						}
						1 => {
							self.chrBank0 = self.shiftReg;
						}
						2 => {
							self.chrBank1 = self.shiftReg;
						}
						3 => {
							self.prgBank = self.shiftReg & 0b1111;
						}
						_ => {}
					}
					self.resetShiftRegister();
				}
			}
		}
		return None;
	}

	fn ppuMapRead(&mut self, ref addr: u16) -> Option<u32> {
		if *addr < 0x2000 {
			let chrBankMode = (self.ctrlReg & 0b10000) >> 4;
			match chrBankMode {
				1 => { // 4k mode
					if *addr < 0x1000 {
						return Some(self.chrBank0 as u32 * 0x1000 + (*addr as u32 & 0x0FFF));
					}
					else {
						return Some(self.chrBank1 as u32 * 0x1000 + (*addr as u32 & 0x0FFF));
					}
				},
				_ => { // 8k mode
					return Some((self.chrBank0 & 0b11110) as u32 * 0x2000 + (*addr as u32 & 0x1FFF));
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
		match self.ctrlReg & 3 {
			0 => {
				return MIRROR::ONESCREEN_LO;
			},
			1 => {
				return MIRROR::ONESCREEN_HI;
			},
			2 => {
				return MIRROR::VERTICAL;
			},
			3 => {
				return MIRROR::HORIZONTAL;
			},
			_ => { panic!("Should never reach this."); }
		}
	}
}