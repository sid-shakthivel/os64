// src/pic.asm

/*
    Programmable interrupt controller manages hardware signals and converts them to software interrupts
    There are 2 PIC's of 8 inputs called master and slave (15 interrupts)
    PIC is initially mapped to the first interrupts however these are used for interrupts thus need to be remapped to 32-47
*/

use crate::ports::outb;
use crate::ports::inb;
use crate::ports::io_wait;
use core::arch::asm;

const PIC1_PORT_COMMAND: u16 = 0x20;
const PIC2_PORT_COMMAND: u16 = 0xA0;

const PIC1_PORT_DATA: u16 = 0x21;
const PIC2_PORT_DATA: u16 = 0xA1;

const PIC1_START_INTERRUPT: u8 = 0x20;
const PIC2_START_INTERRUPT: u8 = 0x28;

const PIC_ACK: u8 = 0x20;

pub fn init_pic() {
    outb(PIC1_PORT_COMMAND, 0x11);
    outb(PIC2_PORT_COMMAND, 0x11);

    outb(PIC1_PORT_DATA, PIC1_START_INTERRUPT);
    outb(PIC2_PORT_DATA, PIC2_START_INTERRUPT);

    outb(PIC1_PORT_DATA, 4);
    outb(PIC2_PORT_DATA, 2);

    outb(PIC1_PORT_DATA, 1);
    outb(PIC2_PORT_DATA, 1);        

    outb(PIC1_PORT_DATA, 0xfd); // Only enable keyboard interrupt
    outb(PIC2_PORT_DATA, 0xff); // Disable Slave
}

// Stop raising interrupts
pub fn mask_interrupt() {

}

// Enable interrupt
pub fn clean_mask() {

}

// Every interrupt from PIC must be acknowledged to confirm interrupt has been handled
pub fn acknowledge_pic(interrupt: u8) {
    outb(0x20, 0x20);
    // if interrupt < PIC2_START_INTERRUPT {
    //     outb(PIC1_PORT_COMMAND, PIC_ACK);
    // } else {
    //     outb(PIC2_PORT_COMMAND, PIC_ACK);
    // }
}