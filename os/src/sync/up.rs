use core::cell::{RefCell, RefMut};

// Uni-Processor synchronization primitives
pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}

// We are responsible for ensuring that 
// the data is only accessed by one core at a time
unsafe impl<T> Sync for UPSafeCell<T> {}

impl <T> UPSafeCell<T> {
    pub unsafe fn new(inner: T) -> Self {
        Self {
            inner: RefCell::new(inner),
        }
    }

    pub fn exclusive_access(&self) -> RefMut<T> {
        self.inner.borrow_mut()
    }
}
