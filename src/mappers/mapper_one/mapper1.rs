#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use sdl2::gfx::imagefilter::add;
use crate::mappers::mapper::{Mapper, MIRROR};
use crate::mappers::mapper_one::control_register::{ControlRegister, PrgMode, ChrMode};
use crate::mappers::mapper_one::chr_register::ChrRegister;
use crate::mappers::mapper_one::prg_register::PrgRegister;

pub struct Mapper1 {
    shiftReg: u8,
    ctrlReg: ControlRegister,
    chrReg: ChrRegister,
    prgReg: PrgRegister,
    numPrgBanks: u8,
    numChrBanks: u8,
    vPrgRam: Vec<u8>
}

impl Mapper1 {
    pub fn new(numPrgBanks: u8, numChrBanks: u8, mirrorType: MIRROR) -> Self {
        Mapper1 {
            shiftReg: 0x10,
            ctrlReg: ControlRegister::new(mirrorType),
            chrReg: ChrRegister::new(),
            prgReg: PrgRegister::new(),
            numPrgBanks,
            numChrBanks,
            vPrgRam: vec![0; 0x2000]
        }
    }

    fn resetShiftRegister(&mut self) -> () {
        self.shiftReg = 0x10;
    }
}

impl Mapper for Mapper1 {
    fn cpuMapRead(&mut self, ref addr: u16) -> Option<u32> {

        if *addr >= 0x6000 && *addr <= 0x7FFF {
            return Some(self.vPrgRam[(*addr & 0x1FFF) as usize] as u32)
        }
        else if *addr >= 0x8000 {
            match self.ctrlReg.getPrgMode() {
                PrgMode::Switch32 => {
                    return Some(self.prgReg.bank32 as u32 * 0x8000 + (*addr & 0x7FFF) as u32)
                }
                PrgMode::FixFirstBank | PrgMode::FixLastBank => {
                    if *addr < 0xC000 {
                        return Some(self.prgReg.bankLo as u32 * 0x4000 + (*addr & 0x3FFF) as u32)
                    } else {
                        return Some(self.prgReg.bankHi as u32 * 0x4000 + (*addr & 0x3FFF) as u32)
                    }
                }
            }
        }

        return None;
    }

    fn cpuMapWrite(&mut self, ref addr: u16, ref val: u8) -> Option<u32> {

        if *addr >= 0x6000 && *addr <= 0x7FFF {
            self.vPrgRam[(*addr & 0x1FFF) as usize] = *val;
        }
        else if *addr >= 0x8000 {
            if val & 0x80 == 0x80 {
                self.shiftReg = 0x10;
            }
            else {
                let complete: bool = (self.shiftReg & 1) == 1;
                self.shiftReg >>= 1;
                self.shiftReg |= (*val & 1) << 4;

                if complete {

                    let register = (*addr >> 13) & 3;
                    match register {
                        0 => {
                            self.ctrlReg.setValues(self.shiftReg & 0x1F);
                        }
                        1 => {
                            if self.ctrlReg.getChrMode() == ChrMode::FourKilo {
                                self.chrReg.bankLo = self.shiftReg & 0x1F;
                            }
                            else {
                                self.chrReg.bank8 = self.shiftReg & 0x1E;
                            }
                        }
                        2 => {
                            if self.ctrlReg.getChrMode() == ChrMode::FourKilo {
                                self.chrReg.bankHi = self.shiftReg & 0x1F;
                            }
                        }
                        3 => {
                            match self.ctrlReg.getPrgMode() {
                                PrgMode::Switch32 => {
                                    self.prgReg.bank32 = self.shiftReg & 0xE >> 1;
                                }
                                PrgMode::FixFirstBank => {
                                    self.prgReg.bankLo = 0;
                                    self.prgReg.bankHi = self.shiftReg & 0xF;
                                }
                                PrgMode::FixLastBank => {
                                    self.prgReg.bankLo = self.shiftReg & 0xF;
                                    self.prgReg.bankHi = self.numPrgBanks - 1;
                                }
                            }
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
        if *addr < 0x2000 {
            if self.numChrBanks == 0 {
                return Some(*addr as u32);
            }


            return match self.ctrlReg.getChrMode() {
                ChrMode::EightKilo => {
                    Some((self.chrReg.bank8 as u32 * 0x2000 + *addr as u32))
                }
                ChrMode::FourKilo => {
                    if *addr < 0x1000 {
                        Some((self.chrReg.bankLo as u32 * 0x1000 + *addr as u32))
                    } else {
                        Some((self.chrReg.bankHi as u32 * 0x1000 + (*addr as u32 & 0x0FFF)))
                    }
                }
            }
        }

        return None;
    }

    fn ppuMapWrite(&mut self, ref addr: u16, ref val: u8) -> Option<u32> {

        if *addr < 0x2000 && self.numChrBanks == 0 {
            return Some(*addr as u32);
        }

        return None;
    }

    fn getMirrorType(&self) -> MIRROR {
        return self.ctrlReg.getMirrorMode();
    }
}