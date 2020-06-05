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

    // background shift registers
    bgShiftPatLo: u16,
    bgShiftPatHi: u16,
    bgShiftAttrLo: u16,
    bgShiftAttrHi: u16,

    // sprite shift registers
    sprShiftPatLo: Vec<u8>,
    sprShiftPatHi: Vec<u8>,

    // background tile info
    bgTileId: u8,
    bgTileAttr: u8,
    bgTileLsb: u8,
    bgTileMsb: u8,

    // sprite info
    vSpriteLine: Vec<u8>,
    spriteLineCount: u8,

    // sprite zero info
    isZeroHitPossible: bool,
    isZeroBeingRendered: bool,


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

            // wipe sprites for next scanline
            self.fSprOver = 0;
            for i in 0..8 {
                self.sprShiftPatLo[i] = 0;
                self.sprShiftPatHi[i] = 0;
            }

            self.fSprZero = 0;
        }

        if self.fNmi == 1 && self.nmiOccured && self.canTrigNmi {
            self.memory.borrow_mut().triggerNMI();
            self.canTrigNmi = false;
        }

        if self.cycle >= 257 && self.cycle <= 320 {
            self.oamAddr = 0;
        }

        if renderEnabled {
            if self.cycle < 258 || (self.cycle > 320 && self.cycle < 337) {

                if self.fBckEnabled == 1 {
                    self.updateBackgroundShiftRegisters();
                }

                if self.fSprEnabled == 1 && self.cycle < 248 {
                    self.updateSpriteShiftRegisters();
                }


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

                    if self.scanLine != SCANLINE_MAX {

                        for i in &mut self.vSpriteLine { *i = 0; }
                        self.spriteLineCount = 0;
                        let mut oamIdx: u8 = 0;
                        self.isZeroBeingRendered = false;

                        while oamIdx < 64 && self.spriteLineCount < 9 {
                            let oamY: u8 = self.memory.borrow_mut().readOam(oamIdx * 4);
                            let spriteSize: u8 = if self.fSprHeight == 0 { 8 } else { 16 };

                            let mut diff: i16 = self.scanLine as i16 - oamY as i16;
                            // the sprite will be rendered on the next scanline!
                            if diff > -1 && diff < spriteSize as i16 {
                                // copy oam entry into scanline vector
                                // increment sprite count
                                if self.spriteLineCount < 8 {

                                    if oamIdx == 0 { self.isZeroHitPossible = true; }

                                    for i in 0..=3 {
                                        self.vSpriteLine[(self.spriteLineCount.clone() * 4 + i) as usize] = self.memory.borrow_mut().readOam(oamIdx * 4 + i)
                                    }
                                    self.spriteLineCount += 1;
                                }
                            }
                            oamIdx += 1;
                        }
                        if self.spriteLineCount > 8 { self.fSprOver = 1 };
                    }
                },
                280..=304 => {
                    if self.scanLine == SCANLINE_MAX {
                        // copy fine y, nametable y, and coarse y to vram address
                        self.v = (self.v & 0b0000010000011111) | (self.t & 0b0111101111100000);
                    }
                },
                340 => {
                    // behold: sprite logic!
                    for i in 0..self.spriteLineCount {
                        let mut sprPatBitsLo: u8 = 0;
                        let mut sprPatBitsHi: u8 = 0;
                        let mut sprPatAddrLo: u16 = 0;
                        let mut sprPatAddrHi: u16 = 0;

                        if self.fSprHeight == 0 {
                            // is the sprite flipped vertically?
                            if self.vSpriteLine[(i * 4 + 2) as usize] & 0x80 == 0 {    // no
                                sprPatAddrLo = (self.fSprTile << 12) as u16 // get pattern table address (left half or right half)
                                                | (self.vSpriteLine[(i * 4 + 1) as usize].clone() << 4) as u16 // tile id multiplied by 16 to get pattern table byte (pattern table tiles are 16 bytes)
                                                | (self.scanLine - self.vSpriteLine[(i * 4) as usize].clone() as u16) as u16; // row offset of tile
                            }
                            else {  // yes
                                sprPatAddrLo = (self.fSprTile << 12) as u16 // get pattern table address (left half or right half)
                                    | (self.vSpriteLine[(i * 4 + 1) as usize].clone() << 4) as u16 // tile id multiplied by 16 to get pattern table byte (pattern table tiles are 16 bytes)
                                    | (7 - (self.scanLine - self.vSpriteLine[(i * 4) as usize].clone() as u16)) as u16; // row offset of tile
                            }
                        }
                        else {
                            if self.vSpriteLine[(i * 4 + 2) as usize] & 0x80 == 0 {
                                // top half of the sprite
                                if (self.scanLine - self.vSpriteLine[(i * 4) as usize] as u16) < 8 {
                                    sprPatAddrLo = ((self.vSpriteLine[(i * 4 + 1) as usize] as u16 & 1 << 12) as u16)
                                        | ((self.vSpriteLine[(i * 4 + 1) as usize] & 0xFE << 4) as u16) // LSB is ignored when fetching tile ID
                                        | ((self.scanLine - self.vSpriteLine[(i * 4) as usize] as u16) & 0x07) as u16;
                                }
                                else {
                                    // bottom half of the sprite
                                    sprPatAddrLo = ((self.vSpriteLine[(i * 4 + 1) as usize] as u16 & 1 << 12) as u16)
                                        | (((self.vSpriteLine[(i * 4 + 1) as usize] & 0xFE + 1) << 4) as u16) // LSB is ignored when fetching tile ID
                                        | ((self.scanLine - self.vSpriteLine[(i * 4) as usize] as u16) & 0x07) as u16;
                                }
                            }
                            else {
                                // top half of the sprite
                                if (self.scanLine - self.vSpriteLine[(i * 4) as usize] as u16) < 8 {
                                    sprPatAddrLo = ((self.vSpriteLine[(i * 4 + 1) as usize] as u16 & 1 << 12) as u16)
                                        | (((self.vSpriteLine[(i * 4 + 1) as usize] & 0xFE + 1) << 4) as u16) // LSB is ignored when fetching tile ID
                                        | (7 - (self.scanLine - self.vSpriteLine[(i * 4) as usize] as u16) & 0x07) as u16;
                                }
                                else {
                                    // bottom half of the sprite
                                    sprPatAddrLo = ((self.vSpriteLine[(i * 4 + 1) as usize] as u16 & 1 << 12) as u16)
                                        | ((self.vSpriteLine[(i * 4 + 1) as usize] & 0xFE << 4) as u16) // LSB is ignored when fetching tile ID
                                        | (7 - (self.scanLine - self.vSpriteLine[(i * 4) as usize] as u16) & 0x07) as u16;
                                }
                            }
                        }

                        sprPatAddrHi = sprPatAddrLo + 8;
                        sprPatBitsLo = self.memory.borrow_mut().readPpuMem(sprPatAddrLo);
                        sprPatBitsHi = self.memory.borrow_mut().readPpuMem(sprPatAddrHi);

                        // flip sprite horizontally
                        if self.vSpriteLine[(i * 4 + 2) as usize] & 0x40 == 1 {

                            let horizontalFlipper = |mut byte: u8| -> u8 {
                                byte = (byte & 0xF0) >> 4 | (byte & 0x0F) << 4;
                                byte = (byte & 0xCC) >> 2 | (byte & 0x33) << 2;
                                byte = (byte & 0xAA) >> 1 | (byte & 0x55) << 1;
                                byte
                            };

                            sprPatBitsHi = horizontalFlipper(sprPatBitsHi);
                            sprPatBitsLo = horizontalFlipper(sprPatBitsLo);
                        }

                        // finally load the bits into the shift registers
                        self.sprShiftPatHi[i as usize] = sprPatBitsHi;
                        self.sprShiftPatLo[i as usize] = sprPatBitsLo;
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

        let mut sprPixel: u8 = 0;
        let mut sprPallete: u8 = 0;
        let mut sprPriority: u8 = 0;

        if self.fSprEnabled == 1 {
            for i in 0..self.spriteLineCount {
                if self.vSpriteLine[(i * 4 + 3) as usize] == 0 {

                    sprPixel = ((self.sprShiftPatHi[i as usize] & 0x80) & 1) << 1 | ((self.sprShiftPatLo[i as usize] & 0x80) & 1);

                    // first four palette entries reserved for background colours
                    sprPallete = (self.vSpriteLine[(i * 4 + 2) as usize] & 0x03) + 0x04;
                    // priority over background (1 means priority)
                    sprPriority = (self.vSpriteLine[(i * 4 + 2) as usize] & 0x20) ^ 1;

                    if sprPriority == 1 {
                        if i == 0 { self.isZeroBeingRendered = true; }
                        break; // lower indexes are higher priority, meaning no successive sprite can trump this one.
                    }
                }
            }
        }

        // combine the background and foreground pixels
        let mut pixel: u8 = 0;
        let mut palette: u8 = 0;

        match bgPixel {
            0 => {
                match sprPixel {
                    1 => {
                        pixel = sprPixel;
                        palette = sprPallete;
                    }
                    _ => {}
                }
            },
            1 => {
                match sprPixel {
                    0 => {
                        pixel = bgPixel;
                        palette = bgPallete;
                    }
                    _ => {
                        match sprPriority {
                            0 => {
                                pixel = bgPixel;
                                palette = bgPallete;
                            },
                            _ => {
                                pixel = sprPixel;
                                palette = sprPallete;
                            }
                        }

                        // if we're rendering sprite zero, and both the background and sprites,
                        // we can have a zero hit
                        if self.isZeroBeingRendered && self.isZeroHitPossible
                            && self.fBckEnabled == 1 && self.fSprEnabled == 1 {

                            // we're not rendering the left-most 8 pixels, so
                            // only generate a hit after the first 8 pixels
                            if (self.fBckLeft | self.fSprLeft) == 0 {
                               if self.cycle > 8 && self.cycle < 258 {
                                   self.fSprZero = 1;
                               }
                            }
                            else {
                                if self.cycle > 0 && self.cycle < 258 {
                                    self.fSprZero = 1;
                                }
                            }
                        }
                    }
                }
            },
            _ => {}
        }

        // call draw function here

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
            sprShiftPatLo: vec![0; 0x0008],
            sprShiftPatHi: vec![0; 0x0008],
            bgTileId: 0,
            bgTileAttr: 0,
            bgTileLsb: 0,
            bgTileMsb: 0,
            vSpriteLine: vec![0; 0x0020],
            spriteLineCount: 0,
            isZeroHitPossible: false,
            isZeroBeingRendered: false,
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

    fn updateBackgroundShiftRegisters(&mut self) -> () {
        self.bgShiftPatLo <<= 1;
        self.bgShiftPatHi <<= 1;
        self.bgShiftAttrLo <<= 1;
        self.bgShiftAttrHi <<= 1;

    }

    fn updateSpriteShiftRegisters(&mut self) -> () {
        for i in 0..self.spriteLineCount {
            // only shift when scanline has hit the start of the sprite
            if self.vSpriteLine[(i * 4 + 3) as usize] > 0 {
                self.vSpriteLine[(i * 4 + 3) as usize] -= 1;
            }
            else {
                self.sprShiftPatLo[i as usize] <<= 1;
                self.sprShiftPatHi[i as usize] <<= 1;
            }
        }
    }

    fn getPatternTable(&mut self) -> () {

    }
}
