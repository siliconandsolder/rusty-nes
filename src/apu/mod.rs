#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use ndarray::{Array, array, Array1, Array2};

use audio::Audio;

use crate::clock::Clocked;
use crate::data_bus::DataBus;
use crate::apu::noise::Noise;
use crate::apu::pulse::Pulse;
use crate::apu::triangle::Triangle;
use crate::apu::dmc::DMC;
use utils::*;

pub mod utils;
pub mod filter;
pub mod pulse;
pub mod triangle;
pub mod noise;
pub mod dmc;
pub mod audio;


/*
 |  0   1   2   3   4   5   6   7    8   9   A   B   C   D   E   F
-----+----------------------------------------------------------------
00-0F  10,254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
10-1F  12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
*/

pub struct Apu<'a> {
    frame: u16,
    globalTime: f64,
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    dmc: DMC<'a>,
    dataBus: Rc<RefCell<DataBus<'a>>>,
    noise: Noise,
    fiveStep: bool,
    frameInterrupt: bool,
    inhibitInterrupt: bool,
    audio: Audio,

    lengthTable: Vec<u8>,
    pulseTable: Vec<f32>,
    tndTable: Vec<f32>,
}

impl<'a> Apu<'a> {
    pub fn new(dataBus: Rc<RefCell<DataBus<'a>>>) -> Self {
        /*
        table:  .byte 10, 254, 20,  2, 40,  4, 80,  6
    .byte 160,  8, 60, 10, 14, 12, 26, 14
    .byte 12,  16, 24, 18, 48, 20, 96, 22
    .byte 192, 24, 72, 26, 16, 28, 32, 30

        */

        let lengthTable: [u8; 32] = [
            10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
            12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
        ];

        let mut pulseTable: Vec<f32> = vec![0.0; 31];
        for i in 0..31 {
            pulseTable[i as usize] = 95.52 / (8128.0 / i as f32 + 100 as f32);
        }

        let mut tndTable: Vec<f32> = vec![0.0; 203];
        for i in 0..203 {
            tndTable[i as usize] = 163.67 / (24329.0 / i as f32 + 100 as f32);
        }

        Apu {
            frame: 0,
            globalTime: 0.0,
            pulse1: Pulse::new(true),
            pulse2: Pulse::new(false),
            triangle: Triangle::new(),
            dmc: DMC::new(dataBus.clone()),
            dataBus: dataBus.clone(),
            noise: Noise::new(),
            fiveStep: false,
            frameInterrupt: false,
            inhibitInterrupt: false,
            audio: Audio::new(),
            lengthTable: Vec::from(lengthTable),
            pulseTable: pulseTable,
            tndTable: tndTable,
        }
    }


