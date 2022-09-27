// src/filesystem.rs

/*
    Driver for the FAT 16 file system (logical way to store, read, write data)
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

#![allow(dead_code)]
#![allow(unused_variables)]

use crate::CONSOLE;
use core::mem;
use spin::Mutex;

use crate::print_serial;

pub struct Fat16 {
    bpb: Option<BiosParameterBlock>,
    start_address: u32,
    fat_address: u32,
    first_data_sector_address: u32,
    root_directory_address: u32,
    initrd: Option<File>,
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
    table_size_16: u16,     // Number of sectors per FAT
    sectors_per_track: u16, // Number of sectors per track
    head_count: u16,
    hidden_sector_count: u32,
    sector_count_32: u32,
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
    bootable_partition_signature: u16,
}

// Stores information on where a file's data/folder are stored on disk along with name, size, creation
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct StandardDirectoryEntry {
    filename: [u8; 8],
    ext: [u8; 3],
    attributes: u8,    // Could be LFN, Directory, Archive
    unused: [u8; 8],   // Reserved for windows NT
    cluster_high: u16, // Always 0
    time: u16,
    date: u16,
    cluster_low: u16,
    file_size: u32,
}

// These always have a regular entry as well, and these are placed before the standard entry and hold extra data
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct LongFileEntry {
    order: u8,             // Since there could be number of LFE's, order is important
    name_start: [u16; 5],  // First 5 characters
    attribute: u8,         // 0x0F
    long_entry_type: u8,   // 0x00
    checksum: u8,          // Checksum generated fo the short file name
    name_middle: [u16; 6], // Next 6 characters
    zero: u16,             // Always 0
    name_end: [u16; 2],    // Final 2 characters
}

// Inspired by the ubiquitous FILE* data type in C
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C, packed)]
pub struct File {
    name: [u8; 8],
    flags: u32,
    size: u32,
    cluster: u32,
    index: u32,
    file_type: FileType,
    offset: i64,
    // permissions: u32,
    // uid: u32,
    // gid: u32,
}

#[derive(Copy, PartialEq, Clone, Debug)]
pub enum FileType {
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
            offset: 0,
        };
    }

    pub fn set_offset(&mut self, new_offset: i64) {
        self.offset = new_offset;
    }

    pub fn get_offset(&self) -> i64 {
        self.offset
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    pub fn read(&mut self, buffer: *mut u8, length: usize) -> Result<u64, &str> {
        if self.file_type != FileType::File {
            return Err("Tried to read on a directory");
        }

        self._modify(buffer, false, length);

        return Ok(0);
    }

    pub fn write(&mut self, buffer: *mut u8, length: usize) -> Result<u64, &str> {
        if self.file_type != FileType::File {
            return Err("Tried to write on a directory");
        }

        self._modify(buffer, true, length);

        return Ok(0);
    }

    pub fn find(&self, filename: &str) -> Result<File, &str> {
        let cluster_address =
            convert_sector_to_bytes(get_lba(self.cluster)) + FS.lock().first_data_sector_address;
        return self._find(filename, cluster_address);
    }

    pub fn find_root(&self, filename: &str) -> Result<File, &str> {
        let root_directory_address = FS.lock().root_directory_address;
        return self._find(filename, root_directory_address);
    }

    pub fn readdir(&mut self) -> Result<*const [File; 64], &str> {
        if self.file_type != FileType::Directory {
            return Err("Tried to open on a file");
        }
        let cluster_address =
            convert_sector_to_bytes(get_lba(self.cluster)) + FS.lock().first_data_sector_address;
        return Ok(unsafe { &*(cluster_address as *const [File; 64]) });
    }

    fn _mk(
        &mut self,
        mut cluster_address: u32,
        filename: &str,
        filetype: FileType,
    ) -> Result<File, &str> {
        // Search the directory for an empty space
        for i in 0..64 {
            let directory_entry = unsafe { &*(cluster_address as *const StandardDirectoryEntry) };

            // Free entry
            if directory_entry.filename[0] == 0 {
                unsafe {
                    let directory_entry_mut =
                        &mut *(cluster_address as *mut StandardDirectoryEntry);
                    let file_name_bytes = filename.as_bytes();

                    for j in 0..8 {
                        if j < file_name_bytes.len() {
                            directory_entry_mut.filename[j] = file_name_bytes[j];
                        }
                    }

                    directory_entry_mut.attributes = match filetype {
                        FileType::Directory => 0x10,
                        FileType::File => 0x20,
                        _ => panic!("Unknown file type"),
                    };

                    directory_entry_mut.cluster_low = get_next_unallocated_cluster().unwrap();

                    let mut node = File::new(
                        directory_entry_mut.cluster_low as u32,
                        directory_entry_mut.file_size,
                        FileType::File,
                    );
                    node.name = directory_entry.filename;
                    return Ok(node);
                }
            };

            cluster_address += 0x20;
        }

        return Err("Directory is full");
    }

    pub fn mkdir_root(&mut self, filename: &str) -> Result<File, &str> {
        if self.file_type != FileType::Directory {
            return Err("Tried to mkdir on a file");
        }

        let root_directory_address = FS.lock().root_directory_address;

        self._mk(root_directory_address, filename, FileType::Directory)
    }

    pub fn mkdir(&mut self, filename: &str) -> Result<File, &str> {
        if self.file_type != FileType::Directory {
            return Err("Tried to mkdir on a file");
        }

        let cluster_address =
            convert_sector_to_bytes(get_lba(self.cluster)) + FS.lock().first_data_sector_address;

        self._mk(cluster_address, filename, FileType::Directory)
    }

    pub fn mkf(&mut self, filename: &str) -> Result<File, &str> {
        if self.file_type != FileType::Directory {
            return Err("Tried to mkf on a file");
        }

        let cluster_address =
            convert_sector_to_bytes(get_lba(self.cluster)) + FS.lock().first_data_sector_address;

        self._mk(cluster_address, filename, FileType::File)
    }

    pub fn mkf_root(&mut self, filename: &str) -> Result<File, &str> {
        if self.file_type != FileType::Directory {
            return Err("Tried to mkf on a file");
        }

        let root_directory_address = FS.lock().root_directory_address;
        self._mk(root_directory_address, filename, FileType::File)
    }

    fn _modify(&mut self, buffer: *mut u8, write: bool, length: usize) {
        let mut cluster_address =
            convert_sector_to_bytes(get_lba(self.cluster)) + FS.lock().first_data_sector_address;

        let mut file_contents = cluster_address as *mut u8;

        let mut total_count = 0;
        let mut i = 0;

        let cluster_clone = self.cluster;

        // Get the correct cluster which needs to be addressed using the current offset
        unsafe {
            let offset = self.offset;
            if offset >= 2048 {
                for j in 0..(offset / 2048) {
                    match get_next_cluster(self.cluster) {
                        None => return, // End of file
                        Some(cluster_num) => {
                            cluster_address = convert_sector_to_bytes(get_lba(cluster_num))
                                + FS.lock().first_data_sector_address;
                            file_contents = cluster_address as *mut u8;
                            self.cluster = cluster_num;
                        }
                    }
                }
                file_contents = file_contents.offset((self.offset % 2048) as isize);
            } else {
                file_contents = file_contents.offset(self.offset as isize);
            }
        }

        while total_count < length {
            unsafe {
                if write {
                    *file_contents.offset(i as isize) = *buffer.offset(total_count as isize);
                } else {
                    *buffer.offset(total_count as isize) = *file_contents.offset(i as isize);
                }

                i += 1;
                total_count += 1;

                if i >= 2048 {
                    panic!("READING OVER A CLUSTER?");
                    i = 0;
                    match get_next_cluster(self.cluster) {
                        None => return, // End of file
                        Some(cluster_num) => {
                            cluster_address = convert_sector_to_bytes(get_lba(cluster_num))
                                + FS.lock().first_data_sector_address;
                            file_contents = cluster_address as *mut u8;
                        }
                    }
                }
            }
        }
        self.offset += length as i64;
        self.cluster = cluster_clone;
    }

    fn _find(&self, filename: &str, mut cluster_address: u32) -> Result<File, &str> {
        if self.file_type != FileType::Directory {
            return Err("Tried to find on a directory");
        }

        let mut filename_buffer: [u8; 12] = [0; 12];

        // Loop through each directory entry within the directory cluster
        for _i in 0..64 {
            let directory_entry = unsafe { &*(cluster_address as *const StandardDirectoryEntry) };

            match directory_entry.filename[0] {
                0x00 => return Err("File cannot be found in this directory"), // Marks the end (no more files/directories)
                0xE5 => panic!("Unused entry"),
                _ => {}
            }

            // Check against attributes of a directory entry
            match directory_entry.attributes {
                0x0F => {
                    // Handle long file name entry
                    let long_file_entry = unsafe { &*(cluster_address as *const LongFileEntry) };
                    let mut index = 0;

                    // Convert the lowercase into uppercase

                    for i in 0..5 {
                        filename_buffer[index] =
                            (long_file_entry.name_start[i] as u8).wrapping_sub(32);
                        index += 1;
                    }

                    for i in 0..6 {
                        filename_buffer[index] =
                            (long_file_entry.name_middle[i] as u8).wrapping_sub(32);
                        index += 1
                    }

                    for i in 0..1 {
                        filename_buffer[index] =
                            (long_file_entry.name_end[i] as u8).wrapping_sub(32);
                        index += 1;
                    }
                }
                0x10 => {
                    // Directory
                    let dos_filename: &str;
                    let filename_clone = filename_buffer.clone();

                    if filename_buffer[0] != 0 {
                        // There is a corresponding long file name for this entry
                        dos_filename = core::str::from_utf8(&filename_clone).unwrap().trim();

                        // Clean up
                        for i in 0..12 {
                            filename_buffer[i] = 0;
                        }
                    } else {
                        // Use the standard filename
                        dos_filename = core::str::from_utf8(&directory_entry.filename)
                            .unwrap()
                            .trim();
                    }

                    let mut split_filename = filename.split(".");

                    if split_filename.next().unwrap() == dos_filename {
                        let mut node = File::new(
                            directory_entry.cluster_low as u32,
                            directory_entry.file_size,
                            FileType::Directory,
                        );
                        node.name = directory_entry.filename;
                        return Ok(node);
                    }
                }
                0x20 => {
                    // Archive

                    let dos_filename: &str;
                    let filename_clone = filename_buffer.clone();

                    if filename_buffer[0] != 0 {
                        // There is a corresponding long file name for this entry
                        dos_filename = core::str::from_utf8(&filename_clone).unwrap().trim_end();

                        // Clean up
                        for i in 0..12 {
                            filename_buffer[i] = 0;
                        }
                    } else {
                        // Use the standard filename
                        dos_filename = core::str::from_utf8(&directory_entry.filename)
                            .unwrap()
                            .trim();
                    }

                    let mut split_filename = filename.split(".");

                    if split_filename.next().unwrap().trim() == dos_filename.trim() {
                        print_serial!("ALL DONE");
                        let mut node = File::new(
                            directory_entry.cluster_low as u32,
                            directory_entry.file_size,
                            FileType::File,
                        );
                        node.name = directory_entry.filename;
                        return Ok(node);
                    }
                }
                _ => panic!("Unknown attribute"),
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
            root_directory_address: 0,
            initrd: None,
        }
    }
}

// Should ensure it's FAT16 and check certain values
fn validate_fat(ebr: &ExtendedBootRecord) -> bool {
    // TODO: Calculate number of clusters and check whether smaller then 65525 and expand

    if ebr.signature != 0x28 && ebr.signature != 0x29 {
        panic!("Invalid signature, {:x}", ebr.signature);
    }
    // if ebr.bootable_partition_signature != 0xAA55 {
    //     panic!("Invalid partition signature");
    // }

    return true;
}

fn get_next_cluster(cluster_num: u32) -> Option<u32> {
    let fat_offset = cluster_num * 2;
    let sector_number = fat_offset / 512;
    let byte_offset = fat_offset % 512;

    let next_cluster = read_fat(sector_number, byte_offset as usize);

    return match next_cluster {
        0xFFF7 => panic!("Bad cluster!"), // Indicates bad cluster
        0xFFF8..=0xFFFF => None,          // Indicates the whole file has been read
        _ => Some(next_cluster as u32),   // Gives next cluster number
    };
}

fn get_next_unallocated_cluster() -> Option<u16> {
    let fat = unsafe { &mut *((FS.lock().fat_address) as *mut [u8; 512]) };
    for i in 0..512 {
        if ((fat[i + 1] as u16) << 8 | (fat[i] as u16)) == 0 {
            fat[i] = 0xFF;
            fat[i + 1] = 0xFF;
            return Some(i as u16);
        }
    }
    return None;
}

/*
    Clusters represent linear addresses, sectors use segment addresses
    LBA represents an indexed location on the disk
*/
fn get_lba(cluster_num: u32) -> u32 {
    return (cluster_num - 2) * (FS.lock().bpb.unwrap().sectors_per_cluster) as u32;
}

