// src/multitask.rs

/*
    Preemptive multitasking is when the CPU splits up it's time between various processes to give the illusion they are happening simultaneously
*/

use spin::Mutex;
use crate::page_frame_allocator::PageFrameAllocator;
use crate::page_frame_allocator::FrameAllocator;
use crate::print;
use crate::vga_text::TERMINAL;
use core::prelude::v1::Some;
use crate::pic::PICS;
use crate::pic::pic_functions;
use crate::ports::outb;


#[derive(Debug, Copy, Clone)]
pub struct Process {
    pid: u64,
    pub rsp: *const u64,
}

pub enum ProcessType {
    Kernel,
    User,
}

static mut COUNTER: u64 = 0;
pub static mut a_process: Option<Process> = None;
pub static mut b_process: Option<Process> = None;
static mut is_a_process: bool = false;
static mut is_first: bool = true;

impl Process {
    pub fn init(func: u64, page_frame_allocator:  &mut PageFrameAllocator, is_kernel: ProcessType) -> Process {
        unsafe {
            let mut rsp = page_frame_allocator.alloc_frame().unwrap(); // Create a stack
            let stack_top = rsp.offset(511) as u64;
            rsp = rsp.offset(511); // Stack grows downwards towards decreasing memory addresses
            
            // These registers are then pushed: RAX -> RBX -> RBC -> RDX -> RSI -> RDI
            // When interrupt is called certain registers are pushed as follows: SS -> RSP -> RFLAGS -> CS -> RIP

            *rsp.offset(-1) = 0x00; // SS (don't have kernel data yet)
            *rsp.offset(-2) = stack_top; // RSP
            *rsp.offset(-3) = 0x200; // RFLAGS
            *rsp.offset(-4) = 0x08; // CS
            *rsp.offset(-5) = func; // RIP
            *rsp.offset(-6) = 0x00; // RAX
            *rsp.offset(-7) = 0x00; // RBX
            *rsp.offset(-8) = 0x00; // RBC
            *rsp.offset(-9) = 0x00; // RDX
            *rsp.offset(-10) = 0x00; // RSI
            *rsp.offset(-11) = 0x00; // RDI
            *rsp.offset(-12) = 0x00; // IRQ Number (0)
            *rsp.offset(-13) = 0x00; // Dummy error thing

            rsp = rsp.offset(-13);

            let new_process = Process {
                pid: COUNTER,
                rsp: rsp,
            };

            COUNTER += 1;

            if a_process.is_none() {
                a_process = Some(new_process);
            } else {
                b_process = Some(new_process);
            }

            return new_process;
        }
    }
}

// Very primitive to begin with
pub fn schedule_process(old_rsp: *const u64) -> *const u64 {
    unsafe {
        if is_a_process == false {
            is_a_process = true;
            if is_first == false {
                let new_struct = Process { rsp: old_rsp, ..b_process.unwrap() };
                b_process = Some(new_struct);
            }
            is_first = true;
            return a_process.unwrap().rsp;
        } else {
            is_a_process = false;
            a_process.unwrap().rsp = old_rsp;
            let new_struct = Process { rsp: old_rsp, ..b_process.unwrap() };
            a_process = Some(new_struct);
            return b_process.unwrap().rsp;
        }
    }
}

extern "C" {
    fn switch_process(rsp: *const u64);
}