// src/lib.rs

#![no_std] // Don't link with Rust standard library

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

extern crate multiboot2;
extern crate x86_64;

use core::panic::PanicInfo;
use crate::vga_text::TERMINAL;
use page_frame_allocator::PageFrameAllocator;
use crate::page_frame_allocator::FrameAllocator;
use crate::pic::PICS;
use crate::pit::PIT;
use core::arch::asm;
use crate::ports::outb;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    TERMINAL.lock().clear();        
    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let memory_end = memory_map_tag.memory_areas().last().expect("Unknown Length").length;

    let mut PAGE_FRAME_ALLOCATOR = page_frame_allocator::PageFrameAllocator::new(boot_info.end_address() as u64, memory_end as u64);    

    // TODO: Fix GDT
    // gdt::init(); 
    unsafe { PIT.lock().init(); }
    PICS.lock().init();
    interrupts::init();

    print!("Finished execution\n");

    unsafe {
        let address1 = process_a as *const ()as u64;
        let process1 = multitask::Process::init(address1, &mut PAGE_FRAME_ALLOCATOR, multitask::ProcessType::Kernel);

        let address2 = process_b as *const ()as u64;
        let process2 = multitask::Process::init(address2, &mut PAGE_FRAME_ALLOCATOR, multitask::ProcessType::Kernel);   
        
        multitask::a_process = Some(process1);
        multitask::b_process = Some(process2); 
    }

    interrupts::enable();

    loop {}
}

// This is an example process func - will eventually be embellished
pub fn process_a() {
    print!("From task 1\n");
    loop{}
}

// This is an example process func - will eventually be embellished
pub fn process_b() {
    print!("From task 2\n");
    loop{}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}

extern "C" {
    fn find_ting();
    fn switch_process(rsp: *const u64);
}
