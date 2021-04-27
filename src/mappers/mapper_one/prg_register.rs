#![allow(non_snake_case)]
#![allow(warnings)]

pub struct PrgRegister {
    pub bankLo: u8,
    pub bankHi: u8,
    pub bank32: u8,
}

impl PrgRegister {
    pub fn new(numBanks: u8) -> Self {
        PrgRegister {
            bankLo: 0,
            bankHi: numBanks - 1,
            bank32: 0
        }
    }
}