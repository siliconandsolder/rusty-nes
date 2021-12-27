#![allow(non_snake_case)]
#![allow(warnings)]

use std::convert::TryFrom;
use crate::mappers::mapper_four::chr_register::ChrRegister;
use crate::mappers::mapper_four::prg_register::PrgRegister;
use crate::mappers::mapper::{MirrorType, Mapper};
use crate::save_load::{Mapper4Data, MapperData};

mod chr_register;
mod prg_register;

pub struct Mapper4 {
    vChrBanks: Vec<u32>,
    vPrgBanks: Vec<u32>,
    vMemRegs: Vec<u32>,
    vPrgRam: Vec<u8>,
    secLastPrgBank: u16,
    lastPrgBank: u16,
    mirrorType: MirrorType,
    target: usize,
    prgBankMode: u8,
    chrInversion: u8,
    writeProtect: bool,
    prgRamEnabled: bool,

    // irq stuff
    irqCounter: u8,
    irqReload: u8,
    irqEnabled: bool,
    irqReady: bool,
}

impl Mapper4 {
    pub fn new(numPrgBanks: u8, numChrBanks: u8, mirrorType: MirrorType) -> Self {

        let mut vPrgBanks: Vec<u32> = vec![0; 4];
        vPrgBanks[1] = 0x2000;
        vPrgBanks[2] = (numPrgBanks as u32 * 2 - 2) * 0x2000;
        vPrgBanks[3] = (numPrgBanks as u32 * 2 - 1) * 0x2000;

        Mapper4 {
            vChrBanks: vec![0; 8],
            vPrgBanks,
            vMemRegs: vec![0; 8],
            vPrgRam: vec![0; 0x2000],
            secLastPrgBank: (numPrgBanks * 2 - 2) as u16,
            lastPrgBank: (numPrgBanks * 2 - 1) as u16,
            mirrorType,
            target: 0,
            prgBankMode: 0,
            chrInversion: 0,
            writeProtect: false,
            prgRamEnabled: false,
            irqCounter: 0,
            irqReload: 0,
            irqEnabled: false,
            irqReady: false
        }
    }
}

impl Mapper for Mapper4 {
    fn cpuMapRead(&mut self, addr: u16) -> Option<u32> {
        return match addr {
            0x6000..=0x7FFF => {
                Some(self.vPrgRam[(addr & 0x1FFF) as usize] as u32)
            }
            0x8000..=0x9FFF => {
                Some((self.vPrgBanks[0] + (addr & 0x1FFF) as u32))
            }
            0xA000..=0xBFFF => {
                Some((self.vPrgBanks[1] + (addr & 0x1FFF) as u32))
            }
            0xC000..=0xDFFF => {
                Some((self.vPrgBanks[2] + (addr & 0x1FFF) as u32))
            }
            0xE000..=0xFFFF => {
                Some((self.vPrgBanks[3] + (addr & 0x1FFF) as u32))
            }
            _ => {
                None
            }
        }
    }

    fn cpuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32> {
        match addr {
            0x6000..=0x7FFF => {
                self.vPrgRam[(addr & 0x1FFF) as usize] = val;
            }
            0x8000..=0x9FFF => {

                if addr % 2 == 0 {
                    self.target = (val & 7) as usize;
                    self.prgBankMode = (val >> 6) & 1;
                    self.chrInversion = (val >> 7) & 1;
                }
                else {
                    self.vMemRegs[self.target] = val as u32;

                    if self.chrInversion == 0 {
                        self.vChrBanks[0] = (self.vMemRegs[0] & 0xFE) * 0x400;
                        self.vChrBanks[1] = (self.vMemRegs[0] | 1) * 0x400; // skip to next bank
                        self.vChrBanks[2] = (self.vMemRegs[1] & 0xFE) * 0x400;
                        self.vChrBanks[3] = (self.vMemRegs[1] | 1) * 0x400;
                        self.vChrBanks[4] = self.vMemRegs[2] * 0x400;
                        self.vChrBanks[5] = self.vMemRegs[3] * 0x400;
                        self.vChrBanks[6] = self.vMemRegs[4] * 0x400;
                        self.vChrBanks[7] = self.vMemRegs[5] * 0x400;
                    }
                    else {
                        self.vChrBanks[0] = self.vMemRegs[2] * 0x400;
                        self.vChrBanks[1] = self.vMemRegs[3] * 0x400;
                        self.vChrBanks[2] = self.vMemRegs[4] * 0x400;
                        self.vChrBanks[3] = self.vMemRegs[5] * 0x400;
                        self.vChrBanks[4] = (self.vMemRegs[0] & 0xFE) * 0x400;
                        self.vChrBanks[5] = (self.vMemRegs[0] | 1) * 0x400; // skip to next bank
                        self.vChrBanks[6] = (self.vMemRegs[1] & 0xFE) * 0x400;
                        self.vChrBanks[7] = (self.vMemRegs[1] | 1) * 0x400;
                    }

                    if self.prgBankMode == 0 {
                        self.vPrgBanks[0] = (self.vMemRegs[6] & 0x3F) * 0x2000;
                        self.vPrgBanks[1] = (self.vMemRegs[7] & 0x3F) * 0x2000;
                        self.vPrgBanks[2] = (self.secLastPrgBank as u32 * 0x2000);
                        self.vPrgBanks[3] = (self.lastPrgBank as u32 * 0x2000);
                    }
                    else {
                        self.vPrgBanks[0] = (self.secLastPrgBank as u32 * 0x2000);
                        self.vPrgBanks[1] = (self.vMemRegs[7] & 0x3F) * 0x2000;
                        self.vPrgBanks[2] = (self.vMemRegs[6] & 0x3F) * 0x2000;
                        self.vPrgBanks[3] = (self.lastPrgBank as u32 * 0x2000);
                    }
                }

            },
            0xA000..=0xBFFF => {
                if addr % 2 == 0 {
                    self.mirrorType = if val & 1 == 0 { MirrorType::Vertical } else { MirrorType::Horizontal };
                }
                else {
                    self.prgRamEnabled = ((val >> 7) & 1 == 1);
                }
            }
            0xC000..=0xDFFF => {
                if addr % 2 == 0 {
                    self.irqReload = val;
                }
                else {
                    self.irqCounter = 0;
                }
            }
            0xE000..=0xFFFF => {
                if addr % 2 == 0 {
                    self.irqEnabled = false;
                }
                else {
                    self.irqEnabled = true;
                }
            }
            _ => {}
        }

        return None;
    }

