use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use core::cell::UnsafeCell;

pub struct Lock<T>
{
    active: AtomicBool,
    data: UnsafeCell<T>
}

impl<T> Lock<T>
{
    pub const fn new(data: T) -> Lock<T>
    {
        Lock
        {
            active: AtomicBool::new(false),
            data: UnsafeCell::new(data)
        }
    }

    pub fn lock(&self) -> &mut T
    {
        if self.active.load(Ordering::Acquire) { panic!("spinlock already locked"); }
        self.active.store(true, Ordering::Release);

        unsafe { &mut *self.data.get() }
    }

    pub fn free(&self)
    {
        self.active.store(false, Ordering::Release);
    }
}

unsafe impl<T> Sync for Lock<T> {}