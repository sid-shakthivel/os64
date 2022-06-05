// src/lib.rs

#![no_std] // Don't link with Rust standard library

mod vga_text;
mod page_frame_allocator;
mod paging;

use core::panic::PanicInfo;
use crate::vga_text::TERMINAL;

extern crate multiboot2;
extern crate x86_64;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    TERMINAL.lock().clear();
    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let memory_end = memory_map_tag.memory_areas().last().expect("Unknown Length").length;
    let mut PAGE_FRAME_ALLOCATOR = page_frame_allocator::PageFrameAllocator::new(boot_info.end_address() as u64, memory_end as u64);    

    print!("Starting Paging\n");
    // paging::map_page(0o177777_777_123_456_123_0000, 0o177777_777_123_456_123_0000, &mut PAGE_FRAME_ALLOCATOR);
    paging::map_page(0xcafebabecafebabe, 0x0000babecafebabe, &mut PAGE_FRAME_ALLOCATOR);
    // paging::map_page(0x0000000000000001, 0xfffffffffffff001, &mut PAGE_FRAME_ALLOCATOR);
    // paging::map_page(0x0000000000000003, 0xfffffffffffff002, &mut PAGE_FRAME_ALLOCATOR);
    print!("Paging Worked!\n");
    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}