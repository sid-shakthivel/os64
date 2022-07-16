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

use crate::print;
use crate::vga_text::TERMINAL;
use multiboot2::load;
use spin::Mutex;
use core::mem;

// VFS

// Inspired by the ubiquitous FILE* data type in C
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct File {
    name: [u8; 32],
    flags: u32,
    file_size: u32,
    cluster: u32,
    file_type: FileType
    // permissions: u32,
    // uid: u32,
    // gid: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FileType {
    File,
    Directory,
    Syslink,
}

impl File {
    pub const fn new(cluster_num: u32, size: u32, file_type: FileType) -> File {
        return File {
            name: [0; 32],
            flags: 0,
            file_size: size,
            cluster: cluster_num,
            file_type: file_type,
        }
    }

    // TODO: Figure out how to copy to a buffer?
    pub fn read(&mut self) {
        if (self.file_type == FileType::File) {
            Self::read_file(self.cluster);
        }

        if (self.file_type == FileType::Directory) {
            print!("Here?\n");
            // Self::read_directory(self.cluster);
            Self::read_directory(self.cluster);
        }
    }

    // TODO: Remove
    pub fn read_root(&mut self, address: u32) {
        let mut cluster_address = address;

        for i in 0..512 {
            let directory_entry = unsafe { &*(cluster_address as *const StandardDirectoryEntry) };
            print_filename(&directory_entry.filename, &directory_entry.ext);

            match directory_entry.filename[0] {
                0x00 => return, // No more files/directories
                0xE5 => panic!("Unused entry"),
                _ => {}
            }
        
            cluster_address += 0x20;
        }
    }

    fn read_directory(cluster_num: u32) {
        let mut cluster_address = (512 * get_lba(cluster_num)) + FS.lock().first_data_sector_address;

        // There are 64 entries in a single cluster
        for i in 0..64 {
            let directory_entry = unsafe { &*(cluster_address as *const StandardDirectoryEntry) };
            print_filename(&directory_entry.filename, &directory_entry.ext);

            match directory_entry.filename[0] {
                0x00 => return, // No more files/directories
                0xE5 => panic!("Unused entry"),
                _ => {}
            }
        
            cluster_address += 0x20;
        }

        match get_next_cluster(cluster_num) {
            None => return, // End of cluster
            Some(cluster_num) => {
                Self::read_directory(cluster_num);
            }
        }
    }

    fn read_file(cluster_num: u32) {
        let cluster_address = (512 * get_lba(cluster_num)) + FS.lock().first_data_sector_address;
        let file_contents = unsafe { (cluster_address as *const u8) }; 
        unsafe {
            print!("Character : {}\n", *file_contents.offset(0));
        }
        match get_next_cluster(cluster_num) {
            None => return, // End of file
            Some(cluster_num) => {
                Self::read_file(cluster_num);
            }
        }
    }
}

// FAT16
struct Fat16 {
    bpb: Option<BiosParameterBlock>,
    start_address: u32,
    fat_address: u32,
    first_data_sector_address: u32,
    root_directory_size: u32,
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

// Should ensure it's FAT16 and check certain values
fn validate_fat(ebr: &ExtendedBootRecord) -> bool {
    // TODO: Calculate number of clusters and check whether smaller then 65525 and expand

    if (ebr.signature != 0x28 && ebr.signature != 0x29) { panic!("Invalid signature, {:x}", ebr.signature); }
    // if (ebr.bootable_partition_signature != 0xAA55) { panic!("Invalid partition signature"); }

    return true;
}

fn get_next_cluster(mut cluster_num: u32) -> Option<u32> {
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

// Clusters represent linear addresses, sectors use segment addresses
// LBA represents an indexed location on the disk
fn get_lba(cluster_num: u32) -> u32 {
    return (cluster_num - 2) * (FS.lock().bpb.unwrap().sectors_per_cluster) as u32;
}

// Uses 16 bits to address clusters
fn read_fat(sector_num: u32, byte_offset: usize) -> u16 {
    let fat =  unsafe { &*((FS.lock().fat_address + sector_num * 512) as *const [u8; 512]) };
    return ((fat[byte_offset+1] as u16) << 8) | (fat[byte_offset] as u16); // Little endian 
}

// Most addresses are calculated sectors and therefore must be converted into bytes to be read/written
fn convert_sector_to_bytes() {

}

static FS: Mutex<Fat16> = Mutex::new(Fat16::new());

// Print filenames in required format
fn print_filename(filename: &[u8], ext: &[u8]) {
    for i in 0..filename.len() {
        if (filename[i] != 0x20) {
            print!("{}", filename[i] as char);
        }
    }

    for i in 0..ext.len() {
        print!("{}", ext[i] as char);
    }

    print!("\n");
}

pub fn init(multiboot_information_address: usize) {
    let boot_info = unsafe { load(multiboot_information_address as usize).unwrap() };

    // TODO: Change to get the first module only since this will always be an fs
    for module in boot_info.module_tags() {
        let bpb = unsafe { &*((module.start_address()) as *const BiosParameterBlock) };

        let ebr_address = module.start_address() + (mem::size_of::<BiosParameterBlock>() as u32);
        let ebr = unsafe { &*(ebr_address as *const ExtendedBootRecord) };

        validate_fat(ebr);

        let first_fat = module.start_address() + (512 * bpb.reserved_sector_count as u32);
        let fat_size: u32 = bpb.table_size_16 as u32;

        let root_directory_sector: u32 = (bpb.reserved_sector_count as u32) + ((bpb.table_count as u32) * fat_size);
        let root_directory_address = module.start_address() + (root_directory_sector * 512);

        let root_directory_size: u32 = ((((bpb.root_entry_count) * 32) + (bpb.bytes_per_sector - 1)) / bpb.bytes_per_sector) as u32;
        let first_data_sector = (root_directory_size * 512) + root_directory_address;

        FS.lock().bpb = Some(*bpb);
        FS.lock().start_address = module.start_address();
        FS.lock().fat_address = first_fat;
        FS.lock().first_data_sector_address = first_data_sector;
        FS.lock().root_directory_size = (root_directory_size * 512);

        // Create the root node
        let mut initrd: File = File::new(root_directory_sector, 512, FileType::Directory);
        print!("{:?}\n", initrd);
        initrd.read_root(root_directory_address);
    }
}