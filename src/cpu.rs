#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use crate::data_bus::*;
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

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum AddressMode {
    Accumulator,
    Implied,
    Immediate,
    Absolute,
    ZeroPage,
    Indirect,
    Relative,
    AbsoluteX,
    AbsoluteY,
    ZeroPageX,
    ZeroPageY,
    IndexedIndirect,
    IndirectIndexed
}

const ACC: AddressMode = AddressMode::Accumulator;
const IMP: AddressMode = AddressMode::Implied;
const IMT: AddressMode = AddressMode::Immediate;
const ABS: AddressMode = AddressMode::Absolute;
const ZPG: AddressMode = AddressMode::ZeroPage;
const IND: AddressMode = AddressMode::Indirect;
const REL: AddressMode = AddressMode::Relative;
const ABS_X: AddressMode = AddressMode::AbsoluteX;
const ABS_Y: AddressMode = AddressMode::AbsoluteY;
const ZPG_X: AddressMode = AddressMode::ZeroPageX;
const ZPG_Y: AddressMode = AddressMode::ZeroPageY;
const IND_X: AddressMode = AddressMode::IndexedIndirect;
const IND_Y: AddressMode = AddressMode::IndirectIndexed;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum OpCode {
    BRK         = 0x00,
    ORA_IND_X   = 0x01,
    SLO_IND_X   = 0x03,
	NOP_ZPG_1   = 0x04,
    ORA_ZPG     = 0x05,
    ASL_ZPG     = 0x06,
    SLO_ZPG     = 0x07,
    PHP         = 0x08,
    ORA_IMT     = 0x09,
    ASL_ACC     = 0x0A,
    NOP_ABS_1   = 0x0C,
    ORA_ABS     = 0x0D,
    ASL_ABS     = 0x0E,
    SLO_ABS     = 0x0F,

    BPL_REL     = 0x10,
    ORA_IND_Y   = 0x11,
    SLO_IND_Y   = 0x13,
    NOP_ZPG_X_1 = 0x14,
    ORA_ZPG_X   = 0x15,
    ASL_ZPG_X   = 0x16,
    SLO_ZPG_X   = 0x17,
    CLC         = 0x18,
    ORA_ABS_Y   = 0x19,
    NOP_1       = 0x1A,
    SLO_ABS_Y   = 0x1B,
    NOP_ABS_X_1 = 0x1C,
    ORA_ABS_X   = 0x1D,
    ASL_ABS_X   = 0x1E,
    SLO_ABS_X   = 0x1F,

    JSR_ABS     = 0x20,
    AND_IND_X   = 0x21,
    RLA_IND_X   = 0x23,
    BIT_ZPG     = 0x24,
    AND_ZPG     = 0x25,
    ROL_ZPG     = 0x26,
    RLA_ZPG     = 0x27,
    PLP         = 0x28,
    AND_IMT     = 0x29,
    ROL_ACC     = 0x2A,
    BIT_ABS     = 0x2C,
    AND_ABS     = 0x2D,
    ROL_ABS     = 0x2E,
    RLA_ABS     = 0x2F,

    BMI_REL     = 0x30,
    AND_IND_Y   = 0x31,
    RLA_IND_Y   = 0x33,
    NOP_ZPG_X_2 = 0x34,
    AND_ZPG_X   = 0x35,
    ROL_ZPG_X   = 0x36,
    RLA_ZPG_X   = 0x37,
    SEC         = 0x38,
    AND_ABS_Y   = 0x39,
    NOP_2       = 0x3A,
    RLA_ABS_Y   = 0x3B,
    NOP_ABS_X_2 = 0x3C,
    AND_ABS_X   = 0x3D,
    ROL_ABS_X   = 0x3E,
    RLA_ABS_X   = 0x3F,

    RTI         = 0x40,
    EOR_IND_X   = 0x41,
    SRE_IND_X   = 0x43,
    NOP_ZPG_4 = 0x44,
    EOR_ZPG     = 0x45,
    LSR_ZPG     = 0x46,
    SRE_ZPG     = 0x47,
    PHA         = 0x48,
    EOR_IMT     = 0x49,
    LSR_ACC     = 0x4A,
    JMP_ABS     = 0x4C,
    EOR_ABS     = 0x4D,
    LSR_ABS     = 0x4E,
    SRE_ABS     = 0x4F,

    BVC_REL     = 0x50,
    EOR_IND_Y   = 0x51,
    SRE_IND_Y   = 0x53,
    NOP_ZPG_X_3 = 0x54,
    EOR_ZPG_X   = 0x55,
    LSR_ZPG_X   = 0x56,
    SRE_ZPG_X   = 0x57,
    CLI         = 0x58,
    EOR_ABS_Y   = 0x59,
    NOP_4       = 0x5A,
    SRE_ABS_Y   = 0x5B,
    NOP_ABS_X_3 = 0x5C,
    EOR_ABS_X   = 0x5D,
    LSR_ABS_X   = 0x5E,
    SRE_ABS_X   = 0x5F,

    RTS         = 0x60,
    ADC_IND_X   = 0x61,
    RRA_IND_X   = 0x63,
    NOP_ZPG_3   = 0x64,
    ADC_ZPG     = 0x65,
    ROR_ZPG     = 0x66,
    RRA_ZPG     = 0x67,
    PLA         = 0x68,
    ADC_IMT     = 0x69,
    ROR_ACC     = 0x6A,
    JMP_IND     = 0x6C,
    ADC_ABS     = 0x6D,
    ROR_ABS     = 0x6E,
    RRA_ABS     = 0x6F,

    BVS_REL     = 0x70,
    ADC_IND_Y   = 0x71,
    RRA_IND_Y   = 0x73,
    NOP_ZPG_X_4 = 0x74,
    ADC_ZPG_X   = 0x75,
    ROR_ZPG_X   = 0x76,
    RRA_ZPG_X   = 0x77,
    SEI         = 0x78,
    ADC_ABS_Y   = 0x79,
    NOP_5       = 0x7A,
    RRA_ABS_Y   = 0x7B,
    NOP_ABS_X_4 = 0x7C,
    ADC_ABS_X   = 0x7D,
    ROR_ABS_X   = 0x7E,
    RRA_ABS_X   = 0x7F,

    NOP_IMM_1   = 0x80,
    STA_IND_X   = 0x81,
    NOP_IMM_2   = 0x82,
    SAX_IND_X   = 0x83,
    STY_ZPG     = 0x84,
    STA_ZPG     = 0x85,
    STX_ZPG     = 0x86,
    SAX_ZPG     = 0x87,
    DEY         = 0x88,
    NOP_IMM_3   = 0x89,
    TXA         = 0x8A,
    STY_ABS     = 0x8C,
    STA_ABS     = 0x8D,
    STX_ABS     = 0x8E,
    SAX_ABS     = 0x8F,

    BCC_REL     = 0x90,
    STA_IND_Y   = 0x91,
    STY_ZPG_X   = 0x94,
    STA_ZPG_X   = 0x95,
    STX_ZPG_Y   = 0x96,
    SAX_ZPG_Y   = 0x97,
    TYA         = 0x98,
    STA_ABS_Y   = 0x99,
    TXS         = 0x9A,
    STA_ABS_X   = 0x9D,

    LDY_IMT     = 0xA0,
    LDA_IND_X   = 0xA1,
    LDX_IMT     = 0xA2,
	LAX_IND_X   = 0xA3,
    LDY_ZPG     = 0xA4,
    LDA_ZPG     = 0xA5,
    LDX_ZPG     = 0xA6,
    LAX_ZPG     = 0xA7,
    TAY         = 0xA8,
    LDA_IMT     = 0xA9,
    TAX         = 0xAA,
    LDY_ABS     = 0xAC,
    LDA_ABS     = 0xAD,
    LDX_ABS     = 0xAE,
    LAX_ABS     = 0xAF,

    BCS_REL     = 0xB0,
    LDA_IND_Y   = 0xB1,
    LAX_IND_Y   = 0xB3,
    LDY_ZPG_X   = 0xB4,
    LDA_ZPG_X   = 0xB5,
    LDX_ZPG_Y   = 0xB6,
    LAX_ZPG_Y   = 0xB7,
    CLV         = 0xB8,
    LDA_ABS_Y   = 0xB9,
    TSX         = 0xBA,
    LDY_ABS_X   = 0xBC,
    LDA_ABS_X   = 0xBD,
    LDX_ABS_Y   = 0xBE,
    LAX_ABS_Y   = 0xBF,

    CPY_IMT     = 0xC0,
    CMP_IND_X   = 0xC1,
    NOP_IMM_4   = 0xC2,
    DCP_IND_X   = 0xC3,
    CPY_ZPG     = 0xC4,
    CMP_ZPG     = 0xC5,
    DEC_ZPG     = 0xC6,
    DCP_ZPG     = 0xC7,
    INY         = 0xC8,
    CMP_IMT     = 0xC9,
    DEX         = 0xCA,
    CPY_ABS     = 0xCC,
    CMP_ABS     = 0xCD,
    DEC_ABS     = 0xCE,
    DCP_ABS     = 0xCF,

    BNE_REL     = 0xD0,
    CMP_IND_Y   = 0xD1,
    DCP_IND_Y   = 0xD3,
    NOP_ZPG_X_5 = 0xD4,
    CMP_ZPG_X   = 0xD5,
    DEC_ZPG_X   = 0xD6,
    DCP_ZPG_X   = 0xD7,
    CLD         = 0xD8,
    CMP_ABS_Y   = 0xD9,
    NOP_6       = 0xDA,
    DCP_ABS_Y   = 0xDB,
    NOP_ABS_X_5 = 0xDC,
    CMP_ABS_X   = 0xDD,
    DEC_ABS_X   = 0xDE,
    DCP_ABS_X   = 0xDF,

    CPX_IMT     = 0xE0,
    SBC_IND_X   = 0xE1,
    NOP_IMM_5   = 0xE2,
    ISC_IND_X   = 0xE3,
    CPX_ZPG     = 0xE4,
    SBC_ZPG     = 0xE5,
    INC_ZPG     = 0xE6,
    ISC_ZPG     = 0xE7,
    INX         = 0xE8,
    SBC_IMT     = 0xE9,
    NOP         = 0xEA,
    SBC_IMT_2   = 0xEB,
    CPX_ABS     = 0xEC,
    SBC_ABS     = 0xED,
    INC_ABS     = 0xEE,
    ISC_ABS     = 0xEF,

    BEQ_REL     = 0xF0,
    SBC_IND_Y   = 0xF1,
    ISC_IND_Y   = 0xF3,
    NOP_ZPG_X_6 = 0xF4,
    SBC_ZPG_X   = 0xF5,
    INC_ZPG_X   = 0xF6,
    ISC_ZPG_X   = 0xF7,
    SED         = 0xF8,
    SBC_ABS_Y   = 0xF9,
    NOP_7       = 0xFA,
    ISC_ABS_Y   = 0xFB,
    NOP_ABS_X_6 = 0xFC,
    SBC_ABS_X   = 0xFD,
    INC_ABS_X   = 0xFE,
    ISC_ABS_X   = 0xFF,

    // for testing purposes only
    //NOP_TEST    = 0xFF
}

