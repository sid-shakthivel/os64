// src/pic.asm

/*
    Programmable interrupt controller manages hardware signals and converts them to software interrupts
    There are 2 PIC's of 8 inputs called master and slave (15 interrupts)
    PIC is initially mapped to the first interrupts however these are used for interrupts thus need to be remapped to 32-47
    +------+-------------+------+-----------------+
    | PIC1 |  Hardware   | PIC2 |    Hardware     |
    +------+-------------+------+-----------------+
    |    0 | Timer       |    8 | Real Time Clock |
    |    1 | Keyboard    |    9 | General I/O     |
    |    2 | PIC 2       |   10 | General I/O     |
    |    3 | COM 2       |   11 | General I/O     |
    |    4 | COM 1       |   12 | PS2 Mouse       |
    |    5 | LPT 2       |   13 | Coprocessor     |
    |    6 | Floppy Disk |   14 | IDE Bus         |
    |    7 | LPT 1       |   15 | IDE Bus         |
    +------+-------------+------+-----------------+
*/

use crate::ports::outb;
use crate::ports::inb;
use crate::ports::io_wait;
use crate::print;
use spin::Mutex;
use crate::TERMINAL;

const PIC1_PORT_COMMAND: u16 = 0x20;
const PIC2_PORT_COMMAND: u16 = 0xA0;

const PIC1_PORT_DATA: u16 = 0x21;
const PIC2_PORT_DATA: u16 = 0xA1;

const PIC1_START_INTERRUPT: u8 = 0x20;
const PIC2_START_INTERRUPT: u8 = 0x28;

const PIC_ACK: u8 = 0x20;

struct Pic {
    offset: u8,
    command: u16,
    data: u16
}

pub struct ChainedPics {
    master: Pic,
    slave: Pic
}

pub trait PicFunctions {
    fn set_mask(&self, interrupt: u8);
    fn clean_mask(&self, interrupt: u8);
    fn acknowledge(&self, interrupt: u8);
}

impl ChainedPics {
    pub const fn new(offset1: u8, offset2: u8) -> ChainedPics {
        return ChainedPics {
            master: Pic {
                offset: offset1,
                command: PIC1_PORT_COMMAND,
                data: PIC1_PORT_DATA,
            },
            slave: Pic {
                offset: offset2,
                command: PIC2_PORT_COMMAND,
                data: PIC2_PORT_DATA,
            },
        };
    }

    pub fn init(&self) {
        // Start initialization
        outb(self.master.command, 0x11);
        outb(self.slave.command, 0x11);
        io_wait();

        outb(self.master.data, self.master.offset); // ICW2 (Offset Master PIC)
        outb(self.slave.data, self.slave.offset); // ICW2 (Offset Slave PIC)
        io_wait();

        outb(self.master.data, 4); // ICW3 (Tell Master PIC Slave PIC Exists)
        outb(self.slave.data, 2); // ICW3 (Tell Slave PIC Cascade Identity)
        io_wait();

        // ECW4 Enable 8086 Mode
        outb(self.master.data, 1); 
        outb(self.slave.data, 1);        
        io_wait();
 
        // f9 - kbd, slave
        // fb - slave
        // ef - mouse
        outb(self.master.data, 0xf9);  // Enable just keyboard and slave
        outb(self.slave.data, 0xef); // Mouse
        io_wait();
    }
}

impl PicFunctions for ChainedPics {
    fn set_mask(&self, interrupt: u8) {
        if interrupt < PIC2_START_INTERRUPT {
            self.master.set_mask(interrupt);
        } else {
            self.slave.set_mask(interrupt);
        }
    }

    fn clean_mask(&self, interrupt: u8) {
        if interrupt < PIC2_START_INTERRUPT {
            self.master.clean_mask(interrupt);
        } else {
            self.slave.clean_mask(interrupt);
        }
    }

    // Both master and slave PIC must be acknowledged on a slave interrupt
    fn acknowledge(&self, interrupt: u8) {
        if interrupt >= 0x28 {
            self.slave.acknowledge(interrupt);
        }

        self.master.acknowledge(interrupt);
    }
}

impl PicFunctions for Pic {
    // Disable interrupt
    fn set_mask(&self, mut interrupt: u8) {
        interrupt -= 0x20;
        let value = inb(self.data) | (1 << interrupt);
        outb(self.data, value);
    }

    // Enable interrupt
    fn clean_mask(&self, mut interrupt: u8) {
        interrupt -= 0x20;
        let value = inb(self.data) & !(1 << interrupt);
        outb(self.data, value);
    }

    // Every interrupt from PIC must be acknowledged to confirm interrupt has been handled
    fn acknowledge(&self, _interrupt: u8) {
        outb(self.command, PIC_ACK);
    }
}

pub static PICS: Mutex<ChainedPics> = Mutex::new(ChainedPics::new(PIC1_START_INTERRUPT, PIC2_START_INTERRUPT));