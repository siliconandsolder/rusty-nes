#![allow(non_snake_case)]
#![allow(warnings)]

use std::path::{PathBuf, Path};
use std::fs;
use std::mem;
use std::fs::File;
use std::io::{BufReader, Read};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use crate::mapper::Mapper;
use crate::mapper0::Mapper0;
use std::borrow::BorrowMut;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Header {
    name: [u8; 3],
    prgSize: u8,
    chrSize: u8,
    mapper1: u8,
    mapper2: u8,
    prgRam: u8,
    tvSys1: u8,
    tvSys2: u8,
    unused: [u8; 5]
}

pub struct Cartridge {
    header: Header,
    mapperId: u8,
    numPrgBanks: u8,
    numChrBanks: u8,
    vPrgMem: Vec<u8>,
    vChrMem: Vec<u8>,
    pMapper: Box<dyn Mapper>
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


        let fileType: u8 = 0;
        let mut numPrgBanks: u8 = 0;
        let mut numChrBanks: u8 = 0;
        let mut prgMem: Vec<u8> = vec![0];
        let mut chrMem: Vec<u8> = vec![0];

        match fileType {
            0 => {},
            1 => {
                numPrgBanks = header.prgSize;
                numChrBanks = header.chrSize;

                prgMem.resize((numPrgBanks as u16 * 16384) as usize, 0);
                for i in 0..prgMem.len() { prgMem[i] = *fIter.next().unwrap(); }

                chrMem.resize((numChrBanks as u16 * 8192) as usize, 0);
                for i in 0..chrMem.len() { chrMem[i] = *fIter.next().unwrap(); }
            },
            2 => {},
            3 => {},
            4 => {},
            _ => panic!("Unknown file type: {}", fileType)
        }

        let mut mapper: Option<Box<dyn Mapper>> = None;

        match mapperId {
            0 => { mapper = Some(Box::new(Mapper0::new(numPrgBanks, numChrBanks))) }
            _ => panic!("Unknown mapper: {}", mapperId)
        }

        return Cartridge {
            header,
            mapperId,
            numChrBanks,
            numPrgBanks,
            vPrgMem: prgMem,
            vChrMem: chrMem,
            pMapper: mapper.unwrap()
        }
    }

    pub fn cpuRead(&mut self, ref addr: u16) -> u8 {
		let mut mapAddr = self.pMapper.cpuMapRead(*addr);
        if mapAddr.is_none() {
            mapAddr = Some(0);
        }
        return self.vPrgMem[mapAddr.unwrap() as usize];
    }

    pub fn cpuWrite(&mut self, ref addr: u16, val: u8) -> () {
        let mapAddr = self.pMapper.cpuMapWrite(*addr);
        if mapAddr.is_some() {
            self.vPrgMem[mapAddr.unwrap() as usize] = val;
        }
    }

    pub fn ppuRead(&mut self, ref addr: u16) -> u8 {
        let mut mapAddr = self.pMapper.ppuMapRead(*addr);
        if mapAddr.is_none() {
            mapAddr = Some(0);
        }
        return self.vChrMem[mapAddr.unwrap() as usize];
    }

    pub fn ppuWrite(&mut self, ref addr: u16, val: u8) -> () {
        let mapAddr = self.pMapper.ppuMapRead(*addr);
        if mapAddr.is_some() {
            self.vChrMem[mapAddr.unwrap() as usize] = val;
        }
    }
}