//@ ignore-on-host

extern crate alloc as core_alloc;

mod alloc {
    #![allow(unstable_name_collisions)]
    #![allow(dead_code)]

    use core::cmp;
    use core::fmt;
    use core::mem;
    use core::ptr::{self, NonNull};
    use core::usize;

    pub use core::alloc::{Layout, LayoutErr};

    fn new_layout_err() -> LayoutErr {
        Layout::from_size_align(1, 3).unwrap_err()
    }

    pub fn handle_alloc_error(layout: Layout) -> ! {
        panic!("encountered allocation error: {:?}", layout)
    }

    pub trait UnstableLayoutMethods {
        fn padding_needed_for(&self, align: usize) -> usize;
        fn repeat(&self, n: usize) -> Result<(Layout, usize), LayoutErr>;
        fn array<T>(n: usize) -> Result<Layout, LayoutErr>;
    }

    impl UnstableLayoutMethods for Layout {
        fn padding_needed_for(&self, align: usize) -> usize {
            let len = self.size();

            let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
            len_rounded_up.wrapping_sub(len)
        }

        fn repeat(&self, n: usize) -> Result<(Layout, usize), LayoutErr> {
            let padded_size = self
                .size()
                .checked_add(self.padding_needed_for(self.align()))
                .ok_or_else(new_layout_err)?;
            let alloc_size = padded_size.checked_mul(n).ok_or_else(new_layout_err)?;

            unsafe {
                // self.align is already known to be valid and alloc_size has been
                // padded already.
                Ok((
                    Layout::from_size_align_unchecked(alloc_size, self.align()),
                    padded_size,
                ))
            }
        }

        fn array<T>(n: usize) -> Result<Layout, LayoutErr> {
            Layout::new::<T>().repeat(n).map(|(k, offs)| {
                debug_assert!(offs == mem::size_of::<T>());
                k
            })
        }
    }

    #[derive(Debug)]
    pub struct Excess(pub NonNull<u8>, pub usize);

    fn size_align<T>() -> (usize, usize) {
        (mem::size_of::<T>(), mem::align_of::<T>())
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct AllocErr;

    impl fmt::Display for AllocErr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("memory allocation failed")
        }
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct CannotReallocInPlace;

    impl CannotReallocInPlace {
        pub fn description(&self) -> &str {
            "cannot reallocate allocator's memory in place"
        }
    }

