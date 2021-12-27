#![allow(non_snake_case)]
#![allow(warnings)]

use crate::save_load::MapperData;
use num_enum::TryFromPrimitive;

#[repr(u8)]
#[derive(PartialOrd, PartialEq, TryFromPrimitive, Debug, Copy, Clone)]
pub enum MirrorType {
    SingleScreenLow = 0,
    SingleScreenHigh = 1,
    Vertical = 2,
    Horizontal = 3,
}

pub trait Mapper {
    fn cpuMapRead(&mut self, addr: u16) -> Option<u32>;
    fn cpuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32>;
    fn ppuMapRead(&mut self, addr: u16) -> Option<u32>;
    fn ppuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32>;

    fn getMirrorType(&self) -> MirrorType;
    fn isPrgRamEnabled(&self) -> bool;

    // irq stuff
    fn checkIrq(&self) -> bool;
    fn clearIrq(&mut self) -> ();
    fn cycleIrqCounter(&mut self) -> ();

    // save states
    fn saveState(&self) -> MapperData;
    fn loadState(&mut self, data: &MapperData) -> ();
}