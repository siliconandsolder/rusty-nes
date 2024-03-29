#![allow(non_snake_case)]
#![allow(warnings)]

use std::convert::TryFrom;
use crate::mappers::mapper::{MirrorType, Mapper};
use crate::save_load::{Mapper0Data, MapperData};

pub struct Mapper0 {
    numPrgBanks: u8,
    numChrBanks: u8,
    mirrorType: MirrorType,
}

impl Mapper0 {
    pub fn new(numPrgBanks: u8, numChrBanks: u8, mirrorType: MirrorType) -> Self {
        Mapper0 {
            numPrgBanks,
            numChrBanks,
            mirrorType,
        }
    }
}

impl Mapper for Mapper0 {
    #[inline]
    fn cpuMapRead(&mut self, ref addr: u16) -> Option<u32> {
        if *addr >= 0x8000 && *addr <= 0xFFFF {
            return match self.numPrgBanks {
                1 => Some((*addr & 0x3FFF) as u32),
                _ => Some((*addr & 0x7FFF) as u32)
            };
        }

        return None;
    }

    #[inline]
    fn cpuMapWrite(&mut self, ref addr: u16, val: u8) -> Option<u32> {
        if *addr >= 0x8000 && *addr <= 0xFFFF {
            return match self.numPrgBanks {
                1 => Some((*addr & 0x3FFF) as u32),
                _ => Some((*addr & 0x7FFF) as u32)
            };
        }

        return None;
    }

    #[inline]
    fn ppuMapRead(&mut self, ref addr: u16) -> Option<u32> {
        if *addr >= 0x0000 && *addr <= 0x1FFF {
            return Some(*addr as u32);
        }

        return None;
    }

    #[inline]
    fn ppuMapWrite(&mut self, ref addr: u16, val: u8) -> Option<u32> {
        return None;    // nothing to write
    }

    #[inline]
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
        MapperData::M0(
            Mapper0Data {
                numPrgBanks: self.numPrgBanks,
                numChrBanks: self.numChrBanks,
                mirrorType: self.mirrorType as u8
            }
        )
    }

    fn loadState(&mut self, data: &MapperData) -> () {
        match data {
            MapperData::M0(data) => {
                self.mirrorType = MirrorType::try_from(data.mirrorType).unwrap();
                self.numChrBanks = data.numChrBanks;
                self.numPrgBanks = data.numPrgBanks;
            }
            _ => { panic!("Wrong mirror type") }
        }
    }
}