### Ramblings

Bochs magic breakpoint is xchg bx, bx

Large Tasks:
- Basic GUI
- Extend usermode/syscall capabilities 
- Extend multitasking
- Polish (GUI (background), Code, etc)

Smaller Tasks:
- REP MOVSB instruction
- Bitflags
- Make malloc multipage (when extending, merge memory)
- Free memory/switch to malloc everywhere
- Improve spinlock
- Fix rust borrow stuff (https://www.youtube.com/watch?v=79phqVpE7cU)
- Add syslinks to newlib makefile
- Mutable into_iter_mut method
- Replace enumerate() with map

Think:
- Switch to usize
- Make things like page frame allocator generic of types - large array would be of certain types
- Make an idle user space process with low priority which always runs
- Syscall management - which ones to write, design of basic syscall functions within syscalls.c

Potential problems:
- New framebuffer stuff may not work with fs
- Double buffering significantly reduces performance
- Fix binutils and get ld to work properly (doesn't build on MacOS)

Now:
Dirty rectangles when dragging windows/raising em
Compare memory - if different write (perhaps using SSE)
Handle keyboard events
General protection fault from syscall

Future:
Handle title bar inactivity, activity
Handle closing of windows

ln -s /usr/local/bin/x86_64-elf-ar x86_64-sidos-ar
ln -s /usr/local/bin/x86_64-elf-as x86_64-sidos-as
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-gcc
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-cc
ln -s /usr/local/bin/x86_64-elf-ranlib x86_64-sidos-ranlib

No extendability for windows below yet (clipping)

Window_apply_bound_clipping(Window* window, int in_recursion, List* dirty_regions)
Window_paint(Window* window, List* dirty_regions, uint8_t paint_children)
Window_move(Window* window, int new_x, int new_y) - method called within the mouse handler method

CLIP the background by making it a window itself

Currently parent => child but only 1 parent and lots of children (for now)

For now - have the main  stuff within window and have smaller helper functions within Desktop (for now)

Should clip against self just in case?
Intersect clipping rects?


