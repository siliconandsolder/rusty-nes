#![allow(non_snake_case)]
#![allow(warnings)]

use std::rc::Rc;
use std::cell::RefCell;
use crate::data_bus::DataBus;
use crate::clock;
use crate::clock::Clocked;

const SCANLINE_VISIBLE_MAX: u16 = 239;
const SCANLINE_POST: u16 = 240;
const SCANLINE_VBLANK_MIN: u16 = 241;
const SCANLINE_VBLANK_MAX: u16 = 260;
const SCANLINE_MAX: u16 = 261;

const CYCLE_MAX: u16 = 340;

const CYCLES_PER_FRAME: u32 = 89342;

pub struct Ppu {
    cycle: u16,
    scanLine: u16,

    frameCycles: u16,
    isOddFrame: bool,

    v: u16,     // vram address
    t: u16,    /* temp vram address
             0x0yyy NN YYYYY XXXXX
                ||| || ||||| +++++-- coarse X scroll
                ||| || +++++-------- coarse Y scroll
                ||| ++-------------- nametable select
                +++----------------- fine Y scroll
                */
    x: u8,      // fine x scroll
    w: u8,      // write toggle
    f: u8,      // frame is even or odd
    prevReg: u8,

    oamAddr: u8,

    bufData: u8,

    nmiOccured: bool,
    canTrigNmi: bool,

    // shift registers
    bgShiftPatLo: u16,
    bgShiftPatHi: u16,
    bgShiftAttrLo: u16,
    bgShiftAttrHi: u16,

    // tile info
    bgTileId: u8,
    bgTileAttr: u8,
    bgTileLsb: u8,
    bgTileMsb: u8,


    // flags
    // PPUCTRL
    fNameTable: u8,
    fIncMode: u8,
    fSprTile: u8,
    fBckTile: u8,
    fSprHeight: u8,
    fMaster: u8,
    fNmi: u8,

    // PPUMASK
    fGrey: u8,
    fBckLeft: u8,
    fSprLeft: u8,
    fBckEnabled: u8,
    fSprEnabled: u8,
    fColour: u8,

    // PPUSTATUS
    fSprOver: u8,
    fSprZero: u8,

    memory: Rc<RefCell<DataBus>>,

}

impl Clocked for Ppu {
    fn cycle(&mut self) {

        let renderEnabled = self.fSprEnabled == 1 || self.fBckEnabled == 1;

        if self.scanLine == SCANLINE_VBLANK_MIN && self.cycle == 1 {
            self.nmiOccured = true;
        }

        if self.scanLine == SCANLINE_MAX && self.cycle == 1 {
            self.fSprZero = 0;
            self.nmiOccured = false;
            self.canTrigNmi = true;
        }

        if self.fNmi == 1 && self.nmiOccured && self.canTrigNmi {
            self.memory.borrow_mut().triggerNMI();
            self.canTrigNmi = false;
        }

        if self.cycle >= 257 && self.cycle <= 320 {
            self.oamAddr = 0;
        }

        if renderEnabled {
            if self.cycle < 257 || (self.cycle > 320 && self.cycle < 337) {

                let vAddr = self.v.clone();
                match (self.cycle - 1) % 8 {
                    0 => {
                        self.loadBackgroundShiftRegisters();
                        self.bgTileId = self.memory.borrow_mut().readPpuMem(
                            0x2000 | (vAddr & 0x0FFF)
                        );
                    },
                    2 => {
                        self.bgTileAttr = self.memory.borrow_mut().readPpuMem(
                            0x23C0 | (vAddr & 0x0C00) | ((vAddr >> 4) & 0x38) | ((vAddr >> 2) & 0x07)
                        );
                    },
                    4 => {
                        self.bgTileLsb = self.memory.borrow_mut().readPpuMem(
                            (self.fBckTile << 12) as u16 +
                                (self.bgTileId << 4) as u16 +
                                vAddr >> 12
                        );
                    },
                    6 => {
                        self.bgTileMsb = self.memory.borrow_mut().readPpuMem(
                            (self.fBckTile << 12) as u16 +
                                (self.bgTileId << 4) as u16 +
                                (vAddr >> 12) + 8
                        );
                    },
                    7 => {
                        self.incrementX();
                    },
                    _ => {}
                }
            }

            match self.cycle {
                256 => {
                    self.incrementY();
                },
                257 => {
                    // copy nametable x and coarse x
                    self.v = (self.v & 0b0111101111100000) | (self.t & 0b0000010000011111);
                },
                280...304 => {
                    if self.scanLine == SCANLINE_MAX {
                        // copy fine y, nametable y, and coarse y to vram address
                        self.v = (self.v & 0b0000010000011111) | (self.t & 0b0111101111100000);
                    }
                },
                _=> {}
            }
        }

        let mut bgPixel: u8 = 0x0000;
        let mut bgPallete: u8 = 0x0000;

        if self.fBckEnabled == 1 {
            let mux: u16 = 0x8000 >> self.x as u16;

            bgPixel = (((self.bgShiftPatHi & mux) & 1) as u8) << 1 | ((self.bgShiftPatLo & mux) & 1) as u8;
            bgPallete = (((self.bgShiftAttrHi & mux) & 1) as u8) << 1 | ((self.bgShiftAttrLo & mux) & 1) as u8;
        }


        // increment cycle and scanline
        self.cycle += 1;
        if self.cycle == CYCLE_MAX + 1 {
            self.cycle = 0;

            self.scanLine += 1;
            if self.scanLine > SCANLINE_MAX {
                self.scanLine = 0;
                self.isOddFrame = !self.isOddFrame;
            }
        }
    }
}