    pub fn write(&mut self, ref addr: u16, data: u8) -> () {
        match *addr {
            0x4000 => {
                self.pulse1.dutyMode = (data & 0b1100_0000) >> 6;
                self.pulse1.lengthHalt = (data & 0b0010_0000) == 0b0010_0000;
                self.pulse1.envLoop = self.pulse1.lengthHalt;
                self.pulse1.constVolume = (data & 0b0001_0000) == 0b0001_0000;
                self.pulse1.envEnabled = !self.pulse1.constVolume;
                self.pulse1.envPeriod = data & 0b0000_1111;
                self.pulse1.volume = self.pulse1.envPeriod;
                self.pulse1.envStart = true;
            }
            0x4001 => {
                self.pulse1.sweepEnabled = (data & 0b1000_0000) == 0b1000_0000;
                self.pulse1.sweepPeriod = (data & 0b0111_0000) >> 4;
                self.pulse1.negate = (data & 0b0000_1000) == 0b0000_1000;
                self.pulse1.shift = data & 0b0000_0111;
                self.pulse1.sweepReload = true;
            }
            0x4002 => {
                self.pulse1.timer = self.pulse1.timer & 0xFF00 | data as u16;
            }
            0x4003 => {
                if !self.pulse1.enabled {
                    return;
                }

                self.pulse1.timer = (self.pulse1.timer & 0x00FF) | ((data as u16 & 0b000_00111 as u16) << 8) as u16;
                self.pulse1.timerPeriod = self.pulse1.timer;
                self.pulse1.lengthCounter = if self.pulse1.enabled { self.lengthTable[(data >> 3) as usize] } else { 0 };
                self.pulse1.envStart = true;
                self.pulse1.dutyValue = 0;
            }
            0x4004 => {
                self.pulse2.dutyMode = (data & 0b1100_0000) >> 6;
                self.pulse2.lengthHalt = (data & 0b0010_0000) == 0b0010_0000;
                self.pulse2.envLoop = self.pulse2.lengthHalt;
                self.pulse2.constVolume = (data & 0b0001_0000) == 0b0001_0000;
                self.pulse2.envEnabled = !self.pulse2.constVolume;
                self.pulse2.envPeriod = data & 0b0000_1111;
                self.pulse2.volume = self.pulse2.envPeriod;
                self.pulse2.envStart = true;
            }
            0x4005 => {
                self.pulse2.sweepEnabled = (data & 0b1000_0000) == 0b1000_0000;
                self.pulse2.sweepPeriod = (data & 0b0111_0000) >> 4;
                self.pulse2.negate = (data & 0b0000_1000) == 0b0000_1000;
                self.pulse2.shift = data & 0b0000_0111;
                self.pulse2.sweepReload = true;
            }
            0x4006 => {
                self.pulse2.timerPeriod = self.pulse2.timerPeriod & 0xFF00 | data as u16;
            }
            0x4007 => {
                if !self.pulse2.enabled {
                    return;
                }
                self.pulse2.timerPeriod = (self.pulse2.timerPeriod & 0x00FF) | ((data as u16 & 0b0000_0111 as u16) << 8) as u16;
                self.pulse2.timer = self.pulse2.timerPeriod;
                self.pulse2.lengthCounter = if self.pulse2.enabled { self.lengthTable[(data >> 3) as usize] } else { 0 };
                self.pulse2.envStart = true;
                self.pulse2.dutyValue = 0;
            }
            0x4008 => {
                self.triangle.linearCounterEnabled = (data & 128) == 128;
                self.triangle.lengthCounterEnabled = !self.triangle.linearCounterEnabled;
                self.triangle.linearCounterPeriod = data & 0b01111111;
            }
            0x400A => {
                self.triangle.timerPeriod = (self.triangle.timerPeriod & 0xFF00) | data as u16;
            }
            0x400B => {
                if !self.triangle.enabled {
                    return;
                }

                self.triangle.timerPeriod = (self.triangle.timerPeriod & 0x00FF) as u16 | ((data as u16 & 0b0000_0111 as u16) << 8) as u16;
                self.triangle.timer = self.triangle.timerPeriod;
                self.triangle.lengthCounterValue = if self.triangle.enabled { self.lengthTable[(data >> 3) as usize] } else { 0 };
                self.triangle.linearCounterReload = true;
            }
            0x400C => {
                self.noise.lengthHalt = (data & 0b0010_0000) == 0b0010_0000;
                self.noise.envLoop = !self.noise.lengthHalt;
                self.noise.envEnabled = !((data & 0b0001_0000) == 0b0001_0000);
                self.noise.envPeriod = data & 0b0000_1111;
                self.noise.constVolume = self.noise.envPeriod;
            }
            0x400E => {
                self.noise.mode = (data & 128) == 128;
                self.noise.timerPeriod = NOISE_TIMER_TABLE[(data & 15) as usize]
            }
            0x400F => {
                if !self.noise.enabled {
                    return;
                }

                self.noise.lengthCounter = if self.noise.enabled { self.lengthTable[(data >> 3) as usize] } else { 0 };
                self.noise.envStart = true;
            }
            0x4010 => {
                self.dmc.irqEnabled = (data & 128) == 128;
                self.dmc.loopEnabled = (data & 64) == 64;
                self.dmc.ratePeriod = DMC_RATE_TABLE[(data & 15) as usize];
            }
            0x4011 => {
                self.dmc.bitCounter = data & 0b0111_1111;
            }
            0x4012 => {
                self.dmc.sampleAddr = (0xC000 + (data as u16 * 64 as u16));
            }
            0x4013 => {
                self.dmc.sampleLength = ((data as u16) << 4) + 1;
            }
            /*
         7654 3210  APUFLAGS ($4015)
            | ||||
               | |||+- Square 1 (0: disable; 1: enable)
               | ||+-- Square 2
               | |+--- Triangle
               | +---- Noise
               +------ DMC
            */
            0x4015 => {
                self.pulse1.enabled = (data & 1) == 1;
                if !self.pulse1.enabled { self.pulse1.lengthCounter = 0; }

                self.pulse2.enabled = (data & 2) == 2;
                if !self.pulse2.enabled { self.pulse2.lengthCounter = 0; }

                self.triangle.enabled = (data & 4) == 4;
                if !self.triangle.enabled { self.triangle.lengthCounterValue = 0; }

                self.noise.enabled = (data & 8) == 8;
                if !self.noise.enabled { self.noise.lengthCounter = 0; }

                self.dmc.enabled = (data & 16) == 16;
                if !self.dmc.enabled {
                    self.dmc.curSampleLength = 0;
                } else {
                    if self.dmc.curSampleLength == 0 {
                        self.dmc.reset();
                    }
                }
            }
            0x4017 => {
                self.fiveStep = (data & 128) == 128;
                self.inhibitInterrupt = (data & 64) == 64;

                if self.inhibitInterrupt {
                    self.frameInterrupt = false;
                }

                if self.fiveStep {
                    self.quarterStep();
                    self.halfStep();
                }
            }
            _ => {}
        }
    }

