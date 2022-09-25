// src/elf.rs

/*
    Executable and Linkable Format which is used to store programs
    Linkers combine elf files into an executable or library (uses sections)
    Loaders load the file into memory (uses segments)


    +-----------+----------------------------------+
    |   Name    |             Purpose              |
    +-----------+----------------------------------+
    | .text     | code                             |
    | .data     | initialised data with read/write |
    | .bss      | unitialised data                 |
    | .roadata  | initialised data with read only  |
    +-----------+----------------------------------+
*/

// TODO: Fix very irritating enum thing when I have wifi

#![allow(dead_code)]
#![allow(unused_variables)]

use crate::page_frame_allocator::{self, FrameAllocator, PAGE_FRAME_ALLOCATOR};
use crate::CONSOLE;
use crate::{paging, print_serial};
use core::mem;

type Elf64Half = u16;
type Elf64Off = u64;
type Elf64Addr = u64;
type Elf64Word = u32;
type Elf64Xword = u64;

const ELF_DATA: u8 = 1; // Little Endian
const ELF_CLASS: u8 = 2; // 64 Bit
const ELF_VERSION: u8 = 1;
const ELF_MACHINE: Elf64Half = 0x3E; // AMD x86-64
const ELF_FLAG_MAG0: u8 = 0x7F;

#[repr(C, packed)]
struct ElfHeader {
    e_ident: [u8; 16],      // Magic number and other info
    e_type: Elf64Half,      // Object file type
    e_machine: Elf64Half,   // Architecture
    e_version: Elf64Word,   // Object file version
    e_entry: Elf64Addr,     // Entry
    e_phoff: Elf64Off,      // Program header table file offset
    e_shoff: Elf64Off,      // Section header table file offset
    e_flags: Elf64Word,     // Processor-specific flags
    e_ehsize: Elf64Half,    // ELF header size in bytes
    e_phentsize: Elf64Half, // Program header table entry size
    e_phnum: Elf64Half,     // Program header table entry count
    e_shentsize: Elf64Half, // Section header table entry size
    e_shnum: Elf64Half,     // Section header table entry count
    e_shstrndx: Elf64Half,  // Section header string table index
}

enum ElfIdent {
    EiMag0 = 0,       // 0x7F
    EiMag1 = 1,       // E
    EiMag2 = 2,       // L
    EiMag3 = 3,       // F
    EiClass = 4,      // Architecture
    EiData = 5,       // Byte order
    EiVersion = 6,    // ELF version
    EiOsabi = 7,      // OS specific
    EiAbiversion = 8, // OS specific
    EiPad = 9,        // Padding
}

