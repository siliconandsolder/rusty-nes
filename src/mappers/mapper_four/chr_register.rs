#![allow(non_snake_case)]
#![allow(warnings)]

pub struct ChrRegister {
    pub doubleBanks: Vec<u8>,
    pub singleBanks: Vec<u8>,
}

impl ChrRegister {
    pub fn new() -> Self {
        ChrRegister {
            doubleBanks: vec![0; 2],
            singleBanks: vec![0; 4]
        }
    }
}