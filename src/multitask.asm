; src/multitask.asm

global switch_process
switch_process:
    mov rsp, rdi
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax
    add rsp, 0x10 
    iretq