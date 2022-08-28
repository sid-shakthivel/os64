; src/isr.asm

section .text

extern exception_handler
extern interrupt_handler
extern pit_handler
extern syscall_handler
extern old_process
extern new_process_rsp

%macro pushaq 0
push rax
push rbx
push rcx
push rdx
push rsi
push rdi
%endmacro

%macro popaq 0
pop rdi
pop rsi
pop rdx
pop rcx
pop rbx
pop rax
%endmacro

%macro handle_no_err_exception 1
global handle_no_err_exception%1
handle_no_err_exception%1:
    push qword 0 ; Dummy error code
    push qword %1 ; Number
    pushaq ; Push registers
    cld
    call exception_handler
    popaq
    add rsp, 0x10 ; Must remove both 64 bit values pushed onto stack
    iretq ; Exit from interrupt
%endmacro

%macro handle_err_exception 1
global handle_err_exception%1
handle_err_exception%1:
    push qword %1
    pushaq
    cld
    call exception_handler
    popaq
    add rsp, 0x10 
    iretq
%endmacro

%macro handle_interrupt 1
global handle_interrupt%1
handle_interrupt%1:
    push qword 0 
    push qword %1
    pushaq
    cld
    call interrupt_handler
    popaq
    add rsp, 0x10 
    iretq
%endmacro

global handle_pit_interrupt
handle_pit_interrupt:
    call pit_handler

    mov rsp, [old_process + 24]
    push qword [old_process + 32]
    push qword [old_process + 24]
    push qword [old_process + 16]
    push qword [old_process + 8]
    push qword [old_process]

    pushaq

    mov rax, cr3
    push rax

    mov rsp, [new_process_rsp]

    pop rax
    ; mov cr3, rax
    popaq

    iretq 


global handle_syscall
handle_syscall:
    cld
    pushaq
    call syscall_handler
    popaq
    iretq

handle_no_err_exception 0
handle_no_err_exception 1
handle_no_err_exception 2
handle_no_err_exception 3
handle_no_err_exception 4
handle_no_err_exception 5
handle_no_err_exception 6
handle_no_err_exception 7
handle_err_exception 8
handle_no_err_exception 9
handle_err_exception 10
handle_err_exception 11
handle_err_exception 12
handle_err_exception 13
handle_err_exception 14
handle_no_err_exception 15
handle_no_err_exception 16
handle_err_exception 17
handle_no_err_exception 18
handle_no_err_exception 19
handle_no_err_exception 20
handle_err_exception 21
handle_no_err_exception 22
handle_no_err_exception 23
handle_no_err_exception 24
handle_no_err_exception 25
handle_no_err_exception 26
handle_no_err_exception 27
handle_no_err_exception 28
handle_err_exception 29
handle_err_exception 30
handle_no_err_exception 31

handle_interrupt 33
handle_interrupt 44