// /src/ports.rs

// Manages all functions related to input/output 

use core::arch::asm;

pub fn outb(port: u16, value: u8) {
    unsafe { outb_raw(port, value); }
}

pub fn inb(port: u16) -> u8 {
    unsafe { inb_raw(port) }
}

pub fn io_wait() {
    outb(0x80, 0);
}

extern "C" {
    fn outb_raw(port: u16, value: u8);
    fn inb_raw(port: u16) -> u8;
}