    impl fmt::Display for CannotReallocInPlace {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.description())
        }
    }

    pub unsafe trait Alloc {
        unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr>;

        unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout);

        #[inline]
        fn usable_size(&self, layout: &Layout) -> (usize, usize) {
            (layout.size(), layout.size())
        }

        unsafe fn realloc(
            &mut self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
        ) -> Result<NonNull<u8>, AllocErr> {
            let old_size = layout.size();

            if new_size >= old_size {
                if let Ok(()) = self.grow_in_place(ptr, layout, new_size) {
                    return Ok(ptr);
                }
            } else if new_size < old_size {
                if let Ok(()) = self.shrink_in_place(ptr, layout, new_size) {
                    return Ok(ptr);
                }
            }

            // otherwise, fall back on alloc + copy + dealloc.
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            let result = self.alloc(new_layout);
            if let Ok(new_ptr) = result {
                ptr::copy_nonoverlapping(
                    ptr.as_ptr(),
                    new_ptr.as_ptr(),
                    cmp::min(old_size, new_size),
                );
                self.dealloc(ptr, layout);
            }
            result
        }

        unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
            let size = layout.size();
            let p = self.alloc(layout);
            if let Ok(p) = p {
                ptr::write_bytes(p.as_ptr(), 0, size);
            }
            p
        }

        unsafe fn alloc_excess(&mut self, layout: Layout) -> Result<Excess, AllocErr> {
            let usable_size = self.usable_size(&layout);
            self.alloc(layout).map(|p| Excess(p, usable_size.1))
        }

        unsafe fn realloc_excess(
            &mut self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
        ) -> Result<Excess, AllocErr> {
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            let usable_size = self.usable_size(&new_layout);
            self.realloc(ptr, layout, new_size)
                .map(|p| Excess(p, usable_size.1))
        }

        unsafe fn grow_in_place(
            &mut self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
        ) -> Result<(), CannotReallocInPlace> {
            let _ = ptr; // this default implementation doesn't care about the actual address.
            debug_assert!(new_size >= layout.size());
            let (_l, u) = self.usable_size(&layout);
            // _l <= layout.size()                       [guaranteed by usable_size()]
            //       layout.size() <= new_layout.size()  [required by this method]
            if new_size <= u {
                Ok(())
            } else {
                Err(CannotReallocInPlace)
            }
        }

        unsafe fn shrink_in_place(
            &mut self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
        ) -> Result<(), CannotReallocInPlace> {
            let _ = ptr; // this default implementation doesn't care about the actual address.
            debug_assert!(new_size <= layout.size());
            let (l, _u) = self.usable_size(&layout);
            //                      layout.size() <= _u  [guaranteed by usable_size()]
            // new_layout.size() <= layout.size()        [required by this method]
            if l <= new_size {
                Ok(())
            } else {
                Err(CannotReallocInPlace)
            }
        }

        fn alloc_one<T>(&mut self) -> Result<NonNull<T>, AllocErr>
        where
            Self: Sized,
        {
            let k = Layout::new::<T>();
            if k.size() > 0 {
                unsafe { self.alloc(k).map(|p| p.cast()) }
            } else {
                Err(AllocErr)
            }
        }

        unsafe fn dealloc_one<T>(&mut self, ptr: NonNull<T>)
        where
            Self: Sized,
        {
            let k = Layout::new::<T>();
            if k.size() > 0 {
                self.dealloc(ptr.cast(), k);
            }
        }

        fn alloc_array<T>(&mut self, n: usize) -> Result<NonNull<T>, AllocErr>
        where
            Self: Sized,
        {
            match Layout::array::<T>(n) {
                Ok(layout) if layout.size() > 0 => unsafe { self.alloc(layout).map(|p| p.cast()) },
                _ => Err(AllocErr),
            }
        }

        unsafe fn realloc_array<T>(
            &mut self,
            ptr: NonNull<T>,
            n_old: usize,
            n_new: usize,
        ) -> Result<NonNull<T>, AllocErr>
        where
            Self: Sized,
        {
            match (Layout::array::<T>(n_old), Layout::array::<T>(n_new)) {
                (Ok(ref k_old), Ok(ref k_new)) if k_old.size() > 0 && k_new.size() > 0 => {
                    debug_assert!(k_old.align() == k_new.align());
                    self.realloc(ptr.cast(), k_old.clone(), k_new.size())
                        .map(NonNull::cast)
                }
                _ => Err(AllocErr),
            }
        }

        unsafe fn dealloc_array<T>(&mut self, ptr: NonNull<T>, n: usize) -> Result<(), AllocErr>
        where
            Self: Sized,
        {
            match Layout::array::<T>(n) {
                Ok(k) if k.size() > 0 => {
                    self.dealloc(ptr.cast(), k);
                    Ok(())
                }
                _ => Err(AllocErr),
            }
        }
    }
}

use core::cell::Cell;
use core::iter;
use core::marker::PhantomData;
use core::mem;
use core::ptr::{self, NonNull};
use core::slice;
use core::str;
use core_alloc::alloc::{alloc, dealloc, Layout};

#[derive(Debug)]
pub struct Bump {
    // The current chunk we are bump allocating within.
    current_chunk_footer: Cell<NonNull<ChunkFooter>>,
}

#[repr(C)]
#[derive(Debug)]
struct ChunkFooter {
    // Pointer to the start of this chunk allocation. This footer is always at
    // the end of the chunk.
    data: NonNull<u8>,

    // The layout of this chunk's allocation.
    layout: Layout,

    // Link to the previous chunk, if any.
    prev: Cell<Option<NonNull<ChunkFooter>>>,

    // Bump allocation finger that is always in the range `self.data..=self`.
    ptr: Cell<NonNull<u8>>,
}

impl Default for Bump {
    fn default() -> Bump {
        Bump::new()
    }
}

impl Drop for Bump {
    fn drop(&mut self) {
        unsafe {
            dealloc_chunk_list(Some(self.current_chunk_footer.get()));
        }
    }
}

