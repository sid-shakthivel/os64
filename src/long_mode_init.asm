; /src/long_mode_init.asm

global long_mode_start

section .text
bits 64
long_mode_start:
  reload_segments

  extern rust_main
  call rust_main

  hlt


