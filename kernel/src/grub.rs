// src/grub.rs

/*
    Grub 2 (GNU bootloader) is a bootloader which uses a header file to configure options
    Grub loads a number of modules(user programs) into certain memory locations which need to be mapped into user pages
    Grub emulates VGA card
    BGA (Bochs Graphic Updator) is accessible via 2 ports (index, data) in which it's possible to enable/disable VBE extentions
    Includes changing screen resolution, dit depth | Latest version is 0xB0C5
*/

#![allow(unused_variables)]

use crate::elf;
use crate::framebuffer;
use crate::fs;
use crate::multitask;
use crate::page_frame_allocator::FrameAllocator;
use crate::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::ports::inpw;
use crate::ports::outpw;
use crate::{print_serial, CONSOLE};
use multiboot2::BootInformation;

const FILESYSTEM_ON: bool = true;
const VBE_DISPI_IOPORT_INDEX: u16 = 0x01CE;
const VBE_DISPI_IOPORT_DATA: u16 = 0x01CF;
const VBE_DISPI_INDEX_ID: u16 = 0;
const VBE_DISPI_INDEX_XRES: u16 = 1;
const VBE_DISPI_INDEX_YRES: u16 = 2;
const VBE_DISPI_INDEX_BPP: u16 = 3;
const VBE_DISPI_INDEX_ENABLE: u16 = 4;
const VBE_DISPI_INDEX_BANK: u16 = 5;
const VBE_DISPI_INDEX_VIRT_WIDTH: u16 = 6;
const VBE_DISPI_INDEX_VIRT_HEIGHT: u16 = 7;
const VBE_DISPI_INDEX_X_OFFSET: u16 = 8;
const VBE_DISPI_INDEX_Y_OFFSET: u16 = 9;
const VBE_DISPI_LFB_ENABLED: u16 = 0x40;

pub fn initialise_userland(boot_info: &BootInformation) {
    let mut i = 0;

    let mut process_index = 0; // This index determines the PID for each process
    for module in boot_info.module_tags() {
        print_serial!(
            "MODULE ADDRESS = 0x{:x} 0x{:x}\n",
            module.start_address(),
            module.module_size()
        );
        // First module will be filesystem if given and constant is true
        if FILESYSTEM_ON && i == 0 {
            fs::init(module.start_address());
        } else {
            // Else, modules are userspace programs
            elf::parse(module.start_address() as u64);

            // Alloc some pages and map them accordingly
            let heap = PAGE_FRAME_ALLOCATOR.lock().alloc_frame();
            PAGE_FRAME_ALLOCATOR.free();

            let user_process = multitask::Process::init(
                multitask::ProcessPriority::High,
                process_index,
                heap as i32,
            );

            // Add process to list of processes
            multitask::PROCESS_SCHEDULAR
                .lock()
                .add_process(user_process);
            multitask::PROCESS_SCHEDULAR.free();

            process_index += 1;
        }
        i += 1;
    }
}

pub fn bga_set_video_mode() {
    assert!(is_bga_available(), "BGA is not available");

    write_bga_register(VBE_DISPI_INDEX_ENABLE, 0x00); // To modify contents of other registers, VBE extensions must be disabled
    write_bga_register(VBE_DISPI_INDEX_XRES, framebuffer::SCREEN_WIDTH as u16);
    write_bga_register(VBE_DISPI_INDEX_YRES, framebuffer::SCREEN_HEIGHT as u16);
    write_bga_register(VBE_DISPI_INDEX_BPP, 0x20);
    write_bga_register(VBE_DISPI_INDEX_ENABLE, 0x01);
    write_bga_register(VBE_DISPI_INDEX_BANK, VBE_DISPI_LFB_ENABLED | 0x1); // Linear frame buffer
}

fn write_bga_register(index: u16, value: u16) {
    outpw(VBE_DISPI_IOPORT_INDEX, index);
    outpw(VBE_DISPI_IOPORT_DATA, value);
}

fn read_bga_register(index: u16) -> u16 {
    outpw(VBE_DISPI_IOPORT_INDEX, index);
    return inpw(VBE_DISPI_IOPORT_DATA);
}

fn is_bga_available() -> bool {
    return read_bga_register(VBE_DISPI_INDEX_ID) == 0xB0C5;
}
