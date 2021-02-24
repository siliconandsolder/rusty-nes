#![allow(non_snake_case)]
#![allow(warnings)]

use std::path::{PathBuf, Path};
use std::fs;
use std::mem;
use std::fs::File;
use std::io::{BufReader, Read};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use crate::mapper::{Mapper, MIRROR};
use crate::mapper0::Mapper0;
use std::borrow::{BorrowMut, Borrow};
use crate::mapper::MIRROR::*;
use crate::mapper1::Mapper1;

const PRG_RAM_START: u16 = 0x6000;
const PRG_RAM_END: u16 = 0x7FFF;


#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Header {
    name: [u8; 4],
    prgSize: u8,
    chrSize: u8,
    mapper1: u8,
    mapper2: u8,
    prgRam: u8,
    tvSys1: u8,
    tvSys2: u8,
    unused: [u8; 5],
}

pub struct Cartridge {
    header: Header,
    mapperId: u8,
    numPrgBanks: u8,
    numChrBanks: u8,
    vPrgMem: Vec<u8>,
    vChrMem: Vec<u8>,
    pMapper: Box<dyn Mapper>,
    mirror: MIRROR,
    hasPrgRam: bool,
}

impl Cartridge {
    pub fn new(romPath: &Path) -> Self {
        let mut fileBuf = fs::read(romPath).unwrap();

        // load header using C-like memory management
        let mut header: Header = unsafe { mem::zeroed() };
        let headerSize = mem::size_of::<Header>();

        unsafe {
            let headerSlice = slice_from_raw_parts_mut(&mut header as *mut _ as *mut u8, headerSize).as_mut().unwrap();
            fileBuf.as_slice().read_exact(headerSlice);
        }

        // get the rest of the cartridge details
        let mut fIter = fileBuf.iter();
        for _ in 0..headerSize { fIter.next(); }

        // don't care about trainer data
        if header.mapper1 & 0x04 == 0x04 {
            for _ in 0..512 { fIter.next(); }
        }

        let mapperId = (header.mapper2 & 0b11110000) | (header.mapper1 >> 4);


        let fileType: u8 = 1;
        let mut numPrgBanks: u8 = 0;
        let mut numChrBanks: u8 = 0;
        let mut prgMem: Vec<u8> = vec![0];
        let mut chrMem: Vec<u8> = vec![0];

        match fileType {
            0 => {}
            1 => {
                numPrgBanks = header.prgSize;
                numChrBanks = header.chrSize;

                prgMem.resize(numPrgBanks as usize * 16384, 0);
                for i in 0..prgMem.len() { prgMem[i] = *fIter.next().unwrap(); }

                chrMem.resize(numChrBanks as usize * 8192, 0);
                for i in 0..chrMem.len() { chrMem[i] = *fIter.next().unwrap(); }
            }
            2 => {}
            3 => {}
            4 => {}
            _ => panic!("Unknown file type: {}", fileType)
        }

        let mut mapper: Option<Box<dyn Mapper>> = None;
        let mirror: MIRROR = if header.mapper1 & 0x01 == 1 { VERTICAL } else { HORIZONTAL };

        match mapperId {
            0 => { mapper = Some(Box::new(Mapper0::new(numPrgBanks, numChrBanks, mirror))) }
            1 => { mapper = Some(Box::new(Mapper1::new(numPrgBanks, numChrBanks))) }
            _ => panic!("Unknown mapper: {}", mapperId)
        }

        let hasPrgRam = header.mapper1 & 2 == 2;


        return Cartridge {
            header,
            mapperId,
            numChrBanks,
            numPrgBanks,
            vPrgMem: prgMem,
            vChrMem: chrMem,
            pMapper: mapper.unwrap(),
            mirror,
            hasPrgRam,
        };
    }

    #[inline]
    pub fn cpuRead(&mut self, ref addr: u16) -> u8 {
        let mut mapAddr = self.pMapper.cpuMapRead(*addr);

        // check if PRG RAM
        if self.hasPrgRam && *addr >= PRG_RAM_START && *addr <= PRG_RAM_END {
            if mapAddr.is_none() {
                return 0;
            }
            return mapAddr.unwrap() as u8;
        }

        if mapAddr.is_none() {
            return 0;
        }
        return *self.vPrgMem.get(mapAddr.unwrap() as usize)
            .unwrap_or_else(|| -> &u8 { &0 });
    }

    #[inline]
    pub fn cpuWrite(&mut self, ref addr: u16, val: u8) -> () {
        // no need to check if battery-backed PRG RAM because we're not returning anything
        let mapAddr = self.pMapper.cpuMapWrite(*addr, val);
        if mapAddr.is_some() {
            match self.vPrgMem.get_mut(mapAddr.unwrap() as usize) {
                Some(x) => { *x = val; }
                _ => {}
            }
        }
    }

    #[inline]
    pub fn ppuRead(&mut self, ref addr: u16) -> u8 {
        let mut mapAddr = self.pMapper.ppuMapRead(*addr);
        if mapAddr.is_none() {
            return 0;
        }
        return *self.vChrMem.get(mapAddr.unwrap() as usize)
            .unwrap_or_else(|| -> &u8 { &0 });
    }

    #[inline]
    pub fn ppuWrite(&mut self, ref addr: u16, val: u8) -> () {
        let mapAddr = self.pMapper.ppuMapWrite(*addr, val);
        if mapAddr.is_some() {
            match self.vChrMem.get_mut(mapAddr.unwrap() as usize) {
                Some(x) => { *x = val; }
                _ => {}
            }
        }
    }

    pub fn getMirrorType(&self) -> MIRROR {
        return self.pMapper.as_ref().borrow().getMirrorType();
    }
}