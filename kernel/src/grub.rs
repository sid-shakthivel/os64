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
use multiboot2::load;
use core::mem;

type Elf64Half = u16;
type Elf64Off = u64;
type Elf64Addr = u64;
type Elf64Word = u32;
// type Elf64Sword = i32;
type Elf64Xword = u64;

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

// enum ElfType {
//     EtNone = 0, // Unknown
//     EtRel = 1, // Relocatable
//     EtExec = 2 // Executable
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

// enum ElfSectionTypes {
//     ShtNull = 0, // Null section
//     ShtProgbits = 1, // Program information
//     ShtSyntab = 2, // Symbol table
//     ShtStrtab = 3, // String table
//     ShtRela = 4, // Relocation
//     ShtNobits = 8, // Not present in file
//     ShtRel = 9, // Relocation
// }

// enum ElfSectionAttributes {
//     ShfWrite = 1, // Writeable section
//     ShfAlloc = 2, // Exists in memory
// }

#[repr(C, packed)]
struct ElfProgramHeader {
    p_type: Elf64Word,
    p_flags: Elf64Word,
    p_offset: Elf64Off,
    p_vaddr: Elf64Addr,
    p_paddr: Elf64Addr,
    p_filesz: Elf64Xword,
    p_memsz: Elf64Xword,
    p_align: Elf64Xword,
}

pub fn initialise_userland(multiboot_information_address: usize, page_frame_allocator: &mut PageFrameAllocator) {
    let boot_info = unsafe { load(multiboot_information_address as usize).unwrap() };

    for module in boot_info.module_tags() {
        // let ptr = module.start_address() as *const ();
        // let code: fn() = unsafe { core::mem::transmute(ptr) };

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

            let user_process = multitask::Process::init(0x3f1000, multitask::ProcessPriority::High, page_frame_allocator);

            // Add process to list of processes
            multitask::PROCESS_SCHEDULAR.lock().add_process(user_process);
            multitask::PROCESS_SCHEDULAR.free();
        }
    }
}

fn elf_check_file(elf_header: &ElfHeader) -> bool {
    if elf_header.e_ident[ElfIdent::EiMag0 as usize] != 0x7F {
        panic!("ELF Header EI_MAG0 incorrect\n");
    }
    else if elf_header.e_ident[ElfIdent::EiMag1 as usize] != ('E' as u8) {
        panic!("ELF header EI_MAG1 incorrect\n");
    }
    else if elf_header.e_ident[ElfIdent::EiMag2 as usize] != ('L' as u8) {
        panic!("ELF header EI_MAG2 incorrect\n");
    }
    else if elf_header.e_ident[ElfIdent::EiMag3 as usize] != ('F' as u8) {
        panic!("ELF header EI_MAG3 incorrect\n");
    } 
    else if elf_header.e_ident[ElfIdent::EiClass as usize] != 2 { // 64 bit
        panic!("Unsupported ELF File class\n");
    } 
    else if elf_header.e_ident[ElfIdent::EiData as usize] != 1 { // little endian
        panic!("Unsupported ELF File byte order\n");
    } 
    else if elf_header.e_ident[ElfIdent::EiVersion as usize] != 1 { // 1
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