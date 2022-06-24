// src/multitask.rs

/*
    Preemptive multitasking is when the CPU splits up its time between various processes to give the illusion they are happening simultaneously
*/

use core::prelude::v1::Some;
use crate::page_frame_allocator::PageFrameAllocator;
use crate::page_frame_allocator::PAGE_SIZE;
use core::mem::size_of;
use crate::spinlock::Lock;
use crate::page_frame_allocator::FrameAllocator;
use crate::paging::Table;
use crate::paging;
use crate::paging::P4;
use crate::print;
use crate::vga_text::TERMINAL;
use crate::paging::Page;

/*
    Processes are running programs with an individual address space, stack and data
    There are kernel processes (run in kernel mode) and user processes (run in user mode)
    Processes will be selected based on what priority they are
*/
#[derive(Debug, Copy, Clone)]
pub struct Process {
    pid: u64,
    pub rsp: *const u64,
    process_priority: ProcessPriority,
    pub cr3: *mut Table,
}

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum ProcessPriority {
    High,
    Low,
}

pub const MAX_PROCESS_NUM: usize = PAGE_SIZE / size_of::<Process>();
static USER_PROCESS_START_ADDRESS: u64 = 0x800000;

/*
    Processes schedular holds all tasks and decides which will be serviced
    TODO: Set next task based on priority
*/
pub struct ProcessSchedular {
    tasks: [Option<Process>; MAX_PROCESS_NUM],
    is_from_kernel: bool,
    process_count: usize,
    current_process_index: usize,
}

impl ProcessSchedular {
    pub const fn new() -> ProcessSchedular {
        ProcessSchedular {
            tasks: [None; MAX_PROCESS_NUM],
            is_from_kernel: true,
            process_count: 0,
            current_process_index: 0
        }
    }

    /*
        Round robin in which there is a single queue so when timer interrupt is triggered the next process is scheduled
        TODO: Switch to priority based round robin
    */
    pub fn schedule_process(&mut self, mut old_rsp: *const u64) -> Option<*const u64> {
        if self.tasks[0].is_none() { return None; }

        if self.is_from_kernel == true {
            // If this is the first process to be called, it stems from kernel and that stack need not be saved
            self.is_from_kernel = false;
        } else {
            // Save the old RSP into the process
            old_rsp = unsafe { old_rsp.offset(5) }; // Adjust RSP by 5 bytes as RSP is pushed onto the stack
            // TODO: Find more efficient way
            let updated_process = Process { rsp: old_rsp, ..self.tasks[self.current_process_index].unwrap() }; 
            self.tasks[self.current_process_index] = Some(updated_process);
            self.current_process_index += 1;
        }

        // Select next process and ensure it's not empty
        let mut current_task = self.tasks[self.current_process_index];
        if current_task.is_none() {
            self.current_process_index = 0;
            current_task = self.tasks[self.current_process_index];
        }
        return Some(current_task.unwrap().rsp);
    }

    pub fn add_process(&mut self, process: Process) {
        if self.process_count > MAX_PROCESS_NUM { panic!("Memory maxed") }
        self.tasks[self.process_count] = Some(process);
        self.process_count += 1;
    }
}

impl Process {
    pub fn init(entrypoint: u64, process_priority: ProcessPriority, pid: u64, page_frame_allocator: &mut PageFrameAllocator) -> Process {
        // Convert physical addresses of process into virtual address to use when switching cr3
        // TODO: add support for multiple pages
        let v_address = USER_PROCESS_START_ADDRESS; 
        let p_address = entrypoint;
        paging::map_page(p_address, v_address, page_frame_allocator, true);

        // Copy current address space
        let new_p4: *mut Table = page_frame_allocator.alloc_frame().unwrap() as *mut _;

        unsafe {
            for i in 0..(*new_p4).entries.len() {
                (*new_p4).entries[i] = (*P4).entries[i];
            }
            
            (*new_p4).entries[511] = Page::new(new_p4 as u64);
        }
        
        let mut rsp = page_frame_allocator.alloc_frame().unwrap(); // Create a stack

        // Setup stack for interrupt
        unsafe {
            let stack_top = rsp.offset(511) as u64;
            rsp = rsp.offset(511); // Stack grows downwards towards decreasing memory addresses
            
            // These registers are then pushed: RAX -> RBX -> RBC -> RDX -> RSI -> RDI
            // When interrupt is called certain registers are pushed as follows: SS -> RSP -> RFLAGS -> CS -> RIP

            *rsp.offset(-1) = 0x20 | 0x3; // SS
            *rsp.offset(-2) = stack_top; // RSP
            *rsp.offset(-3) = 0x202; // RFLAGS TODO: Change to 0x200
            *rsp.offset(-4) = 0x18 | 0x3; // CS
            *rsp.offset(-5) = USER_PROCESS_START_ADDRESS; // RIP
            *rsp.offset(-6) = new_p4 as u64; // CR3
            *rsp.offset(-7) = 0x00; // RAX
            *rsp.offset(-8) = 0x00; // RBX
            *rsp.offset(-9) = 0x00; // RBC
            *rsp.offset(-10) = 0x00; // RDX
            *rsp.offset(-11) = 0x00; // RSI
            *rsp.offset(-12) = 0x00; // RDI

            rsp = rsp.offset(-12);
        }

        Process {
            pid: pid,
            rsp: rsp,
            process_priority: process_priority,
            cr3: new_p4
        }
    }
}

pub static PROCESS_SCHEDULAR: Lock<ProcessSchedular> = Lock::new(ProcessSchedular::new());