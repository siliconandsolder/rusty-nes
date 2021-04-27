#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use crate::data_bus::*;
use crate::opcode_info::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::u8;
use crate::clock::Clocked;
use std::fmt::{Formatter, Error, Debug};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::any::Any;
use std::fs::File;
use std::io::Write;
use crate::opcode_info::OpCode::BRK;

const CARRY_POS: u8 = 0;
const ZERO_POS: u8 = 1;
const INT_POS: u8 = 2;
const DEC_POS: u8 = 3;
const BRK_POS: u8 = 4;
const U_POS: u8 = 5;
const OVER_POS: u8 = 6;
const NEG_POS: u8 = 7;

struct Flags {
    carry: u8,
    zero: u8,
    interrupt: u8,
    decimal: u8,
    brk: u8,
    unused: u8,
    overflow: u8,
    negative: u8,
}

impl Flags {
    pub fn new() -> Flags {
        Flags {
            carry: 0,
            zero: 0,
            interrupt: 1,
            decimal: 0,
            brk: 0,
            unused: 1,
            overflow: 0,
            negative: 0,
        }
    }
}

const STACK_IDX: u16 = 0x0100;

pub struct Cpu<'a> {
    // registers
    regA: u8,
    regY: u8,
    regX: u8,

    // pointers
    pgmCounter: u16,
    stkPointer: u8,

    memory: Rc<RefCell<DataBus<'a>>>,

    // flag(s)
    flags: Flags,
    // Bit 0 - (C) Carry
    // Bit 1 - (Z) Zero
    // Bit 2 - (I) Interrupt
    // Bit 3 - (D) Decimal (unused in the NES)
    // Bit 4 - (B) Break
    // Bit 5 - N/A
    // Bit 6 - (O) Overflow
    // Bit 7 - (N) Negative

    waitCycles: u16,
    isEvenCycle: bool,

    triggerNmi: bool,
    triggerIrq: bool,

    // OAM transfer variables
    isOamTransfer: bool,
    isOamStarted: bool,
    oamByte: u8,
    oamPage: u16,
    oamCycles: u16,

    // debug
    log: File,
    counter: u16,
}

impl<'a> Clocked for Cpu<'a> {
    #[inline]
    fn cycle(&mut self) {
        self.isEvenCycle = !self.isEvenCycle;

        if self.waitCycles != 0 {
            self.waitCycles -= 1;
            return;
        }

        // interrupts
        if self.triggerIrq {
            self.irq();
            return;
        }
        else if self.triggerNmi {
            self.nmi();
            return;
        }

        if self.isOamTransfer {

            // wait for one cycle if not an even cycle
            if !self.isOamStarted && !self.isEvenCycle {
                self.isOamStarted = true;
                return;
            }

            self.doOamTransfer();

            if self.isOamTransfer {
                return;
            }
        }

        let opInfo = &OPCODE_INSTRUCTIONS[self.readMem8(self.pgmCounter) as usize];
        let (target, bytes, increment, boundaryCrossed) = self.getAddressInfo(opInfo.opCode, opInfo.addrMode, self.pgmCounter.wrapping_add(1));

        // print!("PC: {:04X}, A: {:02X}, X: {:02X}, Y: {:02X}, P: {:02X}, SP: {:02X}, INST: {:?}\n",
        //                            self.pgmCounter, self.regA, self.regX, self.regY, self.getFlagValues(), self.stkPointer, opInfo.opCode);

        // if self.pgmCounter == 0xC66E || self.counter == 8991 {
        //     panic!("DONE!");
        // }

        self.pgmCounter = if increment { self.pgmCounter.wrapping_add(bytes) } else { self.pgmCounter };
        self.executeInstruction(opInfo.opCode, target);

        // we add cycles so that potential interrupt cycles are not erased
        self.waitCycles += (opInfo.cycles as u16) - 1;
        if boundaryCrossed { self.waitCycles += opInfo.xCycles as u16; }

        // debug print
    }
}

impl<'a> Cpu<'a> {
    pub fn new(memory: Rc<RefCell<DataBus>>) -> Cpu {

        // load reset vector into program counter
        let lo = memory.borrow().readCpuMem(0xFFFC);
        let hi = memory.borrow().readCpuMem(0xFFFD);
        let prgC = ((hi as u16) << 8) | (lo as u16);


        let mut cpu = Cpu {
            regA: 0,
            regX: 0,
            regY: 0,
            stkPointer: 0xFD,
            memory: memory,
            pgmCounter: prgC,
            flags: Flags::new(),
            waitCycles: 0,
            triggerIrq: false,
            triggerNmi: false,
            isOamTransfer: false,
            isOamStarted: false,
            oamByte: 0,
            oamPage: 0,
            oamCycles: 0,
            log: File::create("log.txt").unwrap(),
            counter: 0,
            isEvenCycle: false,
        };

        cpu.setFlags(0x24);
        return cpu;
    }

    pub fn reset(&mut self) -> () {
        self.stkPointer = self.stkPointer.wrapping_sub(3);
        self.setFlags(0x24);
        self.pgmCounter = self.readMem16(0xFFFC);
    }