// Uses 16 bits to address clusters
fn read_fat(sector_num: u32, byte_offset: usize) -> u16 {
    let fat = unsafe {
        &*((FS.lock().fat_address + convert_sector_to_bytes(sector_num)) as *const [u8; 512])
    };
    return ((fat[byte_offset + 1] as u16) << 8) | (fat[byte_offset] as u16); // Little endian
}

// Most addresses are calculated sectors and therefore must be converted into bytes to be read/written
fn convert_sector_to_bytes(sector: u32) -> u32 {
    return sector * 512;
}

pub static FS: Mutex<Fat16> = Mutex::new(Fat16::new());

pub fn init(start_address: u32) {
    let bpb = unsafe { &*(start_address as *const BiosParameterBlock) };

    let ebr_address = start_address + (mem::size_of::<BiosParameterBlock>() as u32);
    let ebr = unsafe { &*(ebr_address as *const ExtendedBootRecord) };

    validate_fat(ebr);

    let first_fat = start_address + convert_sector_to_bytes(bpb.reserved_sector_count as u32);
    let fat_size: u32 = bpb.table_size_16 as u32;

    let root_directory_sector: u32 =
        (bpb.reserved_sector_count as u32) + ((bpb.table_count as u32) * fat_size);

    let root_directory_address: u32 =
        start_address + convert_sector_to_bytes(root_directory_sector);

    let root_directory_size: u32 = ((((bpb.root_entry_count) * 32) + (bpb.bytes_per_sector - 1))
        / bpb.bytes_per_sector) as u32;
    let first_data_sector: u32 =
        convert_sector_to_bytes(root_directory_size) + root_directory_address;

    let initrd: File = File::new(root_directory_sector, 512, FileType::Directory);

    FS.lock().bpb = Some(bpb.clone());
    FS.lock().start_address = start_address;
    FS.lock().fat_address = first_fat;
    FS.lock().first_data_sector_address = first_data_sector;
    FS.lock().root_directory_address = root_directory_address;
    FS.lock().initrd = Some(initrd);
}

