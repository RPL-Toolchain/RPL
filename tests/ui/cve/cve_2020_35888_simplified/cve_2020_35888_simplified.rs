use std::alloc::{Layout, alloc};

struct DropDetector(u32);

impl Drop for DropDetector {
    fn drop(&mut self) {
        //println!("Dropping value: {} at {:?}", self.0, self as *const _);
    }
}

fn main() {
    let layout = Layout::new::<DropDetector>();

    let ptr: *mut DropDetector = unsafe { alloc(layout) as *mut DropDetector };

    unsafe {
        (*ptr) = DropDetector(12345);
        //~^ ERROR: dropped an possibly-uninitialized value
        std::ptr::drop_in_place(ptr);
    }
}
