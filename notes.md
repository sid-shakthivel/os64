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

Rewrite linked list structure to become more generic
Rectangle Clipping - 
Window 2 -> Window 1
compare 2 edges for now
check whether overlap with edges, pick the one highest one and remove the other one 
make method to print lines (vertical, horizontal) to check stuff
output final clip to framebuffer
2 windows - clipping, subject
Mutablility is not effective (links with global pf_allocator)

Understand test conditions
Add to linked list appropriately and modify list of rects
Remove element correctly if split

    Top
Left -> Right
    Bottom