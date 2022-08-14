; src/multiboot_header.asm

section .multiboot_header
header_start:
    dd 0xe85250d6                ; Magic number (multiboot 2) which identifies header
    dd 0                         ; Architecture 0 (protected mode i386)
    dd header_end - header_start ; Length of multiboot header in bytes (including magic fields)
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start)) ; Checksum 

    ; Framebuffer multiboot tag
    dw 5 ; Type
    dw 1 ; Flags, optional
    dd 20; Size
    dd 1024 ; Width
    dd 768 ; Height 
    dd 32 ; Depth 

    ; Required end tag to terminate
    dw 0    ; Type
    dw 0    ; Flags
    dd 0    ; Size
header_end: