// src/page_frame_allocator.rs

/*
For paging, physical memory is split into 4096 byte chunks and these are physical pages
We need a system in order to fetch and free these pages for different processes (user and kernel)
It returns the physical start address of a page frame
A stack of free pages along with a pointer to the first page will be used in order to keep track of pages
*/

use core::prelude::v1::Some;

/*
Each frame must store a reference to the next frame along with physical data
*/
struct Frame {
    test: u32,
    next_frame: Option<*mut Frame>,
}

struct Stack {
    head: Option<*mut Frame>,
    length: u32,
}

struct SimplePageFrameAllocator {
    memory_start: u32,
    memory_end: u32,
    free_frames: *mut Stack,
    current_page: *mut u32,
}

/*
Basic operations any data structure should be able to do
Can be used when building vectors, stacks, queues, etc
*/
pub trait Operations {
    fn is_empty(&self) -> bool;
    fn push(&mut self, node: impl Operations);
    // fn pop(&mut self) -> impl Operations;
}

// impl Operations for Frame {
//     fn push(&mut self, node: Frame) {
//         self.next_frame = Some(&mut node);
//     }

//     fn is_empty(&self) -> bool {

//     }

//     fn pop(&mut self) -> Frame {

//     }
// }

impl Frame {
    pub fn new() -> Frame {
        Frame { test: 45, next_frame: None }
    }
}

impl Stack {
    
}

// 0x100000 - Stack should be stored here hopefully?

pub trait FrameAllocator {
    fn alloc_frame(&mut self) -> Frame;
    fn free_frame(&mut self, frame: Frame);
}

impl SimplePageFrameAllocator {
    pub fn new(memory_start: u32, memory_end: u32) -> SimplePageFrameAllocator {
        SimplePageFrameAllocator { memory_start: memory_start + 4096, memory_end: memory_end, current_page: unsafe { &mut *(memory_start as *mut u32) }, free_frames: unsafe { &mut *(memory_start as *mut Stack) } }
    }
}

