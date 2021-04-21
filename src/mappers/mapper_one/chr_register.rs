#![allow(non_snake_case)]
#![allow(warnings)]

pub struct ChrRegister {
    pub bankLo: u8,
    pub bankHi: u8,
    pub bank8: u8,
}

impl ChrRegister {
    pub fn new() -> Self {
        ChrRegister {
            bankLo: 0,
            bankHi: 0,
            bank8: 0
        }
    }
}