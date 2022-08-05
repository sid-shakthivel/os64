// src/page_frame_allocator.rs

/*
For paging, physical memory is split into 4096 byte chunks and these are physical pages
We need a system in order to fetch and free these pages for different processes (user and kernel)
It returns the physical start address of a page frame
A stack of free pages along with a pointer to the first page will be used in order to keep track of pages
*/

use crate::{list::Stack, spinlock::Lock};
use core::prelude::v1::Some;
use multiboot2::BootInformation;

pub struct PageFrameAllocator {
    pub free_frames: Stack<u64>,
    pub current_page: u64,
    memory_end: u64,
    page_count: u64,
}

pub trait FrameAllocator {
    fn alloc_frame(&mut self) -> Option<*mut u64>;
    fn free_frame(&mut self, frame_address: *mut u64) -> ();
}

impl FrameAllocator for PageFrameAllocator {
    /*
    Check the free frames stack for frames and pop it from the stack and return the address
    Else increment the address to the next frame and return that
    */
    fn alloc_frame(&mut self) -> Option<*mut u64> {
        if self.free_frames.is_empty() {
            if (self.current_page + 4096) < self.memory_end {
                self.current_page += 4096;
                return Some(self.current_page as *mut u64);
            } else {
                // Reached memory limit
                return None;
            }
        } else {
            return Some(self.free_frames.pop() as *mut u64);
        }
    }

    /*
    Store reference to frame struct in memory and add to start of stack
    Freed page isn't used by any process and thus can be safely written to
    */
    fn free_frame(&mut self, frame_address: *mut u64) {
        self.free_frames.push(frame_address as u64, self.page_count);
        self.page_count += 1;
    }
}

impl PageFrameAllocator {
    pub const fn new() -> Self {
        PageFrameAllocator {
            free_frames: Stack::<u64>::new(),
            current_page: 0,
            memory_end: 0,
            page_count: 0,
        }
    }

    pub fn init(&mut self, boot_info: &BootInformation) {
        let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
        let mut memory_start: u64 =
            (boot_info.end_address() as u64) + (500000 as u64) & 0x000fffff_fffff000;
        memory_start = 4714496 & 0x000fffff_fffff000; // TODO: Fix this temp fix (memory starts after framebuffer)

        let memory_end: u64 = memory_map_tag
            .memory_areas()
            .last()
            .expect("Unknown Length")
            .end_address()
            & 0x000fffff_fffff000;

        self.current_page = memory_start;
        self.memory_end = memory_end;
    }
}

pub static PAGE_FRAME_ALLOCATOR: Lock<PageFrameAllocator> = Lock::new(PageFrameAllocator::new()); 

pub const PAGE_SIZE: usize = 4096;