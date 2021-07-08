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
use sdl2::AudioSubsystem;

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
    pub fn new(dataBus: Rc<RefCell<DataBus<'a>>>, audioSystem: AudioSubsystem) -> Self {
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
            audio: Audio::new(audioSystem),
            lengthTable: Vec::from(lengthTable),
            pulseTable: pulseTable,
            tndTable: tndTable,
        }
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
