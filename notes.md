### Ramblings

Bochs magic breakpoint is xchg bx, bx

TODO:
- Basic GUI (Window Manager)
- Malloc/Free (Memory allocator)
- Port newlib C library and use actual C userland processes
- Extend usermode/syscall capabilities 
- Port dart (git clone https://kernel.googlesource.com/pub/scm/utils/dash/dash)

Refactoring Tasks:
- Free memory properly
- Address todos
- Bitflags
- Improve spinlock
- Fix rust borrow stuff (https://www.youtube.com/watch?v=79phqVpE7cU)

Each window is a doubley linked list of windows of linked list of views (stuff like menus, etc) which contain a buffer of their size which is written to, coordinates, etc
Selected(with mouse)/new windows to the start of linked list with highest Z
Start with the deepest window to the shallowest (recursive algorithm)
Copy each window buffer to screen buffer and use compare memory - if different write (perhaps using SSE)
Give mouse/keyboard event to each window and let them decide whether to process (check position overlaps)

Now:
Malloc/Free
Background (https://forum.osdev.org/viewtopic.php?f=13&t=30154) or render background (rbg file) (https://wiki.osdev.org Drawing_In_a_Linear_Framebuffer)
Improve performance (https://wiki.osdev.org/GUI)

Future:
Dirty rectangles when dragging windows
Double buffering (back/front buffer)
Implement closing windows
Dragging only upon title bar

Nuance to memory manager:
What happens when we allocate more then 1 page and need more memory? ALLOCATE ANOTHER PAGE
On alloc, do we want to split a section of memory? YES
On free, just make the space INACTIVE
If size is too big - just allocate multiple consecutive pages

TESTING
Free list?
Bigger then 1 page allocation stuff
