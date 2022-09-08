// src/lib.rs

#![no_std] // Don't link with Rust standard library
#![feature(core_ffi_c)]

mod allocator;
mod elf;
mod framebuffer;
mod fs;
mod gdt;
mod grub;
mod hashmap;
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
mod string;
mod syscalls;
mod uart;
mod vga_text;
mod writer;

extern crate multiboot2;
#[macro_use]
extern crate bitflags;
extern crate x86_64;

use crate::framebuffer::Rectangle;
use crate::framebuffer::Window;
use crate::framebuffer::DESKTOP;
use crate::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::pic::PICS;
use crate::pit::PIT;
use crate::uart::CONSOLE;
use core::f32::consts::FRAC_1_PI;
use core::panic::PanicInfo;
use framebuffer::FRAMEBUFFER;
use list::Stack;
use multiboot2::load;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    interrupts::disable();

    let boot_info = unsafe { load(multiboot_information_address as usize).unwrap() };
    PAGE_FRAME_ALLOCATOR.lock().init(&boot_info);
    PAGE_FRAME_ALLOCATOR.free();

    uart::init();

    gdt::init();
    PIT.lock().init();
    ps2::init().unwrap();
    interrupts::init();
    PICS.lock().init();

    grub::bga_set_video_mode();

    grub::initialise_userland(&boot_info);

    // framebuffer::init(boot_info.framebuffer_tag().unwrap());

    // setup_wm();

    print_serial!("Execution Finished\n");

    interrupts::enable();

    loop {}
}

fn setup_wm() {
    let window1 = Window::new(
        "Notepad",
        10,
        10,
        300,
        300,
        Some(DESKTOP.lock()),
        framebuffer::WINDOW_BACKGROUND_COLOUR,
    );
    DESKTOP.free();

    let window2 = Window::new(
        "Terminal",
        150,
        150,
        300,
        300,
        Some(DESKTOP.lock()),
        framebuffer::WINDOW_BACKGROUND_COLOUR,
    );
    DESKTOP.free();

    DESKTOP.lock().add_sub_window(window2);
    DESKTOP.free();

    // DESKTOP.lock().add_sub_window(window1);
    // DESKTOP.free();

    DESKTOP.lock().paint(Stack::<Rectangle>::new(), true);
    DESKTOP.free();
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print_serial!("Error: {}", info);
    loop {}
}