    #[inline]
    fn executeInstruction(&mut self, opCode: OpMnemonic, target: Option<u16>) -> () {
        match opCode {
            OpMnemonic::ADC => { self.adc(target) }
            OpMnemonic::AHX => {}
            OpMnemonic::ANC => { self.anc(target) }
            OpMnemonic::AND => { self.and(target) }
            OpMnemonic::ALR => { self.alr(target) }
            OpMnemonic::ARR => { self.arr(target) }
            OpMnemonic::ASL => { self.asl(target) }
            OpMnemonic::AXS => { self.axs(target) }
            OpMnemonic::BCC => { self.bcc(target) }
            OpMnemonic::BCS => { self.bcs(target) }
            OpMnemonic::BEQ => { self.beq(target) }
            OpMnemonic::BIT => { self.bit(target) }
            OpMnemonic::BMI => { self.bmi(target) }
            OpMnemonic::BNE => { self.bne(target) }
            OpMnemonic::BPL => { self.bpl(target) }
            OpMnemonic::BRK => { self.brk(target) }
            OpMnemonic::BVC => { self.bvc(target) }
            OpMnemonic::BVS => { self.bvs(target) }
            OpMnemonic::CLC => { self.clc(target) }
            OpMnemonic::CLD => { self.cld(target) }
            OpMnemonic::CLI => { self.cli(target) }
            OpMnemonic::CLV => { self.clv(target) }
            OpMnemonic::CMP => { self.cmp(target) }
            OpMnemonic::CPX => { self.cpx(target) }
            OpMnemonic::CPY => { self.cpy(target) }
            OpMnemonic::DCP => { self.dcp(target) }
            OpMnemonic::DEC => { self.dec(target) }
            OpMnemonic::DEX => { self.dex(target) }
            OpMnemonic::DEY => { self.dey(target) }
            OpMnemonic::EOR => { self.eor(target) }
            OpMnemonic::INC => { self.inc(target) }
            OpMnemonic::INX => { self.inx(target) }
            OpMnemonic::INY => { self.iny(target) }
            OpMnemonic::ISC => { self.isc(target) }
            OpMnemonic::JMP => { self.jmp(target) }
            OpMnemonic::JSR => { self.jsr(target) }
            OpMnemonic::KIL => {}
            OpMnemonic::LAS => {}
            OpMnemonic::LAX => { self.lax(target) }
            OpMnemonic::LDA => { self.lda(target) }
            OpMnemonic::LDX => { self.ldx(target) }
            OpMnemonic::LDY => { self.ldy(target) }
            OpMnemonic::LSR => { self.lsr(target) }
            OpMnemonic::NOP => { self.nop(target) }
            OpMnemonic::ORA => { self.ora(target) }
            OpMnemonic::PHA => { self.pha(target) }
            OpMnemonic::PHP => { self.php(target) }
            OpMnemonic::PLA => { self.pla(target) }
            OpMnemonic::PLP => { self.plp(target) }
            OpMnemonic::RLA => { self.rla(target) }
            OpMnemonic::ROL => { self.rol(target) }
            OpMnemonic::ROR => { self.ror(target) }
            OpMnemonic::RRA => { self.rra(target) }
            OpMnemonic::RTI => { self.rti(target) }
            OpMnemonic::RTS => { self.rts(target) }
            OpMnemonic::SAX => { self.sax(target) }
            OpMnemonic::SBC => { self.sbc(target) }
            OpMnemonic::SEC => { self.sec(target) }
            OpMnemonic::SED => { self.sed(target) }
            OpMnemonic::SEI => { self.sei(target) }
            OpMnemonic::SHY => { self.shy(target) }
            OpMnemonic::SHX => { self.shx(target) }
            OpMnemonic::SLO => { self.slo(target) }
            OpMnemonic::SRE => { self.sre(target) }
            OpMnemonic::STA => { self.sta(target) }
            OpMnemonic::STX => { self.stx(target) }
            OpMnemonic::STY => { self.sty(target) }
            OpMnemonic::TAS => {}
            OpMnemonic::TAX => { self.tax(target) }
            OpMnemonic::TAY => { self.tay(target) }
            OpMnemonic::TSX => { self.tsx(target) }
            OpMnemonic::TXA => { self.txa(target) }
            OpMnemonic::TXS => { self.txs(target) }
            OpMnemonic::TYA => { self.tya(target) }
            OpMnemonic::XAA => {}
        };
    }

    pub fn setNmi(&mut self) -> () {
        self.triggerNmi = true;
    }

    pub fn setIrq(&mut self) -> () {
        if self.flags.interrupt == 0 {
            self.triggerIrq = true;
        }
    }

    pub fn setDmcStall(&mut self) -> () {
        self.waitCycles += 4;
    }

    pub fn triggerOamTransfer(&mut self, pageAddr: u16) -> () {
        self.isOamTransfer = true;
        self.oamPage = pageAddr;
    }

