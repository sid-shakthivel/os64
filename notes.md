### Ramblings

Bochs magic breakpoint is xchg bx, bx

TODO:
- Port newlib C library
- Extend usermode/syscall capabilities
- GUI (Window Manager)
- Port dart (git clone https://kernel.googlesource.com/pub/scm/utils/dash/dash)
- Malloc/Free (Memory allocator)
- C usermode processes (little book of os dev)

Refactoring Tasks:
- Address todos
- Rewrite PS2 controller, kbd, mouse (bitflags including paging)
- Improve spinlock

Each window is a doubley linked list of windows of linked list of views (stuff like menus, etc) which contain a buffer of their size which is written to, coordinates, etc
Selected(with mouse)/new windows to the start of linked list with highest Z
Start with the deepest window to the shallowest (recursive algorithm)
Copy each window buffer to screen buffer and use compare memory - if different write (perhaps using SSE)
Give mouse/keyboard event to each window and let them decide whether to process (check position overlaps)

Finish refactor to new linked list structure (implement remove too)
Continue clipping system (remove rectangles, add properly (refactoring))
Change colour scheme, title bar, etc
Give each window clipping area 
- Push out (subtract window areas) for subwindows
- Use windows in z order (above)
- Ensure clipping area stuff works

(head) newer -> new -> old (tail)

(head) win2 -> win1 (tail)

make the tail of second list point to first list head
figure out lifetime stuff / make it Window