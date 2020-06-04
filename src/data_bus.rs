#![allow(non_snake_case)]
#![allow(warnings)]

use std::rc::Rc;
use std::cell::RefCell;
use crate::cpu::*;
use crate::ppu::*;
use crate::cartridge::{Cartridge, MIRROR};
use crate::palette::*;


pub struct DataBus {
    cpuMem: Vec<u8>,
    ppuMem: Vec<u8>,
    tblPalette: Vec<u8>,
    tblName: Vec<u8>,
    tblPattern: Vec<u8>,
    oamMem: Vec<u8>,
    interruptMem: Vec<u8>,

    cpu: Option<Rc<RefCell<Cpu>>>,
    ppu: Option<Rc<RefCell<Ppu>>>,
    cartridge: Option<Rc<RefCell<Cartridge>>>
}

impl DataBus {
    pub fn new() -> DataBus {
        DataBus {
            cpuMem: vec![0; 0x0800],
            ppuMem: vec![0; 0x0008],
            tblPalette: vec![0; 0x0020],
            tblName: vec![0; 0x1000],
            tblPattern: vec![0; 0x1000],
            oamMem: vec![0; 0x0100],
            interruptMem: vec![0; 0x0006],
            cpu: None,
            ppu: None,
            cartridge: None
        }
    }

    pub fn attachPpu(&mut self, ppuRef: Rc<RefCell<Ppu>>) -> () {
        self.ppu = Some(Rc::from(ppuRef));
    }

    pub fn attachCpu(&mut self, cpuRef: Rc<RefCell<Cpu>>) -> () {
        self.cpu = Some(Rc::from(cpuRef));
    }

    pub fn attachCartridge(&mut self, cartRef: Rc<RefCell<Cartridge>>) -> () {
        self.cartridge = Some(Rc::from(cartRef))
    }

    pub fn writeCpuMem(&mut self, ref addr: u16, val: u8) -> () {
        if *addr < 0x2000 {
            self.cpuMem[(*addr & 0x07FF) as usize] = val;
        }
        else if *addr >= 0x2000 && *addr <= 0x3FFF {
            self.ppu.as_ref().unwrap().borrow_mut().writeMem(*addr & 0x0007, val);
        }
        else if *addr == 0x4014 {   // special case for OAM writes
            self.ppu.as_ref().unwrap().borrow_mut().writeMem(*addr, val);
        }
        else if *addr == 0x4016 {
            // controller one stuff goes here
        }
        else if *addr == 0x4017 {
            // controller two stuff goes here
        }
        else {
            self.cartridge.as_ref().unwrap().borrow_mut().cpuWrite(*addr, val);
        }
    }

    pub fn readCpuMem(&self, ref addr: u16) -> u8 {
        if *addr < 0x2000 {
            return self.cpuMem[(*addr & 0x07FF) as usize].clone();
        }
        else if *addr >= 0x2000 && *addr <= 0x3FFF {
            return self.ppu.as_ref().unwrap().borrow_mut().readMem(*addr).clone();
        }
        else if *addr == 0x4016 {
            // controller one stuff goes here
            return 0;
        }
        else if *addr == 0x4017 {
            // controller two stuff goes here
            return 0;
        }
        else {
            return self.cartridge.as_ref().unwrap().borrow_mut().cpuRead(*addr);
        }
    }

    pub fn writePpuMem(&mut self, ref addr: u16, val: u8) -> () {
        if *addr < 0x2000 {
            self.tblPattern[*addr as usize] = val;
        }
        else if *addr < 0x3F00 {
            let cart = self.cartridge.as_ref().unwrap();
            let realAddr = *addr & 0x0FFF;

            match cart.borrow().getMirrorType() {
                MIRROR::HORIZONTAL => {
                    match realAddr {
                        a if a < 0x0800 => { self.tblName[(a & 0x03FF) as usize] = val; }
                        _ => { self.tblName[(realAddr & 0x0BFF) as usize] = val; }
                    }
                },
                MIRROR::VERTICAL => {
                    match realAddr {
                        a if a < 0x0800 => { self.tblName[a as usize] = val; }
                        a if a < 0x0C00 => { self.tblName[(a & 0x03FF) as usize] = val; }
                        _ => { self.tblName[(realAddr & 0x07FF) as usize] = val; }
                    }
                },
                _ => { panic!("wat?") }
            }
        }
        else if *addr < 0x4000 {
            let mut realAddr = *addr & 0x001F;
            if realAddr % 4 == 0 { realAddr = 0; }  // fourth byte is transparent (background)
            self.tblPalette[realAddr as usize] = val;
        }
    }

