use core::sync::atomic::{AtomicBool, Ordering};

pub struct SpinMutex<T> {
    lock: AtomicBool,
    data: T,
}

// should implement Deref and Drop
// so every time the SpinGuard is dropped than the lock is free
pub struct SpinGuard<'a, T> {
    lock: &'a AtomicBool,
    data: *mut T,
}

impl<T> SpinMutex<T> {
    pub fn new(data: T) -> Self {
        Self {
            lock: Default::default(), // false
            data,
        }
    }

    // TODO: understand WHY It works without mutable
    // I did that for the globalAllcator static variable that is not mutable
    //pub fn lock<'a>(&'a mut self) -> SpinGuard<'a, T> {
    pub fn lock<'a>(&'a self) -> SpinGuard<'a, T> {
        // Try to swap the lock with a true,
        // swap return the previously value
        // if it was false than the mutex is correctly locked
        while self.lock.swap(true, Ordering::Relaxed) {
            core::hint::spin_loop();
        }

        SpinGuard {
            lock: &self.lock,
            //data: &mut self.data as *mut T
            data: &self.data as *const T as *mut T,
        }
    }
}

impl<'a, T> core::ops::Deref for SpinGuard<'a, T> {
    type Target = T;

    // deref always return a copy value?
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'a, T> core::ops::DerefMut for SpinGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<'a, T> core::ops::Drop for SpinGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.swap(false, Ordering::Relaxed);
    }
}
