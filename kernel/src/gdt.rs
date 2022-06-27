// src/gdt.rs

/*
    Global descriptor table which contains entries about memory segments
    It's pointed to by value in GDTR register
    Entries are 8 bytes
*/

#![allow(dead_code)]
#![allow(unused_variables)]

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


use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;
use x86_64::structures::gdt::SegmentSelector;

pub static mut TSS: TaskStateSegment = TaskStateSegment::new();

use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        unsafe {
            let mut gdt = GlobalDescriptorTable::new();
            let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
            let kernel_data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
            let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
            let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
            let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
            (gdt, Selectors { kernel_code_selector, kernel_data_selector, tss_selector })
        }
    };
}

struct Selectors {
    kernel_code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment, SS, DS, ES, FS, GS};
    
    GDT.0.load();
    unsafe {
        // Reload all segment registers
        CS::set_reg(GDT.1.kernel_code_selector);
        SS::set_reg(GDT.1.kernel_data_selector);
        DS::set_reg(GDT.1.kernel_data_selector);
        ES::set_reg(GDT.1.kernel_data_selector);
        FS::set_reg(GDT.1.kernel_data_selector);
        GS::set_reg(GDT.1.kernel_data_selector);
        load_tss(GDT.1.tss_selector);
    }
}