    fn doOamTransfer(&mut self) -> () {

        // read and write on alternating cycles
        if self.oamCycles % 2 == 0 {
            self.oamByte = self.readMem8(*&self.oamPage);
        }
        else {
            self.memory.borrow_mut().cpuWriteOam(*&self.oamByte);
            self.oamPage = self.oamPage.wrapping_add(1);

            // we've stepped into the next page of memory
            if self.oamPage & 0x00FF == 0 {
                self.resetOamState();
                return;
            }
        }

        self.oamCycles += 1;
    }

    fn resetOamState(&mut self) -> () {
        self.isOamTransfer = false;
        self.isOamStarted = false;
        self.oamByte = 0;
        self.oamPage = 0;
        self.oamCycles = 0;
    }

    fn nmi(&mut self) -> () {
        let hi = (self.pgmCounter >> 8) as u8;
        let lo = (self.pgmCounter & 0x00FF) as u8;
        self.pushStack(hi);
        self.pushStack(lo);
        self.php(None);
        self.sei(None);

        self.pgmCounter = self.readMem16(0xFFFA);
        self.waitCycles = 7;
        self.triggerNmi = false;
    }

    fn irq(&mut self) -> () {
        let hi = (self.pgmCounter >> 8) as u8;
        let lo = (self.pgmCounter & 0x00FF) as u8;
        self.pushStack(hi);
        self.pushStack(lo);
        self.php(None);
        self.sei(None);

        self.pgmCounter = self.readMem16(0xFFFE);
        self.waitCycles = 7;
        self.triggerIrq = false;
    }

    fn adc(&mut self, target: Option<u16>) -> () {
        let oldVal = self.readMem8(target.unwrap()) as u16;
        let newVal = oldVal.wrapping_add(self.regA as u16).wrapping_add(self.flags.carry as u16);

        self.flags.carry = if newVal > 0xFF { 1 } else { 0 };

        // !((M^N) & 0x80) && ((M^result) & 0x80)
        // if the inputs have the same sign, and the input and result have different signs
        if ((self.regA as u16 ^ oldVal) & 0x80) == 0 && ((self.regA as u16 ^ newVal) & 0x80) != 0 {
            self.flags.overflow = 1;
        }
        else {
            self.flags.overflow = 0;
        }

        // set negative and zero flag
        self.setZNFlag(newVal as u8);
        self.regA = newVal as u8;
    }

    fn anc(&mut self, target: Option<u16>) -> () {
        self.and(target);
        self.flags.carry = self.flags.negative;
    }

    fn and(&mut self, target: Option<u16>) -> () {
        self.regA &= self.readMem8(target.unwrap());
        self.setZNFlag(self.regA);
    }

    fn alr(&mut self, target: Option<u16>) -> () {
        self.regA &= self.readMem8(target.unwrap());
        self.flags.carry = if self.regA & 1 == 1 { 1 } else { 0 };
        self.regA >>= 1;
        self.setZNFlag(self.regA);
    }

    fn arr(&mut self, target: Option<u16>) -> () {
        self.and(target);
        self.ror(None);

        let bitFive = (self.regA & 0x20) >> 5;
        let bitSix = (self.regA & 0x40) >> 6;

        self.flags.overflow = bitSix ^ bitFive;
        self.flags.carry = bitSix;
    }

    fn asl(&mut self, target: Option<u16>) -> () {
        if target == None {

            // set the carry flag to Register A's MSB
            self.flags.carry = (self.regA >> 7) & 1;
            self.regA <<= 1;
            self.setZNFlag(self.regA);
        }
        else {
            let mut val = self.readMem8(target.unwrap());

            // set the carry flag to the value's MSB
            self.flags.carry = (val >> 7) & 1;
            val <<= 1;
            self.setZNFlag(val);
            self.writeMem8(target.unwrap(), val);
        }
    }

    // adc, alr

    fn axs(&mut self, target: Option<u16>) -> () {
        let val = self.readMem8(target.unwrap());
        let result = (self.regA & self.regX).wrapping_sub(val);

        self.flags.carry = 0;
        if (self.regA & self.regX) >= val {
            self.flags.carry = 1;
        }

        self.regX = result;
        self.setZNFlag(self.regX);
    }

    fn bcc(&mut self, target: Option<u16>) -> () {
        if self.flags.carry == 0 {
            self.pgmCounter = target.unwrap();
        }
    }

    fn bcs(&mut self, target: Option<u16>) -> () {
        if self.flags.carry != 0 {
            self.pgmCounter = target.unwrap();
        }
    }

    fn beq(&mut self, target: Option<u16>) -> () {
        if self.flags.zero != 0 {
            self.pgmCounter = target.unwrap();
        }
    }

    fn bit(&mut self, target: Option<u16>) -> () {
        let val = self.readMem8(target.unwrap());

        self.flags.zero = if self.regA & val == 0 { 1 } else { 0 };
        self.flags.overflow = (val >> 6) & 1;
        self.flags.negative = (val >> 7) & 1;
    }


    fn bmi(&mut self, target: Option<u16>) -> () {
        if self.flags.negative != 0 {
            self.pgmCounter = target.unwrap();
        }
    }

    fn bne(&mut self, target: Option<u16>) -> () {
        if self.flags.zero == 0 {
            self.pgmCounter = target.unwrap();
        }
    }

