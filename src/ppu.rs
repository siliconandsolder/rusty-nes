#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use std::rc::Rc;
use std::cell::RefCell;
use log::info;
use crate::data_bus::DataBus;
use crate::clock;
use crate::clock::Clocked;
use crate::palette::*;
use sdl2::render::{WindowCanvas, Texture, TextureAccess, TextureCreator};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use crate::ppu_bus::PpuBus;
use std::fs::File;
use std::io::Write;
use sdl2::mouse::SystemCursor::No;
use sdl2::video::WindowContext;
use std::borrow::Borrow;

const SCANLINE_VISIBLE_MAX: u16 = 239;
const SCANLINE_POST: u16 = 240;
const SCANLINE_VBLANK_MIN: u16 = 241;
const SCANLINE_VBLANK_MAX: u16 = 260;
const SCANLINE_MAX: u16 = 261;

const CYCLE_MAX: u16 = 340;

const CYCLES_PER_FRAME: u32 = 89342;

const PIXEL_WIDTH: u32 = 256;
const PIXEL_HEIGHT: u32 = 240;

struct TextureManager<'a> {
    textureCreator: TextureCreator<WindowContext>,
    texture: Option<Texture<'a>>
}

impl<'a> TextureManager<'a> {
    pub fn new(textureCreator: TextureCreator<WindowContext>) -> Self {
        let mut tm = TextureManager {
            textureCreator,
            texture: None
        };
        tm.createTexture();
        return tm;
    }

    pub fn createTexture(&mut self) -> () {
        let tcp = &self.textureCreator as *const TextureCreator<WindowContext>;
        let texture = unsafe { &*tcp }
            .create_texture(
                PixelFormatEnum::RGB24,
                TextureAccess::Streaming,
                PIXEL_WIDTH,
                PIXEL_HEIGHT,
            ).unwrap();
        self.texture = Some(texture);
    }

    pub fn updateTexture(&mut self, pixelBytes: &[u8]) -> () {
        self.texture.as_mut().unwrap().update(None, pixelBytes, (PIXEL_WIDTH * 3) as usize);
    }

    pub fn getTextureRef(&self) -> &Texture<'a> {
        return self.texture.as_ref().unwrap();
    }
}

pub struct Ppu<'a> {
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
    nmiDelay: u8,

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

    dataBus: Rc<RefCell<DataBus<'a>>>,
    ppuBus: PpuBus,

    // SDL pixels (rectangles)
    canvas: Rc<RefCell<WindowCanvas>>,
    textureManager: TextureManager<'a>,
    vPixels: Vec<Rect>,
    vPixelColours: Vec<u8>,
    vPixelPalette: Vec<u8>,
    
    // debug
    log: File

}

