global start ; entry point of kernel

section .text 
bits 32 ; CPU is in protected mode 
start:
  mov dword [0xb8000], 0x2f4b2f4f
  hlt
