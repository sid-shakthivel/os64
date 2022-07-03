// /src/syscalls.rs

use crate::interrupts::Registers;
use crate::print;
use crate::TERMINAL;

#[no_mangle]
pub extern fn syscall_handler(registers: Registers) {
    let unaligned_registers = core::ptr::addr_of!(registers);
    let aligned_registers = unsafe { core::ptr::read_unaligned(unaligned_registers) };

    // print!("{:?}\n", aligned_registers);

    let syscall_id = registers.rax;

    // match syscall_id {
    //     4 => {
    //         // sys_write

    //         let message_length = registers.rdx;
    //         let message: *const char = (0x3f3000) as _;

    //         unsafe {
    //             for i in 0..message_length {
    //                 print!("{}", *(message.offset(i as isize)));
    //             }
    //         }
    //     }
    //     _ => panic!("Unknown Syscall\n");
    // }

    let message_length = registers.rdx;

    // for i in 0..message_length {
        // let char = unsafe { *((0x3f3000 + i) as *const u8) };
        // print!("{}", char as char);
    // }

    print!("Syscall: Writing String\n");
}