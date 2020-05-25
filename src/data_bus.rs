#![allow(non_snake_case)]
#![allow(warnings)]

use std::rc::Rc;
use std::cell::RefCell;
use crate::cpu::*;
use crate::ppu::*;
use crate::cartridge::Cartridge;


pub struct DataBus {
    cpuMem: Vec<u8>,
    ppuMem: Vec<u8>,
    palette: Vec<u8>,
    nmTable: Vec<u8>,
    pnTable: Vec<u8>,
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
            palette: vec![0; 0x0020],
            nmTable: vec![0; 0x0800],
            pnTable: vec![0; 0x2000],
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
        else {
            let cartData = self.cartridge.as_ref().unwrap().borrow_mut().cpuRead(*addr);
        }
    }

    pub fn writePpuMem(&mut self, ref addr: u16, val: u8) -> () {
        if *addr < 0x2000 {
            // pattern memory, this is probably ROM
        }
        else if *addr < 0x3F00 {
            self.ppuMem[*addr as usize] = val;
        }
        else if *addr < 0x4000 {
            // pallete memory
        }
    }

    pub fn readPpuMem(&self, ref addr: u16) -> u8 {
        if *addr < 0x2000 {
            // pattern memory, this is probably ROM
        }
        else if *addr < 0x3F00 {
            return self.ppuMem[*addr as usize].clone();
        }
        else if *addr < 0x4000 {
            // pallete memory
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

        for i in 0..256 {
            self.oamMem[i] = self.cpuMem[cpuAddr as usize + i].clone();
        }

        self.cpu.as_ref().unwrap().borrow_mut().addOamCycles();
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