// src/keyboard.rs

/*
    PS/2 Keyboard which uses serial communication
    Accepts commands and sends scancodes which comply to a scancode set
    Scancode is simply a byte and scan code set is map between ascii characters and bytes sent
*/

use core::panic;

use crate::{print_serial};
use spin::Mutex;
use crate::CONSOLE;
use crate::ps2;

pub struct Keyboard {
    is_upper: bool,
    kbd_us: &'static [char; 0x3A],
    scancode_set: ScancodeSet
}

pub static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard { 
    is_upper: false,
    kbd_us: &['\0', '\0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\0', '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n', '\0', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', '\0', ';', '\'', '`', '\0', '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', '\0', '*', '\0', ' '],
    scancode_set: ScancodeSet::ScancodeSet1
});

#[derive(PartialEq)]
 enum ScancodeSet {
    ScancodeSet1,
    ScancodeSet2,
    ScancodeSet3,
 }

impl Keyboard {
    pub fn init(&mut self) {
        if self.scancode_set != self.get_scancode_set() { self.scancode_set = self.get_scancode_set(); }
        self.enable_scanning();
    }

    fn translate(&self, scancode: u8, uppercase: bool) -> char {
        if scancode > 0x3A { return '0'; }
        
        if uppercase {
            return ((self.kbd_us[scancode as usize] as u8) - 0x20) as char;
        } else {
            return self.kbd_us[scancode as usize];
        }
    }
    
    pub fn handle_keyboard(&mut self) {
        if !ps2::ps2_is_from_mouse() {
            let scancode = ps2::ps2_read(0x60).unwrap();

            match scancode {
                0x26 => print_serial!("l"),
                0x2A => self.is_upper = true, // Left shift pressed
                0x36 => self.is_upper = true, // Right shift pressed
                0xAA => self.is_upper= false, // Left shift released
                0xB6 => self.is_upper = false, // Right shift released
                0x3A => { self.is_upper = !self.is_upper }, // Caps lock pressed
                _ => {
                    let letter = self.translate(scancode, false);
        
                    if letter != '0' {
                        print_serial!("{}", letter);
                    }
                }
            }
        }
    }

    // Enables keyboard
    fn enable_scanning(&self) {
        ps2::ps2_write_device(0, 0xF4).unwrap();
        ps2::ps2_wait_ack().unwrap();
    }

    fn disable_scanning(&self) {
        ps2::ps2_write_device(0, 0xF5).unwrap();
        ps2::ps2_wait_ack().unwrap();
    }

    fn get_scancode_set(&self) -> ScancodeSet {
        ps2::ps2_write_device(0, 0xF0).unwrap();
        ps2::ps2_wait_ack().unwrap();
        ps2::ps2_write_device(0, 0).unwrap();
        ps2::ps2_wait_ack().unwrap();
        
        return match ps2::ps2_read(0x60).unwrap() {
            0x43 => ScancodeSet::ScancodeSet1,
            0x41 => ScancodeSet::ScancodeSet2,
            0x3f => ScancodeSet::ScancodeSet3,
            _ => panic!("Unkown scancode set"),
        };
    }
}