#[inline]
unsafe fn dealloc_chunk_list(mut footer: Option<NonNull<ChunkFooter>>) {
    while let Some(f) = footer {
        footer = f.as_ref().prev.get();
        dealloc(f.as_ref().data.as_ptr(), f.as_ref().layout);
    }
}

unsafe impl Send for Bump {}

#[inline]
pub(crate) fn round_up_to(n: usize, divisor: usize) -> Option<usize> {
    debug_assert!(divisor > 0);
    debug_assert!(divisor.is_power_of_two());
    Some(n.checked_add(divisor - 1)? & !(divisor - 1))
}

// After this point, we try to hit page boundaries instead of powers of 2
const PAGE_STRATEGY_CUTOFF: usize = 0x1000;

// We only support alignments of up to 16 bytes for iter_allocated_chunks.
const SUPPORTED_ITER_ALIGNMENT: usize = 16;
const CHUNK_ALIGN: usize = SUPPORTED_ITER_ALIGNMENT;
const FOOTER_SIZE: usize = mem::size_of::<ChunkFooter>();

// Assert that ChunkFooter is at most the supported alignment. This will give a compile time error if it is not the case
const _FOOTER_ALIGN_ASSERTION: bool = mem::align_of::<ChunkFooter>() <= CHUNK_ALIGN;
const _: [(); _FOOTER_ALIGN_ASSERTION as usize] = [()];

// Maximum typical overhead per allocation imposed by allocators.
const MALLOC_OVERHEAD: usize = 16;

const OVERHEAD: usize = (MALLOC_OVERHEAD + FOOTER_SIZE + (CHUNK_ALIGN - 1)) & !(CHUNK_ALIGN - 1);

// Choose a relatively small default initial chunk size, since we double chunk
// sizes as we grow bump arenas to amortize costs of hitting the global
// allocator.
const FIRST_ALLOCATION_GOAL: usize = 1 << 9;

// The actual size of the first allocation is going to be a bit smaller
// than the goal. We need to make room for the footer, and we also need
// take the alignment into account.
const DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER: usize = FIRST_ALLOCATION_GOAL - OVERHEAD;

#[inline]
fn layout_for_array<T>(len: usize) -> Option<Layout> {
    let layout = Layout::new::<T>();
    let size_rounded_up = round_up_to(layout.size(), layout.align())?;
    let total_size = len.checked_mul(size_rounded_up)?;

    Layout::from_size_align(total_size, layout.align()).ok()
}

/// Wrapper around `Layout::from_size_align` that adds debug assertions.
#[inline]
unsafe fn layout_from_size_align(size: usize, align: usize) -> Layout {
    if cfg!(debug_assertions) {
        Layout::from_size_align(size, align).unwrap()
    } else {
        Layout::from_size_align_unchecked(size, align)
    }
}

#[inline(never)]
fn allocation_size_overflow<T>() -> T {
    panic!("requested allocation size overflowed")
}

impl Bump {
    /// Construct a new arena to bump allocate into.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// # let _ = bump;
    /// ```
    pub fn new() -> Bump {
        Self::with_capacity(0)
    }

    /// Construct a new arena with the specified capacity to bump allocate into.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::with_capacity(100);
    /// # let _ = bump;
    /// ```
    pub fn with_capacity(capacity: usize) -> Bump {
        let chunk_footer = Self::new_chunk(
            None,
            Some(unsafe { layout_from_size_align(capacity, 1) }),
            None,
        );
        Bump {
            current_chunk_footer: Cell::new(chunk_footer),
        }
    }

