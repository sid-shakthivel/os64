// src/gdt.rs

/*
    Global descriptor table which contains entries about memory segments
    It's pointed to by value in GDTR register
    Entries are 8 bytes
    TODO: Include details on how GDT entries are formatted
*/

use core::arch::asm;
use core::mem::size_of;
use crate::print;
use crate::vga_text::TERMINAL;

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

const GDT_MAX_DESCRIPTORS: usize = 5;

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

#[no_mangle]
pub static mut GDTR: gdtr = gdtr { offset: 0, size: 0 };
pub static mut GDT: [gdt_entry; GDT_MAX_DESCRIPTORS] = [gdt_entry { limit_low: 0, base_low: 0, base_middle: 0, access_byte: 0, attributes: 0, base_high: 0 }; GDT_MAX_DESCRIPTORS];

pub fn init() {
    unsafe {
        asm!("cli"); // Disable interrupts

        // TODO: Make sure these are accurate
        GDT[0].edit(0, 0, 0, 0);
        GDT[1].edit(0, 0xFFFFF, 0b10011010, 0xA); // Kernel Code Segment
        GDT[2].edit(0, 0xFFFFF, 0b10010010, 0xC); // Kernel Data Segment
        GDT[3].edit(0, 0xFFFFF, 0b11111010, 0xA); // User Code Segment
        GDT[4].edit(0, 0xFFFFF, 0b11110010, 0xC); // User Data Segment

        // Set gdtr
        let gdt_address = (&GDT[0] as *const gdt_entry) as u64;
        GDTR.size = (size_of::<gdt_entry>() as u16) * (GDT_MAX_DESCRIPTORS as u16) - 1;
        GDTR.offset = gdt_address;

        gdt_flush();
    } 
}

extern "C" {
    fn gdt_flush();
}