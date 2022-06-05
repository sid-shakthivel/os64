// src/paging.rs

/*
Paging allows a kernel to map any virtual address to a physical address
This allows different processes to use the same address space (and addresses) to different parts of memory
This ensures different processes cannot overwrite memory nor access the memory of different processes or the kernel which ensures safety
Page tables specify which frame an address points to
Order is: Page Map Level Table(P4), Page Directory Pointer Table(P3), Page Directory Table(P2), Page Table(P1)

Recursive mapping sets the last entry of P4 to itself 
To access a page table (and edit it), the CPU loops twice through the P4, on second run it acts as a P3, which then points to a P2 which points to a page table entry itself
By modifying the address passed, CPU can access different parts of the paging hierarchy as different tables act as upper tables 
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
        self.entry |= (1 << match flag {
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

    pub is_unused(&self) -> bool {
        return self.entry == 0;
    }

    pub set_unused(&mut self) {
        self.entry = 0;
    }

    pub get_physical_address(&self) -> &*mut u64 {
        return self.entry & 0x000fffff_fffff000;
    }
}

struct Table {
    entries: [Page, 512]
}

impl Table {
    // When unmapping a page, if there are no other entries, the table can be freed from memory
    fn drop_table(&mut self, allocator: &mut PageFrameAllocator) {
        let mut i = ;
        for i in 0..self.entries.len() {
            if self.entries[i] != 0 {
                break
            }
        }
        
        // If there are 512 empty entries, the table may be freed
        if i != 512 {
            allocator.free_frame(self as *const _ as u64);
        }
    }

    /*
    When mapping an address, new tables may have to be created if there are none for a certain memory address
    If there is no table, it is created and then returned
    */
    fn create_next_table(&mut self, index: u64, allocator: &mut PageFrameAllocator) {
        if self.get_table(index).is_none() {
            // Create new table
            let page_frame = allocator.alloc_frame();
            self.entries[index] = Page::new(page_frame);
            self.entries[index].set_flag(Flags::Present);
            self.entries[index].set_flag(Flags::Writable);
        }
        return self.get_table(index);
    }

    // A reference to the actual table is needed instead of an address
    fn get_table(&mut self, index: u64) -> Option<&mut Table> {
        return self.next_table_address(index).map(|address| unsafe { &mut *(address as *mut _) });
    }

    // Each table from the level of hierarchy can be accessed via this formula: next_table_address = (table_address << 9) | (index << 12) in which index refers to certain sections of a virtual address
    fn get_table_address(&mut self, index: u64) -> Option<u64> {
        if (self.entries[index] & 1) {
            return self as *const _ as u64; << 8 | (index >> 12);
        } else {
            None
        }
    }
}

pub const P4: *mut Table = 0xffffffff_fffff000 as *mut _;

/*
    A p1 address looks like this in ocal: 0o177777_777_WWWW_XXX_YYY_ZZZ	
    WWW is the index for P4
    XXX is the index for P3
    YYY is the index for P2
    ZZZ is the index for P1
*/
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

// The index from the address is used to go to or create tables
fn map_page(physical_address: u64, virtual_address: u64, allocator: PageFrameAllocator) {
    let mut p3 = P4.create_next_table(get_p4_index(virtual_address),  &mut allocator);
    let mut p2 = p3.create_next_table(get_p3_index(virtual_address), &mut allocator);
    let mut p1 = p2.create_next_table(get_p2_index(virtual_address), &mut allocator);

    // If the address is not already mapped, create the mapping
    if (p1.entries[get_p1_index(virtual_address)].is_unused()) {
        p1.entries[get_p1_index(virtual_address)] = Page::new(physical_address);
        self.entries[get_p1_index(virtual_address)].set_flag(Flags::Present);
        self.entries[get_p1_index(virtual_address)].set_flag(Flags::Writable);
    }
}

fn unmap_page(virtual_address: u64, allocator: PageFrameAllocator) {
    // Loop through each table and if empty drop it
    let mut p3 = P4.create_next_table(get_p4_index(virtual_address),  &mut allocator);
    p3.drop_table();
    let mut p2 = p3.create_next_table(get_p3_index(virtual_address), &mut allocator);
    p2.drop_table();
    let mut p1 = p2.create_next_table(get_p2_index(virtual_address), &mut allocator);
    p1.drop_table();

    let frame = p1.entries[get_p1_index(virtual_address)];
    if (frame.is_unused() == false) {
        allocator.free_frame(frame.get_physical_address());
        p1.entries[get_p1_index(virtual_address)].set_unused();

        /*
        Translation lookaside buffer
        This buffer cashes the translation of virtual to physical addresses and needs to be updated manually
        */
        use x86_64::instructions::tlb;
        use x86_64::VirtualAddress;
        tlb::flush(VirtualAddress(page.start_address()));
    }
}