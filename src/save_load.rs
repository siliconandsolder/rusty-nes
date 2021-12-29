#![allow(non_snake_case)]
#![allow(warnings)]

use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use serde::{Serialize, Deserialize};
use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::data_bus::DataBus;
use crate::ppu::Ppu;
use crate::ppu_bus::PpuBus;

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveState {
    cart: CartData,
    cpu: CpuData,
    dataBus: BusData,
    ppu: PpuData,
    ppuBus: PpuBusData,
    apu: ApuData,
    mapper: MapperData
}

impl SaveState {
    pub fn save(
        cpu: Rc<RefCell<Cpu>>,
        ppu: Rc<RefCell<Ppu>>,
        apu: Rc<RefCell<Apu>>,
        cart: Rc<RefCell<Cartridge>>,
    ) -> () {
        let saveState = SaveState {
            cart: cart.borrow().saveState(),
            cpu: cpu.borrow().saveState(),
            dataBus: cpu.borrow().saveBusState(),
            ppu: ppu.borrow().saveState(),
            ppuBus: ppu.borrow().saveBusState(),
            apu: apu.borrow().saveState(),
            mapper: cart.borrow().saveMapperState()
        };

        let result = serde_json::to_string(&saveState).unwrap();
        fs::write("./save.json", result).unwrap()
    }

