// /src/syscalls.rs

/*
    System calls are used to call a kernel service from userland as certain actions must be done with privilege
    Syscalls can be used for process management, file management, communication, and information maintainence
    They are invoked with software interrupts
    Sidos syscall design is inspired by posix
*/

use crate::framebuffer::{self, Rectangle, Window, DESKTOP};
use crate::fs::File;
use crate::hashmap::HashMap;
use crate::interrupts::Registers;
use crate::list::Stack;
use crate::multitask::Process;
use crate::multitask::PROCESS_SCHEDULAR;
use crate::print_serial;
use crate::spinlock::Lock;
use crate::CONSOLE;
use bitflags::bitflags;
use core::panic;
use core::str::from_utf8;

/*
    File descriptor table is hashmap of file descriptors which point to actual files
    File table entries are created when a process requests to open a file and this maintains its validity and is used
*/
pub static FILE_TABLE: Lock<HashMap<File>> = Lock::new(HashMap::<File>::new());
pub static mut COUNTER: usize = 0;

bitflags! {
    struct Flags: u32 {
        const O_RDONLY = 0x0000; // Open for reading only
        const O_WRONLY = 0x0001; // Open for writing only
        const O_RDWR = 0x0002; // Open for reading and writing
        const O_CREAT = 0x0200; // Create file if it doesn't exist
    }
}

#[no_mangle]
pub extern "C" fn syscall_handler(registers: Registers) -> i64 {
    let syscall_id = registers.rax;

    return match syscall_id {
        0 => _exit(registers.rbx),
        1 => close(registers.rbx),
        2 => {
            // fstat
            return -1;
        }
        3 => getpid(),
        5 => isatty(registers.rbx),
        6 => kill(registers.rbx, registers.rcx),
        7 => {
            // link
            return -1;
        }
        8 => open(registers.rbx as *const u8, registers.rcx),
        9 => sbrk(registers.rbx as i64),
        10 => write(registers.rbx, registers.rcx as *mut u8, registers.rdx),
        11 => read(registers.rbx, registers.rcx as *mut u8, registers.rdx),
        12 => create_window(registers.rbx, registers.rcx, registers.rdx, registers.rsi),
        13 => desktop_paint(),
        _ => panic!("Unknown Syscall {}\n", syscall_id),
    };
}

// Terminates process without cleaning files
fn _exit(status: u64) -> i64 {
    // Get current pid
    let pid = PROCESS_SCHEDULAR.lock().current_process_index;
    PROCESS_SCHEDULAR.free();

    // Remove from array
    PROCESS_SCHEDULAR.lock().remove_process(pid);
    PROCESS_SCHEDULAR.free();

    print_serial!("TASK {} EXITED WITH STATUS CODE {}\n", pid, status);

    return 0;
}

// Closes a file which is pointed by fd
fn close(file: u64) -> i64 {
    FILE_TABLE.lock().remove(file as usize);
    FILE_TABLE.free();
    return 0; // Successful (-1 unsuccessful)
}

// Query to check if file is a terminal
fn isatty(file: u64) -> i64 {
    if file == 0 || file == 1 || file == 2 {
        return 1;
    }
    return -1;
}

// Returns the process id of the current process
fn getpid() -> i64 {
    // Get current process and return its pid
    let wrapped_process = PROCESS_SCHEDULAR.lock().get_current_process();
    if let Some(process) = wrapped_process {
        return process.pid as i64;
    }
    return -1;
}

// Sends signals to process group or process - may require IPC
fn kill(pid: u64, sig: u64) -> i64 {
    if sig == 0 {
        // If sig is 0, no signal is sent
        return -1;
    }

    // If pid is positive, signal sig is sent to the process

    // If pid is 0, signal is sent to every process in process group

    // If pid is -1, signal is sent to every process in which the calling process has permission to send

    // If pid is under -1, sig is sent to every process in process group whose ID is -pid

    return 0;
}

