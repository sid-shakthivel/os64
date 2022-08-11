// src/vga_text.rs

/*
The vga buffer located at 0xb8000 allows characters to be printed to the screen.
Actual values in memory are not written to as this is only a buffer which maps to VRAM.
Screen has 25 rows of 80 length.
Each entry in the buffer must be formatted like this:
+---------------------------------------------+
|         | 15 | 12-14 | 8-11| 0-7 |          |
+---------------------------------------------+
| | Blink | Background | Foreground | ASCII | |
+---------------------------------------------+
*/

// TODO: Fix having to import use crate::vga_text::TERMINAL; on each file

use crate::writer::Writer;
use core::fmt;
use lazy_static::lazy_static;

#[allow(dead_code)]
enum VgaColours {
    Black = 0,
    Bue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

pub struct Terminal {
    terminal_row: usize,
    terminal_col: usize,
    vga_buffer: &'static mut [[u16; VGA_WIDTH]; VGA_HEIGHT],
}

lazy_static! {
    pub static ref TERMINAL: spin::Mutex<Terminal> = spin::Mutex::new(Terminal {
        terminal_row: 0,
        terminal_col: 0,
        vga_buffer: unsafe { &mut *(0xb8000 as *mut [[u16; VGA_WIDTH]; VGA_HEIGHT]) }, // Make an array pointed at the address
    });
}

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

#[macro_export]
macro_rules! print_vga {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        TERMINAL.lock().write_fmt(format_args!($($arg)*)).unwrap();
    });
}

impl fmt::Write for Terminal {
    // To support the rust formatting system and use the write! macro, the write_str method must be supported
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Terminal {
    fn scroll(&mut self) {
        for i in 0..(VGA_HEIGHT - 1) {
            for j in 0..VGA_WIDTH {
                self.vga_buffer[i][j] = (self.vga_buffer[i + 1][j]).clone()
            }
        }
        self.terminal_row = VGA_HEIGHT - 1;
        self.terminal_col = 0;
        self.clear_row(VGA_HEIGHT - 1);
    }

    fn clear_row(&mut self, row_num: usize) {
        for j in 0..VGA_WIDTH {
            self.vga_buffer[row_num][j] = 0;
        }
    }

    pub fn backspace(&mut self) {
        if self.terminal_col == 0 {
            self.terminal_col = 79;
            self.terminal_row -= 1;
            self.vga_buffer[self.terminal_row][self.terminal_col] = VgaColours::get_vga_entry(
                VgaColours::get_attributes((VgaColours::Black, VgaColours::White)),
                ' ' as u8,
            );
        } else {
            self.terminal_col -= 1;
            self.vga_buffer[self.terminal_row][self.terminal_col] = VgaColours::get_vga_entry(
                VgaColours::get_attributes((VgaColours::Black, VgaColours::White)),
                ' ' as u8,
            );
        }
    }
}

impl VgaColours {
    fn get_vga_entry(attribute: u8, character: u8) -> u16 {
        // First 8 bits is character and last 8 bits are attributes
        return (attribute as u16) << 8 | (character as u16);
    }

    fn get_attributes(colours: (VgaColours, VgaColours)) -> u8 {
        // Background Color, Foreground Color
        return (colours.0 as u8) << 4 | (colours.1 as u8);
    }
}

impl Writer for Terminal {
    fn put_char(&mut self, character: char) {
        match character {
            '\n' => self.newline(),
            _ => {
                let attributes = VgaColours::get_attributes((VgaColours::Black, VgaColours::White));
                self.vga_buffer[self.terminal_row][self.terminal_col] =
                    VgaColours::get_vga_entry(attributes, character as u8);
                self.terminal_col += 1;
                if self.terminal_col >= VGA_WIDTH {
                    self.newline();
                }
            }
        }
    }

    fn newline(&mut self) {
        self.terminal_row += 1;
        self.terminal_col = 0;
        if self.terminal_row >= VGA_HEIGHT {
            self.scroll();
        }
    }

    fn clear(&mut self) {
        for _i in 0..VGA_HEIGHT {
            for _j in 0..VGA_WIDTH {
                self.put_char(' ');
            }
        }
        self.terminal_row = 0;
        self.terminal_col = 0;
    }
}
