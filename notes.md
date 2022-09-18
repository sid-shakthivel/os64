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
- Priority based round robin
- Custom error handling with enums (custom enums for each file, asserts used instead of panic, use of ?)
- Clean code (Remove all the static mut)

Usermode:
- Switching address space is broken with cr3
- IPC process
- Timer bug with RSP

GUI:
- Double buffering with REP MOVSB 
- Gradient with clipping and mouse...
- Handle text on moving
- Bochs issue - mask value must be corrupted
- Porting doom

FS:
- Load userspace programs from fs instead of modules
- Make new verify functions in fs
- Creating new files with fs fails (maybe)

Memory:
- Start of memory
- malloc/free bugs
- Bitflags (paging)

Potential Problems:
- Collisions may fail with hashmap

ln -s /usr/local/bin/x86_64-elf-ar x86_64-sidos-ar
ln -s /usr/local/bin/x86_64-elf-as x86_64-sidos-as
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-gcc
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-cc
ln -s /usr/local/bin/x86_64-elf-ranlib x86_64-sidos-ranlib

<!-- pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
} -->

check if doom copies bits properly to fb
why does it lseek the file size then set back to 0 twice?

offsets at a memory address, 
goes 20 times
(maybe) goes out of bounds