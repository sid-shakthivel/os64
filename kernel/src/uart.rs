// src/uart.rs

/*
    Physical serial ports provide a connector to attach devices (trasmits 1 byte at a time through a single channel)
    Serial ports are bi-directional (half duplex) and are controlled by uart (chip which encodes and decodes data)
    Must supply speed used for sending data (baud rate), error checking, data bits
*/

use spin::Mutex;

use crate::ports::outb;
use crate::ports::inb;
use crate::writer::Writer;
use core::fmt;

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

    if inb(PORT + 0) != 0xae { panic!("Faulty serial!"); }

    outb(PORT + 4, 0x0f); // Set to normal operation mode
}

pub struct Console {
    port: u16
}

pub static CONSOLE: Mutex<Console> = Mutex::new(Console { port: PORT });

#[macro_export] 
macro_rules! print_serial {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        CONSOLE.lock().write_fmt(format_args!($($arg)*)).unwrap();
    });
}

impl fmt::Write for Console {
    // To support the rust formatting system and use the write! macro, the write_str method must be supported
   fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
   }
}

impl Console {    
    fn write_serial(&mut self, character: char) {
        while self.is_transmit_empty() == 0 {};
        outb(self.port, character as u8);
    }
    
    fn read_serial(&self) -> u8 {
        while self.serial_recieved() == 0 {};
        return inb(self.port);
    }
    
    
    fn is_transmit_empty(&self) -> u8 {
        return inb(self.port + 5) & 0x20;
    }
    
    fn serial_recieved(&self) -> u8 {
        return inb(self.port + 5) & 1;
    }
}

impl Writer for Console {
    fn write_string(&mut self, string: &str) {
        for c in string.chars() {
            self.put_char(c);
        }
    }

    fn put_char(&mut self, character: char) {
        self.write_serial(character);
    }
}
