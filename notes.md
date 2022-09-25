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
- Custom error handling with enums (custom enums for each file, asserts used instead of panic, use of ?)
- Clean code 

Usermode:
- Switching address space is broken with cr3
- Timer bug with RSP
- Test IPC
- Priority based round robin

FS:
- Load userspace programs from fs instead of modules
- Fix syscalls and integrate with fs
- Make new verify functions in fs
- Test creating new files with fs fails 

Memory:
- Start of memory bug
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

<!-- // Moves window to the top of the stack and trigers a repaint
// fn raise(&mut self, index: usize) {
//     // Move window if it isn't head (already at the top of the stack)
//     if (&*(*parent).children.head.unwrap()).payload.clone() != self.clone() {
//         let address = (*parent).children.remove_at(index);
//         // kfree(address as *mut u64);
//         (*parent).children.push(self.clone());
//     }
// } -->

