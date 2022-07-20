// src/lib.rs

#![no_std] // Don't link with Rust standard library
// #![feature(const_mut_refs)]

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
mod syscalls;
mod elf;
mod fs;
mod framebuffer;

extern crate multiboot2;
extern crate bitflags;
extern crate bit_field;
extern crate x86_64;

use core::panic::PanicInfo;
use crate::vga_text::TERMINAL;
use crate::framebuffer::TEST_TERMINAL;
use crate::pic::PICS;
use crate::pit::PIT;

use multiboot2::{load};

#[derive(Debug, Clone, Copy)]
struct PsfFont {
    magic: u32,
    version: u32, // Usually 0
    headersize: u32, // Offset of bitmaps
    flags: u32,
    glymph_num: u32, 
    bytes_per_glyph: u32, // Size
    height: u32, // In pixels
    width: u32 // In pixels
}


#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    let boot_info = unsafe { load(multiboot_information_address as usize).unwrap() };

    // TERMINAL.lock().clear();    

    // interrupts::init();
    // gdt::init();
    // PIT.lock().init();
    // PICS.lock().init();

    let mut page_frame_allocator = page_frame_allocator::PageFrameAllocator::new(multiboot_information_address);    

    // paging::identity_map(12, &mut page_frame_allocator, None);

    // fs::init(multiboot_information_address, &mut page_frame_allocator);

    // grub::initialise_userland(multiboot_information_address, &mut page_frame_allocator);
    // bga_set_video_mode();

    framebuffer::init(boot_info.framebuffer_tag().unwrap(), &mut page_frame_allocator);

    TEST_TERMINAL.lock().put_char('A');

    loop {}
}

fn write_pixel(pitch: u32, bpp: u32, x: u32, y: u32, color: u32) {
    let offset = (0x180000 + (y * pitch) + ((x * bpp) / 8)) as *mut u32;
    unsafe { *offset = color; }
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
    print!("Error: {}", info);
    loop {}
}