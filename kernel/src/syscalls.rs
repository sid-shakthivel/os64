// /src/syscalls.rs

/*
    System calls are used to call a kernel service from userland as certain actions must be done with privilege
    Syscalls can be used for process management, file management, communication, and information maintainence
    They are invoked with software interrupts
    Sidos syscall design is inspired by posix
*/

use core::panic;

use crate::interrupts::Registers;
use crate::print_serial;
use crate::print_vga;
use crate::CONSOLE;
use bitflags::bitflags;

bitflags! {
    struct Flags: u32 {
        const O_RDONLY = 0x0000; // Open for reading only
        const O_WRONLY = 0x0001; // Open for writing only
        const O_RDWR = 0x0002; // Open for reading and writing
        const O_ACCMODE = 0x0003; // Mask for above notes
        const O_CREAT = 0x0200; // Create file if it doesn't exist
        const O_EXCL = 0x0800; // Prevent creation if it already exists
    }
}

#[no_mangle]
pub extern "C" fn syscall_handler(registers: Registers) {
    let syscall_id = registers.rax;

    match syscall_id {
        0 => _exit(),
        1 => close(registers.rbx),
        2 => {
            // fstat
        }
        3 => getpid(),
        5 => isatty(registers.rbx),
        6 => kill(registers.rbx, registers.rcx),
        7 => {
            // link
        }
        8 => open(registers.rbx as *const u8, registers.rcx)
        9 => {
            // sbrk
        }
        10 => write(registers.rbx, registers.rcx as *const u8, registers.rdx)
        _ => panic!("Unknown Syscall\n"),
    }

    print_vga!("FINISHED SYSCALL\n");
}

// Terminates process without cleaning files
fn _exit() {
    // Get current process and removes it from the array
}

// Closes a file 
fn close(file: u64) -> i64 {
    // TODO: Need method to check if file is in use, etc
    return 0; // Successful (-1 unsuccessful)
}

// Query to check if file is a terminal
fn isatty(file: u64) -> i64 {
    if file == 0 || file == 1 || file == 2 {
        1
    }
    -1
}

// Returns the process id of the current process
fn getpid() {
    // Get current process and return it's ID
}

// Sends signals to process group or process
fn kill(pid: u64, sig: u64) {
    // If pid is positive, signal sig is sent to the process 

    // If pid is 0, signal is sent to every process in process group

    // If pid is -1, signal is sent to every process in which the calling process has permission to send

    // If pid is under -1, sig is sent to every process in process group whose ID is -pid

    // If sig is 0, no signal is sent
}

// Used to open a file for reading/writing and returns the file number
fn open(name: *const u8, flags: u64) -> i64 {
    /*
        Parse the filename provided appropriately to gain correct node
        Absolute paths start with /
        Relative paths are used when files are within same directory
    */

    // Find file

    let sys_flags =  Flags::from_bits_truncate(controller_config_byte);

    if sys_flags.contains(Flags::O_CREAT) {
        // Create new file 
    } 

    if sys_flags.contains(Flags::O_EXCL) {
        // Prevent creation if the file exists 
    } 

    if sys_flags.contains(Flags::O_RDWR) {
        // Read and write
    }

    0
}

/*
    Dynamically change the amount of space allocated for a process
    Resets the break value and allocates space which is set to zero
    Break value is the first byte of unallocated memory
    If successful, returns the prior break value or else returns -1
*/
fn sbrk(increment: u64) {
    // Get end of program memory

    // Save the old address

    // Check if incrementing heap will clash with something else

    // Increment the break value and return the old value
}

/*
    Writes bytes from buffer to the file specified
    Length must be above 0 and under max value
*/
fn write(file: u64, ptr: *const u8, length: u64) -> i64 {
    if length == 0 { 0 }
    if length > u64::max_value() { -1 }

    // TODO: Ensure that 
    match file {
        1 => {
            // 1 refers to stdout and writes to the console
            for i in 0..length {
                let character = unsafe { *((data + i) as *const u8) };
                print_serial!("{}", character as char);
            }
        }
        _ => {
            // TODO: Get number from filesystem given name somehow
            // Writes data from buffer into a file
            for i in 0..length {
                unsafe {
                    *ptr.offset(i as isize) = 1;
                }
            }
        }
    }

    length as i64
}

