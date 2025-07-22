//@ ignore-on-host

use std::{mem::MaybeUninit, ptr};

pub trait Unsigned: Copy + Default + 'static {
    const U8: u8;

    const U16: u16;

    const U32: u32;

    const U64: u64;

    const U128: u128;

    const USIZE: usize;

    const I8: i8;

    const I16: i16;

    const I32: i32;

    const I64: i64;

    const I128: i128;

    const ISIZE: isize;

    fn to_u8() -> u8;

    fn to_u16() -> u16;

    fn to_u32() -> u32;

    fn to_u64() -> u64;

    fn to_u128() -> u128;

    fn to_usize() -> usize;

    fn to_i8() -> i8;

    fn to_i16() -> i16;

    fn to_i32() -> i32;

    fn to_i64() -> i64;

    fn to_i128() -> i128;

    fn to_isize() -> isize;
}

pub trait ChunkLength<A>: Unsigned {
    /// A `Sized` type matching the size of an array of `Self` elements of `A`.
    type SizedType;
}

pub struct Chunk<A, N = u64>
where
    N: ChunkLength<A>,
{
    left: usize,
    right: usize,
    data: MaybeUninit<N::SizedType>,
}

impl<A, N> Chunk<A, N>
where
    N: ChunkLength<A>,
{
    pub const CAPACITY: usize = N::USIZE;

    /// Get the length of the chunk.
    #[inline]
    pub fn len(&self) -> usize {
        self.right - self.left
    }

    #[inline]
    unsafe fn force_write(index: usize, value: A, chunk: &mut Self) {
        chunk.mut_ptr(index).write(value)
    }

    #[inline]
    unsafe fn mut_ptr(&mut self, index: usize) -> *mut A {
        (&mut self.data as *mut _ as *mut A).add(index)
    }

    #[inline]
    unsafe fn ptr(&self, index: usize) -> *const A {
        (&self.data as *const _ as *const A).add(index)
    }

    #[inline]
    unsafe fn force_copy(from: usize, to: usize, count: usize, chunk: &mut Self) {
        if count > 0 {
            ptr::copy(chunk.ptr(from), chunk.mut_ptr(to), count)
        }
    }

    pub fn insert_from<Iterable, I>(&mut self, index: usize, iter: Iterable)
    where
        Iterable: IntoIterator<Item = A, IntoIter = I>,
        I: ExactSizeIterator<Item = A>,
    {
        let iter = iter.into_iter();
        let insert_size = iter.len();
        if self.len() + insert_size > Self::CAPACITY {
            panic!(
                "Chunk::insert_from: chunk cannot fit {} elements",
                insert_size
            );
        }
        if index > self.len() {
            panic!("Chunk::insert_from: index out of bounds");
        }
        let real_index = index + self.left;
        let left_size = index;
        let right_size = self.right - real_index;
        if self.right == N::USIZE || (self.left >= insert_size && left_size < right_size) {
            unsafe {
                Chunk::force_copy(self.left, self.left - insert_size, left_size, self);
                let mut write_index = real_index - insert_size;
                for value in iter {
                    Chunk::force_write(write_index, value, self);
                    write_index += 1;
                }
            }
            self.left -= insert_size;
        } else if self.left == 0 || (self.right + insert_size <= Self::CAPACITY) {
            unsafe {
                Chunk::force_copy(real_index, real_index + insert_size, right_size, self);
                let mut write_index = real_index;
                for value in iter {
                    Chunk::force_write(write_index, value, self);
                    write_index += 1;
                }
            }
            self.right += insert_size;
        } else {
            unsafe {
                Chunk::force_copy(self.left, 0, left_size, self);
                Chunk::force_copy(real_index, left_size + insert_size, right_size, self);
                let mut write_index = left_size;
                for value in iter {
                    Chunk::force_write(write_index, value, self);
                    write_index += 1;
                }
            }
            self.right -= self.left;
            self.right += insert_size;
            self.left = 0;
        }
    }
}
