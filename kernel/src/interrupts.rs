// src/interrupts.rs

/*
Interrupts are signal which stop the operation flow of a computer in order to perform a set action (pressing a key)
After the CPU performs the action it returns to whatever it was doing
Interrupts are far more efficient then the CPU polling a device
An interrupt descriptor table defines what each interrupt will do
*/

use crate::print_serial;
use core::mem::size_of;
use core::arch::asm;
use crate::pic::PICS;
use crate::pic::PicFunctions;
use crate::keyboard::KEYBOARD;
use crate::mouse::MOUSE;
use crate::pit::PIT;
use crate::multitask::PROCESS_SCHEDULAR;
use crate::gdt::TSS;
use crate::multitask;
use x86_64::addr::VirtAddr;
use crate::uart::CONSOLE;

// 256 entries within the IDT with the first 32 being exceptions
const IDT_MAX_DESCRIPTIONS: u64 = 256;

// Each entry in IDT is 16 bytes
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
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
    Trap, // For exceptions only
    Interrupt // For others
}

pub enum PrivilegeLevel {
    Ring0, // Kernel mode
    Ring1, // Device driver mode
    Ring2, // Device driver mode
    Ring3, // Userspace
}

// These registers are pushed onto the stack on an interrupt
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Registers {
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub num: u64,
    pub error_code: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64
}

// These registers are pushed on an int 
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct IretStack {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64
}

// TODO: Swap these global statics to a mutex or similar
#[no_mangle]
pub static mut IDTR: idtr = idtr { limit: 0, base: 0 };
pub static mut IDT: [idt_entry; 256] = [idt_entry { isr_low: 0, kernel_cs: 0x08, ist: 0, attributes: 0, isr_mid: 0, isr_high: 0, reserved: 0}; 256];

const EXCEPTION_MESSAGES: &'static [&'static str] = &["Divide By Zero", "Debug", "Non-maskable Interrupt", "Breakpoint", "Overflow", "Bound Range Exceeded", "Invalid Opcode", "Device not Available", "Double Fault", "Coprocessor Segment Overrun", "Invalid TSS", "Segment Not Present", "Stack-Segment Fault", "General Protection Fault", "Page Fault", "Reserved", "x87 Floating Point Exception", "Alignment Check", "Machine Check", "SIMD Floating Point Exception", "Virtualisation Exception", "Control Exception", "Hypervisor Injection Exception", "Security Exception", "Reserved"];

#[no_mangle]
pub static mut old_process: IretStack = IretStack::new();

#[no_mangle]
pub static mut new_process_rsp: u64 = 0;

impl idt_entry {
    pub fn edit_entry(vector: usize, raw_func: unsafe extern "C" fn(), gate_type: GateType, privilege_level: PrivilegeLevel) {
        let func_address = (raw_func as *const u64) as u64;
        unsafe {
            IDT[vector].attributes = match gate_type {
                GateType::Trap => 0x8F,
                GateType::Interrupt => 0x8E
            };
    
            match privilege_level {
                PrivilegeLevel::Ring3 => {
                    IDT[vector].attributes |= 1 << 5; 
                    IDT[vector].attributes |= 1 << 6;
                },
                _ => {}
            }
    
            IDT[vector].isr_low = (func_address & 0xFFFF) as u16;
            IDT[vector].isr_mid = ((func_address >> 16) & 0xFFFF) as u16;
            IDT[vector].isr_high = (func_address >> 32) as u32;
        }
    }
}

impl IretStack {
    pub const fn new() -> IretStack {
        IretStack {
            rip: 0,
            cs: 0,
            rflags: 0,
            rsp: 0,
            ss: 0
        }
    }
}

#[no_mangle]
pub extern fn exception_handler(registers: Registers) {
    let unaligned_error_code = core::ptr::addr_of!(registers.error_code); // Packed structs must be aligned properly
    let aligned_error_code = unsafe { core::ptr::read_unaligned(unaligned_error_code) };
    
    // Print a suitable error messages 
    match registers.num {
        0..=22 =>  print_serial!("{}\n", EXCEPTION_MESSAGES[registers.num as usize]),
        27..=31 => print_serial!("{}\n", EXCEPTION_MESSAGES[(registers.num as usize) - 6]),
        _ => print_serial!("Reserved\n"),
    }

    print_serial!("Error Code: {:b}\n", aligned_error_code);

    disable();
    unsafe { asm!("hlt"); }
}

#[no_mangle]
pub extern fn interrupt_handler(registers: Registers) {
    PICS.lock().acknowledge(registers.num as u8); // To allow further interrupts, an acknowledgement must be sent

    match registers.num {
        0x21 => KEYBOARD.lock().handle_keyboard(), // Keyboard
        44 => { MOUSE.lock().handle_mouse_interrupt(); MOUSE.free(); },
        _ => print_serial!("Unknown Interrupt!\n"),
    }
}