    fn bpl(&mut self, target: Option<u16>) -> () {
        if self.flags.negative == 0 {
            self.pgmCounter = target.unwrap();
        }
    }

    fn brk(&mut self, target: Option<u16>) -> () {
        self.pgmCounter += 1;   // on a 6502, BRK advances the program counter by one
        let hi = (self.pgmCounter >> 8) as u8;
        let lo = (self.pgmCounter & 0x00FF) as u8;
        self.pushStack(hi);
        self.pushStack(lo);

        self.flags.brk = 1;
        self.php(None);
        self.sei(None);

        self.pgmCounter = self.readMem16(0xFFFE);
    }

    fn bvc(&mut self, target: Option<u16>) -> () {
        if self.flags.overflow == 0 {
            self.pgmCounter = target.unwrap();
        }
    }

    fn bvs(&mut self, target: Option<u16>) -> () {
        if self.flags.overflow != 0 {
            self.pgmCounter = target.unwrap();
        }
    }

    fn clc(&mut self, target: Option<u16>) -> () {
        self.flags.carry = 0;
    }

    fn cld(&mut self, target: Option<u16>) -> () {
        self.flags.decimal = 0;
    }

    fn cli(&mut self, target: Option<u16>) -> () {
        self.flags.interrupt = 0;
    }

    fn clv(&mut self, target: Option<u16>) -> () {
        self.flags.overflow = 0;
    }

    fn cmp(&mut self, target: Option<u16>) -> () {
        let val = self.readMem8(target.unwrap());
        if self.regA >= val {
            self.flags.carry = 1;
        }
        else {
            self.flags.carry = 0;
        }

        self.setZNFlag(self.regA.wrapping_sub(val));
    }

    fn cpx(&mut self, target: Option<u16>) -> () {
        let val = self.readMem8(target.unwrap());
        if self.regX >= val {
            self.flags.carry = 1;
        }
        else {
            self.flags.carry = 0;
        }

        self.setZNFlag(self.regX.wrapping_sub(val));
    }

    fn cpy(&mut self, target: Option<u16>) -> () {
        let val = self.readMem8(target.unwrap());
        if self.regY >= val {
            self.flags.carry = 1;
        }
        else {
            self.flags.carry = 0;
        }

        self.setZNFlag(self.regY.wrapping_sub(val));
    }

    fn dcp(&mut self, target: Option<u16>) -> () {
        self.dec(target.clone());
        self.cmp(target.clone());
    }

    fn dec(&mut self, target: Option<u16>) -> () {
        let val = self.readMem8(target.unwrap()).wrapping_sub(1);
        self.writeMem8(target.unwrap(), val.clone());

        self.setZNFlag(val);
    }

    fn dex(&mut self, target: Option<u16>) -> () {
        self.regX = self.regX.wrapping_sub(1);

        self.setZNFlag(self.regX);
    }

    fn dey(&mut self, target: Option<u16>) -> () {
        self.regY = self.regY.wrapping_sub(1);

        self.setZNFlag(self.regY);
    }

    fn eor(&mut self, target: Option<u16>) -> () {
        let address = target.unwrap();
        self.regA ^= self.readMem8(address);
        self.setZNFlag(self.regA);
    }

    fn inc(&mut self, target: Option<u16>) -> () {
        let val = self.readMem8(target.unwrap()).wrapping_add(1);
        self.setZNFlag(val);
        self.writeMem8(target.unwrap(), val);
    }

    fn inx(&mut self, target: Option<u16>) -> () {
        self.regX = self.regX.wrapping_add(1);
        self.setZNFlag(self.regX);
    }

    fn iny(&mut self, target: Option<u16>) -> () {
        self.regY = self.regY.wrapping_add(1);
        self.setZNFlag(self.regY);
    }

    fn isc(&mut self, target: Option<u16>) -> () {
        self.inc(target.clone());
        self.sbc(target.clone());
    }

    fn jmp(&mut self, target: Option<u16>) -> () {
        self.pgmCounter = target.unwrap();
    }

    fn jsr(&mut self, target: Option<u16>) -> () {
        self.pgmCounter = self.pgmCounter.wrapping_sub(1);

        let hi = (self.pgmCounter >> 8) as u8;
        let lo = (self.pgmCounter & 0x00FF) as u8;
        self.pushStack(hi);
        self.pushStack(lo);

        self.pgmCounter = target.unwrap();
    }

    fn lax(&mut self, target: Option<u16>) -> () {
        self.lda(target.clone());
        self.ldx(target.clone());
    }

    fn lda(&mut self, target: Option<u16>) -> () {
        let addr = target.unwrap();
        self.regA = self.readMem8(addr);
        self.setZNFlag(self.regA);
    }


    fn ldx(&mut self, target: Option<u16>) -> () {
        let addr = target.unwrap();
        self.regX = self.readMem8(addr);
        self.setZNFlag(self.regX);
    }

    fn ldy(&mut self, target: Option<u16>) -> () {
        self.regY = self.readMem8(target.unwrap());
        self.setZNFlag(self.regY);
    }

