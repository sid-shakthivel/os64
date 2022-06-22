// src/grub.rs

/*
    Grub loads a number of modules into certain memory locations which need to be mapped into user pages
    These modules serve as user programs which will be embellished later
*/

use crate::page_frame_allocator::FrameAllocator;
use crate::page_frame_allocator::PageFrameAllocator;
use crate::vga_text::TERMINAL;
use crate::print;

pub fn map_modules(multiboot_information_address: usize, page_frame_allocator: &mut PageFrameAllocator) -> Option<u64> {
    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };

    for module in boot_info.module_tag() {
        // let ptr = module.start_address() as *const ();
        // let code: fn() = unsafe { core::mem::transmute(ptr) };

        let module_size: isize =  (module.end_address() as isize) - (module.start_address() as isize);

        // TODO: implement method to map over multiple pages
        if module_size > 1024 {
            panic!("Module is too big and requires more then 1 page!")
        } 

        let frame = page_frame_allocator.alloc_frame().unwrap();
        let module_address = module.start_address() as *mut u64;

        unsafe {
            // Map the binary to a new user page
            for i in 0..module_size {
                *frame.offset(i) = *module_address.offset(i);
            }
        }

        let test = frame as u64;
        // print!("GRUB: {:p} {}\n", frame, test);

        // Only 1 module for now thus we can return a pointer to the new frame (if needed)
        // TODO: Implement support for multiple modules
        return Some(frame as u64);
    }
    return None;
}