// TODO: Clean and refactor
#[no_mangle]
pub extern fn pit_handler(iret_stack: IretStack) -> *const u64 {
    // Acknowledge interrupt and timer
    PICS.lock().acknowledge(0x20); 
    PIT.lock().handle_timer();

    let new_stack = PROCESS_SCHEDULAR.lock().schedule_process(iret_stack.rsp);
    PROCESS_SCHEDULAR.free();

    // Update TSS to have a clean stack when coming from user to kernel
    unsafe { TSS.privilege_stack_table[0] = VirtAddr::new(multitask::KERNEL_STACK as u64); }

    unsafe {
        old_process = iret_stack;
        new_process_rsp = new_stack.unwrap() as u64;
    }

    return new_stack.unwrap();
}

pub extern fn enable() {
    unsafe { asm!("sti"); }
}

pub extern fn disable() {
    unsafe { asm!("cli"); }
}

pub fn init() {
    // Setup the idtr structure
    unsafe {
        let idt_address = (&IDT[0] as *const idt_entry) as u64;
        IDTR.limit = (size_of::<idt_entry>() as u16) * (IDT_MAX_DESCRIPTIONS as u16 - 1);
        IDTR.base = idt_address;
    }

    // Exceptions
    idt_entry::edit_entry(0x00, handle_no_err_exception0, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x01, handle_no_err_exception1, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x02, handle_no_err_exception2, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x03, handle_no_err_exception3, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x04, handle_no_err_exception4, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x05, handle_no_err_exception5, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x06, handle_no_err_exception6, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x07, handle_no_err_exception7, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x08, handle_err_exception8, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x09, handle_no_err_exception9, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x0A, handle_err_exception10, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x0B, handle_err_exception11, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x0C, handle_err_exception12, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x0D, handle_err_exception13, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x0E, handle_err_exception14, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x0F, handle_no_err_exception15, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x10, handle_no_err_exception16, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x11, handle_err_exception17, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x12, handle_no_err_exception18, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x13, handle_no_err_exception19, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x14, handle_no_err_exception20, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x15, handle_err_exception21, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x16, handle_no_err_exception22, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x17, handle_no_err_exception23, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x18, handle_no_err_exception24, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x19, handle_no_err_exception25, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x1A, handle_no_err_exception26, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x1B, handle_no_err_exception27, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x1C, handle_no_err_exception28, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x1D, handle_err_exception29, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x1E, handle_err_exception30, GateType::Trap, PrivilegeLevel::Ring3);
    idt_entry::edit_entry(0x1F, handle_no_err_exception31 , GateType::Trap, PrivilegeLevel::Ring3);

    // Interrupts
    idt_entry::edit_entry(0x20, handle_pit_interrupt, GateType::Interrupt, PrivilegeLevel::Ring3); // Timer
    idt_entry::edit_entry(0x21, handle_interrupt33, GateType::Interrupt, PrivilegeLevel::Ring3); // PS2 Keyboard
    idt_entry::edit_entry(0x2c, handle_interrupt44, GateType::Interrupt, PrivilegeLevel::Ring3); // PS2 Mouse

    // Syscall
    idt_entry::edit_entry(0x80, handle_syscall, GateType::Interrupt, PrivilegeLevel::Ring3);
    
    // Load idt
    unsafe { idt_flush(); }
}

extern "C" {
    // ISRS's
    fn handle_no_err_exception0();
    fn handle_no_err_exception1();
    fn handle_no_err_exception2();
    fn handle_no_err_exception3();
    fn handle_no_err_exception4();
    fn handle_no_err_exception5();
    fn handle_no_err_exception6();
    fn handle_no_err_exception7();
    fn handle_err_exception8();
    fn handle_no_err_exception9();
    fn handle_err_exception10();
    fn handle_err_exception11();
    fn handle_err_exception12();
    fn handle_err_exception13();
    fn handle_err_exception14();
    fn handle_no_err_exception15();
    fn handle_no_err_exception16();
    fn handle_err_exception17();
    fn handle_no_err_exception18();
    fn handle_no_err_exception19();
    fn handle_no_err_exception20();
    fn handle_err_exception21();
    fn handle_no_err_exception22();
    fn handle_no_err_exception23();
    fn handle_no_err_exception24();
    fn handle_no_err_exception25();
    fn handle_no_err_exception26();
    fn handle_no_err_exception27();
    fn handle_no_err_exception28();
    fn handle_err_exception29();
    fn handle_err_exception30();
    fn handle_no_err_exception31();
    fn handle_pit_interrupt(); // Timer
    fn handle_interrupt33(); // PPS2 Keyboard
    fn handle_interrupt44(); // PS2 Mouse
    fn handle_syscall(); // Syscall
    fn idt_flush();
}

