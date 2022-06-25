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
extern crate bitflags;
extern crate bit_field;

use core::panic::PanicInfo;
use crate::vga_text::TERMINAL;
use crate::pic::PICS;
use crate::pit::PIT;
use crate::pic::PicFunctions;
use core::arch::asm;
use crate::paging::Table;
use crate::page_frame_allocator::FrameAllocator;


extern "C" {
    pub(crate) static __kernel_end: usize;
}

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    TERMINAL.lock().clear();    

    let mut page_frame_allocator = page_frame_allocator::PageFrameAllocator::new(multiboot_information_address);    

    gdt::init();
    PIT.lock().init();
    PICS.lock().init();
    interrupts::init();
    grub::initialise_userland(multiboot_information_address, &mut page_frame_allocator);

    // interrupts::enable();

    loop {}
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