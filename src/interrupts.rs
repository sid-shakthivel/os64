// src/interrupts.rs

/*
Interrupts are signal which stop the operation flow of a computer in order to perform a set action (pressing a key)
After the CPU performs the action it returns to whatever it was doing
This is far more efficient then the CPU polling a device
An interrupt descriptor table defines what each interrupt will do
*/

// ISR's are interrupt service routines which are called on an interrupt
struct idt_entry {
    isr_low: u16, // low 16 bits of ISR address
    kernel_cs: u16, 
    ist: u8,
    attributes: u8,
    isr_mid: u16,
    isr_high: u32,
    reserved: u32
}

static IDT: &'static  [idt_entry; 256];

struct idtr {
    limit: u16,
    base: u32
}

static IDTR: &'static idtr;


