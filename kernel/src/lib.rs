// src/lib.rs

#![no_std] // Don't link with Rust standard library
#![feature(associated_type_bounds)] // Magic which makes the page frame allocator work
#![feature(generic_associated_types)]
#![feature(const_option)]
#![feature(const_mut_refs)]

mod allocator;
mod elf;
mod framebuffer;
mod fs;
mod gdt;
mod grub;
mod interrupts;
mod keyboard;
mod list;
mod mouse;
mod multitask;
mod page_frame_allocator;
mod paging;
mod pic;
mod pit;
mod ports;
mod ps2;
mod spinlock;
mod syscalls;
mod uart;
mod vga_text;
mod writer;

extern crate multiboot2;
#[macro_use]
extern crate bitflags;
extern crate x86_64;

use crate::framebuffer::DESKTOP;
use crate::mouse::MOUSE;
use crate::page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR};
use crate::pic::PICS;
use crate::pit::PIT;
use crate::uart::CONSOLE;
use crate::vga_text::TERMINAL;
use crate::writer::Writer;
use core::panic::PanicInfo;
use multiboot2::load;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    interrupts::disable();

    let boot_info = unsafe { load(multiboot_information_address as usize).unwrap() };
    PAGE_FRAME_ALLOCATOR.lock().init(&boot_info);
    PAGE_FRAME_ALLOCATOR.free();

    // allocator::extend_memory_region();

    // uart::init();

    gdt::init();
    PIT.lock().init();
    // ps2::init().unwrap();
    interrupts::init();
    PICS.lock().init();
 
    TERMINAL.lock().clear();

    grub::initialise_userland(&boot_info);

    print_vga!("Execution finished\n");

    interrupts::enable();

    // grub::bga_set_video_mode();

    // framebuffer::init(boot_info.framebuffer_tag().unwrap());

    // DESKTOP.lock().create_window(10, 10, 300, 300);
    // DESKTOP.free();

    // DESKTOP.lock().create_window(200, 150, 400, 400);
    // DESKTOP.free();

    // let mouse_x = MOUSE.lock().mouse_x;
    // MOUSE.free();
    // let mouse_y = MOUSE.lock().mouse_y;
    // MOUSE.free();

    // DESKTOP.lock().paint(mouse_x, mouse_y);
    // DESKTOP.free();

    // fs::init(multiboot_information_address);

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print_serial!("Error: {}", info);
    print_vga!("Error: {}", info);
    loop {}
}