#[repr(u16)]
#[derive(PartialEq, Debug, Clone, Copy)]
enum ElfType {
    EtNone = 0, // Unknown
    EtRel = 1,  // Relocatable
    EtExec = 2, // Executable
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct ElfSectionHeader {
    sh_name: Elf64Word, // Section name, index in string table (index is defined in e_shstrndx)
    sh_type: Elf64Word, // Type
    sh_flags: Elf64Xword, // Miscellaneous section atttributes
    sh_addr: Elf64Addr, // Sectoin virtual address
    sh_offset: Elf64Off, // Section file offset
    sh_size: Elf64Xword, // Size of section (in bytes)
    sh_link: Elf64Word, // Index of another section
    sh_info: Elf64Word, // Additional section info
    sh_addralgin: Elf64Xword, // Section alignment
    sh_entsize: Elf64Xword, // Entry size if section holds table
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct ElfProgramHeader {
    p_type: Elf64Word,    // Entry type
    p_flags: Elf64Word,   // Access permission flags
    p_offset: Elf64Off,   // File offset of contents
    p_vaddr: Elf64Addr,   // Virtual address in memory
    p_paddr: Elf64Addr,   // Physical address in memory
    p_filesz: Elf64Xword, // Size of contents in file in bytes
    p_memsz: Elf64Xword,  // Size of contents in memory in bytes
    p_align: Elf64Xword,  // Alignment in memory and file
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u32)]
enum ProgramHeaderType {
    PtNull = 0, // Unused
    PtLoad = 1, // Loadable segment
}

struct ElfSymbol {
    st_name: Elf64Word,  // Symbol name (index in string table)
    st_info: u8,         // Type and binding attributes
    st_other: u8,        // No meaning
    st_shndx: Elf64Half, // Section index
    st_value: Elf64Addr, // Value of symbol
    st_size: Elf64Xword, // Symbol size
}

pub fn parse(file_start: u64) {
    let elf_header = unsafe { &*(file_start as *const ElfHeader) };
    validate_file(elf_header);
    parse_program_headers(file_start, elf_header);
    // parse_section_headers(file_start, elf_header);
}

// Verify file starts with ELF Magic number and is built for the correct system
fn validate_file(elf_header: &ElfHeader) -> bool {
    assert!(
        elf_header.e_ident[ElfIdent::EiMag0 as usize] == ELF_FLAG_MAG0,
        "ELF Header EI_MAG0 incorrect\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiMag1 as usize] == ('E' as u8),
        "ELF header EI_MAG1 incorrect\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiMag2 as usize] == ('L' as u8),
        "ELF header EI_MAG2 incorrect\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiMag3 as usize] == ('F' as u8),
        "ELF header EI_MAG3 incorrect\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiClass as usize] == ELF_CLASS,
        "Unsupported ELF File class\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiData as usize] == ELF_DATA,
        "Unsupported ELF File byte order\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiVersion as usize] == ELF_VERSION,
        "Unsupported ELF version\n"
    );

    assert!(
        elf_header.e_machine == ELF_MACHINE,
        "Unsupported ELF file target\n"
    );

    let test = elf_header.e_type;

    // assert!(test == 1, "Unsupported ELF file type {}", test);

    return true;
}

// Elf program headers specify where segments are located

/*
    Program headers point to segments which contain multiple sections
    These are utilised whilst executing
*/
fn parse_program_headers(file_start: u64, elf_header: &ElfHeader) {
    // Loop through the headers and load each loadable segment into memory
    for i in 0..elf_header.e_phnum {
        let address = file_start
            + elf_header.e_phoff
            + (mem::size_of::<ElfProgramHeader>() as u64) * (i as u64);
        let program_header = unsafe { &*(address as *const ElfProgramHeader) };

        match program_header.p_type {
            1 => {
                // LOAD
                let source = file_start + program_header.p_offset as u64;
                load_segment_into_memory(
                    source,
                    program_header.p_filesz,
                    program_header.p_memsz,
                    program_header.p_vaddr,
                );
            }
            0 => {}
            _ => {}
        }
    }
}

/*
    Section headers point to sections
    These are utilised whilst linking
*/
fn parse_section_headers(file_start: u64, elf_header: &ElfHeader) {
    // Loop through sections
    for i in 0..(elf_header.e_shnum) {
        let address = file_start
            + elf_header.e_shoff
            + (mem::size_of::<ElfSectionHeader>() as u64) * (i as u64);
        let section_header = unsafe { &*(address as *const ElfSectionHeader) };

        let section_type = section_header.sh_type;

        match section_type {
            0 => {} // SHT_NULL
            1 => {} // SHT_PROGBITS
            2 => {
                /*
                SHT_SYMTAB
                    Defines location, type, visibility, and traits of symbols created during compilation/linking
                    Multiple may exist
                */
            }
            3 => {} // SHT_STRTAB
            4 => {} // SHT_RELA
            8 => {} // SHT_NOBITS (BSS)
            9 => {} // SHT_REL
            _ => panic!("Unknown section type {}\n", section_type),
        }
    }
}

fn get_symbol(file_start: u64, section_header: &ElfSectionHeader, index: u64) {
    // Check if index is within range
    let max_entries = section_header.sh_size / section_header.sh_entsize;
    if index > max_entries {
        panic!("Index greater then max number of entries");
    }

    let symbol_address =
        file_start + section_header.sh_offset + (section_header.sh_entsize * index);

    let symbol = unsafe { &*(symbol_address as *const ElfSymbol) };

    match symbol.st_shndx {
        0x00 => {
            // SHN_UNDEF
            panic!("oh no");
        }
        _ => {
            panic!("Unknown {}\n", symbol.st_shndx);
        }
    }
}

fn get_string_table(file_start: u64, elf_header: &ElfHeader) -> ElfSectionHeader {
    let string_table_section_address = file_start
        + elf_header.e_shoff
        + (mem::size_of::<ElfSectionHeader>() as u64) * (elf_header.e_shstrndx as u64);
    let string_table_header =
        unsafe { &*(string_table_section_address as *const ElfSectionHeader) };

    string_table_header.clone()
}

fn get_section_name(file_start: u64, elf_header: &ElfHeader, offset: u64) -> &str {
    let string_table = get_string_table(file_start, elf_header);
    let ptr = (file_start + string_table.sh_offset + offset) as *const u8;
    crate::string::get_string_from_ptr(ptr)
}

fn load_segment_into_memory(source_raw: u64, filesz: u64, memsz: u64, v_address: u64) -> u64 {
    // Allocate appropriate amount of memory
    let rounded_size = page_frame_allocator::round_to_nearest_page(memsz);
    let number_of_pages = page_frame_allocator::get_page_number(rounded_size);

    print_serial!("PAGE NUM = {}\n", number_of_pages);

    let dest = PAGE_FRAME_ALLOCATOR.lock().alloc_frames(number_of_pages) as *mut u64;
    PAGE_FRAME_ALLOCATOR.free();

    let source = source_raw as *mut u64;

    // If the memsz is greater then filesz, extra bytes should store 0
    // for i in 0..(memsz / 8) {
    //     unsafe {
    //         *dest.offset(i as isize) = 0;
    //     }
    // }

    // unsafe {
    //     print_serial!(
    //         "SIZE {:x} POINTER = {:p} {:p}\n",
    //         filesz,
    //         dest,
    //         dest.offset(filesz as isize)
    //     );
    // }

    // // Copy actual segment data
    // for i in 0..(filesz / 8) {
    //     unsafe {
    //         *dest.offset(i as isize) = *source.offset(i as isize);
    //     }
    // }

    unsafe {
        core::ptr::write_bytes(dest as *mut u8, 0, memsz as usize);
        core::ptr::copy_nonoverlapping(source as *mut u8, dest as *mut u8, filesz as usize);
    }

    // Map the physical pages to the virtual address provided
    paging::map_pages(number_of_pages, dest as u64, v_address);

    v_address + (rounded_size)
}
