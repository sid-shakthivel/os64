#![no_std] // Don't link with Rust standard library

use core::panic::PanicInfo;
mod vga_text;
use crate::vga_text::TERMINAL;

#[no_mangle]
pub extern fn rust_main() {
    TERMINAL.lock().clear();
    print!("Hello World {}\n", 42);
    print!("Hello World {}\n", 42);
    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
