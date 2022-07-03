section .text
global start
start:
    bits 64
    xchg bx, bx
    mov rax, 0xDEADBEEF
    jmp $