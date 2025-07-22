//@compile-flags: -Zinline-mir=false
pub fn ensure_buffer_len(mut buffer: Vec<i32>, new_len: usize) -> Vec<i32> {
    if buffer.len() < new_len {
        //~^ ERROR: Use `Vec::set_len` to extend the length of a `Vec`, potentially including uninitialized elements
        if buffer.capacity() < new_len {
            buffer = Vec::with_capacity(new_len);
            //~^ uninit_vec
            //~| uninit_vec
        }
        unsafe {
            buffer.set_len(new_len);
        }
    } else {
        buffer.truncate(new_len);
    }
    buffer
}

fn main() {}
