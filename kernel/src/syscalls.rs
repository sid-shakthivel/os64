// src/syscalls.rs

use crate::interrupts::Registers;
use crate::print;
use crate::vga_text::TERMINAL;

// #[repr(C, packed)]
// #[derive(Debug, Copy, Clone)]
// pub struct SysCallRegisters {
//     rdi: u64,
//     rsi: u64,
//     rdx: u64,
//     rcx: u64,
//     rbx: u64,
//     rax: u64,
// }

#[no_mangle]
pub fn on_syscall(registers: Registers) {
    // Read usermode process stack
    let unaligned_registers = core::ptr::addr_of!(registers);
    let aligned_registers = unsafe { core::ptr::read_unaligned(unaligned_registers) };

    print!("{:?}\n", aligned_registers);

    // let syscall_id = registers.rax;

    // match syscall_id {
    //     4 => {
    //         // Write syscall
    //         let messageLength = registers.rdx;
    //         let message: *const char = registers.rcx as _;

    //         unsafe {
    //             for i in 0..messageLength {
    //                 print!("{}", *(message.Offset(i)));
    //             }
    //         }
    //     }
    //     _ => panic!("Unknown syscall");
    // }
}