impl Ppu {
    pub fn new(mem: Rc<RefCell<DataBus>>) -> Self {
        Ppu {
            cycle: 0,
            scanLine: 0,
            frameCycles: 0,
            isOddFrame: false,
            v: 0,
            t: 0,
            x: 0,
            w: 0,
            f: 0,
            prevReg: 0,
            oamAddr: 0,
            bufData: 0,
            nmiOccured: false,
            canTrigNmi: false,
            bgShiftPatLo: 0,
            bgShiftPatHi: 0,
            bgShiftAttrLo: 0,
            bgShiftAttrHi: 0,
            bgTileId: 0,
            bgTileAttr: 0,
            bgTileLsb: 0,
            bgTileMsb: 0,
            fNameTable: 0,
            fIncMode: 0,
            fSprTile: 0,
            fBckTile: 0,
            fSprHeight: 0,
            fMaster: 0,
            fNmi: 0,
            fGrey: 0,
            fBckLeft: 0,
            fSprLeft: 0,
            fBckEnabled: 0,
            fSprEnabled: 0,
            fColour: 0,
            fSprOver: 0,
            fSprZero: 0,
            memory: mem
        }
    }

    pub fn readMem(&mut self, ref addr: u16) -> u8 {
        match *addr {
            0x0002 => { return self.ppuStatus(); },     // PPU STATUS
            0x0004 => { return self.oamDataRead(); },   // OAM DATA
            0x0007 => { return self.ppuDataRead(); },   // PPU DATA
            _ => panic!("Unknown PPU register: {}", *addr)
        }
        return 0;
    }

    pub fn writeMem(&mut self, ref addr: u16, val: u8) -> () {

        self.prevReg = val;
        match *addr {
            0x0000 => { self.ppuCtrl(val) },        // PPU CONTROL
            0x0001 => { self.ppuMask(val) },        // PPU MASK
            0x0003 => { self.oamAddress(val) },     // OAM ADDRESS
            0x0004 => { self.oamDataWrite(val) },   // OAM DATA
            0x0005 => { self.ppuScroll(val) },      // PPU SCROLL
            0x0006 => { self.ppuAddress(val) },     // PPU ADDRESS
            0x0007 => { self.ppuDataWrite(val) },   // PPU DATA
            0x4014 => { self.oamDma(val) },         // OAM DMA
            _ => panic!("Unknown PPU register: {}", *addr)
        }
    }


    fn ppuCtrl(&mut self, val: u8) -> () {
        self.fNameTable = val & 0b00000011;
        self.fIncMode = (val >> 2) & 1;
        self.fSprTile = (val >> 3) & 1;
        self.fBckTile = (val >> 4) & 1;
        self.fSprHeight = (val >> 5) & 1;
        self.fMaster = (val >> 6) & 1;
        self.fNmi = (val >> 7) & 1;

        self.canTrigNmi = true;
        self.t = ((self.t & 0b0111001111111111) | ((val & 0b00000011) as u16) << 10);
    }

    fn ppuMask(&mut self, val: u8) -> () {
        self.fGrey = val & 1;
        self.fBckLeft = (val >> 1) & 1;
        self.fSprLeft = (val >> 2) & 1;
        self.fBckEnabled = (val >> 3) & 1;
        self.fSprEnabled = (val >> 4) & 1;
        self.fColour = (val >> 5) & 1;
    }

