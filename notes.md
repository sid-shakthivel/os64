### List of useful resources

Bochs magic breakpoint is xchg bx, bx

TODO:
- Port newlib C library
- Extend usermode/syscall capabilities
- GUI 
- Port dart (git clone https://kernel.googlesource.com/pub/scm/utils/dash/dash)
- Malloc/Free
- UART
- PS2 Mouse

Large Tasks:
- Make Window system
- PS/2 Mouse

Refactoring Tasks:
- Make Writer trait which vga_text, framebuffer, uart all use
- Rewrite PS2 controller, kbd, mouse
- Use Bitflags more

Small Tasks:
- Check for mouse in kbd
- Scaling for ps2 mouse
- Fix bochs bug (interrupt 47???), set sample rate correctly, check mouseid, bochs also can't get correct device
- Sample rate, get device, extensions, etc (PS2)