    pub fn load(
        cpu: Rc<RefCell<Cpu>>,
        ppu: Rc<RefCell<Ppu>>,
        apu: Rc<RefCell<Apu>>,
        cart: Rc<RefCell<Cartridge>>,
    ) -> () {
        let saveFile = fs::read("./save.json").unwrap();
        let data: SaveState = serde_json::from_slice(saveFile.as_slice()).unwrap();
        
        cart.borrow_mut().loadState(&data.cart);
        cpu.borrow_mut().loadState(&data.cpu);
        cpu.borrow_mut().loadBusState(&data.dataBus);
        ppu.borrow_mut().loadState(&data.ppu);
        ppu.borrow_mut().loadBusState(&data.ppuBus);
        apu.borrow_mut().loadState(&data.apu);
        cart.borrow_mut().loadMapperState(&data.mapper);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CartData {
    pub(crate) header: CartHeaderData,
    pub(crate) mapperId: u8,
    pub(crate) numPrgBanks: u8,
    pub(crate) numChrBanks: u8,
    pub(crate) vChrMem: Vec<u8>,
    pub(crate) hasPrgRam: bool,
}

#[repr(C, packed)]
#[derive(Serialize, Deserialize, Debug)]
pub struct CartHeaderData {
    pub name: [u8; 4],
    pub prgSize: u8,
    pub chrSize: u8,
    pub mapper1: u8,
    pub mapper2: u8,
    pub prgRam: u8,
    pub tvSys1: u8,
    pub tvSys2: u8,
    pub unused: [u8; 5],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuData {
    pub regA: u8,
    pub regY: u8,
    pub regX: u8,
    pub pgmCounter: u16,
    pub stkPointer: u8,
    pub flags: CpuFlagData,
    pub waitCycles: u16,
    pub isEvenCycle: bool,
    pub triggerNmi: bool,
    pub triggerIrq: bool,
    pub isOamTransfer: bool,
    pub isOamStarted: bool,
    pub oamByte: u8,
    pub oamPage: u16,
    pub oamCycles: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuFlagData {
    pub carry: u8,
    pub zero: u8,
    pub interrupt: u8,
    pub decimal: u8,
    pub brk: u8,
    pub unused: u8,
    pub overflow: u8,
    pub negative: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BusData {
    pub cpuMem: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PpuData {
    pub cycle: u16,
    pub scanLine: u16,
    pub frameCycles: u16,
    pub isOddFrame: bool,
    pub v: u16,
    pub t: u16,
    pub x: u8,
    pub w: u8,
    pub f: u8,
    pub prevReg: u8,
    pub oamAddr: u8,
    pub bufData: u8,
    pub nmiOccured: bool,
    pub forceNmi: bool,
    pub nmiIncoming: bool,
    pub nmiDelay: u8,
    pub bgShiftPatLo: u16,
    pub bgShiftPatHi: u16,
    pub bgShiftAttrLo: u16,
    pub bgShiftAttrHi: u16,
    pub sprShiftPatLo: Vec<u8>,
    pub sprShiftPatHi: Vec<u8>,
    pub bgTileId: u8,
    pub bgTileAttr: u8,
    pub bgTileLsb: u8,
    pub bgTileMsb: u8,
    pub vSpriteLine: Vec<u8>,
    pub spriteLineCount: u8,
    pub isZeroHitPossible: bool,
    pub isZeroBeingRendered: bool,
    pub fNameTable: u8,
    pub fIncMode: u8,
    pub fSprTable: u8,
    pub fBckTile: u8,
    pub fSprHeight: u8,
    pub fMaster: u8,
    pub fNmi: u8,
    pub fGrey: u8,
    pub fBckLeft: u8,
    pub fSprLeft: u8,
    pub fBckEnabled: u8,
    pub fSprEnabled: u8,
    pub fColour: u8,
    pub fSprOver: u8,
    pub fSprZero: u8,
    pub vPixelColours: Vec<u8>,
    pub vPixelPalette: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PpuBusData {
    pub tblPalette: Vec<u8>,
    pub tblName: Vec<u8>,
    pub oamMem: Vec<u8>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApuData {
    pub frame: u16,
    pub pulse1: PulseData,
    pub pulse2: PulseData,
    pub triangle: TriangleData,
    pub dmc: DMCData,
    pub noise: NoiseData,
    pub fiveStep: bool,
    pub frameInterrupt: bool,
    pub inhibitInterrupt: bool,
    pub lengthTable: Vec<u8>,
    pub pulseTable: Vec<f32>,
    pub tndTable: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NoiseData {
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


#[derive(Serialize, Deserialize, Debug)]
pub struct PulseData {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct TriangleData {
    pub enabled: bool,
    pub lengthCounterEnabled: bool,
    pub lengthCounterValue: u8,
    pub linearCounterEnabled: bool,
    pub linearCounterReload: bool,
    pub linearCounterValue: u8,
    pub linearCounterPeriod: u8,
    pub dutyValue: u8,
    pub timer: u16,
    pub timerPeriod: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DMCData {
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
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MapperData {
    M0(Mapper0Data),
    M1(Mapper1Data),
    M2(Mapper2Data),
    M3(Mapper3Data),
    M4(Mapper4Data),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper0Data {
    pub numPrgBanks: u8,
    pub numChrBanks: u8,
    pub mirrorType: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper1Data {
    pub shiftReg: u8,
    pub ctrlReg: Mapper1CtrlRegData,
    pub chrReg: Mapper1ChrRegData,
    pub prgReg: Mapper1PrgRegData,
    pub numPrgBanks: u8,
    pub numChrBanks: u8,
    pub vPrgRam: Vec<u8>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper1CtrlRegData {
    pub mirrorMode: u8,
    pub prgMode: u8,
    pub chrMode: u8,
    pub registerValue: u8
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper1ChrRegData {
    pub bankLo: u8,
    pub bankHi: u8,
    pub bank8: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper1PrgRegData {
    pub bankLo: u8,
    pub bankHi: u8,
    pub bank32: u8,
    pub prgRamEnabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper2Data {
    pub(crate) switchBank: u8,
    pub(crate) lastBank: u8,
    pub(crate) hasChrRam: bool,
    pub(crate) mirrorType: u8
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper3Data {
    pub(crate) chrBank: u8,
    pub(crate) mirrorType: u8
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapper4Data {
    pub vChrBanks: Vec<u32>,
    pub vPrgBanks: Vec<u32>,
    pub vMemRegs: Vec<u32>,
    pub vPrgRam: Vec<u8>,
    pub secLastPrgBank: u16,
    pub lastPrgBank: u16,
    pub mirrorType: u8,
    pub target: usize,
    pub prgBankMode: u8,
    pub chrInversion: u8,
    pub writeProtect: bool,
    pub prgRamEnabled: bool,
    pub irqCounter: u8,
    pub irqReload: u8,
    pub irqEnabled: bool,
    pub irqReady: bool,
}