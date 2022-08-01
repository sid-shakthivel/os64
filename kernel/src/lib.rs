// src/lib.rs

#![no_std] // Don't link with Rust standard library
// #![feature(const_mut_refs)]

// mod vga_text;
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
mod syscalls;
mod elf;
mod fs;
mod framebuffer;
mod uart;
mod ps2;
mod writer;
mod mouse;

extern crate multiboot2;
#[macro_use]
extern crate bitflags;
extern crate x86_64;

use core::panic::PanicInfo;
use crate::framebuffer::{DESKTOP};
use crate::uart::CONSOLE;
use crate::pic::PICS;
use crate::pit::PIT;

use multiboot2::{load};

// TODO: Rectify fact that available memory starts after framebuffer

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    interrupts::disable();
    
    let boot_info = unsafe { load(multiboot_information_address as usize).unwrap() };
    let mut pf_allocator = page_frame_allocator::PageFrameAllocator::new(multiboot_information_address);    
    
    // if boot_info.framebuffer_tag().is_some() {
    //     framebuffer::init(boot_info.framebuffer_tag().unwrap(), &mut pf_allocator);
    // }

    uart::init();

    gdt::init();
    PIT.lock().init();
    ps2::init().unwrap();
    interrupts::init();
    PICS.lock().init();

    interrupts::enable();

    // unsafe {
    //     core::arch::asm!("int 0x00");
    // }

    // DESKTOP.lock().create_window(10, 10, 300, 200, &mut pf_allocator); // small green
    // DESKTOP.free();
    // DESKTOP.lock().create_window(100, 150, 400, 400, &mut pf_allocator); // square red
    // DESKTOP.free();
    // DESKTOP.lock().create_window(200, 100, 200, 600, &mut pf_allocator); // long yellow
    // DESKTOP.free();
    // DESKTOP.lock().paint(); 
    // DESKTOP.free();

    // grub::initialise_userland(&boot_info, &mut pf_allocator);

    print_serial!("End of execution\n");

    // fs::init(multiboot_information_address, &mut page_frame_allocator);
    // grub::initialise_userland(multiboot_information_address, &mut page_frame_allocator);
    // bga_set_video_mode();

    loop {}
}

// fn write_bga_register(index: u16, value: u16) {
//     unsafe {
//         outpw_raw(0x01CE, index);
//         outpw_raw(0x01CF, value)
//     }
// }

// fn bga_set_video_mode() {
//     write_bga_register(4, 0);
//     write_bga_register(1, 1024);
//     write_bga_register(2, 768);
//     write_bga_register(3, 0x20);
//     write_bga_register(4, 1);
//     write_bga_register(5, 0x40 | 0x1); // Linear frame buffer
// }

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print_serial!("Error: {}", info);
    loop {}
}