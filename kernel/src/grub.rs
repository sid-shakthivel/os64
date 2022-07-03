#[warn(non_camel_case_types)]

// src/grub.rs

/*
    Grub loads a number of modules into certain memory locations which need to be mapped into user pages
    These modules serve as user programs which will be embellished later
*/

use crate::page_frame_allocator::FrameAllocator;
use crate::page_frame_allocator::PageFrameAllocator;
use crate::vga_text::TERMINAL;
use crate::multitask;
use crate::print;
use crate::paging::Table;
use multiboot2::load;
use core::mem;
use core::ptr;
use crate::paging;
use crate::gdt::TSS;
use x86_64::addr::VirtAddr;

type Elf64_Half = u16;
type Elf64_Off = u64;
type Elf64_Addr = u64;
type Elf64_Word = u32;
type Elf64_Sword = i32;
type Elf64_Xword = u64;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct ElfHeader {
    e_ident: [u8; 16], // Magic number and other info
    e_type: Elf64_Half, // Object file type
    e_machine: Elf64_Half, // Architecture
    e_version: Elf64_Word, // Object file version
    e_entry: Elf64_Addr, // Entry 
    e_phoff: Elf64_Off, // Program header table file offset
    e_shoff: Elf64_Off, // Section header table file offset
    e_flags: Elf64_Word, // Processor-specific flags
    e_ehsize: Elf64_Half, // ELF header size in bytes
    e_phentsize: Elf64_Half, // Program header table entry size
    e_phnum: Elf64_Half, // Program header table entry count
    e_shentsize: Elf64_Half, // Section header table entry size
    e_shnum: Elf64_Half, // Section header table entry count
    e_shstrndx: Elf64_Half, // Section header string table index
}

enum Elf_Ident {
    EI_MAG0 = 0, // 0x7F
    EI_MAG1 = 1, // E
    EI_MAG2 = 2, // L
    EI_MAG3 = 3, // F
    EI_CLASS = 4, // Architecture
    EI_DATA = 5, // Byte order
    EI_VERSION = 6, // ELF version
    EI_OSABI = 7, // OS specific
    EI_ABIVERSION = 8, // OS specific
    EI_PAD = 9 // Padding
}

enum Elf_Type {
    ET_NONE = 0, // Unknown
    ET_REL = 1, // Relocatable
    ET_EXEC = 2 // Executable
}

#[repr(C, packed)]
struct ElfSectionHeader{
    sh_name: Elf64_Word,
    sh_type: Elf64_Word,
    sh_flags: Elf64_Xword,
    sh_addr: Elf64_Addr,
    sh_offset: Elf64_Off,
    sh_size: Elf64_Xword,
    sh_link: Elf64_Word,
    sh_info: Elf64_Word,
    sh_addralgin: Elf64_Xword,
    sh_entsize: Elf64_Xword,
}

enum ElfSectionTypes {
    SHT_NULL = 0, // Null section
    SHT_PROGBITS = 1, // Program information
    SHT_SYNTAB = 2, // Symbol table
    SHT_STRTAB = 3, // String table
    SHT_RELA = 4, // Relocation
    SHT_NOBITS = 8, // Not present in file
    SHT_REL = 9, // Relocation
}

