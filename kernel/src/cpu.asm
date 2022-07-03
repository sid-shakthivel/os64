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
  ; cli ; Disable interruptsp
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

global switch_process
switch_process: 
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
  add rsp, 0x08 
  iretq 