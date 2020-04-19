#![allow(non_snake_case)]
#![allow(warnings)]

pub struct Memory {
    mem: Vec<u8>
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            mem: vec![0; 0x10000],
        }
    }

    pub fn writeMemory(&mut self, ref addr: u16, val: u8) -> () {
        if *addr < 0x2000 {
            let realAddr = *addr - 0x800 * self.findCPUQuadrant(addr) as u16;
            self.mem[realAddr as usize] = val;
            self.mirrorCPU(addr);
        }
        else {
            panic!("Memory address out of bounds: {}", addr)
        }
    }

    pub fn readMemory(&self, ref addr: u16) -> u8 {
//        if *addr < 0x2000 {
//            return self.mem[*addr as usize].clone();
//        }
//        else {
//            panic!("Memory address out of bounds: {}", *addr);
//        }
        return self.mem[*addr as usize].clone();
    }

    pub fn pushStack(&mut self, stackP: &mut u8, val: u8) -> () {
        self.mem[(0x100 + *stackP as u16) as usize] = val;
        *stackP = stackP.wrapping_add(1);
    }

    pub fn popStack(&mut self, stackP: &mut u8) -> u8 {
        *stackP = stackP.wrapping_sub(1);
        let val = self.mem[(0x100 + *stackP as u16) as usize];
        return val;
    }

    fn mirrorCPU(&mut self, addr: &u16) -> () {
        for i in 1..4 {
            self.mem[i * 0x800 + *addr as usize] = self.mem[*addr as usize]
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
        let mem = Memory::new();
        assert_eq!(mem.mem.len(), 0x10000);
    }

    #[test]
    fn writeToMemHappyPath() -> () {
        let mut mem = Memory::new();
        mem.writeMemory(0x0, 0);
        mem.writeMemory(0x1, 1);
        mem.writeMemory(0x2, 2);

        assert_eq!(mem.mem[0x0], 0);
        assert_eq!((mem.mem[0x0] == mem.mem[0x800]), (mem.mem[0x1000] == mem.mem[0x1800]));

        assert_eq!(mem.mem[0x1], 1);
        assert_eq!((mem.mem[0x1] == mem.mem[0x801]), (mem.mem[0x1001] == mem.mem[0x1801]));

        assert_eq!(mem.mem[0x2], 2);
        assert_eq!((mem.mem[0x2] == mem.mem[0x802]), (mem.mem[0x1002] == mem.mem[0x1802]));
    }

    #[test]
    #[should_panic]
    fn writeToMemOutOfBounds() -> () {
        let mut mem = Memory::new();
        mem.writeMemory(0x2001, 1);
    }

    #[test]
    fn pushPopStack() -> () {
        let mut mem = Memory::new();
        let mut stkPointer: &mut u8 = &mut 0x00;

        mem.pushStack(stkPointer, 1);
        assert_eq!(stkPointer, 0x01);
        assert_eq!(mem.mem[(stkPointer - 1) as usize], 1);

        let val = mem.popStack(stkPointer);
        assert_eq!(val, 1);
        assert_eq!(stkPointer, 0x0100);
    }
}