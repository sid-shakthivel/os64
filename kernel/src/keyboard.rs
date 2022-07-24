// src/keyboard.rs

/*
    PS/2 Keyboard which uses serial communication
    Accept commands and sends scancodes which comply to a scancode set
    TODO: Check status register is ready to be read
*/

use crate::ports::inb;
use crate::{print, print_serial};
use spin::Mutex;
use crate::TERMINAL;
use crate::CONSOLE;

pub struct Keyboard {
    is_upper: bool,
    kbd_us: &'static [char; 0x3A],
}

pub static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard { 
    is_upper: false,
    kbd_us: &['\0', '\0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\0', '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n', '\0', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', '\0', ';', '\'', '`', '\0', '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', '\0', '*', '\0', ' ']
 });

impl Keyboard {
    fn translate(&self, scancode: u8, uppercase: bool) -> char {
        if scancode > 0x3A { return '0'; }
        
        if uppercase {
            return ((self.kbd_us[scancode as usize] as u8) - 0x20) as char;
        } else {
            return self.kbd_us[scancode as usize];
        }
    }
    
    pub fn handle_keyboard(&mut self) {
        let scancode = inb(0x60);

        match scancode {
            0x26 => print!("l"),
            0x2A => self.is_upper = true, // Left shift pressed
            0x36 => self.is_upper = true, // Right shift pressed
            0xAA => self.is_upper= false, // Left shift released
            0xB6 => self.is_upper = false, // Right shift released
            0x3A => { self.is_upper = !self.is_upper }, // Caps lock pressed
            _ => {
                let letter = self.translate(scancode, false);
    
                if letter != '0' {
                    print!("{}", letter);
                    print_serial!("{}", letter);
                }
            }
        }
    }
}

