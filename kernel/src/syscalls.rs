// /src/syscalls.rs

/*
    Syscalls are used to call a kernel service from userland - certain actions must be done with privilege
    They are invoked with software interrupts

    +-----+-----------+
    | RAX |   Name    |
    +-----+-----------+
    |   1 | sys_exit  |
    |   2 | sys_fork  |
    |   3 | sys_read  |
    |   4 | sys_write |
    |   5 | sys_open  |
    |   6 | sys_close |
    +-----+-----------+
*/

use crate::interrupts::Registers;
use crate::print;
use crate::TERMINAL;

// TODO: Add support for more syscalls

#[no_mangle]
pub extern fn syscall_handler(registers: Registers) {
    let syscall_id = registers.rax;
    
    match syscall_id {
        4 => {
            // sys_write
            let message_length = registers.rdx;
            let data = registers.rcx;

            for i in 0..message_length {
                let char = unsafe { *((data + i) as *const u8) };
                print!("{}", char as char);
            }
        }
        _ => panic!("Unknown Syscall\n")
    };
}