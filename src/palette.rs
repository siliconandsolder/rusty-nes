#![allow(non_snake_case)]
#![allow(warnings)]

#[derive(Debug, Clone, Copy)]
pub struct PaletteColour {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl PaletteColour {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        PaletteColour {
            red,
            green,
            blue,
        }
    }
}

pub const PALETTE_ARRAY: [PaletteColour; 64] = [
    PaletteColour::new(84, 84, 84),
    PaletteColour::new(0, 30, 116),
    PaletteColour::new(8, 16, 144),
    PaletteColour::new(48, 0, 136),
    PaletteColour::new(68, 0, 100),
    PaletteColour::new(92, 0, 48),
    PaletteColour::new(84, 4, 0),
    PaletteColour::new(60, 24, 0),
    PaletteColour::new(32, 42, 0),
    PaletteColour::new(8, 58, 0),
    PaletteColour::new(0, 64, 0),
    PaletteColour::new(0, 60, 0),
    PaletteColour::new(0, 50, 60),
    PaletteColour::new(0, 0, 0),
    PaletteColour::new(0, 0, 0),
    PaletteColour::new(0, 0, 0),
    PaletteColour::new(152, 150, 152),
    PaletteColour::new(8, 76, 196),
    PaletteColour::new(48, 50, 236),
    PaletteColour::new(92, 30, 228),
    PaletteColour::new(136, 20, 176),
    PaletteColour::new(160, 20, 100),
    PaletteColour::new(152, 34, 32),
    PaletteColour::new(120, 60, 0),
    PaletteColour::new(84, 90, 0),
    PaletteColour::new(40, 114, 0),
    PaletteColour::new(8, 124, 0),
    PaletteColour::new(0, 118, 40),
    PaletteColour::new(0, 102, 120),
    PaletteColour::new(0, 0, 0),
    PaletteColour::new(0, 0, 0),
    PaletteColour::new(0, 0, 0),
    PaletteColour::new(236, 238, 236),
    PaletteColour::new(76, 154, 236),
    PaletteColour::new(120, 124, 236),
    PaletteColour::new(176, 98, 236),
    PaletteColour::new(228, 84, 236),
    PaletteColour::new(236, 88, 180),
    PaletteColour::new(236, 106, 100),
    PaletteColour::new(212, 136, 32),
    PaletteColour::new(160, 170, 0),
    PaletteColour::new(116, 196, 0),
    PaletteColour::new(76, 208, 32),
    PaletteColour::new(56, 204, 108),
    PaletteColour::new(56, 180, 204),
    PaletteColour::new(60, 60, 60),
    PaletteColour::new(0, 0, 0),
    PaletteColour::new(0, 0, 0),
    PaletteColour::new(236, 238, 236),
    PaletteColour::new(168, 204, 236),
    PaletteColour::new(188, 188, 236),
    PaletteColour::new(212, 178, 236),
    PaletteColour::new(236, 174, 236),
    PaletteColour::new(236, 174, 212),
    PaletteColour::new(236, 180, 176),
    PaletteColour::new(228, 196, 144),
    PaletteColour::new(204, 210, 120),
    PaletteColour::new(180, 222, 120),
    PaletteColour::new(168, 226, 144),
    PaletteColour::new(152, 226, 180),
    PaletteColour::new(160, 214, 228),
    PaletteColour::new(160, 162, 160),
    PaletteColour::new(0, 0, 0),
    PaletteColour::new(0, 0, 0)
];