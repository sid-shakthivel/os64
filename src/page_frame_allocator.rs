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

#[derive(Debug)]
pub struct Frame {
    pub test: u32,
    pub next_frame: Option<*mut Frame>,
}

pub struct Stack {
    pub current: Option<*mut Frame>,
    pub length: u32,
    pub number: u32,
}

pub struct SimplePageFrameAllocator {
    memory_start: u32,
    memory_end: u32,
    pub free_frames: *mut Stack,
    pub current_page: *mut u32,
}

/*
Basic operations any data structure should be able to do
Can be used when building vectors, stacks, queues, etc
*/
pub trait Operations<T> {
    fn is_empty(&self) -> bool;
    fn push(&mut self, node: T);
    fn pop(&mut self) -> Option<T>;
}

impl<T> Operations<T> for Stack {
    fn is_empty(&self) -> bool {
        self.length == 0
    }

    fn push(&mut self, node: T) {
        node.next_frame = self.current;
        self.current = Some(node);
    }

    fn pop(&mut self) -> Option<T> {
        let old_current = self.current.clone();
        self.current = Some(self.current.next_frame);
        some(old_current)
    }
}

impl Frame {
    pub fn new() -> Frame {
        Frame { test: 45, next_frame: None }
    }
}

pub trait FrameAllocator {
    fn alloc_frame(&mut self) -> Frame;
    fn free_frame(&mut self, frame: Frame);
}

impl SimplePageFrameAllocator {
    pub fn new(mut memory_start: u32, memory_end: u32) -> SimplePageFrameAllocator {
        memory_start += 4096;
        SimplePageFrameAllocator { memory_start: memory_start, memory_end: memory_end, current_page: unsafe { &mut *(memory_start as *mut u32) }, free_frames: unsafe { &mut *(memory_start as *mut Stack) } }
    }

    /*
    This test function will setup a simple stack at a variety of page locations from an address
    */
    pub fn setup_stack(&mut self) {
        unsafe {
            (*self.free_frames).length = 0;
            (*self.free_frames).number = 0;
            (*self.free_frames).current = None;

            let test = &mut *(self.current_page as *mut Frame);
            test.test = 10;
            test.next_frame = None;
            (*self.free_frames).current = Some(test);

            self.current_page = self.current_page.offset(1024); // 4096 / 4 as this is a 32 bit pointer

            let best = &mut *(self.current_page as *mut Frame);
            best.test = 15;
            best.next_frame = (*self.free_frames).current;
            (*self.free_frames).current = Some(best);
        }
    }
}

