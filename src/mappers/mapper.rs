#![allow(non_snake_case)]
#![allow(warnings)]

#[derive(Debug, Copy, Clone)]
pub enum MirrorType {
    SingleScreenLow,
    SingleScreenHigh,
    Vertical,
    Horizontal,
}

pub trait Mapper {
    fn cpuMapRead(&mut self, addr: u16) -> Option<u32>;
    fn cpuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32>;
    fn ppuMapRead(&mut self, addr: u16) -> Option<u32>;
    fn ppuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32>;

    fn getMirrorType(&self) -> MirrorType;
}