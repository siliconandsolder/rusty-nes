#![allow(non_snake_case)]
#![allow(warnings)]

use std::cell::RefCell;
use std::rc::Rc;
use crate::cartridge::Cartridge;
use crate::mappers::mapper::MirrorType;
use crate::save_load::PpuBusData;

pub struct PpuBus {
    tblPalette: Vec<u8>,
    tblName: Vec<u8>,
    cart: Option<Rc<RefCell<Cartridge>>>,
    oamMem: Vec<u8>,

}

impl PpuBus {
    pub fn new() -> Self {
        PpuBus {
            tblPalette: vec![0; 0x0020],
            tblName: vec![0; 0x1000],
            cart: None,
            oamMem: vec![0; 0x0100],
        }
    }

    pub fn attachCartridge(&mut self, cart: Rc<RefCell<Cartridge>>) -> () {
        self.cart = Some(cart);
    }

    pub fn saveState(&self) -> PpuBusData {
        PpuBusData {
            tblPalette: self.tblPalette.clone(),
            tblName: self.tblName.clone(),
            oamMem: self.oamMem.clone()
        }
    }

    pub fn loadState(&mut self, data: &PpuBusData) -> () {
        self.tblPalette = data.tblPalette.clone();
        self.tblName = data.tblName.clone();
        self.oamMem = data.oamMem.clone();
    }

    #[inline]
    pub fn readPpuMem(&self, ref addr: u16) -> u8 {
        let adr = *addr & 0x3FFF;
        if adr < 0x2000 {
            return self.cart.as_ref().unwrap().borrow_mut().ppuRead(adr);
        }
        else if adr < 0x3F00 {
            return match self.cart.as_ref().unwrap().borrow().getMirrorType() {
                MirrorType::Horizontal => {
                    match adr {
                        a if a < 0x2800 => { self.tblName[(a & 0x03FF) as usize].clone() }
                        _ => { self.tblName[((adr & 0x03FF) | 0x0400) as usize].clone() }
                    }
                }
                MirrorType::Vertical => {
                    self.tblName[(adr & 0x07FF) as usize].clone()
                }
                MirrorType::SingleScreenLow => {
                    self.tblName[(adr & 0x03FF) as usize].clone()
                }
                MirrorType::SingleScreenHigh => {
                    self.tblName[((adr & 0x03FF) | 0x400) as usize].clone()
                }
            }
        }
        else if adr < 0x4000 {
            let mut realAddr = adr & 0x001F;
            if (realAddr >= 16) && (realAddr % 4 == 0) { realAddr -= 16; }  // fourth byte is transparent (background)
            return self.tblPalette[realAddr as usize].clone();
        }
        return 0;
    }

    #[inline]
    pub fn writePpuMem(&mut self, ref addr: u16, val: u8) -> () {
        let adr = *addr & 0x3FFF;
        if adr < 0x2000 {
            return self.cart.as_ref().unwrap().borrow_mut().ppuWrite(adr, val);
        } else if adr < 0x3F00 {

            match self.cart.as_ref().unwrap().borrow().getMirrorType() {
                MirrorType::Horizontal => {
                    match adr {
                        a if a < 0x2800 => { self.tblName[(a & 0x03FF) as usize] = val; }
                        _ => { self.tblName[((addr & 0x03FF) | 0x0400) as usize] = val; }
                    }
                }
                MirrorType::Vertical => {
                    self.tblName[(adr & 0x07FF) as usize] = val;
                }
                MirrorType::SingleScreenLow => {
                    self.tblName[(adr & 0x03FF) as usize] = val;
                }
                MirrorType::SingleScreenHigh => {
                    self.tblName[((adr & 0x03FF) | 0x400) as usize] = val;
                }
            }
        } else if adr < 0x4000 {
            let mut realAddr = adr & 0x001F;
            if (realAddr >= 16) && (realAddr % 4 == 0) { realAddr -= 16; }  // fourth byte is transparent (background)
            self.tblPalette[realAddr as usize] = val;
        }
    }

    #[inline]
    pub fn writeOam(&mut self, ref addr: u8, val: u8) -> () {
        self.oamMem[*addr as usize] = val;
    }

    #[inline]
    pub fn readOam(&mut self, ref addr: u8) -> u8 {
        return self.oamMem[*addr as usize].clone();
    }
}