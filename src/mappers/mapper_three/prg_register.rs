#![allow(non_snake_case)]
#![allow(warnings)]

pub struct PrgRegister {
    pub eightBanks: Vec<u8>,
    pub secondLastBank: u8,
    pub lastBank: u8
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