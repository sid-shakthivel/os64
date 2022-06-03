// src/page_frame_allocator.rs

/*
For paging, physical memory is split into 4096 byte chunks and these are physical pages
We need a system in order to fetch and free these pages for different processes (user and kernel)
It returns the physical start address of a page frame
A stack of free pages along with a pointer to the first page will be used in order to keep track of pages
*/

use core::prelude::v1::Some;

/*
Each frame must store a reference to the next frame along with physical meta data
*/

#[derive(Debug)]
pub struct Frame {
    pub value: u32,
    pub next_frame: Option<*mut Frame>,
}

pub struct Stack {
    pub current: Option<*mut Frame>,
    pub length: u32,
}

/*
TODO: Change u32 to u8
*/
pub struct PageFrameAllocator {
    memory_start: u32,
    memory_end: u32,
    pub free_frames: *mut Stack,
    pub current_page: *mut u32,
    pub number: u32,
}

/*
Basic operations any data structure should be able to do
Can be used when building vectors, stacks, queues, etc
TODO: Switch to generics
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
        unsafe {
            (*node).next_frame = self.current;
        }
        self.current = Some(node);
    }

    fn pop(&mut self) -> Option<*mut Frame> {
        let old_current = self.current.clone();
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
    fn alloc_frame(&mut self) -> Option<*mut u32>;
    fn free_frame(&mut self, frame_address: *mut u32);
}

impl FrameAllocator for PageFrameAllocator {
    /*
    Check the stack for any known free frames and pop it
    If there are no free frames, increment the pointer to the next frame and return that
    */
    fn alloc_frame(&mut self) -> Option<*mut u32> {
        unsafe {
            if (*self.free_frames).is_empty() {
                // Current Page is a 32 bit address thus 1 page is 1024 32 bits (4096 bytes)
                self.current_page = self.current_page.offset(1024);
                return Some(self.current_page);
            } else {
                return  match (*self.free_frames).pop() {
                    Some(frame) => Some(&mut *(frame as *mut u32)),
                    _ => None
                }
            }
        }
    }

    /*
    Build a new frame struct and store it in the memory address of the freed page
    Push it to the end of the stack
    */
    fn free_frame(&mut self, frame_address: *mut u32) {
        let new_free_frame = unsafe { &mut *(frame_address as *mut Frame) };
        {
            // For testing purposes
            new_free_frame.value = self.number; 
            self.number += 5;
        }
        unsafe { (*self.free_frames).push(new_free_frame) }
    }
}

impl PageFrameAllocator {
    pub fn new(mut memory_start: u32, memory_end: u32) -> PageFrameAllocator {
        memory_start += 4096;
        PageFrameAllocator { memory_start: memory_start, memory_end: memory_end, current_page: unsafe { &mut *(memory_start as *mut u32) }, free_frames: unsafe { &mut *(memory_start as *mut Stack) }, number: 0 }
    }

    /*
    This test function will setup a simple stack at a variety of page locations from an address
    */
    pub fn setup_stack(&mut self) {
        unsafe {
            (*self.free_frames).length = 0;
            (*self.free_frames).current = None;

            // let test = &mut *(self.current_page as *mut Frame);
            // test.value = 10;
            // test.next_frame = None;
            // (*self.free_frames).current = Some(test);

            // self.current_page = self.current_page.offset(1024); // 4096 / 4 as this is a 32 bit pointer

            // let best = &mut *(self.current_page as *mut Frame);
            // best.value = 15;
            // best.next_frame = (*self.free_frames).current;
            // (*self.free_frames).current = Some(best);
        }
    }
}

