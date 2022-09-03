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
use crate::page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR};
use crate::print_serial;
use crate::spinlock::Lock;
use crate::CONSOLE;
use bitflags::bitflags;
use core::panic;

/*
    File descriptor table is hashmap of file descriptors which point to actual files
    File table entries are created when a process requests to open a file and this maintains its validity and is used
*/
pub static FILE_TABLE: Lock<HashMap<File>> = Lock::new(HashMap::<File>::new());
pub static mut COUNTER: i64 = 0;

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
        0 => _exit(),
        1 => close(registers.rbx),
        2 => {
            // fstat
            return -1;
        }
        3 => getpid(),
        4 => isatty(registers.rbx),
        5 => kill(registers.rbx, registers.rcx),
        6 => {
            // link
            return -1;
        }
        7 => open(registers.rbx as *const u8, registers.rcx),
        8 => allocate_pages(registers.rbx),
        9 => write(registers.rbx, registers.rcx as *mut u8, registers.rdx),
        10 => read(registers.rbx, registers.rcx as *mut u8, registers.rdx),
        11 => create_window(registers.rbx, registers.rcx, registers.rdx, registers.rsi),
        12 => desktop_paint(),
        _ => panic!("Unknown Syscall {}\n", syscall_id),
    };
}

// Terminates process without cleaning files
fn _exit() -> i64 {
    // Get current pid
    let pid = PROCESS_SCHEDULAR.lock().current_process_index;
    PROCESS_SCHEDULAR.free();

    // Remove from array
    PROCESS_SCHEDULAR.lock().remove_process(pid);
    PROCESS_SCHEDULAR.free();

    print_serial!("TASK {} EXITED\n", pid);

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

    let filepath = crate::string::get_string_from_ptr(name);

    // Parse filename
    match filepath.as_bytes()[0] {
        0x2F => {
            // Absolute path starting from the root of the entire fs
            match crate::fs::parse_absolute_filepath(filepath) {
                Ok(file) => unsafe {
                    COUNTER += 1;

                    FILE_TABLE.lock().set(COUNTER as usize, file);
                    FILE_TABLE.free();

                    return COUNTER;
                },
                Err(error) => {
                    let file_flags = Flags::from_bits_truncate(flags as u32);

                    if file_flags.contains(Flags::O_CREAT) {
                        let file = crate::fs::create_new_root_file(filepath);
                        unsafe {
                            COUNTER += 1;
                            FILE_TABLE.lock().set(COUNTER as usize, file);
                            FILE_TABLE.free();

                            return COUNTER as i64;
                        }
                    }

                    1
                }
            }
        }
        _ => {
            // Relative path within directory
            // Relative paths are used when files are within same directory
            panic!("RELATIVE FILE PATH of {}\n", filepath.as_bytes()[0]);
        }
    }
}

fn allocate_pages(pages_required: u64) -> i64 {
    let address = PAGE_FRAME_ALLOCATOR.lock().alloc_frames(pages_required);
    PAGE_FRAME_ALLOCATOR.free();

    address as i64
}

/*
    Dynamically change the amount of space allocated for a process
    Resets the break value and allocates space which is set to zero
    Break value is the first byte of unallocated memory
    If successful, returns the prior break value or else returns -1
*/
fn sbrk(increment: u64) -> i64 {
    print_serial!(
        "INCREMENT = {} {} {}\n",
        increment as i32,
        increment as i16,
        increment as i8
    );

    let correct_increment = increment as i16;

    // Get the current process
    let index = PROCESS_SCHEDULAR.lock().current_process_index;
    PROCESS_SCHEDULAR.free();

    let current_process = PROCESS_SCHEDULAR.lock().get_current_process().unwrap();
    PROCESS_SCHEDULAR.free();

    // Save the old address
    let break_value = current_process.heap;

    // Increment the break value and save
    let updated_process = Process {
        heap: (break_value + correct_increment as i32),
        ..PROCESS_SCHEDULAR.lock().tasks[index].unwrap()
    };
    PROCESS_SCHEDULAR.free();

    PROCESS_SCHEDULAR.lock().tasks[index] = Some(updated_process);
    PROCESS_SCHEDULAR.free();

    print_serial!(
        "{} {}\n",
        break_value,
        break_value + correct_increment as i32
    );

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
            for i in 0..(length) {
                let character = unsafe { *buffer.offset(i as isize) };
                print_serial!("{}", character as char);
            }
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
        x,
        y,
        width,
        height,
        Some(DESKTOP.lock()),
        framebuffer::WINDOW_BACKGROUND_COLOUR,
    );
    DESKTOP.free();

    DESKTOP.lock().add_sub_window(new_window);
    DESKTOP.free();

    0
}

/*
    Custom syscall which paints the desktop
*/
fn desktop_paint() -> i64 {
    DESKTOP.lock().paint(Stack::<Rectangle>::new(), true);
    DESKTOP.free();

    0
}