    pub fn read(&mut self, ref addr: u16) -> u8 {
        let mut status: u8 = 0;

        if self.pulse1.lengthCounter > 0 {
            status |= 1;
        }

        if self.pulse2.lengthCounter > 0 {
            status |= 2;
        }

        if self.triangle.lengthCounterValue > 0 {
            status |= 4;
        }

        if self.noise.lengthCounter > 0 {
            status |= 8;
        }

        if self.dmc.curSampleLength > 0 {
            status |= 16;
        }

        if self.frameInterrupt {
            status |= 64;
            self.frameInterrupt = false;
        }

        if self.dmc.irqEnabled {
            status |= 128;
        }

        return status;
    }

    pub fn addSampleToBuffer(&mut self) -> () {
        let pulseOut: f32 = self.pulseTable[(self.pulse1.output() + self.pulse2.output()) as usize];
        let tndOut: f32 = self.tndTable[
            (3 * self.triangle.output()) as usize +
                (2 * self.noise.output()) as usize +
                (self.dmc.output()) as usize
            ];

        // let pulseOut = 95.88 / (100.0
        // 	+ (8128.0 / (  self.pulse1.output() as f32
        // 	+ self.pulse2.output() as f32)));
        // let tndOut = 159.79 / (100.0
        // 	+ (1.0 / (  (self.triangle.output() as f32 / 8227.0)
        // 	+ (self.noise.output() as f32 / 12241.0)
        // 	+ (self.dmc.output() as f32 / 22638.0))));

        self.audio.pushSample(pulseOut + tndOut);
    }

    fn quarterStep(&mut self) -> () {
        self.pulse1.clockEnvelope();
        self.pulse2.clockEnvelope();
        self.triangle.clockLinearCounter();
        self.noise.clockEnvelope();
    }

    fn halfStep(&mut self) -> () {
        self.pulse1.clockSweep();
        self.pulse1.clockLengthCounter();

        self.pulse2.clockSweep();
        self.pulse2.clockLengthCounter();

        self.triangle.clockLengthCounter();

        self.noise.clockLength();
    }
}

impl<'a> Clocked for Apu<'a> {
    fn cycle(&mut self) {
        match self.frame {
            STEP_ONE_CYCLE => {
                self.quarterStep();
            }
            STEP_TWO_CYCLE => {
                self.halfStep();
                self.quarterStep();
            }
            STEP_THREE_CYCLE => {
                self.quarterStep();
            }
            STEP_FOUR_CYCLE => {
                if !self.fiveStep {
                    self.halfStep();
                    self.quarterStep();
                    if !self.inhibitInterrupt {
                        self.frameInterrupt = true;
                        self.dataBus.borrow_mut().triggerCpuIRQ();
                    }
                    self.frame = 0;
                }
            }
            STEP_FIVE_CYCLE => {
                if self.fiveStep {
                    self.quarterStep();
                    self.halfStep();
                    self.frame = 0;
                }
            }
            _ => {}
        }


        if (self.frame % 2) == 0 {
            self.pulse1.clockTimer();
            self.pulse2.clockTimer();
            self.noise.clockTimer();
            self.dmc.clockRate();
        }
        self.triangle.clockTimer();

        self.frame += 1;
    }
}
