#![allow(non_snake_case)]
#![allow(warnings)]

use crate::mappers::mapper::MIRROR;

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
    mirrorMode: MIRROR,
    prgMode: PrgMode,
    chrMode: ChrMode,
    registerValue: u8
}

impl ControlRegister {
    pub fn new(initMirrorType: MIRROR) -> Self {
        ControlRegister {
            mirrorMode: initMirrorType,
            prgMode: PrgMode::FixLastBank,
            chrMode: ChrMode::EightKilo,
            registerValue: 0x0C
        }
    }

    pub fn setValues(&mut self, regVal: u8) {
        self.registerValue = regVal;
        match regVal & 3 {
            0 => {
                self.mirrorMode = MIRROR::ONESCREEN_LO;
            }
            1 => {
                self.mirrorMode = MIRROR::ONESCREEN_HI;
            }
            2 => {
                self.mirrorMode = MIRROR::VERTICAL;
            }
            3 => {
                self.mirrorMode = MIRROR::HORIZONTAL;
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
        self.setValues(self.registerValue | 0x0C);
    }

    pub fn getMirrorMode(&self) -> MIRROR {
        return self.mirrorMode;
    }

    pub fn getPrgMode(&self) -> PrgMode {
        return self.prgMode;
    }

    pub fn getChrMode(&self) -> ChrMode {
        return self.chrMode;
    }
}