global long_mode_start

section .text
bits 64
long_mode_start:
  mov ax, 0
  mov ss, ax
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax

  extern rust_main
  call rust_main

  hlt

global test_pic
test_pic:
  mov al, 0x11
  out 0x20, al
  out 0xA0, al

  mov al, 0x20
  out 0x21, al

  mov al, 0x28
  out 0xA1, al

  mov al, 4
  out 0x21, al

  mov al, 2
  out 0xA1, al

  mov al, 1
  out 0xA1, al
  out 0x21, al

  mov al, 0xfd
  out 0x21, al

  mov al, 0xff
  out 0xA1, al

  ret
