#![allow(non_snake_case)]
#![allow(warnings)]

use crate::mappers::mapper::MirrorType;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PrgMode {
    Switch32,
    FixFirstBank,
    FixLastBank
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ChrMode {
    EightKilo,
    FourKilo,
}

pub struct ControlRegister {
    mirrorMode: MirrorType,
    prgMode: PrgMode,
    chrMode: ChrMode,
    registerValue: u8
}

impl ControlRegister {
    pub fn new(initMirrorType: MirrorType) -> Self {
        ControlRegister {
            mirrorMode: initMirrorType,
            prgMode: PrgMode::FixLastBank,
            chrMode: ChrMode::EightKilo,
            registerValue: 0xC
        }
    }

    pub fn setValues(&mut self, regVal: u8) {
        self.registerValue = regVal;
        match regVal & 3 {
            0 => {
                self.mirrorMode = MirrorType::SingleScreenLow;
            }
            1 => {
                self.mirrorMode = MirrorType::SingleScreenHigh;
            }
            2 => {
                self.mirrorMode = MirrorType::Vertical;
            }
            3 => {
                self.mirrorMode = MirrorType::Horizontal;
            }
            _ => { panic!("Should never reach this."); }
        }

        match (regVal >> 2) & 3 {
            0 | 1 => {
                self.prgMode = PrgMode::Switch32;
            }
            2 => {
                self.prgMode = PrgMode::FixFirstBank;
            }
            3 => {
                self.prgMode = PrgMode::FixLastBank;
            }
            _ => { panic!("Should never reach this."); }
        }

        if (regVal >> 4) & 1 == 1 {
            self.chrMode = ChrMode::FourKilo;
        }
        else {
            self.chrMode = ChrMode::EightKilo;
        }
    }

    pub fn reset(&mut self) -> () {
        self.setValues(self.registerValue | 0xC);
    }

    pub fn getMirrorMode(&self) -> MirrorType {
        return self.mirrorMode;
    }

    pub fn getPrgMode(&self) -> PrgMode {
        return self.prgMode;
    }

    pub fn getChrMode(&self) -> ChrMode {
        return self.chrMode;
    }
}