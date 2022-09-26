// src/page_frame_allocator.rs

/*
For paging, physical memory is split into 4096 byte chunks and these are physical pages
We need a system in order to fetch and free these pages for different processes (user and kernel)
It returns the physical start address of a page frame
A stack of free pages along with a pointer to the first page will be used in order to keep track of pages
*/

use crate::{list::Stack, print_serial, spinlock::Lock, CONSOLE};
use multiboot2::BootInformation;
pub struct PageFrameAllocator {
    pub free_frames: Stack<u64>,
    pub current_page: u64,
    memory_end: u64,
    page_count: u64,
}

pub trait FrameAllocator {
    fn alloc_frame(&mut self) -> *mut u64;
    fn free_frame(&mut self, frame_address: *mut u64) -> ();

    fn alloc_frames(&mut self, pages_required: u64) -> *mut u64;
    fn free_frames(&mut self, frame_address: *mut u64, pages_required: u64) -> ();
}

impl FrameAllocator for PageFrameAllocator {
    //  Allocates 1 physical page of memory
    fn alloc_frame(&mut self) -> *mut u64 {
        if self.free_frames.is_empty() {
            // If the free frames list is empty, bump the address and return that
            let address = self.current_page;
            assert!(address < self.memory_end, "KERNEL RAN OUT OF MEMORY");

            self.current_page += 4096;
            return address as *mut u64;
        } else {
            // Pop from free frames and return that address
            return self.free_frames.pop() as *mut u64;
        }
    }

    // Allocates a continuous amount of pages subsequently
    fn alloc_frames(&mut self, pages_required: u64) -> *mut u64 {
        let address = self.current_page;
        for _i in 0..pages_required {
            self.current_page += 4096;
        }

        if pages_required == 8 {
            print_serial!("0x{:x}\n", self.current_page);
        }

        return address as *mut u64;
    }

    // Frees a continuous amount of memory
    fn free_frames(&mut self, frame_address: *mut u64, pages_required: u64) {
        for i in 0..pages_required {
            unsafe { self.free_frame(frame_address.offset(i as isize)) }
        }
    }

    /*
        Frees a page of memory by adding to stack
        NOTE: Freed page isn't used by any process and thus can be safely written to
    */
    fn free_frame(&mut self, frame_address: *mut u64) {
        // Create new node which stores the page
        self.free_frames
            .push_at_address(frame_address as u64, self.page_count);
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
            round_to_nearest_page((boot_info.end_address() as u64) + 0x1000);

        let memory_end: u64 = round_to_nearest_page(
            memory_map_tag
                .memory_areas()
                .last()
                .expect("Unknown Length")
                .end_address(),
        );

        // print_serial!("MEMORY START = 0x{:x}\n", memory_start);
        // memory_start = 0xe5d000;
        memory_start = 0xf50000;

        // TODO: Fix this fix - very large modules seem to confuse the multiboot2 package

        self.current_page = memory_start;
        self.memory_end = memory_end;
    }
}

pub fn round_to_nearest_page(size: u64) -> u64 {
    ((size as i64 + 4095) & (-4096)) as u64
}

pub fn convert_bytes_to_mb(bytes: u64) -> u64 {
    bytes / 1024 / 1024
}

pub fn get_page_number(size: u64) -> u64 {
    size / (PAGE_SIZE as u64)
}

pub const PAGE_SIZE: usize = 4096;

pub static PAGE_FRAME_ALLOCATOR: Lock<PageFrameAllocator> = Lock::new(PageFrameAllocator::new());
