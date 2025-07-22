use std::cell::UnsafeCell;
use std::rc::Rc;

pub struct Cell<T> {
    pub inner: Rc<UnsafeCell<T>>,
}

/* impl<T> Clone for Cell<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
} */

impl<T> Cell<T> {
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.as_ref().get() }
        //~^ ERROR: Obtaining a mutable reference to the value wrapped by `Rc<UnsafeCell<$T>>` may be unsound
    }
}