impl Default for OpCode {
    fn default() -> Self {
        OpCode::NOP
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum OpMnemonic {
    ADC = 0, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC, CLD, CLI, CLV, CMP, CPX,
    CPY, DCP, DEC, DEX, DEY, EOR, INC, INX, INY, ISC, JMP, JSR, LAX, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA,
    PLP, RLA, ROL, ROR, RRA, RTI, RTS, SAX, SBC, SEC, SED, SEI, SLO, SRE, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA
}

const CARRY_POS: u8 = 0;
const ZERO_POS: u8  = 1;
const INT_POS: u8   = 2;
const DEC_POS: u8   = 3;
const BRK_POS: u8   = 4;
const U_POS: u8     = 5;
const OVER_POS: u8  = 6;
const NEG_POS: u8   = 7;

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
            interrupt: 0,
            decimal: 0,
            brk: 0,
            unused: 0,
            overflow: 0,
            negative: 0
        }
    }
}

const STACK_IDX: u16 = 0x0100;

pub struct Cpu {
    // registers
    regA: u8,
    regY: u8,
    regX: u8,

    // pointers
    pgmCounter: u16,
    stkPointer: u8,

    memory: Rc<RefCell<DataBus>>,

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
    counter: u16
}

impl Clocked for Cpu {
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

        let mem = self.readMem8(self.pgmCounter);
        let opCode = OpCode::try_from(mem).unwrap_or_default();
        let (opMn, addrMode, cycles, xCycles, _) = self.getOpCodeInfo(opCode);
        let (target, bytes, increment, boundaryCrossed) = self.getAddressInfo(opMn, addrMode, self.pgmCounter.wrapping_add(1));

