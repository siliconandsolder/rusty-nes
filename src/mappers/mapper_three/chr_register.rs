#![allow(non_snake_case)]
#![allow(warnings)]

pub struct ChrRegister {
    doubleBanks: Vec<u8>,
    singleBanks: Vec<u8>,
}

impl ChrRegister {
    pub fn new() -> Self {
        ChrRegister {
            doubleBanks: vec![0; 2],
            singleBanks: vec![0; 4]
        }
    }
}