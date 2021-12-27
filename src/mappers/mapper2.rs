#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use std::convert::TryFrom;
use crate::mappers::mapper::{MirrorType, Mapper};
use crate::save_load::{Mapper2Data, MapperData};

pub struct Mapper2 {
    switchBank: u8,
    lastBank: u8,
    hasChrRam: bool,
    mirrorType: MirrorType
}

impl Mapper2 {
    pub fn new(numPrgBanks: u8, numChrBanks: u8, mirrorType: MirrorType) -> Self {
        Mapper2 {
            switchBank: 0,
            lastBank: numPrgBanks - 1,
            hasChrRam: numChrBanks == 0,
            mirrorType: mirrorType
        }
    }
}

impl Mapper for Mapper2 {
    fn cpuMapRead(&mut self, addr: u16) -> Option<u32> {
        return match addr {
            0x8000..=0xBFFF => {
                Some(self.switchBank as u32 * 0x4000 + (addr & 0x3FFF) as u32)
            }
            0xC000..=0xFFFF => {
                Some(self.lastBank as u32 * 0x4000 + (addr & 0x3FFF) as u32)
            }
            _ => {
                None
            }
        };
    }

    fn cpuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32> {
        if addr >= 0x8000 {
            self.switchBank = val & 0x0F;
        }

        return None;
    }

    fn ppuMapRead(&mut self, addr: u16) -> Option<u32> {
        if addr < 0x2000 {
            return Some(addr as u32 & 0x1FFF);
        }

        return None;
    }

    fn ppuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32> {
        if addr < 0x2000 && self.hasChrRam {
            return Some (addr as u32 & 0x1FFF);
        }

        return None;
    }

    fn getMirrorType(&self) -> MirrorType {
        return self.mirrorType;
    }

    fn isPrgRamEnabled(&self) -> bool {
        return false;
    }

    fn checkIrq(&self) -> bool {
        return false;
    }

    fn clearIrq(&mut self) -> () {}

    fn cycleIrqCounter(&mut self) -> () {}

    fn saveState(&self) -> MapperData {
        MapperData::M2(
            Mapper2Data {
                switchBank: self.switchBank,
                lastBank: self.lastBank,
                hasChrRam: self.hasChrRam,
                mirrorType: self.mirrorType as u8
            }
        )
    }

    fn loadState(&mut self, data: &MapperData) -> () {
        match data {
            MapperData::M2(data) => {
                self.mirrorType = MirrorType::try_from(data.mirrorType).unwrap();
                self.switchBank = data.switchBank;
                self.lastBank = data.lastBank;
                self.hasChrRam = data.hasChrRam;
            }
            _ => { panic!("Wrong mapper type") }
        }
    }
}