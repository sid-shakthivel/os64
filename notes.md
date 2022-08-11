### Ramblings

Bochs magic breakpoint is xchg bx, bx

Large Tasks:
- Basic GUI
- Port newlib C library and use actual C userland processes
- Extend usermode/syscall capabilities along with multitasking
- Port dart (git clone https://kernel.googlesource.com/pub/scm/utils/dash/dash)

Smaller Tasks:
- Free memory/switch to malloc everywhere
- Address todos 
- Bitflags
- Improve spinlock
- Fix rust borrow stuff (https://www.youtube.com/watch?v=79phqVpE7cU)
- Optimize memcpy

Think:
- Switch to usize
- Make things like page frame allocator generic of types - large array would be of certain types
- Make an idle user space process with low priority which always runs
- Communicate with VM with syscalls?
- Syscall management - which ones to write, design of basic syscall functions within syscalls.c
- Edit multiboot_header.asm

Potential problems:
- New framebuffer stuff may not work with fs
- Background should clip windows too (may be resolved with DR)
- Double buffering significantly reduces performance
- Modify allocator address when using modules

Strategy:
Each window is a doubley linked list of windows of linked list of views (stuff like menus, etc) which contain a buffer of their size which is written to, coordinates, etc
Selected(with mouse)/new windows to the start of linked list with highest Z
Start with the deepest window to the shallowest (recursive algorithm)
Copy each window buffer to screen buffer and use compare memory - if different write (perhaps using SSE)
Give mouse/keyboard event to each window and let them decide whether to process (check position overlaps)

Now:
General protection fault
Dirty rectangles when dragging windows

Future:
Implement closing windows
Handle keyboard events

ln -s /usr/local/bin/x86_64-elf-ar x86_64-sidos-ar
ln -s /usr/local/bin/x86_64-elf-as x86_64-sidos-as
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-gcc
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-cc
ln -s /usr/local/bin/x86_64-elf-ranlib x86_64-sidos-ranlib

/Users/siddharth/Code/rust/os64/userland/newlib_build/build_raw/x86_64-sidos


3) Write a makefile to fully compile everything for ease of use

Write makefile to fully compile everything - autoconf, newlib, automake, binutils, gcc, etc