    fn lsr(&mut self, target: Option<u16>) -> () {
        if target == None {

            // set the carry flag to Register A's LSB
            self.flags.carry = self.regA & 1;

            self.regA >>= 1;
            self.setZNFlag(self.regA);
        }
        else {
            let mut val = self.readMem8(target.unwrap());

            // set the carry flag to the value's LSB
            self.flags.carry = val & 1;

            val >>= 1;
            self.setZNFlag(val);
            self.writeMem8(target.unwrap(), val);
        }
    }

    fn nop(&mut self, target: Option<u16>) -> () {
        // crickets...
    }

    fn ora(&mut self, target: Option<u16>) -> () {
        self.regA |= self.readMem8(target.unwrap());
        self.setZNFlag(self.regA);
    }

    fn pha(&mut self, target: Option<u16>) -> () {
        self.pushStack(self.regA);
    }

    fn php(&mut self, target: Option<u16>) -> () {
        self.pushStack(self.getFlagValues());
    }

    fn pla(&mut self, target: Option<u16>) -> () {
        self.regA = self.popStack();
        self.setZNFlag(self.regA);
    }

    fn plp(&mut self, target: Option<u16>) -> () {
        let status = self.popStack();
        self.setFlags(status);
    }

    fn rla(&mut self, target: Option<u16>) -> () {
        self.rol(target.clone());
        self.and(target.clone());
    }

    fn rol(&mut self, target: Option<u16>) -> () {
        let oldCarry = self.flags.carry;
        if target == None {
            self.flags.carry = (self.regA >> 7) & 1;
            self.regA <<= 1;
            self.regA |= oldCarry;
            self.setZNFlag(self.regA);
        }
        else {
            let mut val = self.readMem8(target.unwrap());
            self.flags.carry = (val >> 7) & 1;
            val <<= 1;
            val |= oldCarry;
            self.setZNFlag(val);
            self.writeMem8(target.unwrap(), val);
        }
    }

    // failed on ROR at address CF51, caused BEQ to execute when not supposed to
    fn ror(&mut self, target: Option<u16>) -> () {
        let oldCarry = self.flags.carry;
        if target == None {
            self.flags.carry = self.regA & 1;
            self.regA >>= 1;
            self.regA |= (oldCarry << 7);
            self.setZNFlag(self.regA);
        }
        else {
            let mut val = self.readMem8(target.unwrap());
            self.flags.carry = val & 1;
            val >>= 1;
            val |= (oldCarry << 7);
            self.setZNFlag(val);
            self.writeMem8(target.unwrap(), val);
        }
    }

    fn rra(&mut self, target: Option<u16>) -> () {
        self.ror(target.clone());
        self.adc(target.clone());
    }

    // RTI
    fn rti(&mut self, target: Option<u16>) -> () {
        let status = self.popStack();
        self.setFlags(status);
        let lo = self.popStack();
        let hi = self.popStack();
        self.pgmCounter = ((hi as u16) << 8) | lo as u16;
    }
    // RTS
    fn rts(&mut self, target: Option<u16>) -> () {
        let lo = self.popStack();
        let hi = self.popStack();
        self.pgmCounter = ((hi as u16) << 8) | lo as u16;
        self.pgmCounter = self.pgmCounter.wrapping_add(1);
    }

    fn sax(&mut self, target: Option<u16>) -> () {
        self.writeMem8(target.unwrap(), self.regA & self.regX);
    }

    fn sbc(&mut self, target: Option<u16>) -> () {
        let oldVal = self.readMem8(target.unwrap());
        let newVal = self.regA.wrapping_sub(oldVal).wrapping_sub(1 - self.flags.carry);

        let newValInt = self.regA as i32 - oldVal as i32 - (1 - self.flags.carry as i32);
        self.flags.carry = if newValInt >= 0 { 1 } else { 0 };

        // !((M^N) & 0x80) && ((M^result) & 0x80)
        // if the inputs have the same sign, and the input and result have different signs
        if ((self.regA ^ oldVal) & 0x80) != 0 && ((self.regA ^ newVal) & 0x80) != 0 {
            self.flags.overflow = 1;
        }
        else {
            self.flags.overflow = 0;
        }

        // set negative and zero flag
        self.setZNFlag(newVal);
        self.regA = newVal;
    }

    fn sec(&mut self, target: Option<u16>) -> () {
        self.flags.carry = 1;
    }

    fn sed(&mut self, target: Option<u16>) -> () {
        self.flags.decimal = 1;
    }

    fn sei(&mut self, target: Option<u16>) -> () {
        self.flags.interrupt = 1;
    }

    fn shy(&mut self, target: Option<u16>) -> () {
        let address = self.readMem16(target.unwrap());
        self.writeMem8(address, (self.regY & ((address >> 8).wrapping_add(1)) as u8));
    }

    fn shx(&mut self, target: Option<u16>) -> () {
        let address = self.readMem16(target.unwrap());
        self.writeMem8(address, (self.regX & ((address >> 8).wrapping_add(1)) as u8));
    }

    fn slo(&mut self, target: Option<u16>) -> () {
        self.asl(target.clone());
        self.ora(target.clone());
    }

    fn sre(&mut self, target: Option<u16>) -> () {
        self.lsr(target.clone());
        self.eor(target.clone());
    }

