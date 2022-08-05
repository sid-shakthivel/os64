#[warn(non_camel_case_types)]

// src/grub.rs

/*
    Grub loads a number of modules into certain memory locations which need to be mapped into user pages
    These modules serve as user programs which will be embellished later
*/

use multiboot2::{BootInformation};
use crate::elf;
use crate::fs;
use crate::multitask;

const FILESYSTEM_ON: bool = false;

pub fn initialise_userland(boot_info: &BootInformation) {
    let mut i = 0; // TODO: Find a cleaner solution when have wifi
    for module in boot_info.module_tags() {
        // First module will be filesystem if given && constant is true
        if FILESYSTEM_ON && i == 0 { 
            fs::init(module.start_address()); 
        }
        else if FILESYSTEM_ON {
            // Else, modules are userspace programs 
            elf::parse(module.start_address() as u64);
            let user_process = multitask::Process::init(multitask::ProcessPriority::High);

            // Add process to list of processes
            multitask::PROCESS_SCHEDULAR.lock().add_process(user_process);
            multitask::PROCESS_SCHEDULAR.free();   
        }
        i += 1;
    }
}

pub fn bga_set_video_mode() {
    write_bga_register(4, 0);
    write_bga_register(1, 1024);
    write_bga_register(2, 768);
    write_bga_register(3, 0x20);
    write_bga_register(4, 1);
    write_bga_register(5, 0x40 | 0x1); // Linear frame buffer
}

fn write_bga_register(index: u16, value: u16) {
    unsafe {
        // outpw_raw(0x01CE, index);
        // outpw_raw(0x01CF, value);
    }
}

// TODO: write up https://wiki.osdev.org/Bochs_VBE_Extensions