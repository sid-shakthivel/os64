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
mod syscalls;
mod elf;
mod filesystem;

extern crate multiboot2;
extern crate bitflags;
extern crate bit_field;
extern crate x86_64;

use core::panic::PanicInfo;
use crate::vga_text::TERMINAL;
use crate::pic::PICS;
use crate::pit::PIT;
use crate::pic::PicFunctions;
use core::arch::asm;

use multiboot2::load;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    // TODO: Fix the horrific auto formatting in vscode
    TERMINAL.lock().clear();    

    interrupts::init();
    gdt::init();
    PIT.lock().init();
    PICS.lock().init();

    print!("Hello World\n");

    let mut page_frame_allocator = page_frame_allocator::PageFrameAllocator::new(multiboot_information_address);    

    // paging::identity_map(12, &mut page_frame_allocator, None);

    filesystem::init(multiboot_information_address);

    // grub::initialise_userland(multiboot_information_address, &mut page_frame_allocator);

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print!("Error: {}", info);
    loop {}
}

// Bochs magic breakpoint is xchg bx, bx
// Dart: git clone https://kernel.googlesource.com/pub/scm/utils/dash/dash 

// https://fejlesztek.hu/create-a-fat-file-system-image-on-linux/
// https://stackoverflow.com/questions/22385189/add-files-to-vfat-image-without-mounting/29798605#29798605