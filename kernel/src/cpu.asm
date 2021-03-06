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

global outpw_raw
outpw_raw:
  mov dx, di ; Address (16 Bit) 
  mov ax, si ; Value (16 Bit)
  out dx, ax
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

global cause_tf:
  cause_tf:
  xchg bx, bx
  mov rax, 0