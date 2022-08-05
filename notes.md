### List of useful resources

Bochs magic breakpoint is xchg bx, bx

TODO:
- Port newlib C library
- Extend usermode/syscall capabilities
- GUI (Window Manager)
- Port dart (git clone https://kernel.googlesource.com/pub/scm/utils/dash/dash)
- Malloc/Free

Refactoring Tasks:
- Address todos
- Memory allocator (malloc, free)
- Figure out how to get page_frame_allocator as global variable or passed (change linked list structure?)
- Rewrite PS2 controller, kbd, mouse (bitflags including paging)

Each window is a doubley linked list of windows of linked list of views (stuff like menus, etc) which contain a buffer of their size which is written to, coordinates, etc
Selected(with mouse)/new windows to the start of linked list with highest Z
Start with the deepest window to the shallowest (recursive algorithm)
Copy each window buffer to screen buffer and use compare memory - if different write (perhaps using SSE)
Give mouse/keyboard event to each window and let them decide whether to process (check position overlaps)

Make pf allocator globally available
Continue clipping system (remove rectangles, )
Change colour scheme, title bar, etc
Give each window clipping area 
- Push out (subtract window areas) for subwindows
- Use windows in z order (above)
- Ensure clipping area stuff works