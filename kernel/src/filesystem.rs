// src/filesystem.rs

/*
    This is a driver for the FAT 16 file system
    Single linked list of clusters in a table 
    Storage media is a flat array of clusters
    3 areas include: Boot record, FAT, Directory/data area
    Cluster is unit of storage (physically) set by fs
    Sector is unit of storage on a disk drive (FAT level)
*/

use crate::print;
use crate::vga_text::TERMINAL;
use multiboot2::load;
use spin::Mutex;
use core::mem;

// Boot record occupies one sector and is at the start

struct Fat16 {
    bpb: Option<BiosParameterBlock>,
    start_address: u32,
    fat_address: u32,
    first_data_sector_address: u32,
}

impl Fat16 {
    pub const fn new() -> Fat16 {
        Fat16 {
            bpb: None,
            start_address: 0,
            fat_address: 0,
            first_data_sector_address: 0,
        }
    }
}

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

static FS: Mutex<Fat16> = Mutex::new(Fat16::new());

pub fn init(multiboot_information_address: usize) {
    let boot_info = unsafe { load(multiboot_information_address as usize).unwrap() };

    for module in boot_info.module_tags() {
        let bpb = unsafe { &*((module.start_address()) as *const BiosParameterBlock) };

        let ebr_address = module.start_address() + (mem::size_of::<BiosParameterBlock>() as u32);
        let ebr = unsafe { &*(ebr_address as *const ExtendedBootRecord) };

        validate_fs(ebr);

        let first_fat = module.start_address() + (512 * bpb.reserved_sector_count as u32);
        let fat_size: u32 = bpb.table_size_16 as u32;

        let mut root_directory_address: u32 = (bpb.reserved_sector_count as u32) + ((bpb.table_count as u32) * fat_size);
        root_directory_address = module.start_address() + (root_directory_address * 512);

        let root_directory_size: u32 = (bpb.root_entry_count * 32) as u32;
        let first_data_sector = (root_directory_size * 512) + root_directory_address;

        FS.lock().bpb = Some(*bpb);
        FS.lock().start_address = module.start_address();
        FS.lock().fat_address = first_fat;
        FS.lock().first_data_sector_address = first_data_sector;

        create_vfs(root_directory_address, root_directory_size, first_data_sector);
    }
}

// Should ensure it's FAT16 and check certain values
fn validate_fs(ebr: &ExtendedBootRecord) -> bool {
    // TODO: Calculate number of clusters and check whether smaller then 65525 and expand

    if (ebr.signature != 0x28 && ebr.signature != 0x29) { panic!("Invalid signature, {:x}", ebr.signature); }
    // if (ebr.bootable_partition_signature != 0xAA55) { panic!("Invalid partition signature"); }

    return true;
}

// Loops through the root directory
fn create_vfs(mut root_directory_address: u32, root_directory_size: u32, first_data_sector: u32) {
    parse_folder_cluster(root_directory_address, (root_directory_size / 0x20));
}

// Directories contain folders and files 
fn parse_folder_cluster(mut cluster_address: u32, size: u32) {
    for i in 0..size {
        let directory_entry = unsafe { &*(cluster_address as *const StandardDirectoryEntry) };
        print!("{:?}\n", directory_entry);

        match directory_entry.filename[0] {
            0x00 => return, // No more files/directories
            0xE5 => panic!("Unused entry"),
            _ => {}
        }

        if directory_entry.attributes & 0x10 > 0 {
            // Is directory
            let next_cluster_address: u32 = ((512 * directory_entry.cluster_low) as u32) + FS.lock().first_data_sector_address;
            parse_folder_cluster(next_cluster_address, 64);
        } else {
            // Is File
            parse_file_clusters(directory_entry.cluster_low as u32, directory_entry);
        }

        cluster_address += 0x20;
    }
}

fn parse_file_clusters(cluster_num: u32, file_entry: &StandardDirectoryEntry) {
    let cluster_address = (512 * cluster_num) + FS.lock().first_data_sector_address;
    let file_contents = unsafe { (cluster_address as *const u8) }; // Pointer to this in 
    match get_next_cluster(cluster_num) {
        None => return, // End of file
        Some(cluster_num) => {
            parse_file_clusters(cluster_num, file_entry);
        }
    }
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

// Clusters represent linear addresses, sectors use segment addresses
// LBA represents an indexed location on the disk
fn get_lba(cluster_num: u32) -> u32 {
    return (cluster_num - 2) * (FS.lock().bpb.unwrap().sectors_per_cluster) as u32;
}

// Uses 16 bits to address clusters
fn read_fat(sector_num: u32, byte_offset: usize) -> u16 {
    let fat =  unsafe { &*((FS.lock().fat_address + sector_num * 512) as *const [u8; 512]) };
    return ((fat[byte_offset] as u16) << 8) | (fat[byte_offset+1] as u16);
}