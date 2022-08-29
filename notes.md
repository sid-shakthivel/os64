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
- Collisions may fail

Now:
- Implement more syscalls / get inspired more ;)
- See if we can return a value from a syscall (inline assembly kinda thing)

ln -s /usr/local/bin/x86_64-elf-ar x86_64-sidos-ar
ln -s /usr/local/bin/x86_64-elf-as x86_64-sidos-as
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-gcc
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-cc
ln -s /usr/local/bin/x86_64-elf-ranlib x86_64-sidos-ranlib

REMEMBER ABOUT UPPER/LOWER CASES FOR FILESYSTEM

interprocess communication syscall stuff
fix multicluster length files (write syscall)
rename files
create new files/directories (open syscall)

can't implement rn:
kill (need IPC)
fstat (specific struct required

download the elf header file thing
get some ipc links open
get newer syscall stuff open