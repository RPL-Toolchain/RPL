pub trait Fat<T: ?Sized>: AsRef<T> + Sized {
    fn into_box(self, f: impl FnOnce(Self) -> *mut ()) -> Box<T> {
        let mut fat_ptr = self.as_ref() as *const T;
        let data_ptr = &mut fat_ptr as *mut *const T as *mut *mut ();
        //~^ ERROR: wrong assumption of fat pointer layout
        unsafe {
            *data_ptr = f(self);
            Box::from_raw(fat_ptr as *mut T)
        }
    }

    fn to_box(self, f: impl FnOnce(&T) -> *mut ()) -> Box<T> {
        let mut fat_ptr = self.as_ref() as *const T;
        let data_ptr = &mut fat_ptr as *mut *const T as *mut *mut ();
        //~^ ERROR: wrong assumption of fat pointer layout
        unsafe {
            *data_ptr = f(self.as_ref());
            Box::from_raw(fat_ptr as *mut T)
        }
    }
}
