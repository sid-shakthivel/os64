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
- Writeup double buffering
- Gradient with clipping and mouse (working on it)
- Bochs issue - mask value must be corrupted
- Get doom working within syscalls (rewrite them using fs to clean up)
- Doesn't copy to buffer properly (text gets scrambled - roughly correct)
- Cetnre title text within initalise_window_buffer (syscalls)
- Add scancode to Event properly

FS:
- Load userspace programs from fs instead of modules
- Fully integrate the fs
- Make new verify functions in fs
- Creating new files with fs fails (maybe)

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

<!-- // Paint the title text and centre it
// FRAMEBUFFER.lock().draw_string(
//     Some(&self.clipped_rectangles),
//     self.title,
//     self.x + (self.width / 2 - (self.title.as_bytes().len() * 8) as u64 / 2),
//     self.y + (WINDOW_TITLE_HEIGHT - 10) / 2,
//     self.x,
//     self.y,
//     self.width,
//     self.height,
// ); -->

fix draw string moving bug (use buffer coords in thing)
update_buffer_region_to_colour fails with doom second time
syscall to copy to internal buffer fails

after window is over 300 then random failure
the issue lies with copying to the buffer