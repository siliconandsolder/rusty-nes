#![allow(non_snake_case)]
#![allow(warnings)]

use std::cell::RefCell;
use std::rc::Rc;
use crate::cartridge::Cartridge;
use crate::mapper::MIRROR;

pub struct PpuBus {
    tblPalette: Vec<u8>,
    tblName: Vec<u8>,
    cart: Rc<RefCell<Cartridge>>,
    oamMem: Vec<u8>,

}

impl PpuBus {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Self {
        PpuBus {
            tblPalette: vec![0; 0x0020],
            tblName: vec![0; 0x1000],
            cart: cartridge,
            oamMem: vec![0; 0x0100],
        }
    }

    #[inline]
    pub fn readPpuMem(&self, ref addr: u16) -> u8 {
        let addr = *addr & 0x3FFF;
        if addr < 0x2000 {
            return self.cart.borrow_mut().ppuRead(addr);
        }
        else if addr < 0x3F00 {
            let realAddr = addr & 0x0FFF;

            match self.cart.borrow().getMirrorType() {
                MIRROR::HORIZONTAL => {
                    return match realAddr {
                        a if a < 0x0800 => { self.tblName[(a & 0x03FF) as usize].clone() }
                        _ => { self.tblName[(realAddr & 0x0BFF) as usize].clone() }
                    };
                }
                MIRROR::VERTICAL => {
                    return self.tblName[(realAddr & 0x07FF) as usize].clone();
                }
                _ => { panic!("Unrecognized Mirror Type: {:?}", self.cart.borrow().getMirrorType()); }
            }
        } else if addr < 0x4000 {
            let mut realAddr = addr & 0x001F;
            if (realAddr >= 16) && (realAddr % 4 == 0) { realAddr -= 16; }  // fourth byte is transparent (background)
            return self.tblPalette[realAddr as usize].clone();
        }
        return 0;
    }

    #[inline]
    pub fn writePpuMem(&mut self, ref addr: u16, val: u8) -> () {
        let addr = *addr & 0x3FFF;
        if addr < 0x2000 {
            return self.cart.borrow_mut().ppuWrite(addr, val);
        } else if addr < 0x3F00 {
            //let realAddr = addr & 0x0FFF;

            match self.cart.borrow().getMirrorType() {
                MIRROR::HORIZONTAL => {
                    match addr {
                        a if a < 0x2800 => { self.tblName[(a & 0x03FF) as usize] = val; }
                        _ => { self.tblName[(addr & 0x0BFF) as usize] = val; }
                    }
                }
                MIRROR::VERTICAL => {
                    self.tblName[(addr & 0x07FF) as usize] = val;
                }
                _ => { panic!("Unrecognized Mirror Type: {:?}", self.cart.borrow().getMirrorType()); }
            }
        } else if addr < 0x4000 {
            let mut realAddr = addr & 0x001F;
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