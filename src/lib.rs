// src/lib.rs

#![no_std] // Don't link with Rust standard library

mod vga_text;
mod page_frame_allocator;
mod paging;
mod interrupts;
mod ports;
mod pic;

extern crate multiboot2;
extern crate x86_64;

use core::panic::PanicInfo;
use crate::vga_text::TERMINAL;
use page_frame_allocator::PageFrameAllocator;
use crate::page_frame_allocator::FrameAllocator;
use core::arch::asm;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    TERMINAL.lock().clear();        
    // let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    // let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    // let memory_end = memory_map_tag.memory_areas().last().expect("Unknown Length").length;

    // let mut PAGE_FRAME_ALLOCATOR = page_frame_allocator::PageFrameAllocator::new(boot_info.end_address() as u64, memory_end as u64);    

    // let mut address = PAGE_FRAME_ALLOCATOR.alloc_frame().unwrap() as u64;
    // paging::map_page(address, 0x0000000000010000, &mut PAGE_FRAME_ALLOCATOR);

    pic::init_pic();
    interrupts::init_idt();

    // Triggering test interrupts
    unsafe { asm!("sti"); }
    print!("Finished execution\n");
    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}