#[warn(non_camel_case_types)]

// src/grub.rs

/*
    Grub loads a number of modules into certain memory locations which need to be mapped into user pages
    These modules serve as user programs which will be embellished later
*/

use crate::page_frame_allocator::PageFrameAllocator;
use crate::vga_text::TERMINAL;
use crate::multitask;
use crate::print;
use multiboot2::load;
use core::mem;
use crate::elf;

pub fn initialise_userland(multiboot_information_address: usize, page_frame_allocator: &mut PageFrameAllocator) {
    let boot_info = unsafe { load(multiboot_information_address as usize).unwrap() };

    for module in boot_info.module_tags() {
        elf::parse(module.start_address() as u64, page_frame_allocator);
        let user_process = multitask::Process::init(multitask::ProcessPriority::High, page_frame_allocator);

        // Add process to list of processes
        multitask::PROCESS_SCHEDULAR.lock().add_process(user_process);
        multitask::PROCESS_SCHEDULAR.free();
    }
}

    