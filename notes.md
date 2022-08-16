### Ramblings

Bochs magic breakpoint is xchg bx, bx

Large Tasks:
- Basic GUI
- Extend usermode/syscall capabilities 
- Extend multitasking
- Polish (GUI (background), Code, etc)

Smaller Tasks:
- Fix the attriocious formatting
- Bitflags
- Make malloc multipage (when extending, merge memory)
- Free memory/switch to malloc everywhere
- Improve spinlock
- Replace the clones (https://www.youtube.com/watch?v=79phqVpE7cU) 
- Add syslinks to newlib makefile
- Mutable into_iter_mut method
- Replace enumerate() with map
- Fix binutils and get ld to work properly (doesn't build on MacOS)
- Try double buffering once again with REP MOVSB instruction
- Handle keyboard events

Think:
- Switch to usize
- Make things like page frame allocator generic of types - large array would be of certain types
- Make an idle user space process with low priority which always runs
- Syscall management - which ones to write, design of basic syscall functions within syscalls.c

Potential problems:
- New framebuffer stuff may not work with fs
- Double buffering significantly reduces performance
- General protection fault from syscall

ln -s /usr/local/bin/x86_64-elf-ar x86_64-sidos-ar
ln -s /usr/local/bin/x86_64-elf-as x86_64-sidos-as
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-gcc
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-cc
ln -s /usr/local/bin/x86_64-elf-ranlib x86_64-sidos-ranlib
