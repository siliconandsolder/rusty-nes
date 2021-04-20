#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use crate::mapper::{Mapper, MIRROR};
use sdl2::gfx::imagefilter::add;
use sdl2::mouse::SystemCursor::No;
use sdl2::mouse::MouseButton::Middle;

pub struct Mapper1 {
    shiftReg: u8,
    shiftLatch: u8,

    ctrlReg: u8,
    chrBankMode0: u8,
    chrBankMode1: u8,
    prgBankMode: u8,

    chrLo: u8,
    chrHi: u8,
    prgLo: u8,
    prgHi: u8,

    numPrgBanks: u8,
    numChrBanks: u8,
    mirrorMode: MIRROR,

    vPrgRam: Vec<u8>,
}

impl Mapper1 {
    pub fn new(numPrgBanks: u8, numChrBanks: u8) -> Self {
        Mapper1 {
            shiftReg: 0,
            shiftLatch: 0,
            ctrlReg: 0,
            chrBankMode0: 0,
            chrBankMode1: 0,
            prgBankMode: 0,
            chrLo: 0,
            chrHi: 0,
            prgLo: 0,
            prgHi: 0,
            numPrgBanks,
            numChrBanks,
            mirrorMode: MIRROR::ONESCREEN_LO,
            vPrgRam: vec![0; 0x8000],
		}
    }

    fn resetShiftRegister(&mut self) -> () {
        self.shiftReg = 0;
        self.shiftLatch = 0;
    }

    fn updateOffsets(&mut self) -> () {
        match (self.ctrlReg & 0b01100) >> 2 {
            0 | 1 => {

            }
            _ => {}
        }
    }
}

impl Mapper for Mapper1 {
	fn cpuMapRead(&mut self, addr: u16) -> Option<u32> {
		// if addr >= 0x6000 && addr <= 0x7FFF {
        //     return Some(self.vPrgRam[(addr & 0x1FFF) as usize] as u32);
        // }
        // else {
        //     // do PRG ROM stuff
        // }

        return None;
	}

	fn cpuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32> {
        // if addr >= 0x6000 && addr <= 0x7FFF {
        //     return Some((addr & 0x1FFF) as u32);
        // }
        // else {}

        if val & 0x80 == 0x80 {
           self.resetShiftRegister();
        }
        else {
            self.shiftLatch += 1;
            self.shiftReg |= (self.shiftReg >> 1) | ((val & 1) << 4);

            if self.shiftLatch == 5 {

                match (addr >> 13) & 0x03 {
                    0 => {
                        self.ctrlReg = self.shiftReg;

                        match self.shiftReg & 0x3 {
                            0 => {
                                self.mirrorMode = MIRROR::ONESCREEN_LO;
                            }
                            1 => {
                                self.mirrorMode = MIRROR::ONESCREEN_HI;
                            }
                            2 => {
                                self.mirrorMode = MIRROR::VERTICAL;
                            }
                            3 => {
                                self.mirrorMode = MIRROR::HORIZONTAL;
                            }
                            _ => {}
                        }
                        // update offsets

                    }
                    1 => {
                        self.chrBankMode0 = self.shiftReg;
                    }
                    2 => {
                        self.chrBankMode1 = self.shiftReg;
                    }
                    3 => {
                        self.prgBankMode = self.shiftReg;
                    }
                    _ => {}
                }

                self.resetShiftRegister();
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
		return self.mirrorMode;
	}


}