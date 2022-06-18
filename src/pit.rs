// src/pit.rs

/*
    Programmable interval timer is a chip which is used to implement a system clock as it sends interrupts on a regular basis
    Channel 0 (0x40) is connected to IRQ 0
    0x43 is command port
*/

use crate::ports::outb;
use crate::ports::inb;
use crate::print;
use crate::vga_text::TERMINAL;
use spin::Mutex;
use crate::multitask::Process;
use crate::multitask;
use crate::pic::PICS;
use crate::pic::pic_functions;

pub struct pit {
    divisor: u64,
    frequency: u64,
    ticks: u64,
}

const INPUT_CLOCK: u64 = 1193180;
const FREQUENCY: u64  = 100;

pub static PIT: Mutex<pit> = Mutex::new(pit::new(FREQUENCY));

impl pit {
    pub const fn new(frequency: u64) -> pit {
        pit {
            ticks: 0,
            frequency: frequency,
            divisor: INPUT_CLOCK / frequency,
        }
    }

    pub fn init(&self) {
        // Set command byte (0x36)
        let mode = 0b00000000 | 0b00110000 | 0b00000000;
        outb(0x43, mode);
        self.set_frequency();
    }

    pub fn handle_timer(&mut self) {
        self.ticks += 1;
        self.set_frequency();
    }

    fn set_frequency(&self) {
        // To set a frequency, a divisor is sent in bits
        outb(0x40, (self.divisor & 0xFF) as u8);
        outb(0x40, (self.divisor >> 8) as u8);
    }
}

