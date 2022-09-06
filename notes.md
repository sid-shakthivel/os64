### Ramblings

Bochs magic breakpoint is xchg bx, bx

Large Tasks:
- Basic GUI
- Extend usermode/syscall capabilities 
- Extend multitasking
- Polish (GUI, Code, etc)

Other Tasks:
- Fix the formatting (fixed with jetbrains licence?)
- Improve spinlock
- Replace the clones (https://www.youtube.com/watch?v=79phqVpE7cU) 
- Fix binutils and get ld to work properly (doesn't build on MacOS https://github.com/spack/spack/issues/3213)
- Add syslinks to newlib makefile 
- Bitflags
- Priority based round robin
- Memory start in pfa
- Custom error handling with enums (custom emails for each file, asserts used instead of panic, use of ?)

Usermode:
- argx, argrx

GUI:
- Double buffering with REP MOVSB (Bochs is broken, so can't do)
- Handle keyboard events
- Images (https://wiki.osdev.org/Loading_Icons)
- Text with windows (titles, etc)

FS:
- Load userspace programs from fs instead of modules
- Long file names for FAT16
- Make verify functions in fs and fb

Problems:
- New framebuffer stuff may not work with fs
- Double buffering significantly reduces performance
- Switching address space is broken with cr3
- Keyboard and mouse are broken in bochs
- Collisions may fail with hashmap
- Creating new files with fs fails slightly

ln -s /usr/local/bin/x86_64-elf-ar x86_64-sidos-ar
ln -s /usr/local/bin/x86_64-elf-as x86_64-sidos-as
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-gcc
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-cc
ln -s /usr/local/bin/x86_64-elf-ranlib x86_64-sidos-ranlib

