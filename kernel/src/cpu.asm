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

global jump_usermode
jump_usermode:
  cli