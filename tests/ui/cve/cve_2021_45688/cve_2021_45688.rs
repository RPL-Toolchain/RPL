//@revisions: inline normal
//@[inline] compile-flags: -Z inline-mir=true
//@[inline] check-pass
//@[normal] compile-flags: -Z inline-mir=false
use std::io;
use std::slice;

pub fn read_spv<R: io::Read + io::Seek>(x: &mut R) -> io::Result<Vec<u32>> {
    let size = x.seek(io::SeekFrom::End(0))?;
    if size % 4 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "input length not divisible by 4",
        ));
    }
    if size > usize::max_value() as u64 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "input too long"));
    }
    let words = (size / 4) as usize;
    let mut result = Vec::<u32>::with_capacity(words); //~[normal] uninit_vec
    x.seek(io::SeekFrom::Start(0))?;
    unsafe {
        x.read_exact(slice::from_raw_parts_mut(
            //FIXME: a false negative, `result` is not initialized
            //ERROR: it violates the precondition of `std::slice::from_raw_parts_mut` to create a slice from uninitialized data
            result.as_mut_ptr() as *mut u8,
            words * 4,
        ))?;
        result.set_len(words);
        //~[normal]^ERROR: it violates the precondition of `Vec::set_len` to extend a `Vec`'s length without initializing its content in advance
    }
    const MAGIC_NUMBER: u32 = 0x0723_0203;
    if !result.is_empty() && result[0] == MAGIC_NUMBER.swap_bytes() {
        for word in &mut result {
            *word = word.swap_bytes();
        }
    }
    if result.is_empty() || result[0] != MAGIC_NUMBER {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "input missing SPIR-V magic number",
        ));
    }
    Ok(result)
}
