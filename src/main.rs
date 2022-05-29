// src/main.rs

#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
mod vga_text;
use crate::vga_text::TERMINAL;

#[panic_handler] /// This function is called on panic.
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    loop {}
}

#[no_mangle] // doesn't change the name when linking
pub extern "C" fn _start() -> ! {
    print!("Hello World {}'n");
    loop {}
}

