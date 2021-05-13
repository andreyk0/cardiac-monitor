use stm32f1xx_hal::time::Hertz;

use crate::lcd::{TFT_HEIGHT, TFT_WIDTH};

pub const SYS_FREQ: Hertz = Hertz(72_000_000);

pub const UI_HEIGHT: usize = TFT_WIDTH as usize; // width in our screen orientation
pub const UI_WIDTH: usize = TFT_HEIGHT as usize; // width in our screen orientation

/// Number of samples to use as an input into heart rate / SPO2 calculations
pub const MAX30102_NUM_SAMPLES: usize = 160;
// configuration is in board.rs
pub const MAX30102_SAMPLE_RATE: Hertz = Hertz(25);