impl<'a> Clocked for Ppu<'a> {
    #[inline]
    fn cycle(&mut self) {

        let renderEnabled = self.fSprEnabled == 1 || self.fBckEnabled == 1;
        let renderLine = self.scanLine < SCANLINE_VBLANK_MIN - 1;
        let preLine = self.scanLine == SCANLINE_MAX;
        let renderCycle = self.cycle > 1 && self.cycle < 258;
        let fetchCycle = self.cycle > 320 && self.cycle < 338;

        if self.scanLine == SCANLINE_VBLANK_MIN && self.cycle == 1 {
            self.nmiOccured = true;
        }

        if self.scanLine == SCANLINE_MAX && self.cycle == 1 {
            self.fSprZero = 0;
            self.nmiOccured = false;
            self.canTrigNmi = true;
            self.nmiDelay = 0;

            // wipe sprites for next scanline
            self.fSprOver = 0;
            for i in 0..8 {
                self.sprShiftPatLo[i] = 0;
                self.sprShiftPatHi[i] = 0;
            }
        }

        if !self.canTrigNmi && self.nmiDelay > 0 {
            self.nmiDelay -= 1;
            if self.nmiDelay == 0 {
                self.dataBus.borrow_mut().ppuTriggerNMI();
            }
        }

        if self.fNmi == 1 && self.nmiOccured && self.canTrigNmi {
            self.canTrigNmi = false;
            self.nmiDelay = 15;
        }

        // if self.cycle >= 257 && self.cycle <= 320 {
        //     self.oamAddr = 0;
        // }

        if renderEnabled {
            if (renderLine || preLine) && (renderCycle || fetchCycle) {

                if self.fBckEnabled == 1 {
                    self.updateBackgroundShiftRegisters();
                }

                if self.fSprEnabled == 1 && self.cycle < 248 {
                    self.updateSpriteShiftRegisters();
                }


                let vAddr = *&self.v;
                //info!("\nCycle: {}\n", self.cycle);
                match (self.cycle - 1) % 8 {
                    0 => {
                        self.loadBackgroundShiftRegisters();
                        self.bgTileId = self.ppuBus.readPpuMem(
                            0x2000 | (vAddr & 0x0FFF)
                        );
                        //info!("BgTileId: {}\n", self.bgTileId);
                    },
                    2 => {

                        let bgTileAddr = 0x23C0 | (vAddr & 0x0C00) | ((vAddr >> 4) & 0x38) | ((vAddr >> 2) & 0x07);
                        self.bgTileAttr = self.ppuBus.readPpuMem(
                            bgTileAddr
                        );

                        //info!("BgTileAddr: {}\n", bgTileAddr);
                        //info!("BgTileAttr: {}\n", self.bgTileAttr);

                        /* vram address
                        0x0yyy YX YYYYY XXXXX
                           ||| || ||||| +++++-- coarse X scroll
                           ||| || +++++-------- coarse Y scroll
                           ||| ++-------------- nametable select
                           +++----------------- fine Y scroll
                        */
                        // let coarseY =   (vAddr & 0b0000001111100000) >> 5;
                        // let coarseX =   (vAddr & 0b0000000000011111);
                        // if (coarseY & 2) == 2 { self.bgTileAttr >>= 4; }
                        // if (coarseX & 2) == 2 { self.bgTileAttr >>= 2; }
						// self.bgTileAttr &= 3;
                        let shift = ((vAddr & 0x40) >> 4) | (vAddr & 2);
                        self.bgTileAttr = (self.bgTileAttr >> shift as u8) & 3;
                    },
                    4 => {
                        self.bgTileLsb = self.ppuBus.readPpuMem(
                            ((self.fBckTile as u16) << 12) |
                                ((self.bgTileId as u16) << 4) |
                                ((vAddr >> 12) & 0b0111 as u16)
                        );
                    },
                    6 => {
                        self.bgTileMsb = self.ppuBus.readPpuMem(
                            (((self.fBckTile as u16) << 12) |
                                ((self.bgTileId as u16) << 4) |
                                ((vAddr >> 12) & 0b0111 as u16)) + 8 as u16
                        );
                    },
                    7 => {
                        //info!("vAddr before incrementX: {}\n", self.v);
                        self.incrementX();
                        //info!("vAddr after incrementX: {}\n", self.v);
                    },
                    _ => {}
                }
            }

            match self.cycle {
                256 => {
                    //info!("vAddr before incrementY: {}\n", self.v);
                    if renderLine || preLine {
                        self.incrementY();
                    }
                    //info!("vAddr after incrementY: {}\n", self.v);
                },
                257 => {
                    self.loadBackgroundShiftRegisters();
                    // copy nametable x and coarse x
                    //info!("vAddr before X-transfer: {}\n", self.v);
                    if renderLine || preLine {
                        self.v = (self.v & 0xFBE0) | (self.t & 0x041F);
                    }
                    //info!("vAddr after X-transfer: {}\n", self.v);

                    if renderLine {

                        for i in &mut self.vSpriteLine { *i = 0; }
                        self.spriteLineCount = 0;
                        let mut oamIdx: u8 = 0;
                        self.isZeroBeingRendered = false;

                        while oamIdx < 64 && self.spriteLineCount < 9 {
                            let oamY: u8 = self.ppuBus.readOam(oamIdx * 4);
                            let spriteSize: u8 = if self.fSprHeight == 0 { 8 } else { 16 };

                            let mut diff: i16 = self.scanLine as i16 - oamY as i16;
                            // the sprite will be rendered on the next scanline!
                            if diff > -1 && diff < spriteSize as i16 {
                                // copy oam entry into scanline vector
                                // increment sprite count
                                if self.spriteLineCount < 8 {

                                    if oamIdx == 0 { self.isZeroHitPossible = true; }

                                    for i in 0..=3 {
                                        self.vSpriteLine[(self.spriteLineCount * 4 + i) as usize] = self.ppuBus.readOam(oamIdx * 4 + i)
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
                    if preLine {
                        //info!("vAddr before Y-transfer: {}\n", self.v);
                        // copy fine y, nametable y, and coarse y to vram address
                        self.v = (self.v & 0x841F) | (self.t & 0x7BE0);
                        //info!("vAddr after Y-transfer: {}\n", self.v);
                    }
                },
                338 => { self.bgTileId = self.ppuBus.readPpuMem( 0x2000 | (*&self.v & 0x0FFF) ); },
                340 => {
                    self.bgTileId = self.ppuBus.readPpuMem( 0x2000 | (*&self.v & 0x0FFF) );

                    // behold: sprite logic!
                    for i in 0..self.spriteLineCount {
                        let mut sprPatBitsLo: u8 = 0;
                        let mut sprPatBitsHi: u8 = 0;

                        let sprY = self.vSpriteLine[(i * 4) as usize].clone() as u16;
                        let mut sprTile = self.vSpriteLine[(i * 4 + 1) as usize].clone() as u16;
                        let sprAttr = self.vSpriteLine[(i * 4 + 2) as usize].clone() as u16;
                        let mut row = self.scanLine - sprY;

                        let mut sprAddress: u16 = 0;

                        if self.fSprHeight == 0 {
                            // sprite flipped vertically
                            if sprAttr & 0x80 == 0x80 {
                                row = 7 - row;
                            }
                            sprAddress = ((self.fSprTile as u16) << 12) | (sprTile << 4) | row;
                        } else {
                            // sprite flipped vertically
                            if sprAttr & 0x80 == 0x80 {
                                row = 15 - row;
                            }

                            let flagTile: u16 = (self.fSprTile & 1) as u16;
                            sprTile &= 0xFE;
                            if row > 7 {
                                sprTile += 1;
                                row -= 8;
                            }
                            sprAddress = (flagTile << 12) | (sprTile << 4) | row;
                        }

                        // sprPatAddrHi = sprPatAddrLo + 8;
                        sprPatBitsLo = self.ppuBus.readPpuMem(sprAddress);
                        sprPatBitsHi = self.ppuBus.readPpuMem(sprAddress + 8);

                        // flip sprite horizontally
                        if sprAttr & 0x40 == 0x40 {
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

            bgPixel = ((if self.bgShiftPatHi & mux > 0 {1} else {0} as u8) << 1) | if self.bgShiftPatLo & mux > 0 {1} else {0} as u8;
            bgPallete = ((if self.bgShiftAttrHi & mux > 0 {1} else {0} as u8) << 1) | if self.bgShiftAttrLo & mux > 0 {1} else {0} as u8;
        }

        let mut sprPixel: u8 = 0;
        let mut sprPallete: u8 = 0;
        let mut sprPriority: u8 = 0;

        if self.fSprEnabled == 1 {
            for i in 0..self.spriteLineCount {
                if self.vSpriteLine[(i * 4 + 3) as usize] == 0 {

                    sprPixel = ((if self.sprShiftPatHi[i as usize] & 0x80 != 0 {1} else {0}) << 1) | (if self.sprShiftPatLo[i as usize] & 0x80 != 0 {1} else {0});

                    // first four palette entries reserved for background colours
                    sprPallete = (self.vSpriteLine[(i * 4 + 2) as usize] & 0x03) + 0x04;
                    // priority over background (1 means priority)
                    sprPriority = if (self.vSpriteLine[(i * 4 + 2) as usize] & 0x20) == 0x20 {0} else {1};

                    if sprPixel != 0 {
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
                    0 => {},
                    _ => {
                        pixel = sprPixel;
                        palette = sprPallete;
                    }
                }
            },
            _ => {
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
            }
        }

        // call draw function here
        if renderEnabled && self.cycle > 0 && self.cycle < 257 && self.scanLine < SCANLINE_VBLANK_MIN - 1 {
            self.setPixelColour(self.cycle - 1, self.scanLine.clone(), palette, pixel);
        }


        // increment cycle and scanline
        self.cycle += 1;
        if self.cycle == CYCLE_MAX + 1 {
            self.cycle = 0;

            self.scanLine += 1;
            if self.scanLine > SCANLINE_MAX {
                self.scanLine = 0;
                self.isOddFrame = !self.isOddFrame;

                if renderEnabled {
                    self.drawFrame();
                }
            }
        }
    }
}

impl<'a> Ppu<'a> {
    pub fn new(mem: Rc<RefCell<DataBus<'a>>>, canvas: Rc<RefCell<WindowCanvas>>, ppuBus: PpuBus) -> Self {

        let textureManager = TextureManager::new(canvas.borrow_mut().texture_creator());

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
            nmiDelay: 0,
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
            dataBus: mem,
            ppuBus: ppuBus,
            canvas: canvas,
            textureManager: textureManager,
            vPixels: vec![Rect::new(0, 0, 1, 1); 61440],
            vPixelColours: vec![0; (PIXEL_WIDTH * PIXEL_HEIGHT * 3) as usize],
            vPixelPalette: vec![0; (PIXEL_WIDTH * PIXEL_HEIGHT) as usize],
            log: File::create("log.txt").unwrap()
        }
    }

    pub fn readMem(&mut self, ref addr: u16) -> u8 {
        match *addr {
            0x0002 => { return self.ppuStatus(); },     // PPU STATUS
            0x0004 => { return self.oamDataRead(); },   // OAM DATA
            0x0007 => { return self.ppuDataRead(); },   // PPU DATA
            //_ => panic!("Unknown PPU register: {}", *addr)
            _=> {}
        }
        return 0;
    }

    pub fn writeMem(&mut self, ref addr: u16, val: u8) -> () {
        match *addr {
            0x0000 => { self.ppuCtrl(val) },        // PPU CONTROL
            0x0001 => { self.ppuMask(val) },        // PPU MASK
            0x0003 => { self.oamAddress(val) },     // OAM ADDRESS
            0x0004 => { self.oamDataWrite(val) },   // OAM DATA
            0x0005 => { self.ppuScroll(val) },      // PPU SCROLL
            0x0006 => { self.ppuAddress(val) },     // PPU ADDRESS
            0x0007 => { self.ppuDataWrite(val) },   // PPU DATA
            //_ => panic!("Unknown PPU register: {}", *addr)
            _=> {}
        }
        self.prevReg = val;
    }

    pub fn cpuWriteOam(&mut self, val: u8) -> () {
        self.ppuBus.writeOam(*&self.oamAddr, val);
        self.oamAddr = self.oamAddr.wrapping_add(1);
    }

    fn ppuCtrl(&mut self, val: u8) -> () {
        self.fNameTable = val & 3;
        self.fIncMode = (val >> 2) & 1;
        self.fSprTile = (val >> 3) & 1;
        self.fBckTile = (val >> 4) & 1;
        self.fSprHeight = (val >> 5) & 1;
        self.fMaster = (val >> 6) & 1;
        self.fNmi = (val >> 7) & 1;

        if self.nmiDelay == 0 {
            self.canTrigNmi = true;
        }

        //info!("PPUCTRL val: {}, tAddr before PPUCTRL write: {}\n", val, self.t);
        self.t = (self.t & 0xF3FF) | (((val & 0x03) as u16) << 10);
        //info!("PPUCTRL val: {}, tAddr after PPUCTRL write: {}\n", val, self.t);
    }

    fn ppuMask(&mut self, val: u8) -> () {
        self.fGrey = val & 1;
        self.fBckLeft = (val >> 1) & 1;
        self.fSprLeft = (val >> 2) & 1;
        self.fBckEnabled = (val >> 3) & 1;
        self.fSprEnabled = (val >> 4) & 1;
        self.fColour = (val >> 5) & 0b0111;
    }

    fn ppuStatus(&mut self) -> u8 {
        let mut value = self.prevReg & 0x001F;
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
        self.ppuBus.writeOam(self.oamAddr, val);
        self.oamAddr = self.oamAddr.wrapping_add(1);
    }

    fn oamDataRead(&mut self) -> u8 {
        return self.ppuBus.readOam(self.oamAddr);
        // do not increment if v-blank or forced blank
    }

    fn ppuScroll(&mut self, val: u8) -> () {
        if self.w == 0 {
            //info!("PPUSCROLL val: {}, tAddr before first ppuScroll write: {}\n", val, self.t);
            self.t = (self.t & 0xFFE0) | ((val as u16) >> 3);
            self.x = (val & 0x07);
            self.w = 1;
            //info!("PPUSCROLL val: {}, tAddr after first ppuScroll write: {}\n", val, self.t);
        }
        else {

            //info!("PPUSCROLL val: {}, tAddr before second ppuScroll write: {}\n", val, self.t);
            self.t = (self.t & 0x8FFF) | (((val & 0x07) as u16) << 12);
            self.t = (self.t & 0xFC1F) | (((val & 0xF8) as u16) << 2);
            self.w = 0;
            //info!("PPUSCROLL val: {}, tAddr after second ppuScroll write: {}\n", val, self.t);
        }
    }

    fn ppuAddress(&mut self, val: u8) -> () {
        if self.w == 0 {
            //info!("PPUADDR val: {}, tAddr before first ppuAddress write: {}\n", val, self.t);
            self.t = (self.t & 0x80FF) | ((val as u16 & 0x3F) << 8);
            //info!("PPUADDR val: {}, tAddr after first ppuAddress write: {}\n", val, self.t);
            self.w = 1;
        }
        else {
            //info!("PPUADDR val: {}, tAddr before second ppuAddress write: {}\n", val, self.t);
            self.t = (self.t & 0xFF00) | (val as u16);
            //info!("PPUADDR val: {}, tAddr/vAddr after second ppuAddress write: {}\n", val, self.t);
            self.v = self.t;
            self.w = 0;
        }
    }

    fn ppuDataWrite(&mut self, val: u8) -> () {
        let vPtr = *&self.v;
        self.ppuBus.writePpuMem(vPtr, val);
        self.v = if self.fIncMode == 0 { self.v.wrapping_add(1) } else { self.v.wrapping_add(32) };
        //info!("PPUDATA val: {}, vAddr after ppuData write: {}\n", val, self.v);
    }

    fn ppuDataRead(&mut self) -> u8 {

        let mut tempBufData: u8 = 0;
        let vPtr = &self.v;
        let mut ppuData = self.ppuBus.readPpuMem(*vPtr);

        if (self.v & 0x3F00) < 0x3F00 {
            tempBufData = self.bufData;
            self.bufData = ppuData;
            ppuData = tempBufData;
        }
        else {
            // maps to nametable under the palette (palette address minus 0x1000)
            self.bufData = self.ppuBus.readPpuMem(*vPtr - 0x1000);
        }

        self.v = if self.fIncMode == 0 { self.v.wrapping_add(1) } else { self.v.wrapping_add(32) };
        //info!("vAddr after ppuData read: {}\n", self.v);
		return ppuData;
    }

    fn incrementX(&mut self) -> () {
        if self.v & 0x001F == 0x001F {
            self.v &= !0x001F;
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
        self.bgShiftPatLo = (self.bgShiftPatLo & 0xFF00) | self.bgTileLsb as u16;
        self.bgShiftPatHi = (self.bgShiftPatHi & 0xFF00) | self.bgTileMsb as u16;

        self.bgShiftAttrLo = (self.bgShiftAttrLo & 0xFF00) | (if self.bgTileAttr & 0b01 == 1    { 0x00FF } else { 0x0000 });
        self.bgShiftAttrHi = (self.bgShiftAttrHi & 0xFF00) | (if self.bgTileAttr & 0b10 == 0b10 { 0x00FF } else { 0x0000 });
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

    fn setPixelColour(&mut self, x: u16, y: u16, palette: u8, pixel: u8) -> () {
        let mut address = self.ppuBus.readPpuMem(0x3F00 + (palette << 2) as u16 + pixel as u16);
        self.vPixelPalette[(x + (y * PIXEL_WIDTH as u16)) as usize] = address;
    }

    fn drawFrame(&mut self) -> () {
        for i in 0..(PIXEL_WIDTH * PIXEL_HEIGHT) {
            let colour = PALETTE_ARRAY.get((self.vPixelPalette[i as usize] & 0x003F) as usize).unwrap();
            self.vPixelColours[(i * 3) as usize] = colour.red;
            self.vPixelColours[(i * 3 + 1) as usize] = colour.green;
            self.vPixelColours[(i * 3 + 2) as usize] = colour.blue;
        }

        self.textureManager.updateTexture(self.vPixelColours.as_slice());
        self.canvas.borrow_mut().clear();
        self.canvas.borrow_mut().copy(self.textureManager.getTextureRef(), None, None).unwrap();
        self.canvas.borrow_mut().present();
    }
}
