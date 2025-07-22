use memmap::{Mmap, Protection};
use std::{
    io::Result,
    marker::PhantomData,
    ops::Index,
    sync::{Arc, Mutex},
};

/// A buffer that is backed by an anonymous memory map.
///
/// **Anonymously** means that the memory is not backed by a file.
pub struct AnonymousBuffer<T>
where
    T: Sized,
{
    data: Arc<Mutex<Mmap>>,
    phantom: PhantomData<T>,
}

impl<T> AnonymousBuffer<T>
where
    T: Sized,
{
    /// Create a new anonymous buffer of the given size.
    pub fn try_new(size: usize) -> Result<Self> {
        let map = Mmap::anonymous(size, Protection::ReadWrite)?;
        Ok(Self {
            data: Arc::new(Mutex::new(map)),
            phantom: PhantomData,
        })
    }
}

impl<T> Index<usize> for AnonymousBuffer<T>
where
    T: Sized,
{
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        unsafe {
            let mut count = idx;
            let mut p: *const T = self.data.lock().unwrap().ptr() as *const T;
            while count > 0 {
                count -= 1;
                p = p.offset(1);
                //~^ERROR: it is an undefined behavior to offset a pointer using an unchecked integer
            }
            &*p
            //~^ERROR: it is unsound to dereference a pointer that is offset using an unchecked integer
        }
    }
}

fn main() {
    let buf = AnonymousBuffer::<u8>::try_new(1024).unwrap();
    println!("{:?}", &buf[1024]);
}
