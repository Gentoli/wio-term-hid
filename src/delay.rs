use embedded_hal::blocking::delay::DelayMs;

use cortex_m::asm::delay as cycle_delay;

// use wio::prelude::*;
use wio_terminal::hal::time::{Nanoseconds, U32Ext};

pub(crate) struct InstDelay {}

fn ms_to_cycle(ms: u16) -> u32 {
    static PERIOD: Nanoseconds = Nanoseconds(1_000_u32 / 100);
    let delay: Nanoseconds = (ms as u32).ms().into();
    PERIOD.0 / delay.0
}

impl DelayMs<u16> for InstDelay {
    fn delay_ms(&mut self, ms: u16) {
        // Use 100MHz instead of 120MHz to have at least `ms` delay
        cycle_delay(ms_to_cycle(ms));
    }
}
