use cortex_m::asm;
use embedded_hal::blocking::delay::*;

/// Approximate number of cycles per sec for asm delays
const CYCLES_PER_SEC: u32 = 24_000_000;
const CYCLES_PER_MILLIS: u32 = CYCLES_PER_SEC / 1_000;
const CYCLES_PER_MICROS: u32 = CYCLES_PER_SEC / 1_000_000;

pub struct AsmDelay;

impl DelayMs<u32> for AsmDelay {
    fn delay_ms(&mut self, ms: u32) {
        asm::delay(CYCLES_PER_MILLIS * (ms as u32));
    }
}

impl DelayUs<u32> for AsmDelay {
    fn delay_us(&mut self, us: u32) {
        asm::delay(CYCLES_PER_MICROS * (us as u32));
    }
}

pub trait DelayCycles {
    fn delay_cycles(&mut self, cs: u32);
}

impl DelayCycles for AsmDelay {
    #[inline]
    fn delay_cycles(&mut self, cs: u32) {
        asm::delay(cs)
    }
}
