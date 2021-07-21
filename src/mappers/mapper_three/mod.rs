#![allow(non_snake_case)]
#![allow(warnings)]

use crate::mappers::mapper_three::chr_register::ChrRegister;
use crate::mappers::mapper_three::prg_register::PrgRegister;
use crate::mappers::mapper::{MirrorType, Mapper};

mod chr_register;
mod prg_register;

pub struct Mapper3 {
    chrReg: ChrRegister,
    prgReg: PrgRegister,
    numPrgBanks: u8,
    numChrBanks: u8,
    mirrorType: MirrorType,
}

impl Mapper3 {
    pub fn new(numPrgBanks: u8, numChrBanks: u8, mirrorType: MirrorType) -> Self {
        Mapper3 {
            chrReg: ChrRegister::new(),
            prgReg: PrgRegister::new(numPrgBanks),
            numPrgBanks,
            numChrBanks,
            mirrorType
        }
    }
}

impl Mapper for Mapper3 {
    fn cpuMapRead(&mut self, addr: u16) -> Option<u32> {
        todo!()
    }

    fn cpuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32> {
        match addr {
            0x8000..=0x9FFF => {
                if addr % 2 == 0 {

                }
                else {

                }
            }
            _ => {}
        }

        return None;
    }

    fn ppuMapRead(&mut self, addr: u16) -> Option<u32> {
        todo!()
    }

    fn ppuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32> {
        todo!()
    }

    fn getMirrorType(&self) -> MirrorType {
        return self.mirrorType;
    }
}