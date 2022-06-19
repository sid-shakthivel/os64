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
use crate::pic::PICS;
use crate::pit::PIT;
use crate::multitask::PROCESS_SCHEDULAR;
use crate::pic::PicFunctions;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    TERMINAL.lock().clear();        
    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };

    let mut page_frame_allocator = page_frame_allocator::PageFrameAllocator::new(multiboot_information_address);    

    for module in boot_info.module_tag() {
        print!("Module 1: {:?}\n", module);

        let ptr = module.start_address() as *const ();
        let code: fn() = unsafe { core::mem::transmute(ptr) };

        PROCESS_SCHEDULAR.lock().create_process(multitask::ProcessType::Kernel, code, &mut page_frame_allocator);
        PROCESS_SCHEDULAR.free();
    }

    // TODO: Fix GDT
    // gdt::init(); 
    PIT.lock().init();
    PICS.lock().init();
    interrupts::init();

    PROCESS_SCHEDULAR.lock().create_process(multitask::ProcessType::Kernel, process_b, &mut page_frame_allocator);
    PROCESS_SCHEDULAR.free();

    interrupts::enable();
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
