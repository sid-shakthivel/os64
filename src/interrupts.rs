// src/interrupts.rs

/*
Interrupts are signal which stop the operation flow of a computer in order to perform a set action (pressing a key)
After the CPU performs the action it returns to whatever it was doing
This is far more efficient then the CPU polling a device
An interrupt descriptor table defines what each interrupt will do
*/

use crate::print;
use crate::vga_text::TERMINAL;
use core::mem::size_of;
use core::arch::asm;

// 256 entries within the IDT with the first 32 being exceptions
const IDT_MAX_DESCRIPTIONS: u64 = 256;

/*
    Each entry in IDT is 16 bytes
    Two gates include trap (handles exceptions) and interrupt for other interrupts
*/
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)] // By default structs are given padding - this should be disabled
pub struct idt_entry {
    isr_low: u16, // Low 16 bits of ISR address
    kernel_cs: u16, // GDT segment CPU loads before calling ISR
    ist: u8, // Offset into interrupt stack table which is unused (for now)
    attributes: u8, // Type and attributes
    isr_mid: u16, // Mid 16 bits of ISR address
    isr_high: u32, // Upper 32 bits of ISR address
    reserved: u32 // Set to 0
}

#[repr(C, packed)]
pub struct idtr {
    pub limit: u16, // Memory taken up by idt in bytes ((256 - 1) * 16)
    base: u64 // Base address of IDT
}

pub enum GateType {
    Trap,
    Interrupt
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Registers {
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    num: u64,
    error_code: u64
}

#[no_mangle]
pub static mut IDTR: idtr = idtr { limit: 0, base: 0 };
pub static mut IDT: [idt_entry; 256] = [idt_entry { isr_low: 0, kernel_cs: 0x08, ist: 0, attributes: 0, isr_mid: 0, isr_high: 0, reserved: 0}; 256];

const exception_messages: &'static [&'static str] = &["Divide By Zero", "Debug", "Non-maskable Interrupt", "Breakpoint", "Overflow", "Bound Range Exceeded", "Invalid Opcode", "Device not Available", "Double Fault", "Coprocessor Segment Overrun", "Invalid TSS", "Segment Not Present", "Stack-Segment Fault", "General Protection Fault", "Page Fault", "Reserved", "x87 Floating Point Exception", "Alignment Check", "Machine Check", "SIMD Floating Point Exception", "Virtualisation Exception", "Control Exception", "Hypervisor Injection Exception", "Security Exception", "Reserved"];

impl idt_entry {
    pub fn edit_entry(vector: usize, func_address: u64, gate_type: GateType) {
        unsafe {
            IDT[vector].attributes = match gate_type {
                GateType::Trap => 0x8F,
                GateType::Interrupt => 0x8E
            };
            IDT[vector].isr_low = (func_address & 0xFFFF) as u16;
            IDT[vector].isr_mid = ((func_address >> 16) & 0xFFFF) as u16;
            IDT[vector].isr_high = (func_address >> 32) as u32;
        } 
    }
}

