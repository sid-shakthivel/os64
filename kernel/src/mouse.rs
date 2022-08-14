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

use crate::framebuffer;
use crate::framebuffer::FRAMEBUFFER;
use crate::ps2;
use crate::spinlock::Lock;

#[derive(PartialEq)]
pub enum MouseState {
    Up,
    Down,
    Immobile,
}

pub struct Mouse {
    pub mouse_x: u64,
    pub mouse_y: u64,
    mouse_packets: [u8; 4],
    current_byte: usize,
    variety: ps2::PS2Device,
    pub mouse_state: MouseState,
}

pub static MOUSE: Lock<Mouse> = Lock::new(Mouse {
    mouse_x: framebuffer::SCREEN_WIDTH / 2,
    mouse_y: framebuffer::SCREEN_HEIGHT / 2,
    mouse_packets: [0; 4],
    current_byte: 0,
    variety: ps2::PS2Device::PS2Mouse,
    mouse_state: MouseState::Immobile,
});

impl Mouse {
    pub fn init(&mut self) {
        self.enable_z_axis();
        self.enable_5_buttons();
        self.enable_scanning();
    }

    pub fn handle_mouse_interrupt(&mut self) {
        if ps2::ps2_is_from_mouse() {
            let byte = ps2::ps2_read(0x60).unwrap();

            self.mouse_packets[self.current_byte] = byte;

            self.current_byte = (self.current_byte + 1) % 4;

            if self.current_byte == 0 {
                self.handle_mouse_packets();
            }
        }
    }

    fn handle_mouse_packets(&mut self) {
        let mut is_left_clicked = false;
        // Check overflows, if set, discard packet
        if self.mouse_packets[0] & (1 << 7) >= 0x80 || self.mouse_packets[0] & (1 << 6) >= 0x40 {
            return; // TODO: Add an error
        }

        // Bit 3 verifies packet alignment (if wrong, should return error)
        if self.mouse_packets[0] & (1 << 3) != 0x08 {
            return;
        }

        // Left button pressed
        if self.mouse_packets[0] & (1 << 0) == 1 {
            is_left_clicked = true;
            self.mouse_state = MouseState::Down;
        } else {
            self.mouse_state = MouseState::Up;
        }

        // Right button pressed
        // if self.mouse_packets[0] & (1 << 1) == 2 {
        //     return;
        // }

        // Clear mouse coordiantes before updating
        FRAMEBUFFER.lock().fill_rect(
            None,
            self.mouse_x,
            self.mouse_y,
            5,
            5,
            framebuffer::BACKGROUND_COLOUR,
        );
        FRAMEBUFFER.free();

        // X movement and Y movement values must be read as a 9 bit or greater SIGNED value if bit is enabled
        if self.mouse_packets[0] & (1 << 4) == 0x10 {
            self.mouse_x = self
                .mouse_x
                .wrapping_add(self.sign_extend(self.mouse_packets[1]) as u64);
        } else {
            self.mouse_x = self.mouse_x.wrapping_add(self.mouse_packets[1] as u64);
        }

        if self.mouse_packets[0] & (1 << 5) == 0x20 {
            let adjusted_y = self.sign_extend(self.mouse_packets[2]) * -1;
            self.mouse_y = self.mouse_y.wrapping_add(adjusted_y as u64);
        } else {
            let adjusted_y = (self.mouse_packets[2] as i16) * -1;
            self.mouse_y = self.mouse_y.wrapping_add(adjusted_y as u64);
        }

        // if self.mouse_x > 1019 {
        //     self.mouse_x = 1019;
        // }

        // if self.mouse_y > 763 {
        //     self.mouse_y = 763;
        // }

        // if self.mouse_x <= 5{
        //     self.mouse_x = 10;
        // }

        // if self.mouse_y <= 5 {
        //     self.mouse_y = 10;
        // }

        // DESKTOP
        //     .lock()
        //     .handle_mouse_movement(self.mouse_x, self.mouse_y, is_left_clicked);
        // DESKTOP.free();
    }

    fn enable_scanning(&self) {
        ps2::ps2_write_device(1, 0xF4).unwrap(); // Set sample rate command
        ps2::ps2_wait_ack().unwrap();
    }

    fn disable_scanning(&self) {
        ps2::ps2_write_device(1, 0xF5).unwrap(); // Set sample rate command
        ps2::ps2_wait_ack().unwrap();
    }

    fn enable_z_axis(&mut self) {
        self.set_mouse_rate(200);
        self.set_mouse_rate(100);
        self.set_mouse_rate(80);
        if self.get_type() != ps2::PS2Device::PS2MouseScrollWheel {
            panic!("Scroll wheel failed");
        } else {
            self.variety = self.get_type()
        }
    }

    fn enable_5_buttons(&mut self) {
        self.set_mouse_rate(200);
        self.set_mouse_rate(200);
        self.set_mouse_rate(80);
        if self.get_type() != ps2::PS2Device::PS2MouseFiveButtons {
            panic!("5 button mode failed");
        } else {
            self.variety = self.get_type()
        }
    }

    fn get_type(&self) -> ps2::PS2Device {
        return ps2::ps2_identify_device_type(1).unwrap();
    }

    fn sign_extend(&self, packet: u8) -> i16 {
        ((packet as u16) | 0xFF00) as i16
    }

    fn set_mouse_rate(&self, sample_rate: u8) {
        ps2::ps2_write_device(1, 0xF3).unwrap(); // Set sample rate command
        ps2::ps2_wait_ack().unwrap();
        ps2::ps2_write_device(1, sample_rate).unwrap();
        ps2::ps2_wait_ack().unwrap();
    }
}