    /// Allocate a new chunk and return its initialized footer.
    ///
    /// If given, `layouts` is a tuple of the current chunk size and the
    /// layout of the allocation request that triggered us to fall back to
    /// allocating a new chunk of memory.
    fn new_chunk(
        old_size_with_footer: Option<usize>,
        requested_layout: Option<Layout>,
        prev: Option<NonNull<ChunkFooter>>,
    ) -> NonNull<ChunkFooter> {
        unsafe {
            // As a sane default, we want our new allocation to be about twice as
            // big as the previous allocation
            let mut new_size_without_footer =
                if let Some(old_size_with_footer) = old_size_with_footer {
                    let old_size_without_footer = old_size_with_footer - FOOTER_SIZE;
                    old_size_without_footer
                        .checked_mul(2)
                        .unwrap_or_else(|| oom())
                } else {
                    DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER
                };

            // We want to have CHUNK_ALIGN or better alignment
            let mut align = CHUNK_ALIGN;

            // If we already know we need to fulfill some request,
            // make sure we allocate at least enough to satisfy it
            if let Some(requested_layout) = requested_layout {
                align = align.max(requested_layout.align());
                let requested_size = round_up_to(requested_layout.size(), align)
                    .unwrap_or_else(allocation_size_overflow);
                new_size_without_footer = new_size_without_footer.max(requested_size);
            }

            // We want our allocations to play nice with the memory allocator,
            // and waste as little memory as possible.
            // For small allocations, this means that the entire allocation
            // including the chunk footer and mallocs internal overhead is
            // as close to a power of two as we can go without going over.
            // For larger allocations, we only need to get close to a page
            // boundary without going over.
            if new_size_without_footer < PAGE_STRATEGY_CUTOFF {
                new_size_without_footer =
                    (new_size_without_footer + OVERHEAD).next_power_of_two() - OVERHEAD;
            } else {
                new_size_without_footer = round_up_to(new_size_without_footer + OVERHEAD, 0x1000)
                    .unwrap_or_else(|| oom())
                    - OVERHEAD;
            }

            debug_assert_eq!(align % CHUNK_ALIGN, 0);
            debug_assert_eq!(new_size_without_footer % CHUNK_ALIGN, 0);
            let size = new_size_without_footer
                .checked_add(FOOTER_SIZE)
                .unwrap_or_else(allocation_size_overflow);
            let layout = layout_from_size_align(size, align);

            debug_assert!(size >= old_size_with_footer.unwrap_or(0) * 2);

            let data = alloc(layout);
            let data = NonNull::new(data).unwrap_or_else(|| oom());

            // The `ChunkFooter` is at the end of the chunk.
            let footer_ptr = data.as_ptr() as usize + new_size_without_footer;
            debug_assert_eq!((data.as_ptr() as usize) % align, 0);
            debug_assert_eq!(footer_ptr % CHUNK_ALIGN, 0);
            let footer_ptr = footer_ptr as *mut ChunkFooter;

            // The bump pointer is initialized to the end of the range we will
            // bump out of.
            let ptr = Cell::new(NonNull::new_unchecked(footer_ptr as *mut u8));

            ptr::write(
                footer_ptr,
                ChunkFooter {
                    data,
                    layout,
                    prev: Cell::new(prev),
                    ptr,
                },
            );

            NonNull::new_unchecked(footer_ptr)
        }
    }

