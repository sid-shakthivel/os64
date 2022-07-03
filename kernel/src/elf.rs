// src/elf.rs

/*
    Executable and Linkable Format which is used to store programs
    Linkers combine elf files into an executable or library (uses sections)
    Loaders load the file into memory (uses segments)  


    +-----------+----------------------------------+
    |   name    |             purpose              |
    +-----------+----------------------------------+
    | .text     | code                             |
    | .data     | initialised data with read/write |
    | .bss      | unitialised data                 |
    | .roadata  | initialised data with read only  |
    +-----------+----------------------------------+
*/

// TODO: Use enums instead of numbers 

use crate::vga_text::TERMINAL;
use crate::print;
use core::mem;
use crate::page_frame_allocator::PAGE_SIZE;
use crate::page_frame_allocator::PageFrameAllocator;
use crate::page_frame_allocator::FrameAllocator;
use crate::paging;

type Elf64Half = u16;
type Elf64Off = u64;
type Elf64Addr = u64;
type Elf64Word = u32;
type Elf64Xword = u64;

const ELF_DATA: u8 = 1; // Little Endian
const ELF_CLASS: u8 = 2; // 64 Bit
const ELF_VERSION: u8 = 1;
const ELF_MACHINE: Elf64Half = 0x3E; // AMD x86-64

#[repr(C, packed)]
struct ElfHeader {
    e_ident: [u8; 16], // Magic number and other info
    e_type: Elf64Half, // Object file type
    e_machine: Elf64Half, // Architecture
    e_version: Elf64Word, // Object file version
    e_entry: Elf64Addr, // Entry 
    e_phoff: Elf64Off, // Program header table file offset
    e_shoff: Elf64Off, // Section header table file offset
    e_flags: Elf64Word, // Processor-specific flags
    e_ehsize: Elf64Half, // ELF header size in bytes
    e_phentsize: Elf64Half, // Program header table entry size
    e_phnum: Elf64Half, // Program header table entry count
    e_shentsize: Elf64Half, // Section header table entry size
    e_shnum: Elf64Half, // Section header table entry count
    e_shstrndx: Elf64Half, // Section header string table index
}

enum ElfIdent {
    EiMag0 = 0, // 0x7F
    EiMag1 = 1, // E
    EiMag2 = 2, // L
    EiMag3 = 3, // F
    EiClass = 4, // Architecture
    EiData = 5, // Byte order
    EiVersion = 6, // ELF version
    EiOsabi = 7, // OS specific
    EiAbiversion = 8, // OS specific
    EiPad = 9, // Padding
}

// #[repr(u16)]
// #[derive(PartialEq, Debug, Clone, Copy)]
// enum ElfType {
//     EtNone = 0 as u16, // Unknown
//     EtRel = 1 as u16, // Relocatable
//     EtExec = 2 as u16, // Executable
// }

#[repr(C, packed)]
struct ElfSectionHeader{
    sh_name: Elf64Word,
    sh_type: Elf64Word,
    sh_flags: Elf64Xword,
    sh_addr: Elf64Addr,
    sh_offset: Elf64Off,
    sh_size: Elf64Xword,
    sh_link: Elf64Word,
    sh_info: Elf64Word,
    sh_addralgin: Elf64Xword,
    sh_entsize: Elf64Xword,
}

#[repr(C, packed)]
struct ElfProgramHeader {
    p_type: Elf64Word, // Entry type
    p_flags: Elf64Word, // Access permission flags
    p_offset: Elf64Off, // File offset of contents
    p_vaddr: Elf64Addr, // Virtual address in memory
    p_paddr: Elf64Addr, // Physical address in memory
    p_filesz: Elf64Xword, // Size of contents in file
    p_memsz: Elf64Xword, // Size of contents in memory
    p_align: Elf64Xword, // Alignment in memory and file
}

// #[repr(u32)]
// #[derive(PartialEq, Copy, Clone)]
// enum ProgramHeaderType {
//     PtNull = 0, // unused
//     PtLoad = 1, // loadable segment
// }

