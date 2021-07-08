#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use super::utils::SQUARE_SEQUENCE_TABLE;

pub struct Pulse {
    pub isChannelOne: bool,

    pub enabled: bool,
    pub dutyValue: u8,
    pub dutyMode: u8,
    pub output: u8,
    pub lengthHalt: bool,

    pub constVolume: bool,
    pub volume: u8,
    pub envVolume: u8,
    pub envValue: u8,
    pub envPeriod: u8,
    pub envEnabled: bool,
    pub envLoop: bool,
    pub envStart: bool,

    pub sweepEnabled: bool,
    pub sweepReload: bool,
    pub sweepPeriod: u8,
    pub sweepValue: u8,
    pub negate: bool,
    pub shift: u8,

    pub timerPeriod: u16,
    pub timer: u16,
    pub lengthCounter: u8,
    pub sample: u8,
}

impl Pulse {
    pub fn new(isChannelOne: bool) -> Self {
        Pulse {
            isChannelOne: isChannelOne,
            enabled: false,
            dutyValue: 0,
            dutyMode: 0,
            output: 0,
            lengthHalt: false,
            constVolume: false,
            volume: 0,
            envVolume: 0,
            envValue: 0,
            envPeriod: 0,
            envEnabled: false,
            envLoop: false,
            envStart: false,
            sweepEnabled: false,
            sweepReload: false,
            sweepPeriod: 0,
            sweepValue: 0,
            negate: false,
            shift: 0,
            timerPeriod: 0,
            timer: 0,
            lengthCounter: 0,
            sample: 0,
        }
    }

    pub fn writeDuty(&mut self, data: u8) -> () {
        self.dutyMode = (data & 0b1100_0000) >> 6;
        self.lengthHalt = (data & 0b0010_0000) == 0b0010_0000;
        self.envLoop = self.lengthHalt;
        self.constVolume = (data & 0b0001_0000) == 0b0001_0000;
        self.envEnabled = !self.constVolume;
        self.envPeriod = data & 0b0000_1111;
        self.volume = self.envPeriod;
        self.envStart = true;
    }

    pub fn writeSweep(&mut self, data: u8) -> () {
        self.sweepEnabled = (data & 0b1000_0000) == 0b1000_0000;
        self.sweepPeriod = ((data & 0b0111_0000) >> 4);
        self.negate = (data & 0b0000_1000) == 0b0000_1000;
        self.shift = data & 0b0000_0111;
        self.sweepReload = true;
    }

    pub fn writeTimer(&mut self, data: u8) -> () {
        self.timerPeriod = self.timerPeriod & 0xFF00 | data as u16
    }

    pub fn writeLengthCounter(&mut self, data: u8, lenTableVal: u8) -> () {
        self.timerPeriod = (self.timerPeriod & 0x00FF) | ((data as u16 & 0b000_00111 as u16) << 8) as u16;
        self.lengthCounter = if self.enabled { lenTableVal } else { 0 };
        self.envStart = true;
        self.dutyValue = 0;
    }

    pub fn clockTimer(&mut self) -> () {
        if self.timer == 0 {
            self.timer = self.timerPeriod;
            self.dutyValue = (self.dutyValue + 1) % 8;
        }
        else {
            self.timer -= 1;
        }
    }

    pub fn clockLengthCounter(&mut self) -> () {
        if !self.lengthHalt && self.lengthCounter > 0 {
            self.lengthCounter -= 1;
        }
    }

    pub fn clockEnvelope(&mut self) -> () {
        if self.envStart {
            self.envVolume = 15;
            self.envValue = self.envPeriod;
            self.envStart = false;
        }
        else if self.envValue > 0 {
            self.envValue -= 1;
        }
        else {
            self.envValue = self.envPeriod;

            if self.envLoop && self.envVolume == 0 {
                self.envVolume = 15;
            }
            else if self.envVolume > 0 {
                self.envVolume -= 1;
            }
        }
    }

    pub fn clockSweep(&mut self) -> () {
        if self.sweepReload {
            if self.sweepEnabled && self.sweepValue == 0 {
                self.sweep();
            }

            self.sweepValue = self.sweepPeriod;
            self.sweepReload = false;
        }
        else if self.sweepValue > 0 {
            self.sweepValue -= 1;
        }
        else {
            if self.sweepEnabled {
                self.sweep();
            }
            self.sweepValue = self.sweepPeriod;
        }
    }

    pub fn sweep(&mut self) -> () {
        let delta = self.timerPeriod >> self.shift;
        if self.negate {
            self.timerPeriod -= delta;

            if self.isChannelOne {
                self.timerPeriod -= 1;
            }
        }
        else {
            self.timerPeriod += delta;
        }
    }

    pub fn output(&self) -> u8 {
        return if !self.enabled ||
            self.lengthCounter == 0 ||
            self.timerPeriod < 8 ||
            self.timerPeriod > 0x7FF ||
            SQUARE_SEQUENCE_TABLE[self.dutyMode as usize][self.dutyValue as usize] == 0 {
            0
        }
        else if self.envEnabled {
            self.envVolume
        }
        else {
            self.volume
        };
    }
}