    fn sta(&mut self, target: Option<u16>) -> () {
        let addr = target.unwrap();
        self.writeMem8(addr, self.regA.clone());
    }

    fn stx(&mut self, target: Option<u16>) -> () {
        self.writeMem8(target.unwrap(), self.regX.clone());
    }

    fn sty(&mut self, target: Option<u16>) -> () {
        self.writeMem8(target.unwrap(), self.regY.clone());
    }

    fn tax(&mut self, target: Option<u16>) -> () {
        self.regX = self.regA;
        self.setZNFlag(self.regX);
    }

    fn tay(&mut self, target: Option<u16>) -> () {
        self.regY = self.regA;
        self.setZNFlag(self.regY);
    }

    fn tsx(&mut self, target: Option<u16>) -> () {
        self.regX = self.stkPointer;
        self.setZNFlag(self.regX);
    }

    fn txa(&mut self, target: Option<u16>) -> () {
        self.regA = self.regX;
        self.setZNFlag(self.regA);
    }

    fn txs(&mut self, target: Option<u16>) -> () {
        self.stkPointer = self.regX;
    }

    fn tya(&mut self, target: Option<u16>) -> () {
        self.regA = self.regY;
        self.setZNFlag(self.regA);
    }

    #[inline]
    fn getAddressInfo(&mut self, ref opCode: OpMnemonic, ref addrMode: AddressMode, oper: u16) -> (Option<u16>, u16, bool, bool) {
        // target address (option)
        // bytes to increment
        // PC should increment
        // page boundary crossed
        match addrMode {
            AddressMode::Accumulator => {
                return (None, 1, true, false);
            }
            AddressMode::Absolute => {
                let target = self.readMem16(oper);

                return (Some(target), 3, self.pcShouldIncrement(*opCode), false);
            }
            AddressMode::AbsoluteX => {
                let orgTarget: u16 = self.readMem16(oper);
                let newTarget: u16 = orgTarget.wrapping_add(self.regX as u16);

                return (Some(newTarget), 3, self.pcShouldIncrement(*opCode), newTarget & 0xFF00 != orgTarget);
            }
            AddressMode::AbsoluteY => {
                let orgTarget: u16 = self.readMem16(oper);
                let newTarget: u16 = orgTarget.wrapping_add(self.regY as u16);

                return (Some(newTarget), 3, self.pcShouldIncrement(*opCode), newTarget & 0xFF00 != orgTarget);
            }
            AddressMode::Immediate => {
                return (Some(oper), 2, self.pcShouldIncrement(*opCode), false);
            }
            AddressMode::Implied => {
                return (None, 1, true, false);
            }
            AddressMode::Indirect => {
                // only the JMP instruction uses this addressing mode
                let orgAddr = self.readMem16(oper);
                let lo = self.readMem8(orgAddr);
                let hi = self.readMem8(
                    if (orgAddr.wrapping_add(1) & 0x00FF) == 0 { orgAddr & 0xFF00 } else { orgAddr.wrapping_add(1) }
                );
                let target = ((hi as u16) << 8) | (lo as u16);

                return (Some(target), 3, false, false);
            }
            AddressMode::IndirectIndexed => {
                let zpgAddr = self.readMem8(oper);
                let mut storedAddr: u16 = 0x0000;

                if zpgAddr == 0xFF {
                    storedAddr = ((self.readMem8(0x00) as u16) << 8) | self.readMem8(0xFF) as u16;
                }
                else {
                    storedAddr = self.readMem16(zpgAddr as u16);
                }
                let target = storedAddr.wrapping_add(self.regY as u16);

                return (Some(target), 2, self.pcShouldIncrement(*opCode), (storedAddr & 0x00FF) > (target & 0x00FF));
            }
            AddressMode::IndexedIndirect => {
                let zpgAddr = self.readMem8(oper).wrapping_add(self.regX);
                let mut storedAddr: u16 = 0x0000;

                if zpgAddr == 0xFF {
                    storedAddr = ((self.readMem8(0x00) as u16) << 8) | self.readMem8(0xFF) as u16;
                }
                else {
                    storedAddr = self.readMem16(zpgAddr as u16);
                }

                return (Some(storedAddr), 2, self.pcShouldIncrement(*opCode), false);
            }
            AddressMode::Relative => {
                // this addressing mode is only for branching instructions
                let mut jumpOffset = self.readMem8(oper);
                let mut target: u16;

                target = self.pgmCounter.wrapping_add(jumpOffset as u16);
                target = target.wrapping_add(2); // account for opcode and operand byte

                // subtract 256 if offset is supposed to be negative
                if jumpOffset > 0x7F {
                    target = target.wrapping_sub(0x100);
                }

                let branching = self.branchIncrement(*opCode);

                if branching {
                    self.waitCycles += 1;
                }

                return (Some(target), 2, !branching, target & 0xFF00 != (oper.wrapping_sub(1)) & 0xFF00);
            }
            AddressMode::ZeroPage => {
                let target = self.readMem8(oper) as u16;
                return (Some(target), 2, self.pcShouldIncrement(*opCode), false);
            }
            AddressMode::ZeroPageX => {
                let addr = self.readMem8(oper);
                let target = addr.wrapping_add(self.regX) as u16;
                return (Some(target), 2, self.pcShouldIncrement(*opCode), target < addr as u16);
            }
            AddressMode::ZeroPageY => {
                let addr = self.readMem8(oper);
                let target = addr.wrapping_add(self.regY) as u16;
                return (Some(target), 2, self.pcShouldIncrement(*opCode), target < addr as u16);
            }
            _ => {
                panic!("How did you find an unused address mode?!");
            }
        }
    }