pub fn parse(file_start: u64, page_frame_allocator: &mut PageFrameAllocator) {
    let elf_header = unsafe { &*(file_start as *const ElfHeader) };
    validate_file(elf_header);
    parse_program_headers(file_start, elf_header, page_frame_allocator);
    // elf::parse_segment_headers(file_start, elf_header, page_frame_allocator);
}

// Verify file starts with ELF Magic number and is built for the correct system
fn validate_file(elf_header: &ElfHeader) -> bool {
    if elf_header.e_ident[ElfIdent::EiMag0 as usize] != 0x7F { panic!("ELF Header EI_MAG0 incorrect\n"); }
    else if elf_header.e_ident[ElfIdent::EiMag1 as usize] != ('E' as u8) { panic!("ELF header EI_MAG1 incorrect\n"); }
    else if elf_header.e_ident[ElfIdent::EiMag2 as usize] != ('L' as u8) { panic!("ELF header EI_MAG2 incorrect\n"); }
    else if elf_header.e_ident[ElfIdent::EiMag3 as usize] != ('F' as u8) { panic!("ELF header EI_MAG3 incorrect\n"); } 
    else if elf_header.e_ident[ElfIdent::EiClass as usize] != ELF_CLASS { panic!("Unsupported ELF File class\n"); } 
    else if elf_header.e_ident[ElfIdent::EiData as usize] != ELF_DATA { panic!("Unsupported ELF File byte order\n"); } 
    else if elf_header.e_ident[ElfIdent::EiVersion as usize] != ELF_VERSION { panic!("Unsupported ELF version\n"); } 
    else if elf_header.e_machine != ELF_MACHINE { panic!("Unsupported ELF file target\n"); }
    else if elf_header.e_type != 2 { panic!("Unsupported ELF file type"); }
    return true;
}

// Elf program headers specify where segments are located
fn parse_program_headers(file_start: u64, elf_header: &ElfHeader, page_frame_allocator: &mut PageFrameAllocator) {
    // Loop through the headers and load each loadable segment into memory
    for i in 0..(elf_header.e_phnum) {
        let address = file_start + elf_header.e_phoff + (mem::size_of::<ElfProgramHeader>() as u64) * (i as u64);
        let program_header = unsafe { &*(address as *const ElfProgramHeader) };

        match program_header.p_type {
            1 => {
                let source = file_start + program_header.p_offset as u64;
                if program_header.p_memsz != program_header.p_filesz { panic!("Segment is padded with 0's\n"); }
                load_segment_into_memory(source, program_header.p_memsz, i as u64, page_frame_allocator);
            }
            0 => {},
            _ => panic!("Unknown\n"),
        }
    }
}

fn parse_segment_headers(file_start: u64, elf_header: &ElfHeader, page_frame_allocator: &mut PageFrameAllocator) {
    // Loop through sections
    for i in 0..(elf_header.e_shnum) {
        // let address = file_start + elf_header.e_shoff + (mem::size_of::<ElfSectionHeader>() as u64) * (i as u64);
        // let section_header = unsafe { &*(address as *const ElfSectionHeader) };

        // Add support for .bss by checking flags etc
    }
}

fn load_segment_into_memory(source_raw: u64, size: u64, index: u64, page_frame_allocator: &mut PageFrameAllocator) {
    if size > (PAGE_SIZE as u64) { panic!("Segment is over 1 page"); } 

    // Allocate memory for the segment
    // TODO: Implement system to have continuous memory range 
    let dest = page_frame_allocator.alloc_frame().unwrap();
    let source = source_raw as *mut u64;

    // Copy segment data into the memory space
    for i in 0..1024 {
        unsafe { *dest.offset(i) = *source.offset(i);  }
    }

    // Map the physical pages to 0x800000
    let v_address = 0x800000 + (index * 0x1000) as u64;
    paging::map_page(dest as u64, v_address, page_frame_allocator, true, None);
}