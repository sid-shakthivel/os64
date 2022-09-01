### Ramblings

Bochs magic breakpoint is xchg bx, bx

Large Tasks:
- Basic GUI
- Extend usermode/syscall capabilities 
- Extend multitasking
- Polish (GUI (background), Code, etc)

Smaller Tasks:
- Fix the attriocious formatting (fixed with jetbrains licence?)
- Improve spinlock
- Replace the clones (https://www.youtube.com/watch?v=79phqVpE7cU) 
- Fix binutils and get ld to work properly (doesn't build on MacOS https://github.com/spack/spack/issues/3213)
- Add syslinks to newlib makefile 
- Mutable into_iter_mut method
- Handle keyboard events
- Double buffering with REP MOVSB (Bochs is broken, so can't do)
- Bitflags
- Images (https://wiki.osdev.org/Loading_Icons)
- Load userspace programs from fs instead of modules
- Text with windows (titles, etc)

Think:
- Switch to usize
- Make an idle user space process with low priority which always runs

Problems:
- New framebuffer stuff may not work with fs
- Double buffering significantly reduces performance
- Switching address space is broken with cr3
- Keyboard and mouse are broken in bochs
- Collisions may fail with hashmap

ln -s /usr/local/bin/x86_64-elf-ar x86_64-sidos-ar
ln -s /usr/local/bin/x86_64-elf-as x86_64-sidos-as
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-gcc
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-cc
ln -s /usr/local/bin/x86_64-elf-ranlib x86_64-sidos-ranlib

REMEMBER ABOUT UPPER/LOWER CASES FOR FILESYSTEM