    #[inline]
    fn branchIncrement(&self, ref opCode: OpMnemonic) -> bool {
        match *opCode {
            OpMnemonic::BCC => { self.flags.carry == 0 }
            OpMnemonic::BCS => { self.flags.carry == 1 }
            OpMnemonic::BEQ => { self.flags.zero == 1 }
            OpMnemonic::BNE => { self.flags.zero == 0 }
            OpMnemonic::BMI => { self.flags.negative == 1 }
            OpMnemonic::BPL => { self.flags.negative == 0 }
            OpMnemonic::BVS => { self.flags.overflow == 1 }
            OpMnemonic::BVC => { self.flags.overflow == 0 }
            _ => false
        }
    }

    #[inline]
    fn readMem16(&self, ref addr: u16) -> u16 {
        let lo = self.memory.borrow().readCpuMem(*addr);
        let hi = self.memory.borrow().readCpuMem((*addr + 1));
        return (hi as u16) << 8 | lo as u16;
    }

    #[inline]
    fn readMem8(&self, ref addr: u16) -> u8 {
        return self.memory.borrow().readCpuMem(*addr);
    }

    #[inline]
    fn writeMem8(&mut self, ref addr: u16, value: u8) -> () {
        // have to OAM DMA transfer here to prevent violation of borrowing rules
        // TODO: FIX THIS
        match *addr {
            0x4014 => { self.triggerOamTransfer((value as u16) << 8); }
            _ => { self.memory.borrow_mut().writeCpuMem(*addr, value); }
        }
    }

    #[inline]
    fn writeMem16(&self, ref addr: u16, value: u16) -> () {
        let lo = (value & 0x00FF) as u8;
        let hi = (value >> 8) as u8;
        self.memory.borrow_mut().writeCpuMem(*addr, lo);
        self.memory.borrow_mut().writeCpuMem(addr.wrapping_add(1), hi);
    }

    #[inline]
    fn pushStack(&mut self, ref value: u8) -> () {
        self.writeMem8(STACK_IDX | (self.stkPointer as u16), value.clone());
        self.stkPointer = self.stkPointer.wrapping_sub(1);
    }

    #[inline]
    fn popStack(&mut self) -> u8 {
        self.stkPointer = self.stkPointer.wrapping_add(1);
        return self.readMem8(STACK_IDX | (self.stkPointer as u16));
    }

    #[inline]
    fn pcShouldIncrement(&self, ref opCode: OpMnemonic) -> bool {
        match *opCode {
            OpMnemonic::JMP | OpMnemonic::RTS => return false,
            _ => return true
        }
    }

    #[inline]
    fn setZNFlag(&mut self, ref result: u8) -> () {
        self.setNFlag(*result);
        self.setZFlag(*result);
    }

    #[inline]
    fn setZFlag(&mut self, ref result: u8) -> () {
        if *result == 0 {
            self.flags.zero = 1;
        }
        else {
            self.flags.zero = 0;
        }
    }

    #[inline]
    fn setNFlag(&mut self, ref result: u8) -> () {
        if *result & 0x80 != 0 {
            self.flags.negative = 1;
        }
        else {
            self.flags.negative = 0;
        }
    }

    #[inline]
    fn setFlags(&mut self, status: u8) -> () {
        self.flags.carry = (status >> CARRY_POS) & 1;
        self.flags.zero = (status >> ZERO_POS) & 1;
        self.flags.interrupt = (status >> INT_POS) & 1;
        self.flags.decimal = (status >> DEC_POS) & 1;
        self.flags.brk = (status >> BRK_POS) & 1;
        self.flags.unused = 1;
        self.flags.overflow = (status >> OVER_POS) & 1;
        self.flags.negative = (status >> NEG_POS) & 1;
    }

    #[inline]
    fn getFlagValues(&self) -> u8 {
        let mut status: u8 = 0;
        status |= ((self.flags.carry & 1) << CARRY_POS);
        status |= ((self.flags.zero & 1) << ZERO_POS);
        status |= ((self.flags.interrupt & 1) << INT_POS);
        status |= ((self.flags.decimal & 1) << DEC_POS);
        status |= ((self.flags.brk & 1) << BRK_POS);
        status |= (1 << U_POS);
        status |= ((self.flags.overflow & 1) << OVER_POS);
        status |= ((self.flags.negative & 1) << NEG_POS);
        return status;
    }
}

#[cfg(test)]
mod CpuSpc {
    use super::*;

    fn getNewCpu() -> Cpu {
        Cpu::new(Rc::new(RefCell::new(DataBus::new())))
    }