// Copies number of bytes into destination
pub unsafe fn memcpy_cluster(dest: *mut u8, src: *mut u8, index: u32) {
    for i in 0..2048 {
        let offset = ((index * 2048) + i) as isize;
        *(dest.offset(offset)) = *(src.offset(offset));
    }
}

/*
    Parses an absolute filepath and returns a file descriptor
    File descriptor is a unqiue unsigned integer which is used to identify an open file
*/
pub fn parse_absolute_filepath(filepath: &str) -> Result<File, &str> {
    let cleaned_filepath: &str = &filepath[0..filepath.len()]; // Remove the initial / for ease

    let mut current_fd = FS.lock().initrd.unwrap();

    // Split path against /
    let mut filepath_components = cleaned_filepath.split("/");

    match filepath_components.next() {
        Some(first_component) => {
            current_fd = current_fd.find_root(first_component).unwrap();

            // Loop through each other portion and return the file/directory needed
            for component in filepath_components {
                let result = current_fd.find(component);

                if result.is_ok() {
                    current_fd = result.unwrap();
                } else {
                    return Err("File not found");
                }
            }
        }
        None => {
            return Err("Filepath is in incorrect format");
        }
    }

    return Ok(current_fd);
}

// Replace this by parsing the file and creating it if it doesn't exist in a specific path
pub fn create_new_root_file(filename: &str) -> File {
    let mut current_fd = FS.lock().initrd.unwrap();
    let fd = current_fd.mkf_root(filename).unwrap();
    fd
}

pub fn round_to_nearest_cluster(size: u64) -> u64 {
    ((size as i64 + 2047) & (-2048)) as u64
}
