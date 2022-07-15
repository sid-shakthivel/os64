section .data
message:
    db "Hello world"
  
section .text
global start
start:
    bits 64
    mov rdx, 11
    mov rcx, message
    mov rbx, 1
    mov rax, 4
    int 0x80
    jmp $ 