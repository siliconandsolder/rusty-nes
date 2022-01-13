#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use audio::Audio;

use crate::clock::Clocked;
use crate::data_bus::DataBus;
use crate::apu::noise::Noise;
use crate::apu::pulse::Pulse;
use crate::apu::triangle::Triangle;
use crate::apu::dmc::DMC;
use utils::*;
use sdl2::AudioSubsystem;
use crate::save_load::{ApuData, DMCData, NoiseData, PulseData, TriangleData};

pub mod utils;
pub mod filter;
pub mod pulse;
pub mod triangle;
pub mod noise;
pub mod dmc;
pub mod audio;
pub mod callback;


/*
 |  0   1   2   3   4   5   6   7    8   9   A   B   C   D   E   F
-----+----------------------------------------------------------------
00-0F  10,254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
10-1F  12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
*/

pub struct Apu {
    frame: u16,
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    dmc: DMC,
    dataBus: Rc<RefCell<DataBus>>,
    noise: Noise,
    fiveStep: bool,
    frameInterrupt: bool,
    inhibitInterrupt: bool,
    audio: Audio,

    lengthTable: Vec<u8>,
    pulseTable: Vec<f32>,
    tndTable: Vec<f32>,
}

impl Apu {
    pub fn new(dataBus: Rc<RefCell<DataBus>>, audioSystem: Rc<RefCell<AudioSubsystem>>) -> Self {
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
            pulse1: Pulse::new(true),
            pulse2: Pulse::new(false),
            triangle: Triangle::new(),
            dmc: DMC::new(dataBus.clone()),
            dataBus: dataBus.clone(),
            noise: Noise::new(),
            fiveStep: false,
            frameInterrupt: false,
            inhibitInterrupt: false,
            audio: Audio::new(audioSystem),
            lengthTable: Vec::from(lengthTable),
            pulseTable: pulseTable,
            tndTable: tndTable,
        }
    }

    pub fn saveState(&self) -> ApuData {
        ApuData {
            frame: self.frame,
            pulse1: PulseData {
                isChannelOne: self.pulse1.isChannelOne,
                enabled: self.pulse1.enabled,
                dutyValue: self.pulse1.dutyValue,
                dutyMode: self.pulse1.dutyMode,
                output: self.pulse1.output,
                lengthHalt: self.pulse1.lengthHalt,
                constVolume: self.pulse1.constVolume,
                volume: self.pulse1.volume,
                envVolume: self.pulse1.envVolume,
                envValue: self.pulse1.envValue,
                envPeriod: self.pulse1.envPeriod,
                envEnabled: self.pulse1.envEnabled,
                envLoop: self.pulse1.envLoop,
                envStart: self.pulse1.envStart,
                sweepEnabled: self.pulse1.sweepEnabled,
                sweepReload: self.pulse1.sweepReload,
                sweepPeriod: self.pulse1.sweepPeriod,
                sweepValue: self.pulse1.sweepValue,
                negate: self.pulse1.negate,
                shift: self.pulse1.shift,
                timerPeriod: self.pulse1.timerPeriod,
                timer: self.pulse1.timer,
                lengthCounter: self.pulse1.lengthCounter,
                sample: self.pulse1.sample
            },
            pulse2: PulseData {
                isChannelOne: self.pulse2.isChannelOne,
                enabled: self.pulse2.enabled,
                dutyValue: self.pulse2.dutyValue,
                dutyMode: self.pulse2.dutyMode,
                output: self.pulse2.output,
                lengthHalt: self.pulse2.lengthHalt,
                constVolume: self.pulse2.constVolume,
                volume: self.pulse2.volume,
                envVolume: self.pulse2.envVolume,
                envValue: self.pulse2.envValue,
                envPeriod: self.pulse2.envPeriod,
                envEnabled: self.pulse2.envEnabled,
                envLoop: self.pulse2.envLoop,
                envStart: self.pulse2.envStart,
                sweepEnabled: self.pulse2.sweepEnabled,
                sweepReload: self.pulse2.sweepReload,
                sweepPeriod: self.pulse2.sweepPeriod,
                sweepValue: self.pulse2.sweepValue,
                negate: self.pulse2.negate,
                shift: self.pulse2.shift,
                timerPeriod: self.pulse2.timerPeriod,
                timer: self.pulse2.timer,
                lengthCounter: self.pulse2.lengthCounter,
                sample: self.pulse2.sample
            },
            triangle: TriangleData {
                enabled: self.triangle.enabled,
                lengthCounterEnabled: self.triangle.lengthCounterEnabled,
                lengthCounterValue: self.triangle.lengthCounterValue,
                linearCounterEnabled: self.triangle.linearCounterEnabled,
                linearCounterReload: self.triangle.linearCounterReload,
                linearCounterValue: self.triangle.linearCounterValue,
                linearCounterPeriod: self.triangle.linearCounterPeriod,
                dutyValue: self.triangle.dutyValue,
                timer: self.triangle.timer,
                timerPeriod: self.triangle.timerPeriod
            },
            dmc: DMCData {
                enabled: self.dmc.enabled,
                irqEnabled: self.dmc.irqEnabled,
                loopEnabled: self.dmc.loopEnabled,
                ratePeriod: self.dmc.ratePeriod,
                rateValue: self.dmc.rateValue,
                directLoad: self.dmc.directLoad,
                bitCounter: self.dmc.bitCounter,
                freq: self.dmc.freq,
                loadCounter: self.dmc.loadCounter,
                sampleAddr: self.dmc.sampleAddr,
                curSampleAddr: self.dmc.curSampleAddr,
                sampleLength: self.dmc.sampleLength,
                curSampleLength: self.dmc.curSampleLength,
                shift: self.dmc.shift
            },
            noise: NoiseData {
                enabled: self.noise.enabled,
                mode: self.noise.mode,
                output: self.noise.output,
                lengthHalt: self.noise.lengthHalt,
                constVolume: self.noise.constVolume,
                volume: self.noise.volume,
                envVolume: self.noise.envVolume,
                envValue: self.noise.envValue,
                envPeriod: self.noise.envPeriod,
                envEnabled: self.noise.envEnabled,
                envLoop: self.noise.envLoop,
                envStart: self.noise.envStart,
                shift: self.noise.shift,
                timerPeriod: self.noise.timerPeriod,
                timer: self.noise.timer,
                lengthCounter: self.noise.lengthCounter
            },
            fiveStep: self.fiveStep,
            frameInterrupt: self.frameInterrupt,
            inhibitInterrupt: self.inhibitInterrupt,
            lengthTable: self.lengthTable.clone(),
            pulseTable: self.pulseTable.clone(),
            tndTable: self.tndTable.clone()
            
        }
    }

    pub fn loadState(&mut self, data: &ApuData) -> () { 
        self.frame = data.frame;
        self.fiveStep = data.fiveStep;
        self.frameInterrupt = data.frameInterrupt;
        self.inhibitInterrupt = data.inhibitInterrupt;
        self.lengthTable = data.lengthTable.clone();
        self.pulseTable = data.pulseTable.clone();
        self.tndTable = data.tndTable.clone();
        
        // pulse 1
        self.pulse1.isChannelOne = data.pulse1.isChannelOne;
        self.pulse1.enabled = data.pulse1.enabled;
        self.pulse1.dutyValue = data.pulse1.dutyValue;
        self.pulse1.dutyMode = data.pulse1.dutyMode;
        self.pulse1.output = data.pulse1.output;
        self.pulse1.lengthHalt = data.pulse1.lengthHalt;
        self.pulse1.constVolume = data.pulse1.constVolume;
        self.pulse1.volume = data.pulse1.volume;
        self.pulse1.envVolume = data.pulse1.envVolume;
        self.pulse1.envValue = data.pulse1.envValue;
        self.pulse1.envPeriod = data.pulse1.envPeriod;
        self.pulse1.envEnabled = data.pulse1.envEnabled;
        self.pulse1.envLoop = data.pulse1.envLoop;
        self.pulse1.envStart = data.pulse1.envStart;
        self.pulse1.sweepEnabled = data.pulse1.sweepEnabled;
        self.pulse1.sweepReload = data.pulse1.sweepReload;
        self.pulse1.sweepPeriod = data.pulse1.sweepPeriod;
        self.pulse1.sweepValue = data.pulse1.sweepValue;
        self.pulse1.negate = data.pulse1.negate;
        self.pulse1.shift = data.pulse1.shift;
        self.pulse1.timerPeriod = data.pulse1.timerPeriod;
        self.pulse1.timer = data.pulse1.timer;
        self.pulse1.lengthCounter = data.pulse1.lengthCounter;
        self.pulse1.sample = data.pulse1.sample;

        // pulse 2
        self.pulse2.isChannelOne = data.pulse2.isChannelOne;
        self.pulse2.enabled = data.pulse2.enabled;
        self.pulse2.dutyValue = data.pulse2.dutyValue;
        self.pulse2.dutyMode = data.pulse2.dutyMode;
        self.pulse2.output = data.pulse2.output;
        self.pulse2.lengthHalt = data.pulse2.lengthHalt;
        self.pulse2.constVolume = data.pulse2.constVolume;
        self.pulse2.volume = data.pulse2.volume;
        self.pulse2.envVolume = data.pulse2.envVolume;
        self.pulse2.envValue = data.pulse2.envValue;
        self.pulse2.envPeriod = data.pulse2.envPeriod;
        self.pulse2.envEnabled = data.pulse2.envEnabled;
        self.pulse2.envLoop = data.pulse2.envLoop;
        self.pulse2.envStart = data.pulse2.envStart;
        self.pulse2.sweepEnabled = data.pulse2.sweepEnabled;
        self.pulse2.sweepReload = data.pulse2.sweepReload;
        self.pulse2.sweepPeriod = data.pulse2.sweepPeriod;
        self.pulse2.sweepValue = data.pulse2.sweepValue;
        self.pulse2.negate = data.pulse2.negate;
        self.pulse2.shift = data.pulse2.shift;
        self.pulse2.timerPeriod = data.pulse2.timerPeriod;
        self.pulse2.timer = data.pulse2.timer;
        self.pulse2.lengthCounter = data.pulse2.lengthCounter;
        self.pulse2.sample = data.pulse2.sample;
        
        // triangle 
        self.triangle.enabled = data.triangle.enabled;
        self.triangle.lengthCounterEnabled = data.triangle.lengthCounterEnabled;
        self.triangle.lengthCounterValue = data.triangle.lengthCounterValue;
        self.triangle.linearCounterEnabled = data.triangle.linearCounterEnabled;
        self.triangle.linearCounterReload = data.triangle.linearCounterReload;
        self.triangle.linearCounterValue = data.triangle.linearCounterValue;
        self.triangle.linearCounterPeriod = data.triangle.linearCounterPeriod;
        self.triangle.dutyValue = data.triangle.dutyValue;
        self.triangle.timer = data.triangle.timer;
        self.triangle.timerPeriod = data.triangle.timerPeriod;
        
        // dmc
        self.dmc.enabled = data.dmc.enabled;
        self.dmc.irqEnabled = data.dmc.irqEnabled;
        self.dmc.loopEnabled = data.dmc.loopEnabled;
        self.dmc.ratePeriod = data.dmc.ratePeriod;
        self.dmc.rateValue = data.dmc.rateValue;
        self.dmc.directLoad = data.dmc.directLoad;
        self.dmc.bitCounter = data.dmc.bitCounter;
        self.dmc.freq = data.dmc.freq;
        self.dmc.loadCounter = data.dmc.loadCounter;
        self.dmc.sampleAddr = data.dmc.sampleAddr;
        self.dmc.curSampleAddr = data.dmc.curSampleAddr;
        self.dmc.sampleLength = data.dmc.sampleLength;
        self.dmc.curSampleLength = data.dmc.curSampleLength;
        self.dmc.shift = data.dmc.shift;
        
        // Noise
        self.noise.enabled = data.noise.enabled;
        self.noise.mode = data.noise.mode;
        self.noise.output = data.noise.output;
        self.noise.lengthHalt = data.noise.lengthHalt;
        self.noise.constVolume = data.noise.constVolume;
        self.noise.volume = data.noise.volume;
        self.noise.envVolume = data.noise.envVolume;
        self.noise.envValue = data.noise.envValue;
        self.noise.envPeriod = data.noise.envPeriod;
        self.noise.envEnabled = data.noise.envEnabled;
        self.noise.envLoop = data.noise.envLoop;
        self.noise.envStart = data.noise.envStart;
        self.noise.shift = data.noise.shift;
        self.noise.timerPeriod = data.noise.timerPeriod;
        self.noise.timer = data.noise.timer;
        self.noise.lengthCounter = data.noise.lengthCounter;
    }

    pub fn write(&mut self, ref addr: u16, data: u8) -> () {
        match *addr {
            0x4000 => {
                self.pulse1.writeDuty(data);
            }
            0x4001 => {
                self.pulse1.writeSweep(data);
            }
            0x4002 => {
                self.pulse1.writeTimer(data);
            }
            0x4003 => {
                self.pulse1.writeLengthCounter(data, self.lengthTable[(data >> 3) as usize]);
            }
            0x4004 => {
                self.pulse2.writeDuty(data);
            }
            0x4005 => {
                self.pulse2.writeSweep(data);
            }
            0x4006 => {
                self.pulse2.writeTimer(data);
            }
            0x4007 => {
                self.pulse2.writeLengthCounter(data, self.lengthTable[(data >> 3) as usize]);
            }
            0x4008 => {
                self.triangle.writeLinearCounter(data);
            }
            0x400A => {
                self.triangle.writeTimer(data);
            }
            0x400B => {
                self.triangle.writeLengthCounter(data, self.lengthTable[(data >> 3) as usize]);
            }
            0x400C => {
                self.noise.writeEnvelopeVolumeCounter(data);
            }
            0x400E => {
                self.noise.writeLoopNoise(data, NOISE_TIMER_TABLE[(data & 15) as usize]);
            }
            0x400F => {
                self.noise.writeLengthCounter( self.lengthTable[(data >> 3) as usize]);
            }
            0x4010 => {
                self.dmc.writeIrqLoopFreq(data, DMC_RATE_TABLE[(data & 15) as usize]);
            }
            0x4011 => {
                self.dmc.writeLoadCounter(data);
            }
            0x4012 => {
                self.dmc.writeSampleAddress(data);
            }
            0x4013 => {
                self.dmc.writeSampleLength(data);
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
                }
                else {
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

        // if self.frameInterrupt {
        //     status |= 64;
        //     self.frameInterrupt = false;
        // }
        //
        // if self.dmc.irqEnabled {
        //     status |= 128;
        // }

        return status;
    }

    pub fn addSampleToBuffer(&mut self) -> () {
        let pulseOut: f32 = self.pulseTable[(self.pulse1.output() + self.pulse2.output()) as usize];
        let tndOut: f32 = self.tndTable[
            (3 * self.triangle.output()) as usize +
                (2 * self.noise.output()) as usize +
                (self.dmc.output()) as usize
            ];

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

impl Clocked for Apu {
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