    fn ppuMapRead(&mut self, addr: u16) -> Option<u32> {
        return match addr {
            0x0000..=0x03FF => {
                Some((self.vChrBanks[0] + (addr & 0x03FF) as u32))
            }
            0x0400..=0x07FF => {
                Some((self.vChrBanks[1] + (addr & 0x03FF) as u32))
            }
            0x0800..=0x0BFF => {
                Some((self.vChrBanks[2] + (addr & 0x03FF) as u32))
            }
            0x0C00..=0x0FFF => {
                Some((self.vChrBanks[3] + (addr & 0x03FF) as u32))
            }
            0x1000..=0x13FF => {
                Some((self.vChrBanks[4] + (addr & 0x03FF) as u32))
            }
            0x1400..=0x17FF => {
                Some((self.vChrBanks[5] + (addr & 0x03FF) as u32))
            }
            0x1800..=0x1BFF => {
                Some((self.vChrBanks[6] + (addr & 0x03FF) as u32))
            }
            0x1C00..=0x1FFF => {
                Some((self.vChrBanks[7] + (addr & 0x03FF) as u32))
            }
            _ => {
                None
            }
        }
    }

    fn ppuMapWrite(&mut self, addr: u16, val: u8) -> Option<u32> {
        return None;
    }

    fn getMirrorType(&self) -> MirrorType {
        return self.mirrorType;
    }

    fn isPrgRamEnabled(&self) -> bool {
        return self.prgRamEnabled;
    }

    fn checkIrq(&self) -> bool {
        return self.irqReady;
    }

    fn clearIrq(&mut self) -> () {
        self.irqReady = false;
    }

    fn cycleIrqCounter(&mut self) -> () {
        if self.irqCounter == 0 {
            self.irqCounter = self.irqReload;
        }
        else {
            self.irqCounter -= 1;

            if self.irqCounter == 0 && self.irqEnabled {
                self.irqReady = true;
            }
        }
    }

    fn saveState(&self) -> MapperData {
        MapperData::M4(
            Mapper4Data {
                vChrBanks: self.vChrBanks.clone(),
                vPrgBanks: self.vPrgBanks.clone(),
                vMemRegs: self.vMemRegs.clone(),
                vPrgRam: self.vPrgRam.clone(),
                secLastPrgBank: self.secLastPrgBank,
                lastPrgBank: self.lastPrgBank,
                mirrorType: self.mirrorType as u8,
                target: self.target,
                prgBankMode: self.prgBankMode,
                chrInversion: self.chrInversion,
                writeProtect: self.writeProtect,
                prgRamEnabled: self.prgRamEnabled,
                irqCounter: self.irqCounter,
                irqReload: self.irqReload,
                irqEnabled: self.irqEnabled,
                irqReady: self.irqReady
            }
        )
    }

    fn loadState(&mut self, data: &MapperData) -> () {
        match data {
            MapperData::M4(data) => {
                self.vChrBanks = data.vChrBanks.clone();
                self.vPrgBanks = data.vPrgBanks.clone();
                self.vMemRegs = data.vMemRegs.clone();
                self.vPrgRam = data.vPrgRam.clone();
                self.secLastPrgBank = data.secLastPrgBank;
                self.lastPrgBank = data.lastPrgBank;
                self.mirrorType = MirrorType::try_from(data.mirrorType).unwrap();
                self.target = data.target;
                self.prgBankMode = data.prgBankMode;
                self.chrInversion = data.chrInversion;
                self.writeProtect = data.writeProtect;
                self.prgRamEnabled = data.prgRamEnabled;
                self.irqCounter = data.irqCounter;
                self.irqReload = data.irqReload;
                self.irqEnabled = data.irqEnabled;
                self.irqReady = data.irqReady;
            }
            _ => { panic!("Wrong mapper type") }
        }
    }
}