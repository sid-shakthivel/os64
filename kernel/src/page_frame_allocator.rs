
// src/page_frame_allocator.rs

/*
For paging, physical memory is split into 4096 byte chunks and these are physical pages
We need a system in order to fetch and free these pages for different processes (user and kernel)
It returns the physical start address of a page frame
A stack of free pages along with a pointer to the first page will be used in order to keep track of pages
*/

use core::prelude::v1::Some;
use multiboot2::load;

/*
Each frame must store a reference to the next frame along with physical meta data (value is for testing)
*/
#[derive(Debug)]
pub struct Frame {
    pub next_frame: Option<*mut Frame>,
}

pub struct Stack {
    pub current: Option<*mut Frame>,
    pub length: u64,
}

pub struct PageFrameAllocator {
    memory_end: u64,
    pub free_frames: &'static mut Stack,
    pub current_page: *mut u64,
}

/*
Basic operations any data structure should be able to do
Can be used when building vectors, stacks, queues, etc
*/
pub trait Operations {
    fn is_empty(&self) -> bool;
    fn push(&mut self, node: *mut Frame);
    fn pop(&mut self) -> Option<*mut Frame>;
}

impl Operations for Stack {
    fn is_empty(&self) -> bool {
        self.length == 0
    }

    fn push(&mut self, node: *mut Frame) {
        unsafe { (*node).next_frame = self.current; }
        self.current = Some(node);
        self.length += 1;
    }

    fn pop(&mut self) -> Option<*mut Frame> {
        let old_current = self.current.clone();
        self.length -= 1;
        unsafe {
            self.current = match self.current {
                Some(frame) => (*frame).next_frame,
                _ => None
            };
        }
        old_current 
    }
}

pub trait FrameAllocator {
    fn alloc_frame(&mut self) -> Option<*mut u64>;
    fn free_frame(&mut self, frame_address: *mut u64) -> ();
}

impl FrameAllocator for PageFrameAllocator {
    /*
    Check the stack for any known free frames and pop it
    If there are no free frames, increment the pointer to the next frame and return that
    */
    fn alloc_frame(&mut self) -> Option<*mut u64> {
        if self.free_frames.is_empty() {
            // Current Page is a 64 bit address thus 1 page is 512 64 bits (4096 bytes in a page)
            unsafe {
                if (self.current_page.offset(512) as u64) < self.memory_end {
                    self.current_page = self.current_page.offset(512);
                    return Some(self.current_page);
                } else {
                    // Ran out of memory
                    return None;
                }
            }
        } else {
            return  match self.free_frames.pop() {
                Some(frame) => unsafe { Some(&mut *(frame as *mut u64)) }, 
                _ => None
            }
        }
    }

    /*
    Build a new frame struct and store it in the memory address of the freed page
    Push it to the end of the stack
    */
    fn free_frame(&mut self, frame_address: *mut u64) {
        let new_free_frame = unsafe { &mut *(frame_address as *mut Frame) };
        self.free_frames.push(new_free_frame);
    }
}

impl PageFrameAllocator {
    pub fn new(multiboot_information_address: usize) -> PageFrameAllocator {
        let boot_info = unsafe { load(multiboot_information_address as usize).unwrap() };
        let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");  

        let memory_start: u64 = (boot_info.end_address() as u64) + (500000 as u64) & 0x000fffff_fffff000;
        let memory_end: u64 = memory_map_tag.memory_areas().last().expect("Unknown Length").end_address() & 0x000fffff_fffff000;

        let mut page_frame_allocator = PageFrameAllocator { memory_end: memory_end, current_page: unsafe { &mut *(memory_start as *mut u64) }, free_frames: unsafe { &mut *(memory_start as *mut Stack) } };
        page_frame_allocator.setup_stack();
        return page_frame_allocator;
    }

    pub fn setup_stack(&mut self) {
        self.free_frames.length = 0;
        self.free_frames.current = None;
    }
}

pub const PAGE_SIZE: usize = 4096;