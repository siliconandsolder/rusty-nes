#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use std::convert::TryFrom;
use crate::mappers::mapper::{Mapper, MirrorType};
use crate::save_load::{Mapper3Data, MapperData};

pub struct Mapper3 {
    chrBank: u8,
    mirrorType: MirrorType
}

impl Mapper3 {
    pub fn new(mirrorType: MirrorType) -> Self {
        Mapper3 {
            chrBank: 0,
            mirrorType
        }
    }
}

impl Mapper for Mapper3 {
    fn cpuMapRead(&mut self, addr: u16) -> Option<u32> {
        if addr > 0x7FFF {
            return Some(addr as u32 & 0x7FFF);
        }

        return None;
    }

    fn cpuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32> {
        if addr > 0x7FFF {
            self.chrBank = val & 3;
        }

        return None;
    }

    fn ppuMapRead(&mut self, addr: u16) -> Option<u32> {
        return Some(self.chrBank as u32 * 0x2000 + (addr & 0x1FFF) as u32);
    }

    fn ppuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32> {
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
        MapperData::M3(
            Mapper3Data {
                chrBank: self.chrBank,
                mirrorType: self.mirrorType as u8
            }
        )
    }

    fn loadState(&mut self, data: &MapperData) -> () {
        match data {
            MapperData::M3(data) => {
                self.chrBank = data.chrBank;
                self.mirrorType = MirrorType::try_from(data.mirrorType).unwrap();
            }
            _ => { panic!("Wrong mapper type") }
        }
    }
}