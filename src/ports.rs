// /src/ports.rs

// Manages all functions related to input/output 

use core::arch::asm;

// pub fn outb(port: u16, value:  u8) {
//     unsafe {
//         // asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(val));
//         asm! ("outb {1}, {2}", )
//     }
// }

// pub fn inb(port: u16) -> u8 {
//     let result: u8;
//     // llvm_asm!("inb %dx, %al" : "={al}"(result) : "{dx}"(port) :: "volatile");
//     result
// }

pub fn io_wait() {
    unsafe {
        asm!("out 0x80, eax", in("eax") 0x00);
    }
}