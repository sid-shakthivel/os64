// src/mouse.rs

/*
    PS2 Mouse communicates with the PS2 controller using serial communication
    Mouse is enabled on PS2 bys, (0xFA means acknowledge)
    Mouse sends 3/4 byte packets to communicate movement at port 0x60 (bit 5 on status register indicates if from mouse)
    Packets are generated at a rate (100 packets a second) and if mouse is pressed/released
    Byte 1: 
    +------------+------------+-------------+------------+-------+------------+-----------+----------+
    |   Bit 0    |   Bit 1    |    Bit 2    |   Bit 3    | Bit 4 |   Bit 5    |  Bit 6    |  Bit 7   |
    +------------+------------+-------------+------------+-------+------------+-----------+----------+
    | Y Overflow | X Overflow | Y Sign Bit  | X Sign Bit |     1 | Middle Btn | Right Btn | Left Btn |
    +------------+------------+-------------+------------+-------+------------+-----------+----------+
    Byte 2: X Movement
    Byte 3: Y Movement
*/

use spin::Mutex;
use crate::ports::outb;
use crate::ports::inb;
use crate::print_serial;
use crate::ps2::ps2_read;
use crate::print;
use crate::TERMINAL;
use crate::CONSOLE;

pub struct Mouse {
    mouse_x: usize,
    mouse_y: usize,
    mouse_packets: [u8; 3],
    index: usize,
}

// TODO: Make this dynamic with framebuffer size
pub static MOUSE: Mutex<Mouse> = Mutex::new(Mouse {
    mouse_x: 512,
    mouse_y: 384,
    mouse_packets: [0; 3],
    index: 0,
});

impl Mouse {
    pub fn init(&self) {
        // TODO: Move init stuff from ps2 controller enable more buttons/wheel, etc
    }

    pub fn handle_mouse_interrupt(&mut self) {
        let byte = ps2_read(0x60).unwrap();
        self.mouse_packets[self.index] = byte;
        self.index += 1;

        if (self.index > 2) {
            // When we've recieved 3 bytes, update mouse movement
            self.index = 0;
            self.handle_mouse_packets();
        }
    }

    fn handle_mouse_packets(&mut self) {
        // Check overflows, if set, discard packet
        if self.mouse_packets[0] & (1 << 7) == 0x80 || self.mouse_packets[0] & (1 << 6) == 0x40 {
            return; // TODO: Add an error
        }

        // Bit 3 verifies packet alignment
        if self.mouse_packets[0] & (1 << 3) != 0x08 {
            return; 
        }

        // Left button pressed
        if self.mouse_packets[0] & (1 << 0) == 1 {
            return;
        }

        // Right button pressed
        if self.mouse_packets[0] & (1 << 1) == 2 {
            return;
        }

        // X movement and Y movement values must be read as a 9 bit or greater SIGNED value if bit is enabled

        // print!("{:b}\n", self.mouse_packets[0]);

        if self.mouse_packets[0] & (1 << 4) == 0x10 {
            self.mouse_x = self.mouse_x.wrapping_add(self.sign_extend(self.mouse_packets[1]) as usize);
        } else {
            self.mouse_x = self.mouse_x.wrapping_add(self.mouse_packets[1] as usize);
        }

        if self.mouse_packets[0] & (1 << 5) == 0x20 {
            let test = self.sign_extend(self.mouse_packets[2]) * -1;
            self.mouse_y = self.mouse_y.wrapping_add(test as usize);
        } else {
            let test = (self.mouse_packets[2] as i16) * -1;
            self.mouse_y = self.mouse_y.wrapping_add(test as usize);
        }

        // Draw small square to indicate mouse position
        for i in 0..5 {
            for j in 0..5 {
                TERMINAL.lock().draw_pixel(self.mouse_x + j, self.mouse_y + i, 0xFF);
            }
        }
    }

    fn sign_extend(&self, packet: u8) -> i16 {
        ((packet as u16) | 0xFF00) as i16
    }
}