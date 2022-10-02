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
    xchg bx, bx
    pop qword [old_process + 48]
    pop qword [old_process + 56]
    pop qword [old_process + 64]
    pop qword [old_process + 72]
    pop qword [old_process + 80]

    push rax      ;save current rax
    push rbx      ;save current rbx
    push rcx      ;save current rcx
    push rdx      ;save current rdx
    push rbp      ;save current rbp
    push rdi      ;save current rdi
    push rsi      ;save current rsi
    push r8         ;save current r8
    push r9         ;save current r9
    push r10      ;save current r10
    push r11      ;save current r11
    push r12      ;save current r12
    push r13      ;save current r13
    push r14      ;save current r14
    push r15      ;save current r15

    cld
    call pit_handler

    pop r15         ;restore current r15
    pop r14         ;restore current r14
    pop r13         ;restore current r13
    pop r12         ;restore current r12
    pop r11         ;restore current r11
    pop r10         ;restore current r10
    pop r9         ;restore current r9
    pop r8         ;restore current r8
    pop rsi         ;restore current rsi
    pop rdi         ;restore current rdi
    pop rbp         ;restore current rbp
    pop rdx         ;restore current rdx
    pop rcx         ;restore current rcx
    pop rbx         ;restore current rbx
    pop rax         ;restore current rax

    mov rsp, [old_process + 72]

    push qword [old_process + 80]
    push qword [old_process + 72]
    push qword [old_process + 64]
    push qword [old_process + 56]
    push qword [old_process + 48]

    push rax      ;save current rax
    push rbx      ;save current rbx
    push rcx      ;save current rcx
    push rdx      ;save current rdx
    push rbp      ;save current rbp
    push rdi      ;save current rdi
    push rsi      ;save current rsi
    push r8       ;save current r8
    push r9       ;save current r9
    push r10      ;save current r10
    push r11      ;save current r11
    push r12      ;save current r12
    push r13      ;save current r13
    push r14      ;save current r14
    push r15      ;save current r15
    
    mov rax, cr3
    push rax

    mov rsp, [new_process_rsp]

    pop rax
    mov cr3, rax

    pop r15         ;restore current r15
    pop r14         ;restore current r14
    pop r13         ;restore current r13
    pop r12         ;restore current r12
    pop r11         ;restore current r11
    pop r10         ;restore current r10
    pop r9         ;restore current r9
    pop r8         ;restore current r8
    pop rsi         ;restore current rsi
    pop rdi         ;restore current rdi
    pop rbp         ;restore current rbp
    pop rdx         ;restore current rdx
    pop rcx         ;restore current rcx
    pop rbx         ;restore current rbx
    pop rax         ;restore current rax

    iretq 


global handle_syscall
handle_syscall:
    cld
    pushaq
    call syscall_handler
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    add rsp, 0x08 ;; Manoeuvre to preserve the return value as it's stored within rax
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