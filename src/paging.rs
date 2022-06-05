// src/paging.rs

/*
Paging allows a kernel to map any virtual address to a physical address.
This allows different processes to use the same address space (and addresses) to different parts of memory.
This ensures different processes cannot overwrite memory nor access the memory of different processes or the kernel which ensures safety.
Page tables specify which frame an address points to.
*/

/*
Page table entries have a certain 64 bit format which looks like this:
+---------+-----------+------------------+---------------+---------------+-------+-----------+--------+-----------+------------------+-----------+------------+
|    0    |     1     |        2         |       3       |       4       |   6   |     7     |   8    |   9-11    |      12-51       |   52-62   |     63     |
+---------+-----------+------------------+---------------+---------------+-------+-----------+--------+-----------+------------------+-----------+------------+
| present | writable |  user accessible | write through | disable cache | dirty | huge page | global | available | physical address | available | no execute |
+---------+-----------+------------------+---------------+---------------+-------+-----------+--------+-----------+------------------+-----------+------------+
*/

mod vga_text;
mod page_frame_allocator;
use page_frame_allocator::PageFrameAllocator;

enum Flags {
    Present,
    Writable,
    UserAccessible,
    WriteThrough,
    DisableCache,
    Dirty,
    Huge,
    Global,
}

struct Page {
    entry: u64
}

impl Page {
    pub fn new(physical_address: u64) {
        self.entry = 0x000fffff_fffff000 & (physical_address >> 12);
    }

    fn set_flag(&mut self, flag: Flags) {
        self.virtual_address |= (1 << match flag {
            Present => 0,
            Writable => 1,
            UserAccessible => 2,
            WriteThrough => 3,
            DisableCache => 4,
            Dirty => 5,
            Huge => 6,
            Global => 7
        });
    }
}

struct Table {
    entries: [Page, 512]
}

impl Table {
    pub fn get_next_table_address(&mut self, index: u64) -> Option<*mut Table> {
        // Check if the entry is present
        if (self.entries[index] & 1) {
            Some(unsafe { &mut *((self as *const _ as u64 << 9) | (index >> 12) as *mut Table));
        } else {
            None
        }
    }

    fn create_next_table(&mut self, index: u64, allocator: &mut PageFrameAllocator) {
        if self.get_next_table_address(index).is_none() {
            // Create Table
            let page_frame = allocator.alloc_frame();
            self.entries[index] = Page::new(page_frame);
            self.entries[index].set_flag(Flags::Present);
            self.entries[index].set_flag(Flags::Writable);
        }
        return self.get_next_table_address(index);
    }
}

/*
Recursive mapping sets the last entry of P4 to itself 
Order is: Page Map Level Table(P4), Page Directory Pointer Table(P3), Page Directory Table(P2), Page Table(P1)
To access a page table (and edit it), the CPU loops twice through the P4, on second run it acts as a P3, which then points to a P2 which points to a page table entry itself
By modifying the address passed, CPU can access different parts of the paging hierarchy as different tables act as upper tables 
*/

pub const P4: *mut Table = 0xffffffff_fffff000 as *mut _;

fn get_p4_index(virtual_address: u64) {
    return (virtual_address >> 27) & 0o777;
}

fn get_p3_index(virtual_address: u64) {
    return (virtual_address >> 18) & 0o777;
}

fn get_p2_index(virtual_address: u64) {
    return (virtual_address >> 9) & 0o777;
}

fn get_p1_index(virtual_address: u64) {
    return (virtual_address >> 0) & 0o777;
}

fn map_page(physical_address: u64, virtual_address: u64, allocator: PageFrameAllocator) {
    let mut p3 = P4.create_next_table(get_p4_index(virtual_address),  &mut allocator);
    let mut p2 = p3.create_next_table(get_p3_index(virtual_address), &mut allocator);
    let mut p1 = p2.create_next_table(get_p2_index(virtual_address), &mut allocator);

    p1.entries[get_p1_index(virtual_address)] = Page::new(physical_address);
    self.entries[get_p1_index(virtual_address)].set_flag(Flags::Present);
    self.entries[get_p1_index(virtual_address)].set_flag(Flags::Writable);
}