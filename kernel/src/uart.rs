// src/uart.rs

// TODO: Write brief description

use crate::ports::outb;
use crate::ports::inb;
use crate::ports::io_wait;

const PORT: u16 = 0x3f8; // COM1

pub fn init() {
    outb(PORT + 1, 0x00); // Disable interrupts
    outb(PORT + 3, 0x80); // Enable DLAB
    outb(PORT + 0, 0x03); // Set divisor to 3
    outb(PORT + 1, 0x00);
    outb(PORT + 3, 0x03); // 8 Bits, no parity, one stop bit
    outb(PORT + 2, 0xc7); // Enable FIFO
    outb(PORT + 4, 0x0b); // IRQ's enabled
    outb(PORT + 4, 0x1e); // Set in loopback mode
    outb(PORT + 0, 0xae); // Test serial chip

    if (inb(PORT + 0) != 0xae) { panic!("Faulty serial!"); }

    outb(PORT + 4, 0x0f); // Set to normal operation mode
}

fn read_serial() -> u8 {
    while (serial_recieved() == 0) {};
    return inb(PORT);
}

pub fn write_string(string: &str) {
    for c in string.chars() {
       write_serial(c);
    }
}

fn write_serial(character: char) {
    while (is_transmit_empty() == 0) {};
    outb(PORT, (character as u8));
}

fn is_transmit_empty() -> u8 {
    return inb(PORT + 5) & 0x20;
}

fn serial_recieved() -> u8 {
    return inb(PORT + 5) & 1;
}