    fn ppuStatus(&mut self) -> u8 {
        let mut value: u8 = 0;
        value |= self.prevReg & 0b00011111;
        value |= self.fSprOver << 5;
        value |= self.fSprZero << 6;

        if self.nmiOccured {
            value |= 1 << 7;
        }

        self.w = 0;
        self.nmiOccured = false;

        return value;
    }

    fn oamAddress(&mut self, val: u8) -> () {
       self.oamAddr = val;
    }

    fn oamDataWrite(&mut self, val: u8) -> () {
        self.memory.borrow_mut().writeOam(self.oamAddr, val);
        self.oamAddr = self.oamAddr.wrapping_add(1);
    }

    fn oamDataRead(&mut self) -> u8 {
        return self.memory.borrow_mut().readOam(self.oamAddr);
        // do not increment if v-blank or forced blank
    }

    fn ppuScroll(&mut self, val: u8) -> () {
        if self.w == 0 {

            self.t = (self.t & 0b0111111111100000) | ((val & 0b11111000) as u16 >> 3);
            self.x = (self.x & 0) | (val & 0b00000111);
            self.w = 1;
        }
        else {

            self.t &= 0b0000110011100000;
            self.t |= ((val & 0b00000111) as u16) << 12;
            self.t |= ((val & 0b11111000) as u16) << 2;
            self.w = 0;
        }
    }

    fn ppuAddress(&mut self, val: u8) -> () {
        if self.w == 0 {
            self.t = (self.t & 0b0000000011111111) | ((val & 0b00111111) as u16) << 8;
            self.w = 1;
        }
        else {
            self.t = (self.t & 0b0111111100000000) | (val as u16);
            self.v = self.t;
            self.w = 0;
        }
    }

    fn ppuDataWrite(&mut self, val: u8) -> () {
        let vPtr = self.v;
        self.memory.borrow_mut().writePpuMem(vPtr, val);
        self.v = if self.fIncMode == 0 { self.v.wrapping_add(1) } else { self.v.wrapping_add(32) };
    }

    fn ppuDataRead(&mut self) -> u8 {

        let mut tempBufData: u8 = 0;
        let vPtr = &self.v;
        let mut ppuData = self.memory.borrow().readPpuMem(*vPtr);

        if self.v < 0x3F00 {
            tempBufData = self.bufData;
            self.bufData = ppuData;
            ppuData = tempBufData;
        }
        else {
            // maps to nametable under the palette (palette address minus 0x1000)
            self.bufData = self.memory.borrow().readPpuMem(*vPtr - 0x1000);
        }

        self.v = if self.fIncMode == 0 { self.v.wrapping_add(1) } else { self.v.wrapping_add(32) };
		return ppuData;
    }

    fn oamDma(&mut self, val: u8) -> () {
        self.memory.borrow_mut().overWriteOam(val);
    }

    fn incrementX(&mut self) -> () {
        if self.v & 0x001F == 0x001F {
            self.v &= 0x001F;
            self.v ^= 0x0400;
        }
        else {
            self.v += 1;
        }
    }

    fn incrementY(&mut self) -> () {
        if (self.v & 0x7000) != 0x7000 {
            self.v += 0x1000;
        }
        else {
            self.v &= !0x7000;
            let mut y: u16 = (self.v & 0x3E0) >> 5;

            if y == 29 {
                y = 0;
                self.v ^= 0x0800;
            }
            else if y == 31 {
                y = 0;
            }
            else {
                y += 1;
            }
            self.v = (self.v & !0x03E0) | (y << 5);
        }
    }

    fn loadBackgroundShiftRegisters(&mut self) -> () {
        self.bgShiftPatLo &= 0xFF00 | self.bgTileLsb as u16;
        self.bgShiftPatHi &= 0xFF00 | self.bgTileMsb as u16;

        self.bgShiftAttrLo &= 0xFF00 | if self.bgTileAttr & 0x01 == 1 { 0x00FF } else { 0x0000 };
        self.bgShiftAttrHi &= 0xFF00 | if self.bgTileAttr & 0x10 == 1 { 0x00FF } else { 0x0000 };
    }

    fn updateShiftRegisters(&mut self) -> () {
        if self.fBckEnabled == 1 {
            self.bgShiftPatLo <<= 1;
            self.bgShiftPatHi <<= 1;
            self.bgShiftAttrLo <<= 1;
            self.bgShiftAttrHi <<= 1;
        }
    }

    fn getPatternTable(&mut self) -> () {

    }
}