    pub fn reset(&mut self) {
        // Takes `&mut self` so `self` must be unique and there can't be any
        // borrows active that would get invalidated by resetting.
        unsafe {
            let cur_chunk = self.current_chunk_footer.get();

            // Deallocate all chunks except the current one
            let prev_chunk = cur_chunk.as_ref().prev.replace(None);
            dealloc_chunk_list(prev_chunk);

            // Reset the bump finger to the end of the chunk.
            cur_chunk.as_ref().ptr.set(cur_chunk.cast());

            debug_assert!(
                self.current_chunk_footer
                    .get()
                    .as_ref()
                    .prev
                    .get()
                    .is_none(),
                "We should only have a single chunk"
            );
            debug_assert_eq!(
                self.current_chunk_footer.get().as_ref().ptr.get(),
                self.current_chunk_footer.get().cast(),
                "Our chunk's bump finger should be reset to the start of its allocation"
            );
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.alloc_with(|| val)
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_with<F, T>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        #[inline(always)]
        unsafe fn inner_writer<T, F>(ptr: *mut T, f: F)
        where
            F: FnOnce() -> T,
        {
            ptr::write(ptr, f())
        }

        let layout = Layout::new::<T>();

        unsafe {
            let p = self.alloc_layout(layout);
            let p = p.as_ptr() as *mut T;
            inner_writer(p, f);
            &mut *p
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_copy<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Copy,
    {
        let layout = Layout::for_value(src);
        let dst = self.alloc_layout(layout).cast::<T>();

        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), src.len());
            slice::from_raw_parts_mut(dst.as_ptr(), src.len())
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_clone<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Clone,
    {
        let layout = Layout::for_value(src);
        let dst = self.alloc_layout(layout).cast::<T>();

        unsafe {
            for (i, val) in src.iter().cloned().enumerate() {
                ptr::write(dst.as_ptr().add(i), val);
            }

            slice::from_raw_parts_mut(dst.as_ptr(), src.len())
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_str(&self, src: &str) -> &mut str {
        let buffer = self.alloc_slice_copy(src.as_bytes());
        unsafe {
            // This is OK, because it already came in as str, so it is guaranteed to be utf8
            str::from_utf8_unchecked_mut(buffer)
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_fill_with<T, F>(&self, len: usize, mut f: F) -> &mut [T]
    where
        F: FnMut(usize) -> T,
    {
        let layout = layout_for_array::<T>(len).unwrap_or_else(|| oom());
        let dst = self.alloc_layout(layout).cast::<T>();

        unsafe {
            for i in 0..len {
                ptr::write(dst.as_ptr().add(i), f(i));
            }

            let result = slice::from_raw_parts_mut(dst.as_ptr(), len);
            debug_assert_eq!(Layout::for_value(result), layout);
            result
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_fill_copy<T: Copy>(&self, len: usize, value: T) -> &mut [T] {
        self.alloc_slice_fill_with(len, |_| value)
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_fill_clone<T: Clone>(&self, len: usize, value: &T) -> &mut [T] {
        self.alloc_slice_fill_with(len, |_| value.clone())
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_fill_iter<T, I>(&self, iter: I) -> &mut [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut iter = iter.into_iter();
        self.alloc_slice_fill_with(iter.len(), |_| {
            iter.next().expect("Iterator supplied too few elements")
        })
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_fill_default<T: Default>(&self, len: usize) -> &mut [T] {
        self.alloc_slice_fill_with(len, |_| T::default())
    }

    #[inline(always)]
    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        if let Some(p) = self.try_alloc_layout_fast(layout) {
            p
        } else {
            self.alloc_layout_slow(layout)
        }
    }

    #[inline(always)]
    fn try_alloc_layout_fast(&self, layout: Layout) -> Option<NonNull<u8>> {
        unsafe {
            if layout.size() == 0 {
                let ptr = layout.align() as *mut u8;
                return Some(NonNull::new_unchecked(ptr));
            }

            let footer = self.current_chunk_footer.get();
            let footer = footer.as_ref();
            let ptr = footer.ptr.get().as_ptr() as usize;
            let start = footer.data.as_ptr() as usize;
            debug_assert!(start <= ptr);
            debug_assert!(ptr <= footer as *const _ as usize);

            let ptr = ptr.checked_sub(layout.size())?;
            let aligned_ptr = ptr & !(layout.align() - 1);

            if aligned_ptr >= start {
                let aligned_ptr = NonNull::new_unchecked(aligned_ptr as *mut u8);
                footer.ptr.set(aligned_ptr);
                Some(aligned_ptr)
            } else {
                None
            }
        }
    }

    // Slow path allocation for when we need to allocate a new chunk from the
    // parent bump set because there isn't enough room in our current chunk.
    #[inline(never)]
    fn alloc_layout_slow(&self, layout: Layout) -> NonNull<u8> {
        unsafe {
            let size = layout.size();

            // Get a new chunk from the global allocator.
            let current_footer = self.current_chunk_footer.get();
            let current_layout = current_footer.as_ref().layout;
            let new_footer = Bump::new_chunk(
                Some(current_layout.size()),
                Some(layout),
                Some(current_footer),
            );
            debug_assert_eq!(
                new_footer.as_ref().data.as_ptr() as usize % layout.align(),
                0
            );

            // Set the new chunk as our new current chunk.
            self.current_chunk_footer.set(new_footer);

            let new_footer = new_footer.as_ref();

            // Move the bump ptr finger down to allocate room for `val`. We know
            // this can't overflow because we successfully allocated a chunk of
            // at least the requested size.
            let ptr = new_footer.ptr.get().as_ptr() as usize - size;
            // Round the pointer down to the requested alignment.
            let ptr = ptr & !(layout.align() - 1);
            debug_assert!(
                ptr <= new_footer as *const _ as usize,
                "{:#x} <= {:#x}",
                ptr,
                new_footer as *const _ as usize
            );
            let ptr = NonNull::new_unchecked(ptr as *mut u8);
            new_footer.ptr.set(ptr);

            // Return a pointer to the freshly allocated region in this chunk.
            ptr
        }
    }

    pub fn iter_allocated_chunks(&mut self) -> ChunkIter<'_> {
        ChunkIter {
            footer: Some(self.current_chunk_footer.get()),
            bump: PhantomData,
        }
    }

    pub fn allocated_bytes(&self) -> usize {
        let mut footer = Some(self.current_chunk_footer.get());

        let mut bytes = 0;

        while let Some(f) = footer {
            let foot = unsafe { f.as_ref() };

            let ptr = foot.ptr.get().as_ptr() as usize;
            debug_assert!(ptr <= foot as *const _ as usize);

            bytes += foot as *const _ as usize - ptr;

            footer = foot.prev.get();
        }

        bytes
    }

    #[inline]
    unsafe fn is_last_allocation(&self, ptr: NonNull<u8>) -> bool {
        let footer = self.current_chunk_footer.get();
        let footer = footer.as_ref();
        footer.ptr.get() == ptr
    }
}

#[derive(Debug)]
pub struct ChunkIter<'a> {
    footer: Option<NonNull<ChunkFooter>>,
    bump: PhantomData<&'a mut Bump>,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = &'a [mem::MaybeUninit<u8>];
    fn next(&mut self) -> Option<&'a [mem::MaybeUninit<u8>]> {
        unsafe {
            let foot = self.footer?;
            let foot = foot.as_ref();
            let data = foot.data.as_ptr() as usize;
            let ptr = foot.ptr.get().as_ptr() as usize;
            debug_assert!(data <= ptr);
            debug_assert!(ptr <= foot as *const _ as usize);

            let len = foot as *const _ as usize - ptr;
            let slice = slice::from_raw_parts(ptr as *const mem::MaybeUninit<u8>, len);
            self.footer = foot.prev.get();
            Some(slice)
        }
    }
}

impl<'a> iter::FusedIterator for ChunkIter<'a> {}

#[inline(never)]
#[cold]
fn oom() -> ! {
    panic!("out of memory")
}

unsafe impl<'a> alloc::Alloc for &'a Bump {
    #[inline(always)]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, alloc::AllocErr> {
        Ok(self.alloc_layout(layout))
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        // If the pointer is the last allocation we made, we can reuse the bytes,
        // otherwise they are simply leaked -- at least until somebody calls reset().
        if layout.size() != 0 && self.is_last_allocation(ptr) {
            let ptr = NonNull::new_unchecked(ptr.as_ptr().add(layout.size()));
            self.current_chunk_footer.get().as_ref().ptr.set(ptr);
        }
    }

    #[inline]
    unsafe fn realloc(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<u8>, alloc::AllocErr> {
        let old_size = layout.size();

        if old_size == 0 {
            return self.alloc(layout);
        }

        if new_size <= old_size {
            if self.is_last_allocation(ptr)
                 // Only reclaim the excess space (which requires a copy) if it
                 // is worth it: we are actually going to recover "enough" space
                 // and we can do a non-overlapping copy.
                 && new_size <= old_size / 2
            {
                let delta = old_size - new_size;
                let footer = self.current_chunk_footer.get();
                let footer = footer.as_ref();
                footer
                    .ptr
                    .set(NonNull::new_unchecked(footer.ptr.get().as_ptr().add(delta)));
                let new_ptr = footer.ptr.get();
                // NB: we know it is non-overlapping because of the size check
                // in the `if` condition.
                ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), new_size);
                return Ok(new_ptr);
            } else {
                return Ok(ptr);
            }
        }

        if self.is_last_allocation(ptr) {
            // Try to allocate the delta size within this same block so we can
            // reuse the currently allocated space.
            let delta = new_size - old_size;
            if let Some(p) =
                self.try_alloc_layout_fast(layout_from_size_align(delta, layout.align()))
            {
                ptr::copy(ptr.as_ptr(), p.as_ptr(), new_size);
                return Ok(p);
            }
        }

        // Fallback: do a fresh allocation and copy the existing data into it.
        let new_layout = layout_from_size_align(new_size, layout.align());
        let new_ptr = self.alloc_layout(new_layout);
        ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), old_size);
        Ok(new_ptr)
    }
}
