#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use crate::mapper::{Mapper, MIRROR};
use sdl2::gfx::imagefilter::add;

pub struct Mapper1 {
    shiftReg: u8,
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
    fn cpuMapRead(&mut self, ref addr: u16) -> Option<u32> {

        if *addr < 0x8000 {
            return Some(self.vPrgRam[(*addr & 0x1FFF) as usize] as u32);
        }
        else {
            let prgMode = (self.ctrlReg >> 2) & 3;
            match prgMode {
                0 | 1 => {
                    return Some(((self.prgBank & 0x1E) as u16 * 0x8000 + (*addr & 0x7FFF)) as u32);
                }
                2 => {
                    if *addr < 0xC000 {
                        return Some((*addr & 0x3FFF) as u32);
                    }
                    else {
                        return Some((self.prgBank as u16 * 0x4000 + (*addr & 0x3FFF)) as u32);
                    }
                }
                3 => {
                    if *addr >= 0xC000 {
                        return Some(((self.numPrgBanks - 1) as u16 * 0x4000 + (*addr & 0x3FFF)) as u32);
                    }
                    else {
                        return Some((self.prgBank as u16 * 0x4000 + (*addr & 0x3FFF)) as u32);
                    }
                }
                _ => {}
            }
        }

        return None;
    }

    fn cpuMapWrite(&mut self, ref addr: u16, ref val: u8) -> Option<u32> {

        if *addr < 0x8000 {
            self.vPrgRam[(*addr & 0x1FFF) as usize] = *val;
        }
        else if *addr >= 0x8000 {
            if val & 0x80 == 0x80 {
                self.shiftReg = 0;
                self.ctrlReg |= 0xC0;
            }
            else {
                let complete: bool = (self.shiftReg & 1) == 1;
                self.shiftReg >>= 1;
                self.shiftReg |= (*val & 1) << 4;

                if complete {

                    let register = (*addr >> 13) & 3;
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
                            self.prgBank = self.shiftReg;
                        }
                        _ => {}
                    }

                    self.shiftReg = 0x10;
                }
            }
        }


        return None;
    }

    fn ppuMapRead(&mut self, ref addr: u16) -> Option<u32> {
        if *addr < 2000 {
            if self.numChrBanks == 0 {
                return Some(*addr as u32);
            }

            let chrMode = (self.ctrlReg >> 4) & 1;

            match chrMode {
                0 => {
                    return Some(((self.chrBank0 as u16 & 0x1E) * 0x2000 + *addr) as u32);
                }
                1 => {
                    if *addr < 0x1000 {
                        return Some((self.chrBank0 as u16 * 0x1000 + *addr) as u32);
                    }
                    else {
                        return Some((self.chrBank1 as u16 * 0x1000 + (*addr & 0x0FFF)) as u32);
                    }
                }
                _ => {}
            }
        }

        return None;
    }

    fn ppuMapWrite(&mut self, ref addr: u16, ref val: u8) -> Option<u32> {

        if *addr < 2000 && self.numChrBanks == 0 {
            return Some(*addr as u32);
        }

        return None;
    }

    fn getMirrorType(&self) -> MIRROR {
        match self.ctrlReg & 3 {
            0 => {
                return MIRROR::ONESCREEN_LO;
            }
            1 => {
                return MIRROR::ONESCREEN_HI;
            }
            2 => {
                return MIRROR::VERTICAL;
            }
            3 => {
                return MIRROR::HORIZONTAL;
            }
            _ => { panic!("Should never reach this."); }
        }
    }
}