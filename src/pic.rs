// src/pic.asm

/*
    Programmable interrupt controller manages hardware signals and converts them to software interrupts
    There are 2 PIC's of 8 inputs called master and slave (15 interrupts)
    PIC is initially mapped to the first interrupts however these are used for interrupts thus need to be remapped to 32-47
*/

// use crate::ports::outb;
// use crate::ports::inb;
use crate::ports::io_wait;
use core::arch::asm;

const PIC1_PORT_COMMAND: u16 = 0x20;
const PIC2_PORT_COMMAND: u16 = 0xA0;

const PIC1_PORT_DATA: u16 = 0x21;
const PIC2_PORT_DATA: u16 = 0xA1;

const PIC1_START_INTERRUPT: u16 = 0x20;
const PIC2_START_INTERRUPT: u16 = 0x28;

const PIC_ACK: u8 = 0x20;

pub fn init_pic() {
    unsafe {
        // Start initialization
        // outb(PIC1_PORT_COMMAND, 0x11);
        // outb(PIC2_PORT_COMMAND, 0x11);
        asm!("out 0x20, {0}", in(reg_byte) (0x11 as i8));
        asm!("out 0xA0, eax", in("eax") 0x11);
        io_wait();

        // outb(PIC1_PORT_DATA, 0x20); // ICW2 (Offset Master PIC)
        // outb(PIC2_PORT_DATA, 0x28); // ICW2 (Offset Slave PIC)
        asm!("out 0x21, eax", in("eax") 0x20);
        asm!("out 0xA1, eax", in("eax") 0x28);
        io_wait();

        // outb(PIC1_PORT_DATA, 0x04); // ICW3 (Tell Master PIC Slave PIC Exists)
        // outb(PIC2_PORT_DATA, 0x02); // ICW3 (Tell Slave PIC cascade identity)
        asm!("out 0x21, eax", in("eax") 0x04);
        asm!("out 0xA1, eax", in("eax") 0x02);
        io_wait();

        // // ICW4 Enable 8086 Mode
        // outb(PIC1_PORT_DATA, 0x01); 
        // outb(PIC2_PORT_DATA, 0x01);
        asm!("out 0xA1, eax", in("eax") 0x01);
        asm!("out 0x21, eax", in("eax") 0x01);
        io_wait();
        
        // // Enable all interrupts
        // outb(PIC1_PORT_DATA, 0x0);
        // outb(PIC2_PORT_DATA, 0x0);
        asm!("out 0x21, eax", in("eax") 0x00);
        asm!("out 0xA1, eax", in("eax") 0x00);

        asm!("out 0x21, eax", in("eax") 0xfc); // Mask everything but keyboard and timer
        asm!("out 0xA1, eax", in("eax") 0xff); // Mask everything
    }
}

// Stop raising interrupts
pub fn mask_interrupt() {

}

// Enable interrupt
pub fn clean_mask() {

}

// Every interrupt from PIC must be acknowledged to confirm interrupt has been handled
pub fn acknowledge_pic(interrupt: u16) {
    if interrupt < PIC2_START_INTERRUPT {
        // outb(PIC1_PORT_COMMAND, PIC_ACK);
        unsafe {  asm!("out 0x20, eax", in("eax") 0x20); }
    } else {
        // outb(PIC2_PORT_COMMAND, PIC_ACK);
        unsafe {  asm!("out 0xA0, eax", in("eax") 0x20); }
    }
}