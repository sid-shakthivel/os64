// src/ps2.rs

/*
    PS/2 (Personal System 2) controller is part of AIP which is linked to the 8042 chip
    Green/purple ports which connect directly to keyboards and mice
    Scan codes are sets of codes which determine when a key is pressed/repeated/released
    Has 2 buffers for data (one for data recieved and one for data written before it's sent)
    - Data port (0x60) which is used to read/write from PS/2 device/controller
    - Command/Status register (0x64) used to send commands
    - Writing a value to 0x64 sends a command byte whilst reading gets the status byte
*/

use crate::keyboard::KEYBOARD;
use crate::mouse::MOUSE;
use crate::ports::inb;
use crate::ports::outb;
use crate::print_serial;
use crate::uart::CONSOLE;

const PS2_DATA: u16 = 0x60; // Data port
const PS2_STATUS: u16 = 0x64;
const PS2_CMD: u16 = 0x64; // Command port
const TIMEOUT: i16 = 10000;

bitflags! {
    struct ControllerRegister: u8 {
        const KEYBOARD_INTERRUPT_ENABLE = 0b00000001;
        const MOUSE_INTERRUPT_ENABLE = 0b00000010;
        const SYSTEM_FLAG = 0b00000100;
        const IGNORE_KEYBOARD_LOCK = 0b00001000;
        const KEYBOARD_ENABLE = 0b00010000;
        const MOUSE_ENABLE = 0b00100000;
        const KEYBOARD_TRANSLATION = 0b01000000;
        const UNUSED = 0b10000000;
    }
}

#[derive(PartialEq, Debug)]
pub enum PS2Device {
    PS2Mouse,
    PS2MouseScrollWheel,
    PS2MouseFiveButtons,
    MF2Keyboard,
    MF2KeyboardTranslation,
}

pub fn init() -> Result<(), &'static str> {
    // Disable devices so ps2 devices can't mess up init
    ps2_write(PS2_CMD, 0xAD)?;
    ps2_write(PS2_CMD, 0xA7)?;

    // Flush output buffer as data can be stuck in PS2 controller buffer
    inb(PS2_DATA);

    // Set controller configuration byte to disable IRQ's, disable translation
    ps2_write(PS2_CMD, 0x20)?;
    let mut controller_config_byte = ps2_read(PS2_DATA)?;
    let mut controller_config = ControllerRegister::from_bits_truncate(controller_config_byte);
    ControllerRegister::remove(
        &mut controller_config,
        ControllerRegister::KEYBOARD_INTERRUPT_ENABLE,
    );
    ControllerRegister::remove(
        &mut controller_config,
        ControllerRegister::MOUSE_INTERRUPT_ENABLE,
    );
    ControllerRegister::remove(
        &mut controller_config,
        ControllerRegister::KEYBOARD_TRANSLATION,
    );

    // controller_config_byte = controller_config_byte & !(1 << 0) & !(1 << 1) & !(1 << 6);
    ps2_write(PS2_CMD, 0x60)?;
    ps2_write(PS2_DATA, controller_config.bits)?;

    // Perform controller self test
    ps2_write(PS2_CMD, 0xAA)?; // Test controller
    if ps2_read(PS2_DATA)? != 0x55 {
        panic!("Controller self test failed\n");
    }

    ps2_write(PS2_CMD, 0xA8)?; // Enable second PS2 port
    ps2_write(PS2_CMD, 0x20)?;
    controller_config_byte = ps2_read(PS2_DATA)?;
    controller_config = ControllerRegister::from_bits_truncate(controller_config_byte);
    if controller_config.contains(ControllerRegister::MOUSE_ENABLE) {
        panic!("Not dual channel???\n");
    } else {
        ps2_write(PS2_CMD, 0xA7)?;
    }

    // Perform interface tests to test both ports
    ps2_write(PS2_CMD, 0xAB)?;
    if ps2_read(PS2_DATA)? != 0x00 {
        panic!("Interface test failed\n");
    }

    ps2_write(PS2_CMD, 0xA9)?;
    if ps2_read(PS2_DATA)? != 0x00 {
        panic!("Interface test failed\n");
    }

    // Enable both PS2 ports
    ps2_write(PS2_CMD, 0xAE)?;
    ps2_write(PS2_CMD, 0xA8)?;

    // Enable interrupts
    ps2_write(PS2_CMD, 0x20)?;
    controller_config_byte = ps2_read(PS2_DATA)?;
    controller_config = ControllerRegister::from_bits_truncate(controller_config_byte);

    ControllerRegister::set(
        &mut controller_config,
        ControllerRegister::KEYBOARD_INTERRUPT_ENABLE,
        true,
    );
    ControllerRegister::set(
        &mut controller_config,
        ControllerRegister::MOUSE_INTERRUPT_ENABLE,
        true,
    );
    ControllerRegister::set(
        &mut controller_config,
        ControllerRegister::KEYBOARD_TRANSLATION,
        true,
    );

    ps2_write(PS2_CMD, 0x60)?;
    ps2_write(PS2_DATA, controller_config.bits)?;

    // Reset devices
    for i in 0..2 {
        ps2_write_device(i, 0xFF)?;
        let response = ps2_read(PS2_DATA)?;

        if response != 0xFA || ps2_read(PS2_DATA)? != 0xAA {
            panic!("Reading device {} failed with {:x}", i, response);
        }

        // Mouse can send an extra 0x00 byte
        if (inb(PS2_STATUS) & 1) != 0 {
            ps2_read(PS2_DATA)?;
        }
    }

    // Identify devices and initialise them appropriately
    for i in 0..2 {
        match ps2_identify_device_type(i).unwrap() {
            PS2Device::MF2KeyboardTranslation => {
                KEYBOARD.lock().init();
            }
            PS2Device::PS2Mouse => {
                MOUSE.lock().init();
                MOUSE.free();
            }
            _ => panic!("Unknown device"),
        }
    }

    return Ok(());
}

