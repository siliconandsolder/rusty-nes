#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

pub mod control_register;
pub mod prg_register;
pub mod chr_register;

use crate::mappers::mapper::{Mapper, MirrorType};
use crate::mappers::mapper_one::control_register::{ControlRegister, PrgMode, ChrMode};
use crate::mappers::mapper_one::chr_register::ChrRegister;
use crate::mappers::mapper_one::prg_register::PrgRegister;
use crate::save_load::{Mapper1ChrRegData, Mapper1CtrlRegData, Mapper1Data, Mapper1PrgRegData, MapperData};

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
    pub fn new(numPrgBanks: u8, numChrBanks: u8, mirrorType: MirrorType) -> Self {
        Mapper1 {
            shiftReg: 0x10,
            ctrlReg: ControlRegister::new(mirrorType),
            chrReg: ChrRegister::new(),
            prgReg: PrgRegister::new(numPrgBanks),
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
            return Some(self.vPrgRam[(*addr & 0x1FFF) as usize] as u32);
        }
        else if *addr >= 0x8000 {
            match self.ctrlReg.getPrgMode() {
                PrgMode::Switch32 => {
                    return Some(self.prgReg.bank32 as u32 * 0x8000 + (*addr & 0x7FFF) as u32);
                }
                PrgMode::FixFirstBank | PrgMode::FixLastBank => {
                    return if *addr < 0xC000 {
                        Some(self.prgReg.bankLo as u32 * 0x4000 + (*addr & 0x3FFF) as u32)
                    } else {
                        Some(self.prgReg.bankHi as u32 * 0x4000 + (*addr & 0x3FFF) as u32)
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
                self.ctrlReg.reset();
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
                                self.chrReg.bank8 = (self.shiftReg & 0x1E) >> 1;
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
                                    self.prgReg.bank32 = (self.shiftReg & 0xE) >> 1;
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

                            self.prgReg.prgRamEnabled = ((self.shiftReg >> 4) & 1 != 1);
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

    fn getMirrorType(&self) -> MirrorType {
        return self.ctrlReg.getMirrorMode();
    }

    fn isPrgRamEnabled(&self) -> bool {
        return self.prgReg.isPrgRamEnabled();
    }

    fn checkIrq(&self) -> bool {
        return false;
    }

    fn clearIrq(&mut self) -> () {}

    fn cycleIrqCounter(&mut self) -> () {}

    fn saveState(&self) -> MapperData {
        MapperData::M1(
            Mapper1Data {
                shiftReg: self.shiftReg,
                ctrlReg: Mapper1CtrlRegData {
                    mirrorMode: self.ctrlReg.getMirrorMode() as u8,
                    prgMode: self.ctrlReg.getPrgMode() as u8,
                    chrMode: self.ctrlReg.getChrMode() as u8,
                    registerValue: self.ctrlReg.getRegisterValue()
                },
                chrReg: Mapper1ChrRegData {
                    bankLo: self.chrReg.bankLo,
                    bankHi: self.chrReg.bankHi,
                    bank8: self.chrReg.bank8
                },
                prgReg: Mapper1PrgRegData {
                    bankLo: self.prgReg.bankLo,
                    bankHi: self.prgReg.bankHi,
                    bank32: self.prgReg.bank32,
                    prgRamEnabled: self.prgReg.prgRamEnabled
                },
                numPrgBanks: self.numPrgBanks,
                numChrBanks: self.numChrBanks,
                vPrgRam: self.vPrgRam.clone()
            }
        )
    }

    fn loadState(&mut self, data: &MapperData) -> () {
        match data {
            MapperData::M1(data) => {
                self.shiftReg = data.shiftReg;
                self.numPrgBanks = data.numPrgBanks;
                self.numChrBanks = data.numChrBanks;
                self.vPrgRam = data.vPrgRam.clone();

                // ctrl reg
                self.ctrlReg.setValues(data.ctrlReg.registerValue);

                // chr reg
                self.chrReg.bankLo = data.chrReg.bankLo;
                self.chrReg.bankHi = data.chrReg.bankHi;
                self.chrReg.bank8 = data.chrReg.bank8;

                // prg reg
                self.prgReg.bankLo = data.prgReg.bankLo;
                self.prgReg.bankHi = data.prgReg.bankHi;
                self.prgReg.bank32 = data.prgReg.bank32;
                self.prgReg.prgRamEnabled = data.prgReg.prgRamEnabled;
            }
            _ => { panic!("Wrong mapper type") }
        }
    }
}