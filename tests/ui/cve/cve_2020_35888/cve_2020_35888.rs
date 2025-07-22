use std::alloc::{Layout, alloc};

pub struct Array<T> {
    size: usize,
    ptr: *mut T,
}

impl<T> Array<T>
where
    T: Clone,
{
    pub fn new_from_template(size: usize, template: &T) -> Self {
        let objsize = std::mem::size_of::<T>();
        let layout = Layout::from_size_align(size * objsize, 8).unwrap();
        let ptr = unsafe { alloc(layout) as *mut T };
        for i in 0..size {
            unsafe {
                (*(ptr.wrapping_offset(i as isize))) = template.clone();
                //~^ ERROR: dropped an possibly-uninitialized value
            }
        }
        Self { size, ptr }
    }
}

#[derive(Clone, Debug)]
struct DropDetector(u32);

impl Drop for DropDetector {
    fn drop(&mut self) {
        println!("Dropping value: {} at {:?}", self.0, self as *const _);
    }
}
fn main() {
    // let array = Array::new_from_template(2, &DropDetector(12345));
    // for i in 0..array.size {
    //     // drop elements
    //     unsafe {
    //         let ptr = array.ptr.wrapping_offset(i as isize);
    //         std::ptr::drop_in_place(ptr);
    //     }
    // }
}
