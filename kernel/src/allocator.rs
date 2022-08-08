// allocator.rs

/*
    Contains implementations for malloc, free, brk, etc
    TODO: Implement some version of brk, sbrk, syscalls etc
    Uses a free list allocator, which traverses a list of memory blocks until it finds a block which can fit the size
*/

use crate::{
    list::{Node, Stack},
    page_frame_allocator::{self, FrameAllocator, PAGE_FRAME_ALLOCATOR},
    paging, print_serial,
    spinlock::Lock,
    CONSOLE,
};

/*
   +--------+------+-------+
   | Header | Data | Align |
   +--------+------+-------+
*/

#[derive(Clone, Debug)]
struct MemoryBlock {
    is_free: bool, // Indicates whether the memory block is available to be used
    size: u64,
    data: *mut u64, // Pointer to any data which is held within TODO: make this generic?
}

impl MemoryBlock {
    fn new(data: *mut u64, size: u64) -> MemoryBlock {
        MemoryBlock {
            is_free: true,
            size,
            data,
        }
    }

    unsafe fn get_header_address(&self) -> *mut u64 {
        let size = core::mem::size_of::<Node<MemoryBlock>>();
        return self.data.offset(-1 * (size / 8) as isize);
    }
}

static MEMORY_BLOCK_LIST: Lock<Stack<MemoryBlock>> = Lock::new(Stack::<MemoryBlock>::new());

/*
    Recives the size of variables in bytes which are to be used
    Returns pointer to data region
*/
pub fn malloc(mut size: u64) -> Option<*mut u64> {
    // If size is greater then 1 page, allocate multiple pages straight through PFA
    if size > 4096 {}

    // Must align block
    size = align(size);

    print_serial!("SIZE IS {}\n", size);

    let (index, wrapped_memory_block) = find_first_fit(size);

    match wrapped_memory_block {
        Some(memory_block) => {
            // If block is larger then memory required, split region and add parts to list
            // WARNING: Size's aren't calculated exactly so there may be times in which this doesn't work effectively
            if memory_block.size > size {
                // Remove old memory block
                MEMORY_BLOCK_LIST.lock().remove_at(index as usize);
                MEMORY_BLOCK_LIST.free();

                // Create new memory block for malloc'd memory
                let mut address = unsafe { memory_block.get_header_address() };

                let size_of_node = core::mem::size_of::<Node<MemoryBlock>>();
                let dp_address = unsafe { address.offset((size_of_node / 8) as isize) };

                let new_memory_block = MemoryBlock::new(dp_address, size);

                MEMORY_BLOCK_LIST
                    .lock()
                    .push(address as u64, new_memory_block);
                MEMORY_BLOCK_LIST.free();

                // Add remaining section of block
                address = unsafe { dp_address.offset((size / 8) as isize) };

                let dp_address = unsafe { address.offset((size_of_node / 8) as isize) };

                let new_memory_block = MemoryBlock::new(dp_address, (memory_block.size - size));

                MEMORY_BLOCK_LIST
                    .lock()
                    .push(address as u64, new_memory_block);
                MEMORY_BLOCK_LIST.free();
                
            } else {
                return Some(memory_block.data);
            }
        }
        None => {
            // No memory blocks can be found, thus must allocate more memory
            extend_memory_region();

            return malloc(size);
        }
    }
    MEMORY_BLOCK_LIST.free();

    return None;
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
    for (i, memory_block) in MEMORY_BLOCK_LIST.lock().into_iter().enumerate() {
        if memory_block.unwrap().payload.is_free && memory_block.unwrap().payload.size > size {
            return (i as u64, Some(memory_block.unwrap().payload.clone()));
        }
    }
    MEMORY_BLOCK_LIST.free();
    return (0, None);
}

// Extends accessible memory region of kernel heap by another page (4096 bytes)
// WARNING: May want to expand when having a functional userspace
pub fn extend_memory_region() {
    // Allocate another page
    let address = PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap();
    PAGE_FRAME_ALLOCATOR.free();

    print_serial!("ADDRESS OF MEMORY {:p}\n", address);

    // Calculate data pointer address
    let size = core::mem::size_of::<Node<MemoryBlock>>();
    // Divide by 8 as u64 is 8 bytes
    let dp_address = unsafe { address.offset((size / 8) as isize) };

    print_serial!("Size of memory is {}\n", size);
    print_serial!("DATA ADDRESS is {:p}\n", dp_address);

    let new_memory_block = MemoryBlock::new(dp_address, page_frame_allocator::PAGE_SIZE as u64);

    // Add to linked list
    MEMORY_BLOCK_LIST
        .lock()
        .push(address as u64, new_memory_block);
    MEMORY_BLOCK_LIST.free();
}