fn ps2_write(port: u16, byte: u8) -> Result<u8, &'static str> {
    let mut timeout = TIMEOUT;
    while (inb(PS2_STATUS) & 2) > 0 {
        timeout -= 1;
        if timeout < 0 {
            print_serial!("PS2 WRITE FAILED\n");
            return Err("PS2 Write Failed");
        }
    }
    outb(port, byte);
    return Ok(0);
}

pub fn ps2_read(port: u16) -> Result<u8, &'static str> {
    let mut timeout = TIMEOUT;
    while (inb(PS2_STATUS) & 1) == 0 {
        timeout -= 1;
        if timeout < 0 {
            print_serial!("PS2 READ FAILED\n");
            return Err("PS2 Read Failed");
        }
    }

    return Ok(inb(port));
}

pub fn ps2_is_from_mouse() -> bool {
    return inb(PS2_STATUS) & (1 << 5) == 0x20;
}

pub fn ps2_write_device(device_num: u16, byte: u8) -> Result<u8, &'static str> {
    return match device_num {
        0 => {
            ps2_write(PS2_DATA, byte)?;
            return Ok(0);
        }
        1 => {
            ps2_write(PS2_CMD, 0xD4)?;
            ps2_write(PS2_DATA, byte)?;
            return Ok(0);
        }
        _ => Err("Unknown device"),
    };
}

// Must wait to recieve acknowledgement from device (0xFA)
pub fn ps2_wait_ack() -> Result<bool, &'static str> {
    while ps2_read(PS2_DATA)? != 0xFA {}
    return Ok(true);
}

pub fn ps2_identify_device_type(device_num: u16) -> Result<PS2Device, &'static str> {
    ps2_write_device(device_num, 0xF5)?; // Send disable scanning command
    ps2_wait_ack()?;

    ps2_write_device(device_num, 0xF2)?; // Send identify command
    ps2_wait_ack()?;

    let mut response = ps2_read(PS2_DATA)?;
    return match response {
        0x00 => Ok(PS2Device::PS2Mouse),
        0x03 => Ok(PS2Device::PS2MouseScrollWheel),
        0x04 => Ok(PS2Device::PS2MouseFiveButtons),
        0xAB => {
            response = ps2_read(PS2_DATA)?;
            return match response {
                0x41 | 0xC1 => Ok(PS2Device::MF2KeyboardTranslation),
                0x83 => Ok(PS2Device::MF2Keyboard),
                _ => Err("Unknown device"),
            };
        }
        _ => Err("Unknown device"),
    };
}
