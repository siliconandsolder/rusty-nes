#![allow(non_snake_case)]
#![allow(warnings)]

use std::path::{PathBuf, Path};
use std::fs;
use std::mem;
use std::fs::File;
use std::io::{BufReader, Read};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use crate::mappers::mapper0::Mapper0;
use crate::mappers::mapper::{Mapper, MirrorType};
use crate::mappers::mapper::MirrorType::{Vertical, Horizontal};
use crate::mappers::mapper_one::Mapper1;
use crate::mappers::mapper2::Mapper2;
use crate::mappers::mapper3::Mapper3;
use crate::mappers::mapper_four::Mapper4;
use crate::save_load::{CartData, CartHeaderData, MapperData};

const PRG_RAM_START: u16 = 0x6000;
const PRG_RAM_END: u16 = 0x7FFF;

enum FileType {
    AiNES,  // Archaic iNES
    iNES,
    NES2
}

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


        let mut numPrgBanks: u8 = 0;
        let mut numChrBanks: u8 = 0;
        let mut prgMem: Vec<u8> = vec![0];
        let mut chrMem: Vec<u8> = vec![0];

        let mut fileType: FileType = FileType::AiNES;
        if header.mapper2 & 0xC == 0x8 { fileType = FileType::NES2; }
        if header.mapper2 & 0xC == 0x0 && !header.unused.iter().any(|el| *el != 0) { fileType = FileType::iNES; }

        match fileType {
            FileType::AiNES => {

            }
            FileType::iNES => {
                numPrgBanks = header.prgSize;
                numChrBanks = header.chrSize;

                prgMem.resize(numPrgBanks as usize * 0x4000, 0);
                for i in 0..prgMem.len() { prgMem[i] = *fIter.next().unwrap(); }

                if numChrBanks == 0 {
                    chrMem.resize(0x2000, 0);
                }
                else {
                    chrMem.resize(numChrBanks as usize * 0x2000, 0);
                }

                let mut idx: usize = 0;
                while let Some(el) = fIter.next() {
                    chrMem[idx] = *el;
                    idx += 1;
                }
            }
            FileType::NES2 => {

            }
        }

        let mut mapper: Option<Box<dyn Mapper>> = None;
        let mirror: MirrorType = if header.mapper1 & 0x01 == 1 { Vertical } else { Horizontal };

        match mapperId {
            0 => { mapper = Some(Box::new(Mapper0::new(numPrgBanks, numChrBanks, mirror))) }
            1 => { mapper = Some(Box::new(Mapper1::new(numPrgBanks, numChrBanks, mirror))) }
            2 => { mapper = Some(Box::new(Mapper2::new(numPrgBanks, numChrBanks, mirror))) }
            3 => { mapper = Some(Box::new(Mapper3::new(mirror))) }
            4 => { mapper = Some(Box::new(Mapper4::new(numPrgBanks, numChrBanks, mirror))) }
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
            hasPrgRam,
        };
    }

    #[inline]
    pub fn cpuRead(&mut self, ref addr: u16) -> u8 {
        let mut mapAddr = self.pMapper.cpuMapRead(*addr);

        // check if PRG RAM
        if self.pMapper.isPrgRamEnabled() && *addr >= PRG_RAM_START && *addr <= PRG_RAM_END {
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

    pub fn cycleIrq(&mut self) -> () {
        self.pMapper.cycleIrqCounter();
    }

    pub fn checkIrq(&mut self) -> bool {
        if self.pMapper.checkIrq() {
            self.pMapper.clearIrq();
            return true;
        }
        return false;
    }

    pub fn getMirrorType(&self) -> MirrorType {
        return self.pMapper.getMirrorType();
    }
    
    pub fn saveState(&self) -> CartData {
        CartData{
            header: CartHeaderData {
                name: self.header.name,
                prgSize: self.header.prgSize,
                chrSize: self.header.chrSize,
                mapper1: self.header.mapper1,
                mapper2: self.header.mapper2,
                prgRam: self.header.prgRam,
                tvSys1: self.header.tvSys1,
                tvSys2: self.header.tvSys2,
                unused: self.header.unused
            },
            mapperId: self.mapperId,
            numPrgBanks: self.numPrgBanks,
            numChrBanks: self.numChrBanks,
            vChrMem: self.vChrMem.clone(),
            hasPrgRam: self.hasPrgRam
        }
    }
    
    pub fn loadState(&mut self, data: &CartData) -> () {
        self.header.name = data.header.name;
        self.header.prgSize = data.header.prgSize;
        self.header.chrSize = data.header.chrSize;
        self.header.mapper1 = data.header.mapper1;
        self.header.mapper2 = data.header.mapper2;
        self.header.prgRam = data.header.prgRam;
        self.header.tvSys1 = data.header.tvSys1;
        self.header.tvSys2 = data.header.tvSys2;
        self.header.unused = data.header.unused;

        self.mapperId = data.mapperId;
        self.numPrgBanks = data.numPrgBanks;
        self.numChrBanks = data.numChrBanks;
        self.vChrMem = data.vChrMem.clone();
        self.hasPrgRam = data.hasPrgRam;
    }

    pub fn saveMapperState(&self) -> MapperData {
        return self.pMapper.saveState();
    }

    pub fn loadMapperState(&mut self, data: &MapperData) -> () {
        self.pMapper.loadState(data);
    }
}