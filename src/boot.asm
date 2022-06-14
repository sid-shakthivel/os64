; src/boot.asm

extern long_mode_start
global start

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
    jmp gdt64.code:long_mode_start

    hlt

setup_paging:
    ; Point P4 to P3 to P2
    ; Fill P2 with 512 entries 

    mov eax, p3_table
    or eax, 0b11 ; Present, Writeable
    mov [p4_table], eax

    mov eax, p2_table
    or eax, 0b11 ; Present, Writeable
    mov [p3_table], eax

    mov eax, p1_table_1
    or eax, 0b11 ; Present, Writeable
    mov [p2_table + 0 * 8], eax

    mov eax, p1_table_2
    or eax, 0b11
    mov [p2_table + 1 * 8], eax

    mov eax, p1_table_3
    or eax, 0b11
    mov [p2_table + 2 * 8], eax

    mov ecx, 0

.map_p1_table_1
    mov eax, 0x1000
    mul ecx
    or eax, 0b11 ; Present, Writable
    mov [p1_table_1 + ecx * 8], eax ; 8 bit entries

    inc ecx
    cmp ecx, 512
    jne .map_p1_table_1

    mov ebx, 0

.map_p1_table_2
    mov eax, 0x1000
    mul ecx
    or eax, 0b11 ; Present, Writable
    mov [p1_table_2 + ebx * 8], eax ; 8 bit entries

    inc ebx
    inc ecx
    cmp ebx, 512
    jne .map_p1_table_2

    mov ebx, 0

.map_p1_table_3
    mov eax, 0x1000
    mul ecx
    or eax, 0b11 ; Present, Writable
    mov [p1_table_3 + ebx * 8], eax ; 8 bit entries

    inc ebx
    inc ecx
    cmp ebx, 512
    jne .map_p1_table_3

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

section .rodata
gdt64:
    dq 0 ; null entry
.code: equ $ - gdt64
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code segment
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
p1_table_1:
    resb 4096
p1_table_2:
    resb 4096
p1_table_3:
    resb 4096
stack_bottom:
    resb 16384
stack_top:
