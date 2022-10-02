### Ramblings

Bochs magic breakpoint is xchg bx, bx

Todo:
Fix PIT Bug
IPC (stubs in syscalls.c/syscalls.rs tommorow)

ln -s /usr/local/bin/x86_64-elf-ar x86_64-sidos-ar
ln -s /usr/local/bin/x86_64-elf-as x86_64-sidos-as
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-gcc
ln -s /usr/local/bin/x86_64-elf-gcc x86_64-sidos-cc
ln -s /usr/local/bin/x86_64-elf-ranlib x86_64-sidos-ranlib

Things to add:

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
