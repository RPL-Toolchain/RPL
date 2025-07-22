//@check-pass

use std::{
    cell::UnsafeCell,
    sync::{atomic::AtomicBool, Mutex},
};

enum ThisOrThat<T, U> {
    This(T),
    That(U),
}

/// `LazyTransform<T, U>` is a synchronized holder type, that holds a value of
/// type T until it is lazily converted into a value of type U.
pub struct LazyTransform<T, U> {
    initialized: AtomicBool,
    lock: Mutex<()>,
    value: UnsafeCell<Option<ThisOrThat<T, U>>>,
}

unsafe impl<T, U> Sync for LazyTransform<T, U>
where
    T: Sync + Send,
    U: Sync, // fix: Sync + Send
{
}
