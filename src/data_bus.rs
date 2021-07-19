#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use std::rc::Rc;
use std::cell::RefCell;
use log::info;
use crate::cpu::*;
use crate::ppu::*;
use crate::cartridge::*;
use crate::mappers::mapper::MirrorType::*;
use crate::palette::*;
use crate::controller::Controller;
use crate::clock::Clocked;
use crate::apu::Apu;
use sdl2::event::Event;

// Split the buses in two. One for CPU-PPU intercommunication, one for PPU data reads and writes.

pub struct DataBus<'a> {
    cpuMem: Vec<u8>,

    cpu: Option<Rc<RefCell<Cpu<'a>>>>,
    ppu: Option<Rc<RefCell<Ppu<'a>>>>,
    apu: Option<Rc<RefCell<Apu<'a>>>>,
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    controller1: Option<Rc<RefCell<Controller>>>,
}

impl<'a> DataBus<'a> {
    pub fn new() -> Self {
        DataBus {
            cpuMem: vec![0; 0x0800],
            cpu: None,
            ppu: None,
            apu: None,
            cartridge: None,
            controller1: None,
        }
    }

    pub fn attachPpu(&mut self, ppuRef: Rc<RefCell<Ppu<'a>>>) -> () {
        self.ppu = Some(ppuRef);
    }

    pub fn attachCpu(&mut self, cpuRef: Rc<RefCell<Cpu<'a>>>) -> () {
        self.cpu = Some(cpuRef);
    }

    pub fn attachApu(&mut self, apuRef: Rc<RefCell<Apu<'a>>>) {
        self.apu = Some(apuRef);
    }

    pub fn attachCartridge(&mut self, cartRef: Rc<RefCell<Cartridge>>) -> () {
        self.cartridge = Some(cartRef)
    }

    pub fn attachController1(&mut self, con1Ref: Rc<RefCell<Controller>>) -> () {
        self.controller1 = Some(con1Ref);
    }

    #[inline]
    pub fn writeCpuMem(&mut self, ref addr: u16, val: u8) -> () {
        if *addr < 0x2000 {
            self.cpuMem[(*addr & 0x07FF) as usize] = val;
        }
        else if *addr < 0x4000 {
            //info!("Calling register: {} with value {}", *addr & 0007, val);
            self.ppu.as_ref().unwrap().borrow_mut().writeMem(*addr & 0x0007, val);
        }
        else if *addr == 0x4016 {
            self.controller1.as_ref().unwrap().borrow_mut().writeState(val);
        }
        else if (*addr > 0x3FFF && *addr < 0x4014) || *addr == 0x4015 || *addr == 0x4017 {
            self.apu.as_ref().unwrap().borrow_mut().write(*addr, val);
        }
        else {
            self.cartridge.as_ref().unwrap().borrow_mut().cpuWrite(*addr, val);
        }
    }

    #[inline]
    pub fn readCpuMem(&self, ref addr: u16) -> u8 {
        if *addr < 0x2000 {
            return self.cpuMem[(*addr & 0x07FF) as usize].clone();
        }
        else if *addr < 0x4000 {
            return self.ppu.as_ref().unwrap().borrow_mut().readMem(*addr & 0x0007).clone();
        }
        else if *addr == 0x4015 {
            return self.apu.as_ref().unwrap().borrow_mut().read(*addr);
        }
        else if *addr == 0x4016 {
            return self.controller1.as_ref().unwrap().borrow_mut().getState();
        }
        else if *addr == 0x4017 {
            // controller two stuff goes here
            return 0;
        }
        else {
            return self.cartridge.as_ref().unwrap().borrow_mut().cpuRead(*addr);
        }
    }

    #[inline]
    pub fn cpuWriteOam(&mut self, val: u8) -> () {
        self.ppu.as_ref().unwrap().borrow_mut().cpuWriteOam(val);
    }

    #[inline]
    pub fn getControllerInput(&mut self) -> () {
        self.controller1.as_ref().unwrap().borrow_mut().cycle();
    }

    pub fn setControllerEvents(&mut self, events: Vec<Event>) -> () {
        self.controller1.as_ref().unwrap().borrow_mut().setEvents(events);
    }

    pub fn setDmcCpuStall(&mut self) -> () {
        self.cpu.as_ref().unwrap().borrow_mut().setDmcStall();
    }

    pub fn triggerCpuIRQ(&mut self) -> () {
        self.cpu.as_ref().unwrap().borrow_mut().setIrq();
    }

    pub fn ppuTriggerNMI(&mut self) -> () {
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
        };
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