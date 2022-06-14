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

const GDT_MAX_DESCRIPTORS: usize = 3;

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
pub static mut GDT: [u64; GDT_MAX_DESCRIPTORS] = [0; GDT_MAX_DESCRIPTORS];

fn get_descriptor(is_code: bool, is_present: bool, is_64: bool) -> u64 {
    let mut descriptor = 0;
    if is_code == true {
        descriptor |= (1<<43);
        descriptor |= (1<<44);
    }

    if is_present == true {
        descriptor |= (1<<47);
    }

    if is_64 == true {
        descriptor |= (1<<53);
    }

    return descriptor;
}

pub fn init() {
    unsafe {
        asm!("cli"); // Disable interrupts

        // TODO: Make sure these are accurate
        // GDT[0].edit(0, 0, 0, 0);
        // GDT[1].edit(0, 0xFFFFF, 0b10011010, 0xA); // Kernel Code Segment
        // GDT[2].edit(0, 0xFFFFF, 0b10010010, 0b0010); // Kernel Data Segment
        // GDT[3].edit(0, 0xFFFFF, 0b11111010, 0b0010); // User Code Segment
        // GDT[5].edit(0, 0xFFFFF, 0b11110010, 0b0010); // User Data Segment

        GDT[1] = get_descriptor(true, true, true);
        GDT[2] = get_descriptor(false, true, true);

        print!("{:x}", GDT[1]);
        print!("{:x}", GDT[2]);

        // Set gdtr
        let gdt_address = (&GDT[0] as *const u64) as u64;
        GDTR.size = (size_of::<u64>() as u16) * (GDT_MAX_DESCRIPTORS as u16) - 1;
        GDTR.offset = gdt_address;



        gdt_flush();
    } 
}

extern "C" {
    fn gdt_flush();
}