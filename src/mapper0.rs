#![allow(non_snake_case)]
#![allow(warnings)]

use crate::mapper::Mapper;

pub struct Mapper0 {
    numPrgBanks: u8,
    numChrBanks: u8
}

impl Mapper0 {
    pub fn new(numPrgBanks: u8, numChrBanks: u8) -> Self {
        Mapper0 {
            numPrgBanks,
            numChrBanks
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
    fn cpuMapWrite(&mut self, ref addr: u16) -> Option<u32> {
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
    fn ppuMapWrite(&mut self, ref addr: u16) -> Option<u32> {
        return None;    // nothing to write
    }
}