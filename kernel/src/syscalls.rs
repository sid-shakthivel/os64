// /src/syscalls.rs

/*
    System calls are used to call a kernel service from userland as certain actions must be done with privilege
    Syscalls can be used for process management, file management, communication, and information maintainence
    They are invoked with software interrupts
    Sidos syscall design is inspired by posix
*/

use crate::framebuffer::{self, Event, Rectangle, Window, DESKTOP, FRAMEBUFFER};
use crate::fs::File;
use crate::grub::{DOOM1_WAD_OFFSET, DOOM1_WAD_ORIGINAL, DOOM_SIZE};
use crate::hashmap::HashMap;
use crate::interrupts::Registers;
use crate::list::Stack;
use crate::multitask::PROCESS_SCHEDULAR;
use crate::page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR};
use crate::print_serial;
use crate::spinlock::Lock;
use crate::CONSOLE;
use bitflags::bitflags;
use core::panic;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct SlimmedWindow {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    x_final: i32,
    y_final: i32,
}

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

    // print_serial!("SYSCALL {}\n", syscall_id);

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
        11 => create_window(registers.rbx, registers.rcx, registers.rdi, registers.rsi),
        12 => desktop_paint(),
        13 => get_event(),
        14 => draw_string(
            registers.rbx as *const u8,
            registers.rcx as *const SlimmedWindow,
        ),
        15 => lseek(registers.rdx, registers.rcx as i64, registers.rbx),
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

    let mut filepath = crate::string::get_string_from_ptr(name);
    filepath = &filepath[0..filepath.len() - 1];

    if filepath == "DOOM1.WAD" {
        print_serial!("Opening doom1.wad");
        return 325;
    }

    print_serial!("OPEN'ING {:?}\n", filepath.as_bytes());

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
            // panic!("RELATIVE FILE PATH of {}\n", filepath.as_bytes()[0]);
            return -1;
        }
    }
}

fn allocate_pages(pages_required: u64) -> i64 {
    let address = PAGE_FRAME_ALLOCATOR.lock().alloc_frames(pages_required);
    PAGE_FRAME_ALLOCATOR.free();
    unsafe {
        *address = 1;
        *address = 0;
    }
    print_serial!(
        "ALLOC PAGES = {} 0x{:x} {:p}\n",
        pages_required,
        address as i64,
        address
    );
    address as i64
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
        2 => {
            // 2 refers to stderr and writes to the console
            for i in 0..(length) {
                let character = unsafe { *buffer.offset(i as isize) };
                print_serial!("{}", character as char);
            }
        }
        _ => {
            panic!("OH DOOM");
            // Other files can be written to through the fs
            // let wrapped_fd = FILE_TABLE.lock().get(file as usize);
            // FILE_TABLE.free();
            // match wrapped_fd {
            //     Some(mut fd) => {
            //         fd.write(buffer, length as usize).unwrap();
            //     }
            //     None => {
            //         return -1;
            //     }
            // }
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
            // Copies length bytes to a buffer
            print_serial!("READ'ING {} {:p} {}\n", file, buffer, length);
            unsafe {
                let src_buffer = DOOM1_WAD_ORIGINAL as *mut u8;
                for i in 0..length {
                    *buffer.offset(i as isize) = *src_buffer.offset(i as isize);
                }
            }

            // File system is out of action /
            // let wrapped_fd = FILE_TABLE.lock().get(file as usize);
            // FILE_TABLE.free();
            // match wrapped_fd {
            //     Some(mut fd) => {
            //         fd.read(buffer, length as usize).unwrap();
            //     }
            //     None => {
            //         return -1;
            //     }
            // }
        }
    }

    length as i64
}

/*
    Custom syscall which creates a new window
*/
fn create_window(x: u64, y: u64, width: u64, height: u64) -> i64 {
    let new_window = Window::new(
        "Window",
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

/*
    Returns the key that a user pressed
*/
fn get_event() -> i64 {
    let event = DESKTOP.lock().handle_event().unwrap();
    DESKTOP.free();
    unsafe { event.offset(0) as i64 }
}

fn draw_string(string_ptr: *const u8, window: *const SlimmedWindow) -> i64 {
    let mut string = crate::string::get_string_from_ptr(string_ptr);

    print_serial!("str = {}\n", string);

    string = &string[0..string.len() - 1]; // Remove null terminator?

    print_serial!("str = {}\n", string);

    let v = unsafe { core::ptr::read_unaligned(window) };
    FRAMEBUFFER.lock().draw_string(
        None,
        string,
        v.x_final as u64,
        v.y_final as u64,
        v.x as u64,
        v.y as u64,
        v.width as u64,
        v.height as u64,
    );

    return 0;
}

fn lseek(file: u64, offset: i64, whence: u64) -> i64 {
    print_serial!("LSEEK'ING {} {} {}\n", file, offset, whence);

    if offset < 0 {
        panic!("oh dear");
    }

    match whence {
        0 => {
            // SEEK_SET (begining of file)
            unsafe {
                DOOM1_WAD_OFFSET = (offset as u64) + DOOM1_WAD_ORIGINAL;
                return 0;
            }
        }
        1 => {
            // SEEK_CUR (current location of file)
            unsafe {
                DOOM1_WAD_OFFSET += offset as u64;
                return 0;
            }
        }
        2 => {
            // SEEK_END (end of file)
            unsafe {
                DOOM1_WAD_OFFSET = DOOM1_WAD_ORIGINAL + DOOM_SIZE + offset as u64;
                return 0;
            }
        }
        _ => panic!("OH NOP"),
    }

    return -1;
}
