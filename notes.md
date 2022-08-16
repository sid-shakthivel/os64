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
- Replace any is_some() with if lets
- Fix binutils and get ld to work properly (doesn't build on MacOS)

Think:
- Switch to usize
- Make things like page frame allocator generic of types - large array would be of certain types
- Make an idle user space process with low priority which always runs
- Syscall management - which ones to write, design of basic syscall functions within syscalls.c
- Only repaint sections of window which have been updated

Potential problems:
- New framebuffer stuff may not work with fs
- Double buffering significantly reduces performance

Now:
Dirty rectangles when dragging windows
Raise windows
Compare memory - if different write (perhaps using SSE)
Handle keyboard events
Mouse pointer - make old mouse dirty, repaint it appropriately

Future:
General protection fault from syscall

ln -s /usr/local/bin/x86_64-elf-ar x86_64-sidos-ar
ln -s /usr/local/bin/x86_64-elf-as x86_64-sidos-as
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-gcc
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-cc
ln -s /usr/local/bin/x86_64-elf-ranlib x86_64-sidos-ranlib

Phantom's fixed
Some parts of border still persist 
Fix how windows look in order to 
Clip out the top bit?