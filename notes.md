### Ramblings

Bochs magic breakpoint is xchg bx, bx

TODO:
- Basic GUI (Window Manager)
- Malloc/Free (Memory allocator)
- Extend usermode/syscall capabilities 
- Port newlib C library and use actual C userland processes
- Port dart (git clone https://kernel.googlesource.com/pub/scm/utils/dash/dash)

Refactoring Tasks:
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
Fix bugs
Change colour scheme, title bar, uniformity, etc
Ensure freeing/dragging works properly
Get background rendered properly onto screen

Future:
Double buffering
Dirty rectangles when dragging windows
Build a small kernel land application
Implement closing windows

(HEAD) Red => Green (TAIL)

Red (Forground)
Green (Background)
Green should not render completly as red is above and is ahead of red anyway

check if overlaps with clipping_rect
Red over green (Correct)

Figure out a better way to check against clipping rects?

When we get to green
Should recognize there is red ahead of it
Should split the shape up
Should subtract the green away
Should loop through clipping rects and render them

Green - subject
Red - clip

Clipping rects are what we want to render
2nd window not rendered?
max_x is the issue

16:21