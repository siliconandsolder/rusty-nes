#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

pub struct Noise {
    pub enabled: bool,
    pub mode: bool,
    pub output: u8,
    pub lengthHalt: bool,

    pub constVolume: u8,
    pub volume: u8,
    pub envVolume: u8,
    pub envValue: u8,
    pub envPeriod: u8,
    pub envEnabled: bool,
    pub envLoop: bool,
    pub envStart: bool,

    pub shift: u16,

    pub timerPeriod: u16,
    pub timer: u16,
    pub lengthCounter: u8,
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            enabled: false,
            mode: false,
            output: 0,
            lengthHalt: false,
            constVolume: 0,
            volume: 0,
            envVolume: 0,
            envValue: 0,
            envPeriod: 0,
            envEnabled: false,
            envLoop: false,
            envStart: false,
            shift: 0,
            timerPeriod: 0,
            timer: 0,
            lengthCounter: 0,
        }
    }


    pub fn writeEnvelopeVolumeCounter(&mut self, data: u8) -> () {
        self.lengthHalt = (data & 0b0010_0000) == 0b0010_0000;
        self.envLoop = !self.lengthHalt;
        self.envEnabled = !((data & 0b0001_0000) == 0b0001_0000);
        self.envPeriod = data & 0b0000_1111;
        self.constVolume = self.envPeriod;
    }

    pub fn writeLoopNoise(&mut self, data: u8, noiseTimerVal: u16) -> () {
        self.mode = (data & 128) == 128;
        self.timerPeriod = noiseTimerVal;
    }

    pub fn writeLengthCounter(&mut self, lenTableVal: u8) -> () {
        self.lengthCounter = if self.enabled { lenTableVal } else { 0 };
        self.envStart = true;
    }

    pub fn clockTimer(&mut self) -> () {
        if self.timer == 0 {
            self.timer = self.timerPeriod;
            let shiftBit: u16 = if self.mode { 6 } else { 1 };
            let feedBack: u16 = (self.shift & 1) ^ ((self.shift >> shiftBit) & 1);
            self.shift >>= 1;
            self.shift |= feedBack << 14;
        }
        else {
            self.timer -= 1;
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

    pub fn clockLength(&mut self) -> () {
        if !self.lengthHalt && self.lengthCounter > 0 {
            self.lengthCounter -= 1;
        }
    }

    pub fn output(&mut self) -> u8 {
        if !self.enabled ||
            self.lengthCounter == 0 ||
            self.shift & 1 == 1 {
            return 0;
        }

        return if self.envEnabled {
            self.envVolume
        }
        else {
            self.constVolume
        };
    }
}
