
use lazy_static::lazy_static;
use ndarray::Array2;
use ndarray::Array1;
use ndarray::array;

pub const STEP_ONE_CYCLE: u16 = 3729;
pub const STEP_TWO_CYCLE: u16 = 7457;
pub const STEP_THREE_CYCLE: u16 = 11186;
pub const STEP_FOUR_CYCLE: u16 = 14914;
pub const STEP_FOUR_CYCLE_PLUS_ONE: u16 = 14915;
pub const STEP_FIVE_CYCLE: u16 = 18641;
pub const CPU_TICK_TIME: f64 = 1.0 / 1789773.0;
pub const CPU_FREQ: f64 = 1789773.0;
pub const AUDIO_HERTZ: u16 = 44100;
pub const HARMONICS: u8 = 100;

const BUFFER_SIZE: u16 = 2048;


pub const SQUARE_SEQUENCE_TABLE: [[u8; 8]; 4] = [
     [0, 1, 0, 0, 0, 0, 0, 0],
     [0, 1, 1, 0, 0, 0, 0, 0],
     [0, 1, 1, 1, 1, 0, 0, 0],
     [1, 0, 0, 1, 1, 1, 1, 1],
];

lazy_static! {



/*
15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0
 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15
*/

	pub static ref TRIANGLE_SEQUENCE_TABLE: Array1<u8> = array![
		15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
 		0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15
	];

	pub static ref NOISE_TIMER_TABLE: Array1<u16> = array![
		4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068
	];

	pub static ref DMC_RATE_TABLE: Array1<u16> = array![
		428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54
	];
}
