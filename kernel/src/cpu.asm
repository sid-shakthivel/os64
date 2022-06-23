; /src/cpu.asm

global inb_raw
inb_raw:
  mov dx, di ; Address, first parameter
  in al, dx
  ret

global outb_raw
outb_raw:
  mov dx, di ; Address, first parameter
  mov al, sil ; Value, second parameter
  out dx, al
  ret

; Load IDT
global idt_flush    
idt_flush:
  ; cli ; Disable interrupts
  extern IDTR
  lidt [IDTR]
  ret

global gdt_flush
gdt_flush:
  extern GDTR
  lgdt  [GDTR]

  push 0x08
  lea rax, [rel reload_cs]
  push rax
  retfq

reload_cs:
  mov ax, 0x10
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  ret

global flush_tlb
flush_tlb:
  push rax
  mov rax, cr3
  mov cr3, rax
  pop rax
  ret

global switch_process
switch_process:
  ; ; Write address of P4 table to CR3 register
  ; mov rax, rsi
  ; mov cr3, rax
  
  ; ; Enable pAE paging
  ; mov rax, cr4
  ; or rax, 1 << 5,
  ; mov cr4, rax

  ; ; Set long mode bit in EFER MSR
  ; mov rcx, 0xC0000080
  ; rdmsr
  ; or rax, 1 << 8
  ; wrmsr

  ; ; Enable paging
  ; mov rax, cr0
  ; or rax, 1 << 31 | 1 << 0
  ; mov cr0, rax

  mov ax, 0x20 | 0x3 ; All segment registers must be equal to ss (user data segment)
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax

  ; Switch stacks and then pop registers and iret
  mov rsp, rdi
  pop rdi
  pop rsi
  pop rdx
  pop rcx
  pop rbx
  pop rax
  add rsp, 0x10 
  iretq 