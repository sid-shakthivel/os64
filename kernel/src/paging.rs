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

#![allow(dead_code)]
#![allow(unused_variables)]

use crate::print;
use crate::vga_text::TERMINAL;
use crate::page_frame_allocator;
use page_frame_allocator::PageFrameAllocator;
use crate::page_frame_allocator::FrameAllocator;
use core::prelude::v1::Some;

#[allow(dead_code)]
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

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Page {
    pub entry: u64
}

impl Page {
    pub fn new(physical_address: u64) -> Page {
        Page { entry: (0x000fffff_fffff000 & (physical_address) | (1 << 0) | (1 << 1))  }
    }

    fn set_flag(&mut self, flag: Flags) {
        self.entry |= 1 << match flag {
            Flags::Present => 0,
            Flags::Writable => 1,
            Flags::UserAccessible => 2,
            Flags::WriteThrough => 3,
            Flags::DisableCache => 4,
            Flags::Dirty => 5,
            Flags::Huge => 6,
            Flags::Global => 7
        };
    }

    pub fn is_unused(&self) -> bool {
        return self.entry == 0;
    }

    pub fn set_unused(&mut self) {
        self.entry = 0;
    }

    pub fn get_physical_address(&self) -> *mut u64 {
        let p_address = (self.entry & 0x000fffff_fffff000);
        return p_address as *mut u64;
    }

    pub fn entry(&self) -> u64 {
        return self.entry & 0x000fffff_fffff000;
    }
}

#[repr(C, packed)]
pub struct Table {
    pub entries: [Page; 512]
}

impl Table {
    // When unmapping a page, if there are no other entries, the table can be freed from memory
    fn drop_table(&mut self, allocator: &mut PageFrameAllocator) {
        let i = 0;
        for i in 0..self.entries.len() {
            if self.entries[i].entry != 0 {
                break
            }
        }
        
        // If there are 512 empty entries, the table may be freed
        if i == 512 {
            print!("Dropping table\n");
            allocator.free_frame(self as *const _ as *mut u64);
        }
    }

    /*
    When mapping an address, new tables may have to be created if there are none for a certain memory address
    If there is no table, it is created and then returned
    */
    fn create_next_table(&mut self, index: usize, allocator: &mut PageFrameAllocator) -> &mut Table {
        if self.get_table(index).is_none() {
            print!("building new table\n");
            let page_frame = allocator.alloc_frame();
            self.entries[index] = Page::new(page_frame.unwrap() as u64);
        } 
        return self.get_table(index).expect("why not working");
    }

    // A reference to the actual table is needed instead of an address
    fn get_table<'a>(&'a mut self, index: usize) -> Option<&'a mut Table> {
        return self.get_table_address(index).map(|address| unsafe { &mut *(address as *mut _) });
    }

    // Each table from the level of hierarchy can be accessed via this formula: next_table_address = (table_address << 9) | (index << 12) in which index refers to certain sections of a virtual address
    fn get_table_address(&mut self, index: usize) -> Option<u64> {
        if self.entries[index].entry & 1 > 0 {
            return Some(((self as *const _ as u64) << 9) | ((index as u64) >> 12));
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

    TODO: make these methods not global functions
*/
fn get_p4_index(virtual_address: u64) -> usize {
    return ((virtual_address >> 39) & 0x1ff) as usize;
}

fn get_p3_index(virtual_address: u64) -> usize {
    return ((virtual_address >> 30) & 0x1ff) as usize;
}

fn get_p2_index(virtual_address: u64) -> usize {
    return ((virtual_address >> 21) & 0x1ff) as usize;
}

fn get_p1_index(virtual_address: u64) -> usize {
    return ((virtual_address >> 12) & 0x1ff) as usize;
}

// The index from the address is used to go to or create tables
// TODO: swap is_user bool for enum
pub fn map_page(physical_address: u64, virtual_address: u64, allocator: &mut PageFrameAllocator, optional_p4: Option<*mut Table>, is_user: bool) {   
    assert!(virtual_address < 0x0000_8000_0000_0000 || virtual_address >= 0xffff_8000_0000_0000, "invalid address: 0x{:x}", virtual_address);

    let mut p4 = unsafe { &mut *P4 };

    if optional_p4.is_none() == false {
        p4 = unsafe { &mut *(optional_p4.unwrap()) };
    }

    let p3 = p4.create_next_table(get_p4_index(virtual_address),  allocator);
    let p2 = p3.create_next_table(get_p3_index(virtual_address), allocator);
    // let p1 = p2.create_next_table(get_p2_index(virtual_address), allocator);

    unsafe {
        let page_frame = allocator.alloc_frame();
        p2.entries[4] = Page::new(page_frame.unwrap() as u64);

        let test = p2.entries[4].entry & 0x000fffff_fffff000;

        let p1: *mut Table = test as *mut _;

        let p1_index = get_p1_index(virtual_address);
        (*p1).entries[p1_index] = Page::new(physical_address);

        // Add option for kernel only pages
        (*p1).entries[p1_index].set_flag(Flags::UserAccessible);
    }

    /*
        Translation lookaside buffer
        This buffer cashes the translation of virtual to physical addresses and needs to be updated manually
    */

    unsafe { flush_tlb(); }
}

pub fn unmap_page(virtual_address: u64, allocator: &mut PageFrameAllocator) {
    // Loop through each table and if empty drop it
    let p4 = unsafe { &mut *P4 };
    let p3 = p4.create_next_table(get_p4_index(virtual_address),  allocator);
    p3.drop_table(allocator);
    let p2 = p3.create_next_table(get_p3_index(virtual_address), allocator);
    p2.drop_table(allocator);
    let p1 = p2.create_next_table(get_p2_index(virtual_address), allocator);
    p1.drop_table(allocator);

    let frame = p1.entries[get_p1_index(virtual_address)];
    if frame.is_unused() == false {
        allocator.free_frame(frame.get_physical_address());
        p1.entries[get_p1_index(virtual_address)].set_unused();

        unsafe { flush_tlb(); }
    }
}

extern "C" {
    fn flush_tlb();
}