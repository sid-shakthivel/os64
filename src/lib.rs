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

extern crate multiboot2;
extern crate x86_64;

use core::panic::PanicInfo;
use crate::vga_text::TERMINAL;
use page_frame_allocator::PageFrameAllocator;
use crate::page_frame_allocator::FrameAllocator;
use crate::pic::PICS;
use crate::pit::PIT;
use core::arch::asm;
use crate::ports::outb;
use crate::multitask::PROCESS_SCHEDULAR;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    TERMINAL.lock().clear();        
    let mut page_frame_allocator = page_frame_allocator::PageFrameAllocator::new(multiboot_information_address);    

    // TODO: Fix GDT
    // gdt::init(); 
    PIT.lock().init();
    PICS.lock().init();
    interrupts::init();

    print!("Finished execution\n");

    PROCESS_SCHEDULAR.lock().create_process(multitask::ProcessType::Kernel, process_a, &mut page_frame_allocator);
    PROCESS_SCHEDULAR.free();

    PROCESS_SCHEDULAR.lock().create_process(multitask::ProcessType::Kernel, process_b, &mut page_frame_allocator);
    PROCESS_SCHEDULAR.free();

    interrupts::enable();

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
    fn find_ting();
    fn switch_process(rsp: *const u64);
}
