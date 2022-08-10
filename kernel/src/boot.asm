; src/boot.asm

extern long_mode_start
global start

section .multiboot_header
header_start:
    dd 0xe85250d6                ; Magic number (multiboot 2) which identifies header
    dd 0                         ; Architecture 0 (protected mode i386)
    dd header_end - header_start ; Length of multiboot header in bytes (including magic fields)
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start)) ; Checksum 

    ; insert optional multiboot tags here
    dw 5 ; Type
    dw 1 ; Flags, optional
    dd 20; Size
    dd 1024 ; Width
    dd 768 ; Height 
    dd 32 ; Depth 

    ; required end tag to terminate
    dw 0    ; Type
    dw 0    ; Flagsw
    dd 0    ; Size
header_end:

section .rodata
gdt64:
    dq 0 ; null entry
    dq 0x002098000000ffff ; kernel code segment
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

; Identity map function 
section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
p1_tables:
    resb 49152 ; Identity map the first 24MB
stack_bottom:
    resb 16384
stack_top:

section .text
bits 32
start:
    mov esp, stack_top ; Stack grows downwards
    mov edi, ebx; Multiboot information structure

    call setup_paging

    ; Recursive paging in which the last entry points to the first

    mov eax, p4_table
    or eax, 0b11 ; Present, Writeable
    mov [p4_table + 511 * 8], eax

    call enable_paging

    lgdt [gdt64.pointer] ; Load the new GDT
    jmp 0x08:long_mode_start

    hlt

setup_paging:
    ; Point P4 to P3 to P2
    ; Fill P2 with 512 entries 

    mov eax, p3_table
    or eax, 0b111 ; Present, Writeable
    mov [p4_table], eax

    mov eax, p2_table
    or eax, 0b111 ; Present, Writeable
    mov [p3_table], eax

    mov eax, p1_tables
    mov ecx, 0

.map_p2_table
    or eax, 0b111
    mov [p2_table + ecx * 8], eax
    inc ecx
    add eax, 4096
    cmp ecx, 12
    jne .map_p2_table

    mov ecx, 0

.map_p1_tables
    mov eax, 0x1000
    mul ecx
    or eax, 0b111 ; Present, Writeable, User
    mov [p1_tables + ecx * 8], eax
    inc ecx
    cmp ecx, 6144 ; Make 12 tables
    jne .map_p1_tables

    mov ecx, 0

    ret

enable_paging:
    ; Write address of P4 table to CR3 register
    mov eax, p4_table
    mov cr3, eax
    
    ; Enable pAE paging
    mov eax, cr4
    or eax, 1 << 5,
    mov cr4, eax

    ; Set long mode bit in EFER MSR
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; Enable paging
    mov eax, cr0
    or eax, 1 << 31 | 1 << 0
    mov cr0, eax

    ret