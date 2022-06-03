// src/lib.rs

#![no_std] // Don't link with Rust standard library

use core::panic::PanicInfo;
mod vga_text;
mod page_frame_allocator;
use crate::vga_text::TERMINAL;

extern crate multiboot2;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    TERMINAL.lock().clear();
    // let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    // let memory_map_tag = boot_info.memory_map_tag()
    //     .expect("Memory map tag required\n");
    // print!("memory areas:\n");
    // for area in memory_map_tag.memory_areas() {
    //     print!("    start: 0x{:x}, length: 0x{:x}\n", area.base_addr, area.length);
    // }

    unsafe {
        let mut stack_page_frame_alloc = page_frame_allocator::SimplePageFrameAllocator::new(0x120000, 0x7ee0000);
        print!("First Page: {:p}\n", stack_page_frame_alloc.current_page);
        stack_page_frame_alloc.setup_stack();
        print!("Second Page: {:p}\n", stack_page_frame_alloc.current_page);
        match (*stack_page_frame_alloc.free_frames).current {
            Some(test) => {
                print!("{:p}\n", test);
                match (*test).next_frame {
                    Some(best) => {
                        print!("{:p}\n", best);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    // let test = &mut *(0x120000 as *mut u32);
    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}
