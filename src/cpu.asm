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
  
  mov ax, 0
  mov ss, ax
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  
  ret

global find_ting
find_ting:
  xchg bx, bx 
  ret