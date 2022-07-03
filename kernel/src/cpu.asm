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
  cli ; Disable interrupts
  extern IDTR
  lidt [IDTR]
  ret

global flush_tlb
flush_tlb:
  push rax
  mov rax, cr3
  mov cr3, rax
  pop rax
  ret