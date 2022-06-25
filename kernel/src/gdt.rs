// src/gdt.rs

/*
    Global descriptor table which contains entries about memory segments
    It's pointed to by value in GDTR register
    Entries are 8 bytes
*/

#![allow(dead_code)]
#![allow(unused_variables)]

use core::mem::size_of;
use crate::interrupts;
use bitflags::__impl_bitflags;
use bitflags::bitflags;

/*
    Base: 32 bit address of where segment begins
    Limit: 20 bit value of maximum address
    Access Byte:
    +---------+-----------------+-----------------+------------------+----------------+---------------+------------------------+-------------+
    |    7    |        6        |        5        |        4         |       3        |       2       |           1            |      0      |
    +---------+-----------------+-----------------+------------------+----------------+---------------+------------------------+-------------+
    | Present | Privilege level | Privilege level | Descriptor type  | Executable bit | Direction dit | Readable/writeable bit | Accessed bt |
    +---------+-----------------+-----------------+------------------+----------------+---------------+------------------------+-------------+
    Flag:
    +-------------+------+-----------+----------+
    |      3      |  2   |     1     |    0     |
    +-------------+------+-----------+----------+
    | Granularity | Size | Long Mode | Reserved |
    +-------------+------+-----------+----------+
*/

// TODO: Fix GDT

const GDT_MAX_DESCRIPTORS: usize = 6;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct gdt_entry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access_byte: u8,
    attributes: u8,
    base_high: u8
}

#[repr(C, packed)]
pub struct gdtr {
    size: u16, // Size of table subtracted by 1
    offset: u64, // Linear address of GDT
}

impl gdt_entry {
    pub fn edit(&mut self, base: u64, limit: u64, access_byte: u8, flags: u8) {
        self.limit_low = (limit & 0xFFFF) as u16;
        self.base_low = (base & 0xFFFF) as u16;
        self.base_middle = ((base >> 16) & 0xFF) as u8;
        self.access_byte = access_byte;
        self.attributes = ((limit >> 16) & 0xF) as u8;
        self.attributes |= flags & 0xF0;
        self.base_high = ((base >> 24) & 0xFF) as u8;
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct tss_64 {
    reserved0: u32,
    rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    reserved1: u64,
    ist1: u64,
    ist2: u64,
    ist3: u64,
    ist4: u64,
    ist5: u64,
    ist6: u64,
    ist7: u64,
    reserved2: u64,
    reserved3: u16,
    iomap: u16
}

impl tss_64 {
    pub const fn new() -> tss_64 {
        tss_64 {
            reserved0: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            reserved1: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            reserved2: 0,
            reserved3: 0,
            iomap: 0
        }
    }
}

#[no_mangle]
pub static mut GDTR: gdtr = gdtr { offset: 0, size: 0 };
// pub static mut GDT: [gdt_entry; GDT_MAX_DESCRIPTORS] = [gdt_entry { limit_low: 0, base_low: 0, base_middle: 0, access_byte: 0, attributes: 0, base_high: 0 }; GDT_MAX_DESCRIPTORS];
pub static mut GDT: [u64; GDT_MAX_DESCRIPTORS] = [0; GDT_MAX_DESCRIPTORS];

pub static mut TSS: tss_64 = tss_64::new();

#[no_mangle]
pub static mut TSS_OFFSET: u64 = (5 * core::mem::size_of::<u64>() as u64) | 3;

pub fn set_entry(is_code: bool, is_kernel: bool, is_present: bool) -> u64 {
    let mut entry: u64 = 0;

    if is_code {
        entry |= 1 << 43;
    }

    entry |= 1 << 44; // Descriptor type

    if is_present {
        entry |= 1 << 47;
    }

    entry |= 1 << 53; // 64 Bit

    if is_kernel == false {
        // User mode segments
        entry |= 1 << 45;
        entry |= 1 << 46;
    }
    
    return entry
}

bitflags! {
    struct DescriptorFlags: u64 {
        const CONFORMING        = 1 << 42;
        const EXECUTABLE        = 1 << 43;
        const USER_SEGMENT      = 1 << 44;
        const PRESENT           = 1 << 47;
        const LONG_MODE         = 1 << 53;
    }
}

pub fn init() {
    interrupts::disable();
    unsafe {
        GDT[0] = 0;
        GDT[1] = 0x002098000000ffff;
        GDT[2] = 0x008092000000ffff;
        GDT[3] = 0x0020f8000000ffff;
        GDT[4] = 0x0080f2000000ffff;

        unsafe {
            use bit_field::BitField;

            // Handle TSS
            let ptr = &TSS as *const _ as u64;
            let size = (size_of::<tss_64>() - 1) as u64;

            let mut low = PRESENT.bits();

            low.set_bits(16..40, ptr.get_bits(0..24));
            low.set_bits(56..64, ptr.get_bits(24..32));
            low.set_bits(0..16, size);
            low.set_bits(40..44, 0b1001);

            let mut high = 0;
            high.set_bits(0..32, ptr.get_bits(32..64));

            GDT[5] = low;
        }

        // Set gdtr
        let gdt_address = (&GDT[0] as *const u64) as u64;
        GDTR.size = (size_of::<u64>() as u16) * (GDT_MAX_DESCRIPTORS as u16) - 1;
        GDTR.offset = gdt_address;

        gdt_flush(); 
        flush_tss();
    } 
}

extern "C" {
    fn gdt_flush();
    fn flush_tss();
}