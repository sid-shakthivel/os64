; /src/long_mode_init.asm

global long_mode_start

section .text
bits 64
long_mode_start:
  mov ax, 0x00
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax

  extern rust_main
  call rust_main

  hlt


