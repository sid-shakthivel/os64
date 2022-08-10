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

use crate::page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR};
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
    pub entry: u64,
}

impl Page {
    pub fn new(physical_address: u64) -> Page {
        let entry = (0x000fffff_fffff000 & physical_address) | 0b111;
        Page { entry: entry }
    }

    fn set_flag(&mut self, flag: Flags) {
        self.entry |= 1
            << match flag {
                Flags::Present => 0,
                Flags::Writable => 1,
                Flags::UserAccessible => 2,
                Flags::WriteThrough => 3,
                Flags::DisableCache => 4,
                Flags::Dirty => 5,
                Flags::Huge => 6,
                Flags::Global => 7,
            };
    }

    pub fn is_unused(&self) -> bool {
        return self.entry == 0;
    }

    pub fn set_unused(&mut self) {
        self.entry = 0;
    }

    pub fn get_physical_address(&self) -> *mut u64 {
        let p_address = self.entry & 0x000fffff_fffff000;
        return p_address as *mut u64;
    }
}

#[repr(C, packed)]
pub struct Table {
    pub entries: [Page; 512],
}

impl Table {
    // When unmapping a page, if there are no other entries, the table can be freed from memory
    fn drop_table(&mut self) {
        let i = 0;
        for i in 0..self.entries.len() {
            if self.entries[i].entry != 0 {
                break;
            }
        }

        // If there are 512 empty entries, the table may be freed
        if i == 512 {
            PAGE_FRAME_ALLOCATOR
                .lock()
                .free_frame(self as *const _ as *mut u64);
            PAGE_FRAME_ALLOCATOR.free();
        }
    }

    /*
    When mapping an address, new tables may have to be created if there are none for a certain memory address
    If there is no table, it is created and then returned
    */
    fn create_next_table(&mut self, index: usize) -> &mut Table {
        if self.get_table(index).is_none() {
            let page_frame = PAGE_FRAME_ALLOCATOR.lock().alloc_frame();
            PAGE_FRAME_ALLOCATOR.free();
            self.entries[index] = Page::new(page_frame as u64);
        }
        return self.get_table(index).expect("why not working");
    }

    // Return address of table
    fn get_table<'a>(&'a mut self, index: usize) -> Option<&'a mut Table> {
        if self.entries[index].entry & 1 > 0 {
            let table_address = self.entries[index].entry & 0x000fffff_fffff000;
            return unsafe { Some(&mut *(table_address as *mut _)) };
        } else {
            None
        }
    }

    /*
        A p1 address looks like this in ocal: 0o177777_777_WWWW_XXX_YYY_ZZZ
        WWW is the index for P4
        XXX is the index for P3
        YYY is the index for P2
        ZZZ is the index for P1
    */

    fn get_indexes(virtual_address: u64) -> (usize, usize, usize, usize) {
        let p1_index = ((virtual_address >> 12) & 0x1ff) as usize;
        let p2_index = ((virtual_address >> 21) & 0x1ff) as usize;
        let p3_index = ((virtual_address >> 30) & 0x1ff) as usize;
        let p4_index = ((virtual_address >> 39) & 0x1ff) as usize;
        return (p1_index, p2_index, p3_index, p4_index);
    }
}

pub const P4: *mut Table = 0xffffffff_fffff000 as *mut _;

// The index from the address is used to go to or create tables
pub fn map_page(physical_address: u64, virtual_address: u64, is_user: bool) {
    assert!(
        virtual_address < 0x0000_8000_0000_0000 || virtual_address >= 0xffff_8000_0000_0000,
        "invalid address: 0x{:x}",
        virtual_address
    );

    let p4 = unsafe { &mut *P4 };

    let (p1_index, p2_index, p3_index, p4_index) = Table::get_indexes(virtual_address);

    let p3 = p4.create_next_table(p4_index);
    let p2 = p3.create_next_table(p3_index);
    let p1 = p2.create_next_table(p2_index);

    p1.entries[p1_index] = Page::new(physical_address);

    // Translation lookaside buffer - cashes the translation of virtual to physical addresses and needs to be updated manually
    unsafe {
        flush_tlb();
    }
}

pub fn unmap_page(virtual_address: u64) {
    // Loop through each table and if empty drop it
    let p4 = unsafe { &mut *P4 };

    let (p1_index, p2_index, p3_index, p4_index) = Table::get_indexes(virtual_address);

    let p3 = p4.create_next_table(p4_index);
    p3.drop_table();
    let p2 = p3.create_next_table(p3_index);
    p2.drop_table();
    let p1 = p2.create_next_table(p2_index);
    p1.drop_table();

    let frame = p1.entries[p1_index];
    if frame.is_unused() == false {
        PAGE_FRAME_ALLOCATOR
            .lock()
            .free_frame(frame.get_physical_address());
        PAGE_FRAME_ALLOCATOR.free();
        p1.entries[p1_index].set_unused();

        unsafe {
            flush_tlb();
        }
    }
}

/*
    Identity maps a specified amount of megabytes from address 0
    Usage identity_map(16) would identity map the first 16 MB of memory
*/
pub fn identity_map(megabytes: u64) {
    for address in 0..(megabytes * 256) {
        map_page(address * 4096, address * 4096, true);
    }
}

pub fn identity_map_from(physical_address: u64, virtual_address: u64, megabytes: u64) {
    for address in 0..(megabytes * 256) {
        let p_address = physical_address + (address * 4096);
        let v_address = virtual_address + (address * 4096);
        map_page(p_address, v_address, false);
    }
}

// Creates a deep clone of the paging system
pub fn deep_clone() -> *mut Table {
    unsafe {
        let p4 = &mut *P4;
        let new_p4: *mut Table = PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as *mut _;
        PAGE_FRAME_ALLOCATOR.free();
        for i in 0..(*p4).entries.len() - 1 {
            if (*p4).entries[i].entry != 0 {
                let new_p3: *mut Table = PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as *mut _;
                PAGE_FRAME_ALLOCATOR.free();
                let p3 = p4.get_table(i).unwrap();

                for j in 0..(*p3).entries.len() {
                    if (*p3).entries[j].entry != 0 {
                        let new_p2: *mut Table =
                            PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as *mut _;
                        PAGE_FRAME_ALLOCATOR.free();
                        let p2 = p3.get_table(j).unwrap();

                        for k in 0..(*p2).entries.len() {
                            if (*p2).entries[k].entry != 0 {
                                let new_p1: *mut Table =
                                    PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as *mut _;
                                PAGE_FRAME_ALLOCATOR.free();
                                let p1 = p2.get_table(k).unwrap();

                                for l in 0..(*p1).entries.len() {
                                    if (*p1).entries[l].entry != 0 {
                                        (*new_p1).entries[l] =
                                            Page::new((*p1).entries[l].entry as u64);
                                    }
                                }

                                (*new_p2).entries[k] = Page::new(new_p1 as u64);
                            }
                        }

                        (*new_p3).entries[j] = Page::new(new_p2 as u64);
                    }
                }

                (*new_p4).entries[i] = Page::new(new_p3 as u64);
            }

            (*new_p4).entries[511] = Page::new(new_p4 as u64); // Recursive mapping
        }
        return new_p4;
    }
}

extern "C" {
    fn flush_tlb();
}