    #[test]
    fn getOpcodeInfoHappyPath() {
        let cpu = getNewCpu();
        let (mnemonic, addrType, cpuCycles, extraCycles, bytes) = cpu.getOpCodeInfo(OpCode::INC_ABS_X);
        assert_eq!(mnemonic, OpMnemonic::INC);
        assert_eq!(addrType, ABS_X);
        assert_eq!(cpuCycles, 7);
        assert_eq!(extraCycles, 0);
        assert_eq!(bytes, 3);
    }

    #[test]
    #[should_panic]
    fn getOpcodeInfoShouldFail() {
        let cpu = getNewCpu();
        cpu.getOpCodeInfo(OpCode::NOP_TEST);
    }

    #[test]
    fn getAddressInfoAccumulator() {
        let mut cpu = getNewCpu();
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, ACC, 0);
        assert_eq!(target, None);
        assert_eq!(bytes, 1);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, false);
    }

    #[test]
    fn getAddressInfoAbsolute() {
        let mut cpu = getNewCpu();
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, ABS, 0);
        assert_eq!(target, Some(0));
        assert_eq!(bytes, 3);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, false);
    }

    #[test]
    fn getAddressInfoAbsoluteX() {
        let mut cpu = getNewCpu();
        cpu.memory.borrow_mut().writeCpuMem(0, 0xFF);
        cpu.regX = 1;
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, ABS_X, 0);
        assert_eq!(target, Some(0x0100));
        assert_eq!(bytes, 3);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, true);
    }

    #[test]
    fn getAddressInfoAbsoluteY() {
        let mut cpu = getNewCpu();
        cpu.memory.borrow_mut().writeCpuMem(0, 0xFF);
        cpu.regY = 1;
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, ABS_Y, 0);
        assert_eq!(target, Some(0x0100));
        assert_eq!(bytes, 3);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, true);
    }

    #[test]
    fn getAddressInfoImmediate() {
        let mut cpu = getNewCpu();
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, IMT, 0);
        assert_eq!(target, Some(0));
        assert_eq!(bytes, 2);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, false);
    }

    #[test]
    fn getAddressInfoImplied() {
        let mut cpu = getNewCpu();
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, IMP, 0);
        assert_eq!(target, None);
        assert_eq!(bytes, 1);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, false);
    }

    #[test]
    fn getAddressInfoIndirectIndexed() {
        let mut cpu = getNewCpu();
        cpu.memory.borrow_mut().writeCpuMem(0, 0xFF);
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, IND_Y, 0);
        assert_eq!(target, Some(255));
        assert_eq!(bytes, 2);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, true);
    }

    #[test]
    fn getAddressInfoIndexedIndirect() {
        let mut cpu = getNewCpu();
        cpu.regX = 1;
        cpu.memory.borrow_mut().writeCpuMem(0, 0xFE);
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, IND_X, 0);
        assert_eq!(target, Some(255));
        assert_eq!(bytes, 2);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, true);
    }

    #[test]
    fn getAddressInfoRelative() {
        let mut cpu = getNewCpu();
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, REL, 0);
        assert_eq!(target, Some(0));
        assert_eq!(bytes, 2);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, false);
    }

    #[test]
    fn getAddressInfoZeroPage() {
        let mut cpu = getNewCpu();
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, ZPG, 0);
        assert_eq!(target, Some(0));
        assert_eq!(bytes, 2);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, false);
    }

    #[test]
    fn getAddressInfoZeroPageX() {
        let mut cpu = getNewCpu();
        cpu.regX = 1;
        cpu.memory.borrow_mut().writeCpuMem(0, 0xFF);
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, ZPG_X, 0);
        assert_eq!(target, Some(0));
        assert_eq!(bytes, 2);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, true);
    }

    #[test]
    fn getAddressInfoZeroPageY() {
        let mut cpu = getNewCpu();
        cpu.regY = 1;
        cpu.memory.borrow_mut().writeCpuMem(0, 0xFF);
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, ZPG_Y, 0);
        assert_eq!(target, Some(0));
        assert_eq!(bytes, 2);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, true);
    }

    #[test]
    fn readMem16Spec() {
        let mut cpu = getNewCpu();
        cpu.memory.borrow_mut().writeCpuMem(0, 0x01);
        cpu.memory.borrow_mut().writeCpuMem(1, 0x01);
        let result = cpu.readMem16(0);
        assert_eq!(result, 257)
    }

    #[test]
    fn setZFlagOnSpec() {
        let mut cpu = getNewCpu();
        cpu.setZFlag(0);
        assert_eq!(cpu.flags.zero, 1)
    }

    #[test]
    fn setZFlagOffSpec() {
        let mut cpu = getNewCpu();
        cpu.setZFlag(1);
        assert_eq!(cpu.flags.zero, 0)
    }

    #[test]
    fn setNFlagOnSpec() {
        let mut cpu = getNewCpu();
        cpu.setNFlag(0x80);
        assert_eq!(cpu.flags.negative, 1);
    }

    #[test]
    fn setNFlagOffSpec() {
        let mut cpu = getNewCpu();
        cpu.setNFlag(0x7F);
        assert_eq!(cpu.flags.negative, 0)
    }
}

