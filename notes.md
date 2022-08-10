### Ramblings

Bochs magic breakpoint is xchg bx, bx

Large Tasks:
- Basic GUI
- Port newlib C library and use actual C userland processes
- Extend usermode/syscall capabilities along with multitasking
- Port dart (git clone https://kernel.googlesource.com/pub/scm/utils/dash/dash)

Smaller Tasks:
- Free memory/switch to malloc
- Address todos 
- Bitflags
- Improve spinlock
- Fix rust borrow stuff (https://www.youtube.com/watch?v=79phqVpE7cU)
- Optimize memcpy

Think:
- Switch to usize
- Make things like page frame allocator generic of types - large array would be of certain types

Potential problems:
- New framebuffer stuff may not work with fs
- Background should clip windows too (may be resolved with DR)
- Double buffering drastically reduces performance

Strategy:
Each window is a doubley linked list of windows of linked list of views (stuff like menus, etc) which contain a buffer of their size which is written to, coordinates, etc
Selected(with mouse)/new windows to the start of linked list with highest Z
Start with the deepest window to the shallowest (recursive algorithm)
Copy each window buffer to screen buffer and use compare memory - if different write (perhaps using SSE)
Give mouse/keyboard event to each window and let them decide whether to process (check position overlaps)

Now:
C user space programs (No elf?)
Dirty rectangles when dragging windows

Future:
Implement closing windows
Handle keyboard events

- Just compile a c file into a binary which can be used 
- Figure out how to compile with a standard library, etc