// Used to open a file for reading/writing and returns the file number
fn open(name: *const u8, flags: u64) -> i64 {
    // Get name of file
    let len = strlen(name);

    let filepath_array = unsafe { core::slice::from_raw_parts(name, len) };

    let filepath = from_utf8(filepath_array).unwrap().trim();

    // Parse filename
    match filepath_array[0] {
        0x2F => {
            // Absolute path starting from the root of the entire fs
            let file = crate::fs::parse_absolute_filepath(filepath).unwrap();

            unsafe {
                FILE_TABLE.lock().set(COUNTER, file);
                FILE_TABLE.free();
                COUNTER += 1;

                return COUNTER as i64;
            }
        }
        _ => {
            // Relative path within directory
            // Relative paths are used when files are within same directory
            panic!("RELATIVE FILE PATH of {}\n", filepath_array[0]);
        }
    }

    // let file_flags = Flags::from_bits_truncate(flags as u32);

    // if file_flags.contains(Flags::O_CREAT) {
    //     // Create new file
    // }

    // if file_flags.contains(Flags::O_RDWR) {
    //     // Read and write
    // }
}

/*
    Dynamically change the amount of space allocated for a process
    Resets the break value and allocates space which is set to zero
    Break value is the first byte of unallocated memory
    If successful, returns the prior break value or else returns -1
*/
fn sbrk(increment: i64) -> i64 {
    // Get the current process
    let index = PROCESS_SCHEDULAR.lock().current_process_index;
    PROCESS_SCHEDULAR.free();

    let current_process = PROCESS_SCHEDULAR.lock().get_current_process().unwrap();
    PROCESS_SCHEDULAR.free();

    // Save the old address
    let break_value = current_process.heap;

    // TODO: Check boundries to avoid overwritting

    // Increment the break value and save
    let updated_process = Process {
        heap: break_value + increment,
        ..PROCESS_SCHEDULAR.lock().tasks[index].unwrap()
    };
    PROCESS_SCHEDULAR.free();

    PROCESS_SCHEDULAR.lock().tasks[index] = Some(updated_process);
    PROCESS_SCHEDULAR.free();

    // Return old break
    return break_value as i64;
}

/*
    Writes given length of bytes from buffer to the file specified
    Length must be above 0 and under max value
*/
fn write(file: u64, buffer: *mut u8, length: u64) -> i64 {
    if length == 0 {
        return 0;
    }
    if length > u64::max_value() {
        return -1;
    }

    match file {
        1 => {
            // 1 refers to stdout and writes to the console
            for i in 0..(length + 3) {
                let character = unsafe { *buffer.offset(i as isize) };
                print_serial!("{}\n", character as char);
            }
            print_serial!("\n");
        }
        _ => {
            // Other files can be written to through the fs
            let wrapped_fd = FILE_TABLE.lock().get(file as usize);
            FILE_TABLE.free();
            match wrapped_fd {
                Some(mut fd) => {
                    fd.write(buffer, length as usize).unwrap();
                }
                None => {
                    return -1;
                }
            }
        }
    }

    length as i64
}

/*
    Reads given length of bytes into the buffer
*/
fn read(file: u64, buffer: *mut u8, length: u64) -> i64 {
    // TODO: Account for special files like stdin

    match file {
        0 => {
            // stdin
            panic!("STDIN\n");
        }
        _ => {
            let wrapped_fd = FILE_TABLE.lock().get(file as usize);
            FILE_TABLE.free();
            match wrapped_fd {
                Some(mut fd) => {
                    fd.read(buffer, length as usize).unwrap();
                }
                None => {
                    return -1;
                }
            }
        }
    }

    length as i64
}

/*
    Custom syscall which creates a new window
*/
fn create_window(x: u64, y: u64, width: u64, height: u64) -> i64 {
    let new_window = Window::new(
        10,
        10,
        300,
        300,
        Some(DESKTOP.lock()),
        framebuffer::WINDOW_BACKGROUND_COLOUR,
    );
    DESKTOP.free();

    DESKTOP.lock().add_sub_window(new_window);
    DESKTOP.free();

    0
}

/*
    Custom syscall which paints the scene
*/
fn desktop_paint() -> i64 {
    DESKTOP.lock().paint(Stack::<Rectangle>::new(), true);
    DESKTOP.free();

    0
}

fn strlen(mut string: *const u8) -> usize {
    let mut count = 0;
    loop {
        count += 1;
        unsafe {
            if *string == 0 {
                return count as usize;
            }
            string = string.offset(1);
        }
    }
}
