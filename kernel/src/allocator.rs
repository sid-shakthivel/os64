// allocator.rs

/*
    Contains implementations for malloc, free, brk, etc
    Uses a free list allocator, which traverses a list of memory blocks until it finds a block which can fit the size
*/

use crate::{
    list::{Node, Stack},
    page_frame_allocator::{self, FrameAllocator, PAGE_FRAME_ALLOCATOR},
    spinlock::Lock,
};

// Divide by 8 as u64 is 8 bytes and a *mut u64 points to 8 bytes
const NODE_MEMORY_BLOCK_SIZE: isize = (core::mem::size_of::<Node<MemoryBlock>>() / 8) as isize;

/*
   +--------+------+-------+
   | Header | Data | Align |
   +--------+------+-------+
*/

#[derive(Clone, Debug, PartialEq)]
struct MemoryBlock {
    is_free: bool, // Indicates whether the memory block is available to be used
    size: u64,
    data: *mut u64, // Pointer to any data which is held within
}

impl MemoryBlock {
    fn new(data: *mut u64, size: u64, is_free: bool) -> MemoryBlock {
        MemoryBlock {
            is_free,
            size,
            data,
        }
    }
}

static FREE_MEMORY_BLOCK_LIST: Lock<Stack<MemoryBlock>> = Lock::new(Stack::<MemoryBlock>::new());

/*
    Recives the size of data in bytes which is to be used
    Returns pointer to data region
*/
pub fn kmalloc(mut size: u64) -> *mut u64 {
    // Must align block by 8
    size = align(size);

    let (index, wrapped_memory_block) = find_first_fit(size);

    match wrapped_memory_block {
        Some(memory_block) => {
            // If block is larger then memory required, split region and add parts to list
            if memory_block.size > size {
                // Remove old memory block
                FREE_MEMORY_BLOCK_LIST.lock().remove_at(index as usize);
                FREE_MEMORY_BLOCK_LIST.free();

                // Create new memory block for malloc'd memory
                let mut address = unsafe { get_header_address(memory_block.data) };
                let dp = create_new_memory_block(size, address, false);

                // Add remaining section of block
                address = unsafe { address.offset(NODE_MEMORY_BLOCK_SIZE + size as isize) };
                create_new_memory_block(memory_block.size - size, address, true);

                return dp;
            } else {
                return memory_block.data;
            }
        }
        None => {
            // No memory blocks can be found, thus must allocate more memory according to how many bytes needed
            let pages_required = page_frame_allocator::round_to_nearest_page(size);

            extend_memory_region(pages_required);

            return kmalloc(size);
        }
    }
}

/*
    Recives pointer to memory address
    Frees a memory region which can later be allocated
*/
pub fn kfree(dp: *mut u64) {
    let header_address = unsafe { get_header_address(dp) };
    let header = unsafe { &mut *(header_address as *mut Node<MemoryBlock>) };

    // Add node to linked list of free nodes
    header.payload.is_free = true;
    FREE_MEMORY_BLOCK_LIST
        .lock()
        .push(header_address as u64, header.payload.clone());
    FREE_MEMORY_BLOCK_LIST.free();

    FREE_MEMORY_BLOCK_LIST.free();

    // Check next node to merge memory regions together to alleviate fragmentation
    // NOTE: Since a stack is used, the node is added to the top of the stack so there is only a next 
    if let Some(next_node) = header.next {
        let next_header = unsafe { &mut *next_node };

        // Get total size of other region and update memory block
        header.payload.size += next_header.payload.size;

        // Remove other region from linked list since updated
        FREE_MEMORY_BLOCK_LIST.lock().remove(next_header);
        FREE_MEMORY_BLOCK_LIST.free();
    }
}

/*
    For faster memory access, blocks should be aligned by machine word (8 for x64)
*/
fn align(size: u64) -> u64 {
    ((size as i64 + 7) & (-8)) as u64
}

/*
    Uses First-fit algorithm
    Recieves size to determine whether a block will fit or not
    Returns first memory block which fits the size
*/
fn find_first_fit(size: u64) -> (u64, Option<MemoryBlock>) {
    for (i, memory_block) in FREE_MEMORY_BLOCK_LIST.lock().into_iter().enumerate() {
        if memory_block.unwrap().payload.is_free && memory_block.unwrap().payload.size > size {
            FREE_MEMORY_BLOCK_LIST.free();
            return (i as u64, Some(memory_block.unwrap().payload.clone()));
        }
    }
    FREE_MEMORY_BLOCK_LIST.free();
    return (0, None);
}

// Extends accessible memory region of kernel heap by another page (4096 bytes)
pub fn extend_memory_region(pages: u64) {
    // Allocate another page
    let address = PAGE_FRAME_ALLOCATOR.lock().alloc_frames(pages);
    PAGE_FRAME_ALLOCATOR.free();

    let size = (page_frame_allocator::PAGE_SIZE as u64) * pages;
    create_new_memory_block(size, address, true);
}

/*
    Create a new memory block of a certain size
    Recieves size of block and address in which to create a new block
*/
fn create_new_memory_block(size: u64, address: *mut u64, is_free: bool) -> *mut u64 {
    let dp_address = unsafe { address.offset(NODE_MEMORY_BLOCK_SIZE) };
    let new_memory_block = MemoryBlock::new(dp_address, size, is_free);

    if is_free {
        // Push to linked list
        FREE_MEMORY_BLOCK_LIST
            .lock()
            .push(address as u64, new_memory_block);
        FREE_MEMORY_BLOCK_LIST.free();
    } else {
        // Add meta data regardless
        unsafe {
            *(address as *mut MemoryBlock) = new_memory_block;
        }
    }

    return dp_address;
}

/*
    Recives pointer to data
    Returns pointer to address of header
*/
unsafe fn get_header_address(dp: *mut u64) -> *mut u64 {
    return dp.offset(-1 * (NODE_MEMORY_BLOCK_SIZE) as isize);
}
