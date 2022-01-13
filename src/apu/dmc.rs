#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use std::cell::RefCell;
use crate::data_bus::DataBus;
use std::rc::Rc;

pub struct DMC {
    pub enabled: bool,
    pub irqEnabled: bool,
    pub loopEnabled: bool,
    pub ratePeriod: u16,
    pub rateValue: u16,
    pub directLoad: u8,
    pub bitCounter: u8,
    pub freq: u8,
    pub loadCounter: u8,
    pub sampleAddr: u16,
    pub curSampleAddr: u16,
    pub sampleLength: u16,
    pub curSampleLength: u16,
    pub shift: u8,
    pub dataBus: Rc<RefCell<DataBus>>,

}

impl DMC {
    pub fn new(dataBus: Rc<RefCell<DataBus>>) -> Self {
        DMC {
            enabled: false,
            irqEnabled: false,
            loopEnabled: false,
            ratePeriod: 0,
            rateValue: 0,
            directLoad: 0,
            bitCounter: 0,
            freq: 0,
            loadCounter: 0,
            sampleAddr: 0,
            curSampleAddr: 0,
            sampleLength: 0,
            curSampleLength: 0,
            shift: 0,
            dataBus: dataBus,
        }
    }

    pub fn writeIrqLoopFreq(&mut self, data: u8, ratePeriod: u16) -> () {
        self.irqEnabled = (data & 128) == 128;
        self.loopEnabled = (data & 64) == 64;
        self.ratePeriod = ratePeriod;
    }

    pub fn writeLoadCounter(&mut self, data: u8) -> () {
        self.bitCounter = data & 0b0111_1111;
    }

    pub fn writeSampleAddress(&mut self, data: u8) -> () {
        self.sampleAddr = (0xC000 | (data as u16 * 64 as u16));
    }

    pub fn writeSampleLength(&mut self, data: u8) -> () {
        self.sampleLength = ((data as u16) << 4) | 1;
    }

    pub fn clockRate(&mut self) -> () {
        if !self.enabled {
            return;
        }

        self.clockReader();

        if self.rateValue == 0 {
            self.rateValue = self.ratePeriod;
            self.updateShift();
        }
        else {
            self.rateValue -= 1;
        }
    }

    pub fn clockReader(&mut self) -> () {
        if self.curSampleLength > 0 && self.bitCounter == 0 {
            self.dataBus.borrow_mut().setDmcCpuStall();
            self.shift = self.dataBus.borrow().readCpuMem(self.curSampleAddr);

            self.curSampleAddr = self.curSampleAddr.wrapping_add(1);
            if self.curSampleAddr == 0 {
                self.curSampleAddr = 0x8000;
            }

            self.bitCounter = 8;

            self.curSampleLength -= 1;
            if self.curSampleLength == 0 && self.loopEnabled {
                self.reset();
            }
        }
    }

    pub fn reset(&mut self) -> () {
        self.curSampleAddr = self.sampleAddr;
        self.curSampleLength = self.sampleLength;
    }

    pub fn output(&self) -> u8 {
        return self.directLoad;
    }

    fn updateShift(&mut self) -> () {
        if self.bitCounter == 0 {
            return;
        }

        if self.shift & 1 == 1 {
            if self.directLoad <= 125 {
                self.directLoad += 2;
            }
        }
        else {
            if self.directLoad >= 2 {
                self.directLoad -= 2;
            }
        }

        self.shift >>= 1;
        self.bitCounter -= 1;
    }
}
