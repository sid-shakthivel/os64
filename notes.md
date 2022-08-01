### List of useful resources

Bochs magic breakpoint is xchg bx, bx

TODO:
- Port newlib C library
- Extend usermode/syscall capabilities
- GUI 
- Port dart (git clone https://kernel.googlesource.com/pub/scm/utils/dash/dash)
- Malloc/Free

Large Tasks:
- Make Window system

Refactoring Tasks:
- Address todos
- Memory allocator
- Figure out how to get page_frame_allocator as global variable or passed (change linked list structure?)
- Rewrite PS2 controller, kbd, mouse

Each window is a doubley linked list of windows of linked list of views (stuff like menus, etc) which contain a buffer of their size which is written to, coordinates, etc
Selected(with mouse)/new windows to the start of linked list with highest Z
Start with the deepest window to the shallowest (recursive algorithm)
Copy each window buffer to screen buffer and use compare memory - if different write (perhaps using SSE)
Give mouse/keyboard event to each window and let them decide whether to process (check position overlaps)

Fix ps2 bug
PFA
Continue WMS
Use bitflags more