enum ElfSectionAttributes {
    SHF_WRITE = 1, // Writeable section
    SHF_ALLOC = 2, // Exists in memory
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct ElfProgramHeader {
    p_type: Elf64_Word,
    p_flags: Elf64_Word,
    p_offset: Elf64_Off,
    p_vaddr: Elf64_Addr,
    p_paddr: Elf64_Addr,
    p_filesz: Elf64_Xword,
    p_memsz: Elf64_Xword,
    p_align: Elf64_Xword,
}

pub fn initialise_userland(multiboot_information_address: usize, page_frame_allocator: &mut PageFrameAllocator) {
    let boot_info = unsafe { load(multiboot_information_address as usize).unwrap() };

    for module in boot_info.module_tags() {
        // let ptr = module.start_address() as *const ();
        // let code: fn() = unsafe { core::mem::transmute(ptr) };

        let module_size: isize =  (module.end_address() as isize) - (module.start_address() as isize);
        let address = module.start_address() as usize;

        unsafe {
            let elf_header = &*(module.start_address() as *const ElfHeader);
            elf_check_file(elf_header);

            for i in 0..(elf_header.e_phnum as usize)
            {
                let addr = address + (elf_header.e_phoff as usize) + mem::size_of::<ElfProgramHeader>() * i;
                let program_header = &*(addr as *const ElfProgramHeader);
                let program_type = program_header.p_type;

                if program_type == 1 // load
                {
                    let source = address + program_header.p_offset as usize;
                    let file_size = program_header.p_filesz as usize;
                    let mem_size = program_header.p_memsz as usize;

                    if mem_size != file_size { panic!("Mem size != File size\n"); }

                    let dest = page_frame_allocator.alloc_with_address(source, mem_size).unwrap(); // May need to specifically map this into different memory

                    // unsafe {
                    //     let p_address = dest as u64;
                    //     let v_address = 0x800000 + (i * 0x1000) as u64;
                    //     paging::map_page(p_address, v_address, page_frame_allocator, true, None);
                    // }

                    print!("{:p}\n", dest);

                    // If p_memsz exceeds p_filesz, then the remaining bits are to be cleared with zeros
                    // ptr::write_bytes(dest as *mut u8, 0, mem_size);
                    // ptr::copy_nonoverlapping(source as *mut u8, dest as *mut u8, file_size);
                }
            }

            // Apparently reading section headers is unnecessary for data
            // for i in 0..(elf_header.e_shnum as usize)
            // {
            //     let addr = address + (elf_header.e_shoff as usize) + mem::size_of::<ElfSectionHeader>() * i;
            //     let section_header = &*(addr as *const ElfSectionHeader);
            //     let flags = section_header.sh_flags;

            //     if flags & (1 << 2) > 0 { // Check if flag contains alloc
            //         // TODO: Add support for .bss data
            //     }
            // }

            let user_process = multitask::Process::init(0x3f2000, multitask::ProcessPriority::High, page_frame_allocator);

            // Add process to list of processes
            multitask::PROCESS_SCHEDULAR.lock().add_process(user_process);
            multitask::PROCESS_SCHEDULAR.free();
        }
    }
}

fn elf_check_file(elf_header: &ElfHeader) -> bool {
    let unaligned = core::ptr::addr_of!(elf_header);
    let aligned = unsafe { core::ptr::read_unaligned(unaligned) };

    if elf_header.e_ident[Elf_Ident::EI_MAG0 as usize] != 0x7F {
        panic!("ELF Header EI_MAG0 incorrect\n");
    }
    else if elf_header.e_ident[Elf_Ident::EI_MAG1 as usize] != ('E' as u8) {
        panic!("ELF header EI_MAG1 incorrect\n");
    }
    else if elf_header.e_ident[Elf_Ident::EI_MAG2 as usize] != ('L' as u8) {
        panic!("ELF header EI_MAG2 incorrect\n");
    }
    else if elf_header.e_ident[Elf_Ident::EI_MAG3 as usize] != ('F' as u8) {
        panic!("ELF header EI_MAG3 incorrect\n");
    } 
    else if elf_header.e_ident[Elf_Ident::EI_CLASS as usize] != 2 { // 64 bit
        panic!("Unsupported ELF File class\n");
    } 
    else if elf_header.e_ident[Elf_Ident::EI_DATA as usize] != 1 { // little endian
        panic!("Unsupported ELF File byte order\n");
    } 
    else if elf_header.e_ident[Elf_Ident::EI_VERSION as usize] != 1 { // 1
        panic!("Unsupported ELF version\n");
    } 
    else if elf_header.e_machine != 0x3E { // AMD 64
        panic!("Unsupported ELF file target\n");
    }
    else if elf_header.e_type != 1 && elf_header.e_type != 2 { // relocatable or executable
        panic!("Unsupported ELF file type");
    }
    return true;
}

extern "C" {
    fn switch_process(rsp: *const u64, p4: *const Table);
}