pub fn init_idt() {
    unsafe {
        let idt_address = (&IDT[0] as *const idt_entry) as u64;
        IDTR.limit = (size_of::<idt_entry>() as u16) * (IDT_MAX_DESCRIPTIONS as u16 - 1);
        IDTR.base = idt_address;

        idt_entry::edit_entry(0, (handle_no_err_exception0 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(1, (handle_no_err_exception1 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(2, (handle_no_err_exception2 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(3, (handle_no_err_exception3 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(4, (handle_no_err_exception4 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(5, (handle_no_err_exception5 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(6, (handle_no_err_exception6 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(7, (handle_no_err_exception7 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(8, (handle_err_exception8 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(9, (handle_no_err_exception9 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(10, (handle_err_exception10 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(11, (handle_err_exception11 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(12, (handle_err_exception12 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(13, (handle_err_exception13 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(14, (handle_err_exception14 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(15, (handle_no_err_exception15 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(16, (handle_no_err_exception16 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(17, (handle_err_exception17 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(18, (handle_no_err_exception18 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(19, (handle_no_err_exception19 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(20, (handle_no_err_exception20 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(21, (handle_err_exception21 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(22, (handle_no_err_exception22 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(23, (handle_no_err_exception23 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(24, (handle_no_err_exception24 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(25, (handle_no_err_exception25 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(26, (handle_no_err_exception26 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(27, (handle_no_err_exception27 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(28, (handle_no_err_exception28 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(29, (handle_err_exception29 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(30, (handle_err_exception30 as *const u64) as u64, GateType::Trap);
        idt_entry::edit_entry(31, (handle_no_err_exception31 as *const u64) as u64, GateType::Trap);

        // idt_entry::edit_entry(32, (handle_interrupt0 as *const u64) as u64, GateType::Interrupt);
        idt_entry::edit_entry(33, (handle_interrupt1 as *const u64) as u64, GateType::Interrupt);
    
        idt_flush();
    }
}

#[no_mangle]
pub extern fn exception_handler(registers: Registers) {
    let unaligned_registers = core::ptr::addr_of!(registers);
    let aligned_registers = unsafe { core::ptr::read_unaligned(unaligned_registers) };

    if aligned_registers.num < 22 {
        print!("{}\n", exception_messages[aligned_registers.num as usize]);
    } else if aligned_registers.num > 27 { 
        print!("{}\n", exception_messages[(aligned_registers.num as usize) - 6]);
    } else {
        print!("Reserved\n");
    }

    print!("{:?}\n", aligned_registers);
}

#[no_mangle]
pub extern fn interrupt_handler(registers: Registers) {
    let unaligned_registers = core::ptr::addr_of!(registers);
    let aligned_registers = unsafe { core::ptr::read_unaligned(unaligned_registers) };

    print!("Interrupt!\n");
    print!("{:?}\n", aligned_registers);
    // TODO: Depending on interrupt call function eg keyboard, timer, etc
    // TODO: Call PIC acknowledge
    unsafe { asm!("cli; hlt"); }
}

extern "C" {
    fn handle_no_err_exception0(registers: Registers);
    fn handle_no_err_exception1(registers: Registers);
    fn handle_no_err_exception2(registers: Registers);
    fn handle_no_err_exception3(registers: Registers);
    fn handle_no_err_exception4(registers: Registers);
    fn handle_no_err_exception5(registers: Registers);
    fn handle_no_err_exception6(registers: Registers);
    fn handle_no_err_exception7(registers: Registers);
    fn handle_err_exception8(registers: Registers);
    fn handle_no_err_exception9(registers: Registers);
    fn handle_err_exception10(registers: Registers);
    fn handle_err_exception11(registers: Registers);
    fn handle_err_exception12(registers: Registers);
    fn handle_err_exception13(registers: Registers);
    fn handle_err_exception14(registers: Registers);
    fn handle_no_err_exception15(registers: Registers);
    fn handle_no_err_exception16(registers: Registers);
    fn handle_err_exception17(registers: Registers);
    fn handle_no_err_exception18(registers: Registers);
    fn handle_no_err_exception19(registers: Registers);
    fn handle_no_err_exception20(registers: Registers);
    fn handle_err_exception21(registers: Registers);
    fn handle_no_err_exception22(registers: Registers);
    fn handle_no_err_exception23(registers: Registers);
    fn handle_no_err_exception24(registers: Registers);
    fn handle_no_err_exception25(registers: Registers);
    fn handle_no_err_exception26(registers: Registers);
    fn handle_no_err_exception27(registers: Registers);
    fn handle_no_err_exception28(registers: Registers);
    fn handle_err_exception29(registers: Registers);
    fn handle_err_exception30(registers: Registers);
    fn handle_no_err_exception31(registers: Registers);
    fn handle_interrupt0(registers: Registers); // Timer
    fn handle_interrupt1(registers: Registers); // Keyboard
    fn idt_flush();
}