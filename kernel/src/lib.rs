// src/lib.rs

#![no_std] // Don't link with Rust standard library
#![feature(const_mut_refs)]

mod vga_text;
mod page_frame_allocator;
mod paging;
mod interrupts;
mod ports;
mod pic;
mod keyboard;
mod pit;
mod gdt;
mod multitask;
mod spinlock;
mod grub;

extern crate multiboot2;

use core::panic::PanicInfo;
use crate::vga_text::TERMINAL;
use crate::pic::PICS;
use crate::pit::PIT;
use crate::multitask::PROCESS_SCHEDULAR;
use crate::multitask::ProcessType;
use crate::multitask::ProcessPriority;
use crate::pic::PicFunctions;
use core::arch::asm;
use crate::paging::Table;
use crate::page_frame_allocator::FrameAllocator;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    TERMINAL.lock().clear();    

    let mut page_frame_allocator = page_frame_allocator::PageFrameAllocator::new(multiboot_information_address);    

    // gdt::init();
    PIT.lock().init();
    PICS.lock().init();
    interrupts::init();
    let mapped_module = grub::map_modules(multiboot_information_address, &mut page_frame_allocator).unwrap();

    let user_process = multitask::Process::init(mapped_module, ProcessType::User, ProcessPriority::High, 0, &mut page_frame_allocator);

    unsafe {
        switch_process(user_process.rsp, user_process.cr3);
    }

    // interrupts::enable();
    // PICS.lock().set_mask(0x20);

    loop {}
}

// This is an example process func - will eventually be embellished
pub fn process_a() {
    print!("From task 1\n");
    loop{}
}

// This is an example process func - will eventually be embellished
pub fn process_b() {
    print!("From task 2\n");
    loop{}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print!("Error: {}", info);
    loop {}
}

extern "C" {
    fn switch_process(rsp: *const u64, p4: *const Table);
}

// Bochs magic breakpoint is xchg bx, bx