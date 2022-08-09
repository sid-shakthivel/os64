// src/filesystem.rs

/*
    This is a driver for the FAT 16 file system (logical way to store, read, write data)
    Single linked list of clusters in a table 
    Storage media is a flat array of clusters
    3 areas include: Boot record, FAT, Directory/data area
    Cluster is unit of storage (physically) set by fs
    Sector is unit of storage on a disk drive (FAT level)
*/

/*
    Virtual File System is abstraction on top of a FS which allows programs to work with any filesystem
    Node graph which represents files/directories which have methods (read, write, etc)
*/

// TODO: Long File Names

#![allow(dead_code)]
#![allow(unused_variables)]

use spin::Mutex;
use core::mem;

use crate::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::page_frame_allocator::FrameAllocator;

struct Fat16 {
    bpb: Option<BiosParameterBlock>,
    start_address: u32,
    fat_address: u32,
    first_data_sector_address: u32,
    root_directory_size: u32,
}

// Boot record occupies one sector and is at the start
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct BiosParameterBlock {
    jmp: [u8; 3],
    oem: [u8; 8],
    bytes_per_sector: u16, 
    sectors_per_cluster: u8, 
    reserved_sector_count: u16,
    table_count: u8, 
    root_entry_count: u16,
    sector_count_16: u16,
    media_type: u8,
    table_size_16: u16, // Number of sectors per FAT
    sectors_per_track: u16, // Number of sectors per track
    head_count: u16,
    hidden_sector_count: u32,
    sector_count_32: u32
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct ExtendedBootRecord {
    drive_number: u8,
    nt_flags: u8,
    signature: u8,
    serial: u32,
    volume_label: [char; 11],
    system_ud_string: u64,
    bootcode: [u8; 448],
    bootable_partition_signature: u16
}

// Stores information on where a file's data/folder are stored on disk along with name, size, creation
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct StandardDirectoryEntry {
    filename: [u8; 8],
    ext: [u8; 3],
    attributes: u8,
    unused: [u8; 8],
    cluster_high: u16,
    time: u16,
    date: u16,
    cluster_low: u16,
    file_size: u32,
}

// These always have a regular entry as well, and these are placed before the standard entry
struct LongDirectoryEntry {
    order: u8,
    name_start: [char; 5],
    attribute: u8,
    long_entry_type: u8,
    checksum: u8,
    name_middle: [char; 6],
    zero: u16,
    name_end: [char; 2]
}

// Inspired by the ubiquitous FILE* data type in C
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct File {
    name: [u8; 8],
    flags: u32,
    size: u32,
    cluster: u32,
    index: u32,
    file_type: FileType,
    // permissions: u32,
    // uid: u32,
    // gid: u32,
}

#[derive(Copy, PartialEq, Clone, Debug)]
enum FileType {
    File,
    Directory,
    Syslink,
}

impl File {
    pub const fn new(cluster_num: u32, size: u32, file_type: FileType) -> File {
        return File {
            name: [0; 8],
            flags: 0,
            index: 0,
            size: size,
            cluster: cluster_num,
            file_type: file_type,
        }
    }

    pub fn read(&mut self, buffer: *mut u8) -> Result<u64, &str> {
        if self.file_type != FileType::File { return Err("Tried to read on a directory"); }

        self._modify(buffer, false);
        self.index = 0;
        return Ok(0);
    }

    pub fn write(&mut self, buffer: *mut u8)-> Result<u64, &str> {
        if self.file_type != FileType::File { return Err("Tried to write on a directory"); }

        self._modify(buffer, true);
        self.index = 0;
        return Ok(0);
    }

    pub fn find(&mut self, filename: &str) -> Result<File, &str> {
        let cluster_address = convert_sector_to_bytes(get_lba(self.cluster)) + FS.lock().first_data_sector_address;
        return self._find(filename, cluster_address);
    }

    pub fn find_root(&mut self, filename: &str) -> Result<File, &str> {
        let first_data_sector_address: u32 = FS.lock().first_data_sector_address;
        let root_directory_size: u32 = FS.lock().root_directory_size;
        let root_directory_address = first_data_sector_address - root_directory_size;

        return self._find(filename, root_directory_address);
    }

    pub fn readdir(&mut self) -> Result<*const [File; 64], &str> {
        if self.file_type != FileType::Directory { return Err("Tried to open on a file"); }
        let cluster_address = convert_sector_to_bytes(get_lba(self.cluster)) + FS.lock().first_data_sector_address;
        return Ok(unsafe { &*(cluster_address as *const [File; 64]) });
    }

    pub fn mkdir(&mut self, filename: &str) -> Result<File, &str> {
        if self.file_type != FileType::Directory { return Err("Tried to mkdir on a file"); }

        let mut cluster_address = convert_sector_to_bytes(get_lba(self.cluster)) + FS.lock().first_data_sector_address;
        
        // Search for an empty space 
        for _i in 0..64 {
            let directory_entry = unsafe { &*(cluster_address as *const StandardDirectoryEntry) };

            // Free entry
            if directory_entry.filename[0] == 0 {
                unsafe {
                    let directory_entry_mut = &mut *(cluster_address as *mut StandardDirectoryEntry);
                    let file_name_bytes = filename.as_bytes();

                    for j in 0..8 {
                        if j < file_name_bytes.len() { directory_entry_mut.filename[j] = file_name_bytes[j]; }
                    }

                    directory_entry_mut.attributes = 0x10;
                    directory_entry_mut.cluster_low = get_next_unallocated_cluster().unwrap();
    
                    let mut node = File::new(directory_entry_mut.cluster_low as u32, directory_entry_mut.file_size, FileType::Directory);
                    node.name = directory_entry.filename;
                    return Ok(node);
                }
            };

            cluster_address += 0x20;
        }

        return Err("Directory is full");
    }

    fn _modify(&mut self, buffer: *mut u8, write: bool) {
        let cluster_address = convert_sector_to_bytes(get_lba(self.cluster)) + FS.lock().first_data_sector_address;

        unsafe {
            let file_contents = cluster_address as *mut u8;
            if write { memcpy_cluster(file_contents, buffer, self.index); }
            else { memcpy_cluster(buffer, file_contents, self.index); }
        }

        let saved_cluster_num = self.cluster;

        match get_next_cluster(self.cluster) {
            None => return, // End of file
            Some(cluster_num) => {
                self.index += 1;
                self.cluster = cluster_num;
                return self._modify(buffer, write);
            }
        }
    }

    fn _find(&mut self, filename: &str, mut cluster_address: u32) -> Result<File, &str> {
        if filename.len() > 8 { return Err("File is too big"); }
        if self.file_type != FileType::Directory { return Err("Tried to find on a directory"); }

        for _i in 0..64 {
            let directory_entry = unsafe { &*(cluster_address as *const StandardDirectoryEntry) };

            match directory_entry.filename[0] {
                0x00 => return Err("File cannot be found in this directory"), // No more files/directories
                0xE5 => panic!("Unused entry"),
                _ => {}
            }

            let dos_filename = core::str::from_utf8(&directory_entry.filename).unwrap().trim();
            let dos_extension = core::str::from_utf8(&directory_entry.ext).unwrap().trim();
            let mut split_filename = filename.split(".");

            // Check if entry matches 
            if split_filename.next().unwrap() == dos_filename {
                if directory_entry.attributes & 0x10 > 0 {
                    let mut node = File::new(directory_entry.cluster_low as u32, directory_entry.file_size, FileType::Directory);
                    node.name = directory_entry.filename;
                    return Ok(node);
                } else {
                    let mut node = File::new(directory_entry.cluster_low as u32, directory_entry.file_size, FileType::File);
                    node.name = directory_entry.filename;
                    return Ok(node);
                }
            }
        
            cluster_address += 0x20;
        }

        // TODO: Add support for directories which are greater then a cluster
        return Err("File cannot be found in this directory");
    }
}

impl Fat16 {
    pub const fn new() -> Fat16 {
        Fat16 {
            bpb: None,
            start_address: 0,
            fat_address: 0,
            first_data_sector_address: 0,
            root_directory_size: 0,
        }
    }
}

// Should ensure it's FAT16 and check certain values
fn validate_fat(ebr: &ExtendedBootRecord) -> bool {
    // TODO: Calculate number of clusters and check whether smaller then 65525 and expand

    if ebr.signature != 0x28 && ebr.signature != 0x29 { panic!("Invalid signature, {:x}", ebr.signature); }
    // if (ebr.bootable_partition_signature != 0xAA55) { panic!("Invalid partition signature"); }

    return true;
}

fn get_next_cluster(cluster_num: u32) -> Option<u32> {
    let fat_offset = cluster_num * 2;
    let sector_number = fat_offset / 512;
    let byte_offset = fat_offset % 512;

    let next_cluster = read_fat(sector_number, byte_offset as usize);

    return match next_cluster {
        0xFFF7 => panic!("Bad cluster!"), // Indicates bad cluster
        0xFFF8..=0xFFFF => None, // Indicates the whole file has been read
        _ => Some(next_cluster as u32) // Gives next cluster number
    }
}

fn get_next_unallocated_cluster() -> Option<u16> {
    let fat =  unsafe { &mut *((FS.lock().fat_address) as *mut [u8; 512]) };
    for i in 0..512 {
        if ((fat[i+1] as u16) << 8 | (fat[i] as u16)) == 0 {
            fat[i] = 0xFF;
            fat[i+1] = 0xFF;
            return Some(i as u16);
        }
    }
    return None;
}

// Clusters represent linear addresses, sectors use segment addresses
// LBA represents an indexed location on the disk
fn get_lba(cluster_num: u32) -> u32 {
    return (cluster_num - 2) * (FS.lock().bpb.unwrap().sectors_per_cluster) as u32;
}

// Uses 16 bits to address clusters
fn read_fat(sector_num: u32, byte_offset: usize) -> u16 {
    let fat =  unsafe { &*((FS.lock().fat_address + convert_sector_to_bytes(sector_num)) as *const [u8; 512]) };
    return ((fat[byte_offset+1] as u16) << 8) | (fat[byte_offset] as u16); // Little endian 
}

// Most addresses are calculated sectors and therefore must be converted into bytes to be read/written
fn convert_sector_to_bytes(sector: u32) -> u32 {
    return sector * 512;
}

static FS: Mutex<Fat16> = Mutex::new(Fat16::new());

// Print filenames in required format
fn print_filename(filename: &[u8], ext: &[u8]) {
    for i in 0..filename.len() {
        if filename[i] != 0x20 {
            // print!("{}", filename[i] as char);
        }
    }

    for i in 0..ext.len() {
        // print!("{}", ext[i] as char);
    }

    // print!("\n");
}

pub fn init(start_address: u32) {
    let bpb = unsafe { &*(start_address as *const BiosParameterBlock) };

    let ebr_address = start_address + (mem::size_of::<BiosParameterBlock>() as u32);
    let ebr = unsafe { &*(ebr_address as *const ExtendedBootRecord) };

    validate_fat(ebr);

    let first_fat = start_address + convert_sector_to_bytes(bpb.reserved_sector_count as u32);
    let fat_size: u32 = bpb.table_size_16 as u32;

    let root_directory_sector: u32 = (bpb.reserved_sector_count as u32) + ((bpb.table_count as u32) * fat_size);
    let root_directory_address: u32 = start_address + convert_sector_to_bytes(root_directory_sector);

    let root_directory_size: u32 = ((((bpb.root_entry_count) * 32) + (bpb.bytes_per_sector - 1)) / bpb.bytes_per_sector) as u32;
    let first_data_sector: u32 = convert_sector_to_bytes(root_directory_size) + root_directory_address;

    FS.lock().bpb = Some(*bpb);
    FS.lock().start_address = start_address;
    FS.lock().fat_address = first_fat;
    FS.lock().first_data_sector_address = first_data_sector;
    FS.lock().root_directory_size = convert_sector_to_bytes(root_directory_size);

    let dest = PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as *mut u8;
    PAGE_FRAME_ALLOCATOR.free();

    let initrd: File = File::new(root_directory_sector, 512, FileType::Directory);
}

// Copies number of bytes into destination
pub unsafe fn memcpy_cluster(dest: *mut u8, src: *mut u8, index: u32) {
    for i in 0..2048 {
        let offset = ((index * 2048) + i) as isize;
        *(dest.offset(offset)) = *(src.offset(offset));
    }
} 