    pub fn readPpuMem(&self, ref addr: u16) -> u8 {
        if *addr < 0x2000 {
            return self.tblPattern[*addr as usize].clone();
        }
        else if *addr < 0x3F00 {
            let cart = self.cartridge.as_ref().unwrap();
            let realAddr = *addr & 0x0FFF;

            match cart.borrow().getMirrorType() {
                MIRROR::HORIZONTAL => {
                    return match realAddr {
                        a if a < 0x0800 => { self.tblName[(a & 0x03FF) as usize] }
                        _ => { self.tblName[(realAddr & 0x0BFF) as usize] }
                    }
                },
                MIRROR::VERTICAL => {
                    return match realAddr {
                        a if a < 0x0800 => { self.tblName[a as usize] }
                        a if a < 0x0C00 => { self.tblName[(a & 0x03FF) as usize] }
                        _ => { self.tblName[(realAddr & 0x07FF) as usize] }
                    }
                },
                _ => { panic!("wat?") }
            }

            return self.tblName[(*addr & 0x1FFF) as usize].clone();
        }
        else if *addr < 0x4000 {
            let mut realAddr = *addr & 0x001F;
            if realAddr % 4 == 0 { realAddr = 0; }  // fourth byte is transparent (background)
            return self.tblPalette[realAddr as usize].clone();
        }
        return 0;
    }

    pub fn writeOam(&mut self, ref addr: u8, val: u8) -> () {
        self.oamMem[*addr as usize] = val;
    }

    pub fn readOam(&mut self, ref addr: u8) -> u8 {
        return self.oamMem[*addr as usize].clone();
    }

    pub fn overWriteOam(&mut self, val: u8) -> () {
        let cpuAddr: u16 = (val << 16) as u16;
        self.cpu.as_ref().unwrap().borrow_mut().triggerOamTransfer(cpuAddr);
    }

    pub fn triggerNMI(&mut self) -> () {
        self.cpu.as_ref().unwrap().borrow_mut().setNmi();
    }

    pub fn pushStack(&mut self, stackP: &mut u8, val: u8) -> () {
        self.cpuMem[(0x100 + *stackP as u16) as usize] = val;
        *stackP = stackP.wrapping_add(1);
    }

    pub fn popStack(&mut self, stackP: &mut u8) -> u8 {
        *stackP = stackP.wrapping_sub(1);
        let val = self.cpuMem[(0x100 + *stackP as u16) as usize];
        return val;
    }

    pub fn getInterruptOffset(&self, ref addr: u16) -> u8 {
        return match *addr {
            0xFFFA => 0,
            0xFFFB => 1,
            0xFFFC => 2,
            0xFFFD => 3,
            0xFFFE => 4,
            0xFFFF => 5,
            _ => panic!("Interrupt memory out of bounds: {}", *addr)
        }
    }

    fn mirrorCPU(&mut self, addr: &u16) -> () {
        for i in 1..4 {
            self.cpuMem[i * 0x800 + *addr as usize] = self.cpuMem[*addr as usize]
        }
    }

    fn findCPUQuadrant(&self, addr: &u16) -> u8 {
        match *addr {
            a if a < 0x0800 => 0,
            a if a < 0x1000 => 1,
            a if a < 0x1800 => 2,
            _ => 3
        }
    }
}


#[cfg(test)]
mod MemorySpec {
    use super::*;

    #[test]
    fn happyPath() -> () {
        let mem = DataBus::new();
        assert_eq!(mem.cpuMem.len(), 0x10000);
    }

    #[test]
    fn writeToMemHappyPath() -> () {
        let mut mem = DataBus::new();
        mem.writeCpuMem(0x0, 0);
        mem.writeCpuMem(0x1, 1);
        mem.writeCpuMem(0x2, 2);

        assert_eq!(mem.cpuMem[0x0], 0);
        assert_eq!((mem.cpuMem[0x0] == mem.cpuMem[0x800]), (mem.cpuMem[0x1000] == mem.cpuMem[0x1800]));

        assert_eq!(mem.cpuMem[0x1], 1);
        assert_eq!((mem.cpuMem[0x1] == mem.cpuMem[0x801]), (mem.cpuMem[0x1001] == mem.cpuMem[0x1801]));

        assert_eq!(mem.cpuMem[0x2], 2);
        assert_eq!((mem.cpuMem[0x2] == mem.cpuMem[0x802]), (mem.cpuMem[0x1002] == mem.cpuMem[0x1802]));
    }

    #[test]
    #[should_panic]
    fn writeToMemOutOfBounds() -> () {
        let mut mem = DataBus::new();
        mem.writeCpuMem(0x2001, 1);
    }

    #[test]
    fn pushPopStack() -> () {
        let mut mem = DataBus::new();
        let mut stkPointer: &mut u8 = &mut 0x00;

        mem.pushStack(stkPointer, 1);
        assert_eq!(stkPointer, 0x01);
        assert_eq!(mem.cpuMem[(stkPointer - 1) as usize], 1);

        let val = mem.popStack(stkPointer);
        assert_eq!(val, 1);
        assert_eq!(stkPointer, 0x0100);
    }
}