#[warn(non_camel_case_types)]

// src/grub.rs

/*
    Grub loads a number of modules into certain memory locations which need to be mapped into user pages
    These modules serve as user programs which will be embellished later
*/

use crate::page_frame_allocator::PageFrameAllocator;
use multiboot2::{BootInformation};
use crate::elf;
use crate::fs;
use crate::multitask;

const FILESYSTEM_ON: bool = false;

pub fn initialise_userland(boot_info: &BootInformation, page_frame_allocator: &mut PageFrameAllocator) {
    let mut i = 0; // TODO: Find a cleaner solution when have wifi
    for module in boot_info.module_tags() {
        // First module will be filesystem if given && constant is true
        if FILESYSTEM_ON && i == 0 { 
            fs::init(module.start_address(), page_frame_allocator); 
        }
        else if FILESYSTEM_ON {
            // Else, modules are userspace programs 
            elf::parse(module.start_address() as u64, page_frame_allocator);
            let user_process = multitask::Process::init(multitask::ProcessPriority::High, page_frame_allocator);

            // Add process to list of processes
            multitask::PROCESS_SCHEDULAR.lock().add_process(user_process);
            multitask::PROCESS_SCHEDULAR.free();   
        }
        i += 1;
    }
}

    