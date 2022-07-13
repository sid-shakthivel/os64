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

// Boot record occupies one sector and is at the start

struct Fat16 {
    bpb: Option<BiosParameterBlock>,
    start_address: u32,
    fat_address: u32
}

impl Fat16 {
    pub const fn new() -> Fat16 {
        Fat16 {
            bpb: None,
            start_address: 0,
            fat_address: 0,
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

struct EBR {
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

        let first_fat = module.start_address() + (512 * bpb.reserved_sector_count as u32);
        let fat_size: u32 = bpb.table_size_16 as u32;

        let mut root_directory_address: u32 = (bpb.reserved_sector_count as u32) + ((bpb.table_count as u32) * fat_size);
        root_directory_address = module.start_address() + (root_directory_address * 512);

        let root_directory_size: u32 = (((bpb.root_entry_count * 32) + (bpb.bytes_per_sector - 1)) / bpb.bytes_per_sector).into();
        let first_data_sector = (root_directory_size * 512) + root_directory_address;

        FS.lock().bpb = Some(*bpb);
        FS.lock().start_address = module.start_address();
        FS.lock().fat_address = first_fat;

        create_vfs(root_directory_address, root_directory_size, first_data_sector);
    }
}

// Should ensure it's FAT16
// fn validate_fs() -> bool {
//     let data_sectors = bpb.sector_count_16;
//     let total_clusters = data_sectors / (bpb.sectors_per_cluster as u16);

//     print!("{}\n", total_clusters);
// }

fn get_lba(cluster_num: u32) -> u32 {
    return (cluster_num - 2) * (FS.lock().bpb.unwrap().sectors_per_cluster) as u32;
}

fn get_next_cluster(cluster_num: u32) {
    let fat_offset = cluster_num * 32;
    let sector_number = fat_offset / 512;
    let byte_offset = fat_offset % 512;

    print!("{} {}\n", sector_number, byte_offset);

    // TODO: Check value to get next cluster, etc
    let fat = FS.lock().fat_address as *const u8;

    unsafe {
        print!("{:x}\n", *(fat.offset(0)));
        print!("{:x}\n", *(fat.offset(1)));
        print!("{:x}\n", *(fat.offset(2)));
    }
}

fn create_vfs(mut root_directory_address: u32, root_directory_size: u32, first_data_sector: u32) {
    let limit = root_directory_size / 0x20; // Each entry in root directory is 0x20 bytes
    for i in 0..limit {
        let entry = unsafe { &*(root_directory_address as *const StandardDirectoryEntry) };
        print!("{:?}\n", entry);

        let mut data_address = 0;
        data_address = (512 * get_lba(entry.cluster_low as u32)) + first_data_sector;

        let data = unsafe { &*(data_address as *const StandardDirectoryEntry) };
        print!("{:?}\n", data);

        root_directory_address += 0x20;

        get_next_cluster(entry.cluster_low as u32);
    }
}