extern exception_handler
; isr_no_err_stub 1
; isr_no_err_stub 2
; isr_no_err_stub 3
; isr_no_err_stub 4
; isr_no_err_stub 5
; isr_no_err_stub 6
; isr_no_err_stub 7
; isr_err_stub    8
; isr_no_err_stub 9
; isr_err_stub    10
; isr_err_stub    11
; isr_err_stub    12
; isr_err_stub    13
; isr_err_stub    14
; isr_no_err_stub 15
; isr_no_err_stub 16
; isr_err_stub    17
; isr_no_err_stub 18
; isr_no_err_stub 19
; isr_no_err_stub 20
; isr_no_err_stub 21
; isr_no_err_stub 22
; isr_no_err_stub 23
; isr_no_err_stub 24
; isr_no_err_stub 25
; isr_no_err_stub 26
; isr_no_err_stub 27
; isr_no_err_stub 28
; isr_no_err_stub 29
; isr_err_stub    30
; isr_no_err_stub 31

%macro handle_no_err_exception 1
global handle_no_err_exception%1
handle_no_err_exception%1:
    push rax
    cld
    call exception_handler
    pop rax
    iretq
%endmacro

handle_no_err_exception 0

; %macro isr_err_stub 1
; isr_stub_%+%1:
;     push %1
;     pushaq
;     cld
;     call exception_handler
;     popq
;     add rsp, 0x08
;     iretq 
; %endmacro

; %macro isr_no_err_stub 1
; isr_stub_%+%1:
;     push %1
;     pushaq
;     cld
;     call exception_handler
;     popq
;     add rsp, 0x08
;     iretq 
; %endmacro

; .macro pushaq
;     push %rax
;     push %rcx
;     push %rdx
;     push %rbx
;     push %rbp
;     push %rsi
;     push %rdi
; .endm # pushaq


; .macro popaq
;     pop %rdi
;     pop %rsi
;     pop %rbp
;     pop %rbx
;     pop %rdx
;     pop %rcx
;     pop %rax
; .endm # popaq

global idt_flush    

idt_flush:
    extern IDTR
    lidt [IDTR]
    ret