        // print!("PC: {:04X}, A: {:02X}, X: {:02X}, Y: {:02X}, P: {:02X}, SP: {:02X}, INST: {:?}\n",
        //                            self.pgmCounter, self.regA, self.regX, self.regY, self.getFlagValues(), self.stkPointer, opCode);

        // if self.pgmCounter == 0xC66E || self.counter == 8991 {
        //     panic!("DONE!");
        // }

        self.pgmCounter = if increment { self.pgmCounter.wrapping_add(bytes) } else { self.pgmCounter };
        self.executeInstruction(opMn, target);

        // we add cycles so that potential interrupt cycles are not erased
        self.waitCycles += cycles as u16;
        if boundaryCrossed { self.waitCycles += xCycles as u16; }

        // debug print

    }
}

impl Cpu {

    pub fn new (memory: Rc<RefCell<DataBus>>) -> Cpu {

        // load reset vector into program counter
        let lo = memory.borrow().readCpuMem(0xFFFC);
        let hi = memory.borrow().readCpuMem(0xFFFD);
        let prgC = ((hi as u16) << 8) + (lo as u16);


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
            isEvenCycle: false
        };

        cpu.setFlags(0x24);
        return cpu;
    }

    pub fn reset(&mut self) -> () {
        self.stkPointer = self.stkPointer.wrapping_sub(3);
        self.setFlags(0x24);
        self.pgmCounter = self.readMem16(0xFFFC);
    }

    fn executeInstruction(&mut self, opCode: OpMnemonic, target: Option<u16>) -> () {
        match opCode {
            OpMnemonic::ADC => { self.adc(target) },
            OpMnemonic::AND => { self.and(target) },
            OpMnemonic::ASL => { self.asl(target) },
            OpMnemonic::BCC => { self.bcc(target) },
            OpMnemonic::BCS => { self.bcs(target) },
            OpMnemonic::BEQ => { self.beq(target) },
            OpMnemonic::BIT => { self.bit(target) },
            OpMnemonic::BMI => { self.bmi(target) },
            OpMnemonic::BNE => { self.bne(target) },
            OpMnemonic::BPL => { self.bpl(target) },
            OpMnemonic::BRK => { self.brk(target) },
            OpMnemonic::BVC => { self.bvc(target) },
            OpMnemonic::BVS => { self.bvs(target) },
            OpMnemonic::CLC => { self.clc(target) },
            OpMnemonic::CLD => { self.cld(target) },
            OpMnemonic::CLI => { self.cli(target) },
            OpMnemonic::CLV => { self.clv(target) },
            OpMnemonic::CMP => { self.cmp(target) },
            OpMnemonic::CPX => { self.cpx(target) },
            OpMnemonic::CPY => { self.cpy(target) },
			OpMnemonic::DCP => { self.dcp(target) },
            OpMnemonic::DEC => { self.dec(target) },
            OpMnemonic::DEX => { self.dex(target) },
            OpMnemonic::DEY => { self.dey(target) },
            OpMnemonic::EOR => { self.eor(target) },
            OpMnemonic::INC => { self.inc(target) },
            OpMnemonic::INX => { self.inx(target) },
            OpMnemonic::INY => { self.iny(target) },
            OpMnemonic::ISC => { self.isc(target) },
            OpMnemonic::JMP => { self.jmp(target) },
            OpMnemonic::JSR => { self.jsr(target) },
            OpMnemonic::LAX => { self.lax(target) },
            OpMnemonic::LDA => { self.lda(target) },
            OpMnemonic::LDX => { self.ldx(target) },
            OpMnemonic::LDY => { self.ldy(target) },
            OpMnemonic::LSR => { self.lsr(target) },
            OpMnemonic::NOP => { self.nop(target) },
            OpMnemonic::ORA => { self.ora(target) },
            OpMnemonic::PHA => { self.pha(target) },
            OpMnemonic::PHP => { self.php(target) },
            OpMnemonic::PLA => { self.pla(target) },
            OpMnemonic::PLP => { self.plp(target) },
            OpMnemonic::RLA => { self.rla(target) },
            OpMnemonic::ROL => { self.rol(target) },
            OpMnemonic::ROR => { self.ror(target) },
            OpMnemonic::RRA => { self.rra(target) },
            OpMnemonic::RTI => { self.rti(target) },
            OpMnemonic::RTS => { self.rts(target) },
            OpMnemonic::SAX => { self.sax(target) },
            OpMnemonic::SBC => { self.sbc(target) },
            OpMnemonic::SEC => { self.sec(target) },
            OpMnemonic::SED => { self.sed(target) },
            OpMnemonic::SEI => { self.sei(target) },
            OpMnemonic::SLO => { self.slo(target) },
            OpMnemonic::SRE => { self.sre(target) },
            OpMnemonic::STA => { self.sta(target) },
            OpMnemonic::STX => { self.stx(target) },
            OpMnemonic::STY => { self.sty(target) },
            OpMnemonic::TAX => { self.tax(target) },
            OpMnemonic::TAY => { self.tay(target) },
            OpMnemonic::TSX => { self.tsx(target) },
            OpMnemonic::TXA => { self.txa(target) },
            OpMnemonic::TXS => { self.txs(target) },
            OpMnemonic::TYA => { self.tya(target) },
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

    fn adc(&mut self, target: Option<u16>) ->() {
        let oldVal = self.readMem8(target.unwrap());
        let newVal = oldVal.wrapping_add(self.regA).wrapping_add(self.flags.carry);

        self.flags.carry = if newVal < oldVal {1} else {0};

        // !((M^N) & 0x80) && ((M^result) & 0x80)
        // if the inputs have the same sign, and the input and result have different signs
        if ((self.regA ^ oldVal) & 0x80) == 0 && ((self.regA ^ newVal) & 0x80) != 0 {
            self.flags.overflow = 1;
        }
        else {
            self.flags.overflow = 0;
        }

        // set negative and zero flag
        self.setZNFlag(newVal);
        self.regA = newVal;
    }

    fn and(&mut self, target: Option<u16>) -> () {
        self.regA &= self.readMem8(target.unwrap());
        self.setZNFlag(self.regA);
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

        self.flags.zero = if self.regA & val == 0 {1} else {0};
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
        let hi = (self.pgmCounter >> 8) as u8;
        let lo = (self.pgmCounter & 0x00FF) as u8;
        self.pushStack(hi);
        self.pushStack(lo);
        self.php(None);
        self.sei(None);

        self.pgmCounter = self.readMem16(0xFFFE);
        self.flags.brk = 1;
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
        self.regA ^= self.readMem8(target.unwrap());
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
        } else {
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

    fn getAddressInfo(&self, ref opCode: OpMnemonic, ref addrMode: AddressMode, oper: u16) -> (Option<u16>, u16, bool, bool) {
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

                return (Some(target), 3, false, false)
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

                return(Some(target), 2, self.pcShouldIncrement(*opCode), (storedAddr & 0x00FF) > (target & 0x00FF));
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

                return (Some(target), 2, !self.branchIncrement(*opCode), target & 0xFF00 != (oper.wrapping_sub(1)) & 0xFF00);
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

    fn branchIncrement(&self, ref opCode: OpMnemonic) -> bool {
        match *opCode {
            OpMnemonic::BCC => { self.flags.carry == 0 },
            OpMnemonic::BCS => { self.flags.carry == 1 },
            OpMnemonic::BEQ => { self.flags.zero == 1 },
            OpMnemonic::BNE => { self.flags.zero == 0 },
            OpMnemonic::BMI => { self.flags.negative == 1 },
            OpMnemonic::BPL => { self.flags.negative == 0 },
            OpMnemonic::BVS => { self.flags.overflow == 1 },
            OpMnemonic::BVC => { self.flags.overflow == 0 },
            _ => false
        }
    }

    fn readMem16(&self, ref addr: u16) -> u16 {
        let lo = self.memory.borrow().readCpuMem(*addr);
        let hi = self.memory.borrow().readCpuMem((*addr + 1));
        return (hi as u16) << 8 | lo as u16;
    }

    fn readMem8(&self, ref addr: u16) -> u8 {
        return self.memory.borrow().readCpuMem(*addr);
    }

    fn writeMem8(&mut self, ref addr: u16, value: u8) -> () {
        // have to OAM DMA transfer here to prevent violation of borrowing rules
        // TODO: FIX THIS
        match *addr {
            0x4014  => { self.triggerOamTransfer((value as u16) << 8); },
            _ => { self.memory.borrow_mut().writeCpuMem(*addr, value); }
        }
    }

    fn writeMem16(&self, ref addr: u16, value: u16) -> () {
        let lo = (value & 0x00FF) as u8;
        let hi = (value >> 8) as u8;
        self.memory.borrow_mut().writeCpuMem(*addr, lo);
        self.memory.borrow_mut().writeCpuMem(addr.wrapping_add(1), hi);
    }

    fn pushStack(&mut self, ref value: u8) -> () {
        self.writeMem8(STACK_IDX | (self.stkPointer as u16), value.clone());
        self.stkPointer = self.stkPointer.wrapping_sub(1);
    }

    fn popStack(&mut self) -> u8 {
        self.stkPointer = self.stkPointer.wrapping_add(1);
        return self.readMem8(STACK_IDX | (self.stkPointer as u16));
    }

    fn pcShouldIncrement(&self, ref opCode: OpMnemonic) -> bool {
        match *opCode {
            OpMnemonic::JMP | OpMnemonic::RTS => return false,
            _ => return true
        }
    }

    fn setZNFlag(&mut self, ref result: u8) -> () {
        self.setNFlag(*result);
        self.setZFlag(*result);
    }

    fn setZFlag(&mut self, ref result: u8) -> () {
        if *result == 0 {
            self.flags.zero = 1;
        }
        else {
            self.flags.zero = 0;
        }
    }

    fn setNFlag(&mut self, ref result: u8) -> () {
        if *result & 0x80 != 0 {
            self.flags.negative = 1;
        }
        else {
            self.flags.negative = 0;
        }
    }

    fn setFlags(&mut self, status: u8) -> () {
        self.flags.carry =      (status >> CARRY_POS) & 1;
        self.flags.zero =       (status >> ZERO_POS) & 1;
        self.flags.interrupt =  (status >> INT_POS) & 1;
        self.flags.decimal =    (status >> DEC_POS) & 1;
        //self.flags.brk =        (status >> BRK_POS) & 1;
        self.flags.unused =     1;
        self.flags.overflow =   (status >> OVER_POS) & 1;
        self.flags.negative =   (status >> NEG_POS) & 1;
    }

    fn getFlagValues(&self) -> u8 {
        let mut status: u8 = 0;
        status |= ((self.flags.carry & 1) << CARRY_POS);
        status |= ((self.flags.zero & 1) << ZERO_POS);
        status |= ((self.flags.interrupt & 1) << INT_POS);
        status |= ((self.flags.decimal & 1) << DEC_POS);
        //status |= ((self.flags.brk & 1) << BRK_POS);
        status |= (1 << U_POS);
        status |= ((self.flags.overflow & 1) << OVER_POS);
        status |= ((self.flags.negative & 1) << NEG_POS);
        return status;
    }

//    fn flagOn(&mut self, bitPos: u8) -> () {
//        self.flags |= (1 << bitPos);
//    }
//
//    fn flagOff(&mut self, bitPos: u8) -> () {
//        self.flags &= !(1 << bitPos);
//    }
//
//    fn getFlagBit(&self, bitPos: u8) -> u8 {
//        return 1 << bitPos;
//    }

    fn getOpCodeInfo(&self, opcode: OpCode) -> (OpMnemonic, AddressMode, u8, u8, u8) {
        // Opcode Mnemonic
        // Address Mode
        // CPU cycles
        // extra CPU cycles if page boundary crossed
        // number of bytes in opcode + operand
        return match opcode {
            // 0x00
            OpCode::BRK =>          (OpMnemonic::BRK, IMP, 7, 0, 1),
            OpCode::ORA_IND_X =>    (OpMnemonic::ORA, IND_X, 6, 0, 2),
            OpCode::SLO_IND_X =>    (OpMnemonic::SLO, IND_X, 8, 0, 2),
            OpCode::ORA_ZPG =>      (OpMnemonic::ORA, ZPG, 3, 0, 2),
            OpCode::ASL_ZPG =>      (OpMnemonic::ASL, ZPG, 3, 0, 2),
            OpCode::SLO_ZPG =>      (OpMnemonic::SLO, ZPG, 5, 0, 2),
            OpCode::PHP =>          (OpMnemonic::PHP, IMP, 3, 0, 1),
            OpCode::ORA_IMT =>      (OpMnemonic::ORA, IMT, 2, 0, 2),
            OpCode::ASL_ACC =>      (OpMnemonic::ASL, ACC, 2, 0, 2),
            OpCode::ORA_ABS =>      (OpMnemonic::ORA, ABS, 4, 0, 3),
            OpCode::ASL_ABS =>      (OpMnemonic::ASL, ABS, 6, 0, 3),
            OpCode::SLO_ABS =>      (OpMnemonic::SLO, ABS, 6, 0, 3),

            // 0x10
            OpCode::BPL_REL =>      (OpMnemonic::BPL, REL, 2, 1, 2),
            OpCode::ORA_IND_Y =>    (OpMnemonic::ORA, IND_Y, 5, 1, 2),
            OpCode::SLO_IND_Y =>    (OpMnemonic::SLO, IND_Y, 8, 1, 2),
            OpCode::ORA_ZPG_X =>    (OpMnemonic::ORA, ZPG_X, 4, 0, 2),
            OpCode::ASL_ZPG_X =>    (OpMnemonic::ASL, ZPG_X, 6, 0, 2),
            OpCode::SLO_ZPG_X =>    (OpMnemonic::SLO, ZPG_X, 6, 0, 2),
            OpCode::CLC =>          (OpMnemonic::CLC, IMP, 2, 0, 1),
            OpCode::ORA_ABS_Y =>    (OpMnemonic::ORA, ABS_Y, 4, 1, 3),
            OpCode::SLO_ABS_Y =>    (OpMnemonic::SLO, ABS_Y, 7, 1, 3),
            OpCode::ORA_ABS_X =>    (OpMnemonic::ORA, ABS_X, 4, 1, 3),
            OpCode::ASL_ABS_X =>    (OpMnemonic::ASL, ABS_X, 7, 0, 3),
            OpCode::SLO_ABS_X =>    (OpMnemonic::SLO, ABS_X, 7, 0, 3),

            // 0x20
            OpCode::JSR_ABS =>      (OpMnemonic::JSR, ABS, 6, 0, 3),
            OpCode::AND_IND_X =>    (OpMnemonic::AND, IND_X, 6, 0, 2),
            OpCode::RLA_IND_X =>    (OpMnemonic::RLA, IND_X, 8, 0, 2),
            OpCode::BIT_ZPG =>      (OpMnemonic::BIT, ZPG, 3, 0, 2),
            OpCode::AND_ZPG =>      (OpMnemonic::AND, ZPG, 3, 0, 2),
            OpCode::ROL_ZPG =>      (OpMnemonic::ROL, ZPG, 5, 0, 2),
            OpCode::RLA_ZPG =>      (OpMnemonic::RLA, ZPG, 5, 0, 2),
            OpCode::PLP =>          (OpMnemonic::PLP, IMP, 4, 0, 1),
            OpCode::AND_IMT =>      (OpMnemonic::AND, IMT, 4, 0, 2),
            OpCode::ROL_ACC =>      (OpMnemonic::ROL, ACC, 6, 0, 1),
            OpCode::BIT_ABS =>      (OpMnemonic::BIT, ABS, 4, 0, 3),
            OpCode::AND_ABS =>      (OpMnemonic::AND, ABS, 4, 0, 3),
            OpCode::ROL_ABS =>      (OpMnemonic::ROL, ABS, 6, 0, 3),
            OpCode::RLA_ABS =>      (OpMnemonic::RLA, ABS, 6, 0, 3),

            // 0x30
            OpCode::BMI_REL =>      (OpMnemonic::BMI, REL, 2, 1, 2),
            OpCode::AND_IND_Y =>    (OpMnemonic::AND, IND_Y, 5, 1, 2),
            OpCode::RLA_IND_Y =>    (OpMnemonic::RLA, IND_Y, 8, 1, 2),
            OpCode::AND_ZPG_X =>    (OpMnemonic::AND, ZPG_X, 4, 0, 2),
            OpCode::ROL_ZPG_X =>    (OpMnemonic::ROL, ZPG_X, 6, 0, 2),
            OpCode::RLA_ZPG_X =>    (OpMnemonic::RLA, ZPG_X, 6, 0, 2),
            OpCode::SEC =>          (OpMnemonic::SEC, IMP, 2, 0, 1),
            OpCode::AND_ABS_Y =>    (OpMnemonic::AND, ABS_Y, 4, 1, 3),
            OpCode::RLA_ABS_Y =>    (OpMnemonic::RLA, ABS_Y, 7, 1, 3),
            OpCode::AND_ABS_X =>    (OpMnemonic::AND, ABS_X, 4, 1, 3),
            OpCode::ROL_ABS_X =>    (OpMnemonic::ROL, ABS_X, 7, 0, 3),
            OpCode::RLA_ABS_X =>    (OpMnemonic::RLA, ABS_X, 7, 0, 3),

            // 0x40
            OpCode::RTI =>          (OpMnemonic::RTI, IMP, 6, 0, 1),
            OpCode::EOR_IND_X =>    (OpMnemonic::EOR, IND_X, 6, 0, 2),
            OpCode::SRE_IND_X =>    (OpMnemonic::SRE, IND_X, 8, 0, 2),
            OpCode::EOR_ZPG =>      (OpMnemonic::EOR, ZPG, 3, 0, 2),
            OpCode::LSR_ZPG =>      (OpMnemonic::LSR, ZPG, 5, 0, 2),
            OpCode::SRE_ZPG =>      (OpMnemonic::SRE, ZPG, 5, 0, 2),
            OpCode::PHA =>          (OpMnemonic::PHA, IMP, 3, 0, 1),
            OpCode::EOR_IMT =>      (OpMnemonic::EOR, IMT, 2, 0, 2),
            OpCode::LSR_ACC =>      (OpMnemonic::LSR, ACC, 2, 0, 1),
            OpCode::JMP_ABS =>      (OpMnemonic::JMP, ABS, 3, 0, 3),
            OpCode::EOR_ABS =>      (OpMnemonic::EOR, ABS, 4, 0, 3),
            OpCode::LSR_ABS =>      (OpMnemonic::LSR, ABS, 6, 0, 3),
            OpCode::SRE_ABS =>      (OpMnemonic::SRE, ABS, 6, 0, 3),

            // 0x50
            OpCode::BVC_REL =>      (OpMnemonic::BVC, REL, 2, 1, 2),
            OpCode::EOR_IND_Y =>    (OpMnemonic::EOR, IND_Y, 5, 1, 2),
            OpCode::SRE_IND_Y =>    (OpMnemonic::SRE, IND_Y, 8, 1, 2),
            OpCode::EOR_ZPG_X =>    (OpMnemonic::EOR, ZPG_X, 4, 0, 2),
            OpCode::LSR_ZPG_X =>    (OpMnemonic::LSR, ZPG_X, 6, 0, 2),
            OpCode::SRE_ZPG_X =>    (OpMnemonic::SRE, ZPG_X, 6, 0, 2),
            OpCode::CLI =>          (OpMnemonic::CLI, IMP, 2, 0, 1),
            OpCode::EOR_ABS_Y =>    (OpMnemonic::EOR, ABS_Y, 4, 1, 3),
            OpCode::SRE_ABS_Y =>    (OpMnemonic::SRE, ABS_Y, 7, 1, 3),
            OpCode::EOR_ABS_X =>    (OpMnemonic::EOR, ABS_X, 4, 1, 3),
            OpCode::LSR_ABS_X =>    (OpMnemonic::LSR, ABS_X, 7, 0, 3),
            OpCode::SRE_ABS_X =>    (OpMnemonic::SRE, ABS_X, 7, 0, 3),

            // 0x60
            OpCode::RTS =>          (OpMnemonic::RTS, IMP, 6, 0, 1),
            OpCode::ADC_IND_X =>    (OpMnemonic::ADC, IND_X, 6, 0, 2),
            OpCode::RRA_IND_X =>    (OpMnemonic::RRA, IND_X, 8, 0, 2),
            OpCode::ADC_ZPG =>      (OpMnemonic::ADC, ZPG, 3, 0, 2),
            OpCode::ROR_ZPG =>      (OpMnemonic::ROR, ZPG, 5, 0, 2),
            OpCode::RRA_ZPG =>      (OpMnemonic::RRA, ZPG, 5, 0, 2),
            OpCode::PLA =>          (OpMnemonic::PLA, IMP, 4, 0, 1),
            OpCode::ADC_IMT =>      (OpMnemonic::ADC, IMT, 2, 0, 2),
            OpCode::ROR_ACC =>      (OpMnemonic::ROR, ACC, 2, 0, 1),
            OpCode::JMP_IND =>      (OpMnemonic::JMP, IND, 5, 0, 2),
            OpCode::ADC_ABS =>      (OpMnemonic::ADC, ABS, 4, 0, 3),
            OpCode::ROR_ABS =>      (OpMnemonic::ROR, ABS, 6, 0, 3),
            OpCode::RRA_ABS =>      (OpMnemonic::RRA, ABS, 6, 0, 3),

            // 0x70
            OpCode::BVS_REL =>      (OpMnemonic::BVS, REL, 2, 1, 2),
            OpCode::ADC_IND_Y =>    (OpMnemonic::ADC, IND_Y, 5, 1, 2),
            OpCode::RRA_IND_Y =>    (OpMnemonic::RRA, IND_Y, 8, 1, 2),
            OpCode::ADC_ZPG_X =>    (OpMnemonic::ADC, ZPG_X, 4, 0, 2),
            OpCode::ROR_ZPG_X =>    (OpMnemonic::ROR, ZPG_X, 6, 0, 2),
            OpCode::RRA_ZPG_X =>    (OpMnemonic::RRA, ZPG_X, 6, 0, 2),
            OpCode::SEI =>          (OpMnemonic::SEI, IMP, 2, 0, 1),
            OpCode::ADC_ABS_Y =>    (OpMnemonic::ADC, ABS_Y, 4, 1, 3),
            OpCode::RRA_ABS_Y =>    (OpMnemonic::RRA, ABS_Y, 7, 1, 3),
            OpCode::ADC_ABS_X =>    (OpMnemonic::ADC, ABS_X, 4, 1, 3),
            OpCode::ROR_ABS_X =>    (OpMnemonic::ROR, ABS_X, 7, 0, 3),
            OpCode::RRA_ABS_X =>    (OpMnemonic::RRA, ABS_X, 7, 0, 3),

            // 0x80
            OpCode::STA_IND_X =>    (OpMnemonic::STA, IND_X, 6, 0, 2),
            OpCode::SAX_IND_X =>    (OpMnemonic::SAX, IND_X, 6, 0, 2),
            OpCode::STY_ZPG =>      (OpMnemonic::STY, ZPG, 3, 0, 2),
            OpCode::STA_ZPG =>      (OpMnemonic::STA, ZPG, 3, 0, 2),
            OpCode::STX_ZPG =>      (OpMnemonic::STX, ZPG, 3, 0, 2),
            OpCode::SAX_ZPG =>      (OpMnemonic::SAX, ZPG, 3, 0, 2),
            OpCode::DEY =>          (OpMnemonic::DEY, IMP, 2, 0, 1),
            OpCode::TXA =>          (OpMnemonic::TXA, IMP, 2, 0, 1),
            OpCode::STY_ABS =>      (OpMnemonic::STY, ABS, 4, 0, 3),
            OpCode::STA_ABS =>      (OpMnemonic::STA, ABS, 4, 0, 3),
            OpCode::STX_ABS =>      (OpMnemonic::STX, ABS, 4, 0, 3),
            OpCode::SAX_ABS =>      (OpMnemonic::SAX, ABS, 4, 0, 3),

            // 0x90
            OpCode::BCC_REL =>      (OpMnemonic::BCC, REL, 2, 1, 1),
            OpCode::STA_IND_Y =>    (OpMnemonic::STA, IND_Y, 6, 0, 2),
            OpCode::STY_ZPG_X =>    (OpMnemonic::STY, ZPG_X, 4, 0, 2),
            OpCode::STA_ZPG_X =>    (OpMnemonic::STA, ZPG_X, 4, 0, 2),
            OpCode::STX_ZPG_Y =>    (OpMnemonic::STX, ZPG_Y, 4, 0, 2),
            OpCode::SAX_ZPG_Y =>    (OpMnemonic::SAX, ZPG_Y, 4, 0, 2),
            OpCode::TYA =>          (OpMnemonic::TYA, IMP, 2, 0, 1),
            OpCode::STA_ABS_Y =>    (OpMnemonic::STA, ABS_Y, 5, 0, 3),
            OpCode::TXS =>          (OpMnemonic::TXS, IMP, 2, 0, 1),
            OpCode::STA_ABS_X =>    (OpMnemonic::STA, ABS_X, 5, 0, 3),

            // 0xA0
            OpCode::LDY_IMT =>      (OpMnemonic::LDY, IMT, 2, 0, 2),
            OpCode::LDA_IND_X =>    (OpMnemonic::LDA, IND_X, 6, 0, 2),
            OpCode::LDX_IMT =>      (OpMnemonic::LDX, IMT, 2, 0, 2),
            OpCode::LAX_IND_X =>    (OpMnemonic::LAX, IND_X, 6, 0, 2),
            OpCode::LDY_ZPG =>      (OpMnemonic::LDY, ZPG, 3, 0, 2),
            OpCode::LDA_ZPG =>      (OpMnemonic::LDA, ZPG, 3, 0, 2),
            OpCode::LDX_ZPG =>      (OpMnemonic::LDX, ZPG, 3, 0, 2),
            OpCode::LAX_ZPG =>      (OpMnemonic::LAX, ZPG, 3, 0, 2),
            OpCode::TAY =>          (OpMnemonic::TAY, IMP, 2, 0, 1),
            OpCode::LDA_IMT =>      (OpMnemonic::LDA, IMT, 2, 0, 2),
            OpCode::TAX =>          (OpMnemonic::TAX, IMP, 2, 0, 1),
            OpCode::LDY_ABS =>      (OpMnemonic::LDY, ABS, 4, 0, 3),
            OpCode::LDA_ABS =>      (OpMnemonic::LDA, ABS, 4, 0, 3),
            OpCode::LDX_ABS =>      (OpMnemonic::LDX, ABS, 4, 0, 3),
            OpCode::LAX_ABS =>      (OpMnemonic::LAX, ABS, 4, 0, 2),

            // 0xB0
            OpCode::BCS_REL =>      (OpMnemonic::BCS, REL, 2, 1, 2),
            OpCode::LDA_IND_Y =>    (OpMnemonic::LDA, IND_Y, 5, 1, 2),
            OpCode::LAX_IND_Y =>    (OpMnemonic::LAX, IND_Y, 5, 1, 2),
            OpCode::LDY_ZPG_X =>    (OpMnemonic::LDY, ZPG_X, 4, 0, 2),
            OpCode::LDA_ZPG_X =>    (OpMnemonic::LDA, ZPG_X, 4, 0, 2),
            OpCode::LDX_ZPG_Y =>    (OpMnemonic::LDX, ZPG_Y, 4, 0, 2),
            OpCode::LAX_ZPG_Y =>    (OpMnemonic::LAX, ZPG_Y, 4, 0, 2),
            OpCode::CLV =>          (OpMnemonic::CLV, IMP, 2, 0, 1),
            OpCode::LDA_ABS_Y =>    (OpMnemonic::LDA, ABS_Y, 4, 1, 3),
            OpCode::TSX =>          (OpMnemonic::TSX, IMP, 2, 0, 1),
            OpCode::LDY_ABS_X =>    (OpMnemonic::LDY, ABS_X, 4, 1, 3),
            OpCode::LDA_ABS_X =>    (OpMnemonic::LDA, ABS_X, 4, 1, 3),
            OpCode::LDX_ABS_Y =>    (OpMnemonic::LDX, ABS_Y, 4, 1, 3),
            OpCode::LAX_ABS_Y =>    (OpMnemonic::LAX, ABS_Y, 4, 1, 2),

            // 0xC0
            OpCode::CPY_IMT =>      (OpMnemonic::CPY, IMT, 2, 0, 2),
            OpCode::CMP_IND_X =>    (OpMnemonic::CMP, IND_X, 6, 0, 2),
            OpCode::DCP_IND_X =>    (OpMnemonic::DCP, IND_X, 8, 0, 2),
            OpCode::CPY_ZPG =>      (OpMnemonic::CPY, ZPG, 3, 0, 2),
            OpCode::CMP_ZPG =>      (OpMnemonic::CMP, ZPG, 3, 0, 2),
            OpCode::DEC_ZPG =>      (OpMnemonic::DEC, ZPG, 5, 0, 2),
            OpCode::DCP_ZPG =>      (OpMnemonic::DCP, ZPG, 5, 0, 2),
            OpCode::INY =>          (OpMnemonic::INY, IMP, 2, 0, 1),
            OpCode::CMP_IMT =>      (OpMnemonic::CMP, IMT, 2, 0, 2),
            OpCode::DEX =>          (OpMnemonic::DEX, IMP, 2, 0, 1),
            OpCode::CPY_ABS =>      (OpMnemonic::CPY, ABS, 4, 0, 3),
            OpCode::CMP_ABS =>      (OpMnemonic::CMP, ABS, 4, 0, 3),
            OpCode::DEC_ABS =>      (OpMnemonic::DEC, ABS, 6, 0, 3),
            OpCode::DCP_ABS =>      (OpMnemonic::DCP, ABS, 6, 0, 3),

            // 0xD0
            OpCode::BNE_REL =>      (OpMnemonic::BNE, REL, 2, 1, 2),
            OpCode::CMP_IND_Y =>    (OpMnemonic::CMP, IND_Y, 5, 1, 2),
            OpCode::DCP_IND_Y =>    (OpMnemonic::DCP, IND_Y, 8, 1, 2),
            OpCode::CMP_ZPG_X =>    (OpMnemonic::CMP, ZPG_X, 4, 0, 2),
            OpCode::DEC_ZPG_X =>    (OpMnemonic::DEC, ZPG_X, 6, 0, 2),
            OpCode::DCP_ZPG_X =>    (OpMnemonic::DCP, ZPG_X, 6, 0, 2),
            OpCode::CLD =>          (OpMnemonic::CLD, IMP, 2, 0, 1),
            OpCode::CMP_ABS_Y =>    (OpMnemonic::CMP, ABS_Y, 4, 1, 3),
            OpCode::DCP_ABS_Y =>    (OpMnemonic::DCP, ABS_Y, 7, 1, 3),
            OpCode::CMP_ABS_X =>    (OpMnemonic::CMP, ABS_X, 4, 1, 3),
            OpCode::DEC_ABS_X =>    (OpMnemonic::DEC, ABS_X, 7, 0, 3),
            OpCode::DCP_ABS_X =>    (OpMnemonic::DCP, ABS_X, 7, 0, 3),

            // 0xE0
            OpCode::CPX_IMT =>      (OpMnemonic::CPX, IMT, 2, 0, 2),
            OpCode::SBC_IND_X =>    (OpMnemonic::SBC, IND_X, 6, 0, 2),
            OpCode::ISC_IND_X =>    (OpMnemonic::ISC, IND_X, 8, 0, 2),
            OpCode::CPX_ZPG =>      (OpMnemonic::CPX, ZPG, 3, 0, 2),
            OpCode::SBC_ZPG =>      (OpMnemonic::SBC, ZPG, 3, 0, 2),
            OpCode::INC_ZPG =>      (OpMnemonic::INC, ZPG, 5, 0, 2),
            OpCode::ISC_ZPG =>      (OpMnemonic::ISC, ZPG, 5, 0, 2),
            OpCode::INX =>          (OpMnemonic::INX, IMP, 2, 0, 1),
            OpCode::SBC_IMT =>      (OpMnemonic::SBC, IMT, 2, 0, 2),
            OpCode::NOP =>          (OpMnemonic::NOP, IMP, 2, 0, 1),
            OpCode::SBC_IMT_2 =>    (OpMnemonic::SBC, IMT, 2, 0, 2),
            OpCode::CPX_ABS =>      (OpMnemonic::CPX, ABS, 4, 0, 3),
            OpCode::SBC_ABS =>      (OpMnemonic::SBC, ABS, 4, 0, 3),
            OpCode::INC_ABS =>      (OpMnemonic::INC, ABS, 6, 0, 3),
            OpCode::ISC_ABS =>      (OpMnemonic::ISC, ABS, 6, 0, 3),

            // 0xF0
            OpCode::BEQ_REL =>      (OpMnemonic::BEQ, REL, 2, 1, 2),
            OpCode::SBC_IND_Y =>    (OpMnemonic::SBC, IND_Y, 5, 1, 2),
            OpCode::ISC_IND_Y =>    (OpMnemonic::ISC, IND_Y, 8, 1, 2),
            OpCode::SBC_ZPG_X =>    (OpMnemonic::SBC, ZPG_X, 4, 0, 2),
            OpCode::INC_ZPG_X =>    (OpMnemonic::INC, ZPG_X, 6, 0, 2),
            OpCode::ISC_ZPG_X =>    (OpMnemonic::ISC, ZPG_X, 6, 0, 2),
            OpCode::SED =>          (OpMnemonic::SED, IMP, 2, 0, 1),
            OpCode::SBC_ABS_Y =>    (OpMnemonic::SBC, ABS_Y, 4, 1, 3),
            OpCode::ISC_ABS_Y =>    (OpMnemonic::ISC, ABS_Y, 7, 1, 3),
            OpCode::SBC_ABS_X =>    (OpMnemonic::SBC, ABS_X, 4, 1, 3),
            OpCode::INC_ABS_X =>    (OpMnemonic::INC, ABS_X, 7, 0, 3),
            OpCode::ISC_ABS_X =>    (OpMnemonic::ISC, ABS_X, 7, 0, 3),

            // extra NOPs
            OpCode::NOP_ZPG_1 =>    (OpMnemonic::NOP, ZPG, 3, 0, 3),
            OpCode::NOP_ABS_1 =>    (OpMnemonic::NOP, ABS, 4, 0, 3),
            OpCode::NOP_ZPG_X_1 =>  (OpMnemonic::NOP, ZPG_X, 4, 0, 3),
            OpCode::NOP_1 =>        (OpMnemonic::NOP, IMP, 2, 0, 3),
            OpCode::NOP_ABS_X_1 =>  (OpMnemonic::NOP, ABS_X, 4, 1, 3),
            OpCode::NOP_ZPG_X_2 =>  (OpMnemonic::NOP, ZPG_X, 4, 0, 3),
            OpCode::NOP_2 =>        (OpMnemonic::NOP, IMP, 2, 0, 3),
            OpCode::NOP_ABS_X_2 =>  (OpMnemonic::NOP, ABS_X, 4, 1, 3),
            OpCode::NOP_ZPG_4 =>    (OpMnemonic::NOP, ZPG, 3, 0, 3),
            OpCode::NOP_ZPG_X_3 =>  (OpMnemonic::NOP, ZPG_X, 4, 0, 3),
            OpCode::NOP_4 =>        (OpMnemonic::NOP, IMP, 2, 0, 3),
            OpCode::NOP_ABS_X_3 =>  (OpMnemonic::NOP, ABS_X, 4, 1, 3),
            OpCode::NOP_ZPG_3 =>    (OpMnemonic::NOP, ZPG, 3, 0, 3),
            OpCode::NOP_ZPG_X_4 =>  (OpMnemonic::NOP, ZPG_X, 4, 0, 3),
            OpCode::NOP_5 =>        (OpMnemonic::NOP, IMP, 2, 0, 3),
            OpCode::NOP_ABS_X_4 =>  (OpMnemonic::NOP, ABS_X, 4, 1, 3),
            OpCode::NOP_IMM_1 =>    (OpMnemonic::NOP, IMT, 2, 0, 3),
            OpCode::NOP_IMM_2 =>    (OpMnemonic::NOP, IMT, 2, 0, 3),
            OpCode::NOP_IMM_3 =>    (OpMnemonic::NOP, IMT, 2, 0, 3),
            OpCode::NOP_IMM_4 =>    (OpMnemonic::NOP, IMT, 2, 0, 3),
            OpCode::NOP_ZPG_X_5 =>  (OpMnemonic::NOP, IMT, 2, 0, 3),
            OpCode::NOP_6 =>        (OpMnemonic::NOP, IMP, 2, 0, 3),
            OpCode::NOP_ABS_X_5 =>  (OpMnemonic::NOP, ABS_X, 4, 1, 3),
            OpCode::NOP_IMM_5 =>    (OpMnemonic::NOP, IMT, 2, 0, 3),
            OpCode::NOP_ZPG_X_6 =>  (OpMnemonic::NOP, ZPG_X, 4, 0, 3),
            OpCode::NOP_7 =>        (OpMnemonic::NOP, IMP, 2, 0, 3),
            OpCode::NOP_ABS_X_6 =>  (OpMnemonic::NOP, ABS_X, 4, 1, 3),
        }
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
        let cpu = getNewCpu();
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, ACC, 0);
        assert_eq!(target, None);
        assert_eq!(bytes, 1);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, false);
    }

    #[test]
    fn getAddressInfoAbsolute() {
        let cpu = getNewCpu();
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
        let cpu = getNewCpu();
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, IMT, 0);
        assert_eq!(target, Some(0));
        assert_eq!(bytes, 2);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, false);
    }

    #[test]
    fn getAddressInfoImplied() {
        let cpu = getNewCpu();
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, IMP, 0);
        assert_eq!(target, None);
        assert_eq!(bytes, 1);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, false);
    }

    #[test]
    fn getAddressInfoIndirectIndexed() {
        let cpu = getNewCpu();
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
        let cpu = getNewCpu();
        let (target, bytes, shouldInc, boundaryCrossed) = cpu.getAddressInfo(OpMnemonic::NOP, REL, 0);
        assert_eq!(target, Some(0));
        assert_eq!(bytes, 2);
        assert_eq!(shouldInc, true);
        assert_eq!(boundaryCrossed, false);
    }

    #[test]
    fn getAddressInfoZeroPage() {
        let cpu = getNewCpu();
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

