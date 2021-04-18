#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use crate::mapper::{Mapper, MIRROR};
use sdl2::gfx::imagefilter::add;
use sdl2::mouse::SystemCursor::No;

pub struct Mapper1 {
    shiftReg: u8,
    shiftLatch: u8,

    ctrlReg: u8,
    chrBank0: u8,
    chrBank1: u8,
    prgBank: u8,

    numPrgBanks: u8,
    numChrBanks: u8,

    vPrgRam: Vec<u8>,
}

impl Mapper1 {
    pub fn new(numPrgBanks: u8, numChrBanks: u8) -> Self {
        Mapper1 {
            shiftReg: 0,
            shiftLatch: 0,
            ctrlReg: 0,
            chrBank0: 0,
            chrBank1: 0,
            prgBank: 0,
            numPrgBanks,
            numChrBanks,
            vPrgRam: vec![0; 0x8000],
		}
    }

    fn resetShiftRegister(&mut self) -> () {
        self.shiftReg = 0x10;
    }
}

impl Mapper for Mapper1 {
	fn cpuMapRead(&mut self, addr: u16) -> Option<u32> {
		if addr >= 0x6000 && addr <= 0x7FFF {
            return Some(self.vPrgRam[(addr & 0x1FFF) as usize] as u32);
        }
        else {
            // do PRG ROM stuff
        }

        return None;
	}

	fn cpuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32> {
        if addr >= 0x6000 && addr <= 0x7FFF {
            return Some((addr & 0x1FFF) as u32);
        }
        else {
            if val & 0x80 == 0x80 {
                self.shiftLatch = 0;
                self.shiftReg = 0x10;
            }
            else {
                self.shiftLatch += 1;
                self.shiftReg |= (self.shiftReg >> 1) | ((val & 1) << 4);

                if self.shiftLatch == 5 {

                }
            }
        }

        return None;
	}

	fn ppuMapRead(&mut self, addr: u16) -> Option<u32> {
		todo!()
	}

	fn ppuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32> {
		todo!()
	}

	fn getMirrorType(&self) -> MIRROR {
		todo!()
	}
}