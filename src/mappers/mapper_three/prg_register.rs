#![allow(non_snake_case)]
#![allow(warnings)]

pub struct PrgRegister {
    eightBanks: Vec<u8>,
    secondLastBank: u8,
    lastBank: u8
}

impl PrgRegister {
    pub fn new(numBanks: u8) -> Self {
        PrgRegister {
            eightBanks: vec![0; 2],
            secondLastBank: numBanks - 2,
            lastBank: numBanks - 1
        }
    }
}