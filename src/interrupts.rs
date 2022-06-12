// src/interrupts.rs

/*
Interrupts are signal which stop the operation flow of a computer in order to perform a set action (pressing a key)
After the CPU performs the action it returns to whatever it was doing
This is far more efficient then the CPU polling a device
An interrupt descriptor table defines what each interrupt will do
*/

use crate::print;
use crate::vga_text::TERMINAL;
use core::arch::asm;
use core::mem::size_of;
use core::prelude::v1::Some;
use lazy_static::lazy_static;

// 256 entries within the IDT with the first 32 being exceptions
const IDT_MAX_DESCRIPTIONS: u64 = 256;

// ISR's are interrupt service routines which are called on an interrupt
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct idt_entry {
    isr_low: u16, // low 16 bits of ISR address
    kernel_cs: u16, // GDT segment CPU loads before calling ISR
    ist: u8, // 0
    attributes: u8, // Type and attributes
    isr_mid: u16, // mid 16 bits of ISR address
    isr_high: u32, // upper 32 bits of ISR address
    reserved: u32 // Set to 0
}

#[repr(C, packed)]
pub struct idtr {
    pub limit: u16,
    base: u64
}

impl idtr {
    fn new() -> idtr {
        idtr {
            limit: 0,
            base: 0
        }
    }
}

#[no_mangle]
pub static mut IDTR: idtr = unsafe { idtr { limit: 0, base: 0 } };
pub static mut idt: [idt_entry; 256] = [idt_entry { isr_low: 0, kernel_cs: 0x08, ist: 0, attributes: 0, isr_mid: 0, isr_high: 0, reserved: 0}; 256];

pub fn init_idt() {
    unsafe {
        let idt_address = (&idt[0] as *const idt_entry) as u64;
        IDTR.limit = (size_of::<idt_entry>() as u16) * (IDT_MAX_DESCRIPTIONS as u16 - 1);
        IDTR.base = idt_address;

        let func_address = (handle_no_err_exception0 as *const u64) as u64;

        idt[0].attributes = 0x8E;
        idt[0].isr_low = (func_address & 0xFFFF) as u16;
        idt[0].isr_mid = ((func_address >> 16) & 0xFFFF) as u16;
        idt[0].isr_high = (func_address >> 32) as u32;

        let idtr_address = (&IDTR as *const idtr);

        idt_flush();
    }
}

#[no_mangle]
pub extern fn exception_handler(rax: u64) {
    print!("Yo you divided 0 by 0");
    unsafe { asm!("cli; hlt"); }
}

extern "C" {
    fn handle_no_err_exception0();
    fn idt_flush();
}