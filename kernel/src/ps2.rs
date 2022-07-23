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

use crate::ports::outb;
use crate::ports::inb;
use crate::ports::io_wait;
use crate::print_serial;
use crate::uart::CONSOLE;
use crate::framebuffer::TERMINAL;
use crate::print;

const PS2_DATA: u16 = 0x60; // Data port
const PS2_STATUS: u16 = 0x64;
const PS2_CMD: u16 = 0x64; // Command port
const TIMEOUT: i16 = 10000;

enum PS2Device {
    Mouse,
    Keyboard,
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

    print_serial!("{:b}\n", controller_config_byte);
    controller_config_byte = controller_config_byte & !(1 << 0) & !(1 << 1) & !(1 << 6);
    ps2_write(PS2_CMD, 0x60)?;
    ps2_write(PS2_DATA, controller_config_byte)?;

    // Perform controller self test
    ps2_write(PS2_CMD, 0xAA)?; // Test controller

    let mut test = ps2_read(PS2_DATA)?;
    if test != 0x55 {
        panic!("Controller self test failed\n");
    }

    // print_serial!("It's {:x}\n", test);

    ps2_write(PS2_CMD, 0xA8)?; // Enable second PS2 port
    ps2_write(PS2_CMD, 0x20)?; 
    controller_config_byte = ps2_read(PS2_DATA)?;
    if (controller_config_byte & (1 << 5) > 0) {
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
    controller_config_byte = controller_config_byte | (1 << 0) | (1 << 1);
    ps2_write(PS2_CMD, 0x60)?;
    ps2_write(PS2_DATA, 0b01000011)?;

    // 0b1110011

    // Reset devices
    for i in 0..2 {
        ps2_write_device(i, 0xFF)?;
        let mut response = ps2_read(PS2_DATA)?;

        if response != 0xFA || ps2_read(PS2_DATA)? != 0xAA {
            panic!("Reading device {} failed with {:x}", i, response);
        }

        // Mouse can send an extra 0x00 byte
        if (inb(PS2_STATUS) & 1) != 0 {
            ps2_read(PS2_DATA)?;
        }
    }

    // Identify devices
    for i in 0..2 {
        ps2_identify_device_type(i)?;
    }

    // Enable keyboard
    ps2_write_device(0, 0xF4)?;
    while ps2_read(PS2_DATA)? != 0xFA {} // Wait for ACK

    // Enable mouse
    ps2_write_device(1, 0xF4)?;
    while ps2_read(PS2_DATA)? != 0xFA {} // Wait for ACK

    print_serial!("Here\n");

    return Ok(());
}

fn ps2_write(port: u16, byte: u8) -> Result<u8, &'static str> {
    let mut timeout = TIMEOUT;
    while((inb(PS2_STATUS) & 2) > 0) {
        timeout -= 1;
        if timeout < 0 {
            print!("PS2 WRITE FAILED\n");
            return Err("PS2 Write Failed");
        }
    }
    outb(port, byte);
    return Ok(0);
}

fn ps2_read(port: u16) -> Result<u8, &'static str> {
    let mut timeout = TIMEOUT;
    while((inb(PS2_STATUS) & 1) == 0) {
        timeout -= 1;
        if timeout < 0 {
            print!("PS2 READ FAILED\n");
            return Err("PS2 Read Failed");
        }
    }

    return Ok(inb(port));
}

fn ps2_write_device(device_num: u16, byte: u8) -> Result<u8, &'static str> {
    return match device_num {
        0 => {
            ps2_write(PS2_DATA, byte)?;
            return Ok(0);
        },
        1 => {
            ps2_write(PS2_CMD, 0xD4)?;
            ps2_write(PS2_DATA, byte)?;
            return Ok(0);
        },
        _ => Err("Unknown device"),
    }
}

fn ps2_identify_device_type(device_num: u16) -> Result<PS2Device, &'static str> {
    ps2_write_device(device_num, 0xF5)?; // Send disable scanning command
    while ps2_read(PS2_DATA)? != 0xFA {} // Wait for ACK

    ps2_write_device(device_num, 0xF2)?; // Send identify command
    while ps2_read(PS2_DATA)? != 0xFA {} // Wait for ACK

    let byte1 = inb(PS2_DATA);
    let byte2 = inb(PS2_DATA);

//    return match byte1 {
//         0x00 => Ok(PS2Device::Mouse),
//         0xAB => Ok(PS2Device::Keyboard),
//         _ => panic!("Unknown device {:x} {:x}\n", byte1, byte2),
//     }
    return Ok(PS2Device::Keyboard);
}