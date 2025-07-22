//@compile-flags: -Zinline-mir-threshold=200
//@compile-flags: -Zinline-mir-forwarder-threshold=200
//@compile-flags: -Zinline-mir-hint-threshold=200

use std::cell::Cell;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ptr::NonNull;
use std::slice;

pub trait BitStore: Copy {
    /// The width, in bits, of this type.
    const BITS: u8 = size_of::<Self>() as u8 * 8;

    /// The number of bits required to index a bit inside the type. This is
    /// always log<sub>2</sub> of the type’s bit width.
    const INDX: u8 = Self::BITS.trailing_zeros() as u8;

    /// The bitmask to turn an arbitrary number into a bit index. Bit indices
    /// are always stored in the lowest bits of an index value.
    const MASK: u8 = Self::BITS - 1;

    /// The value with all bits unset.
    const FALSE: Self;

    /// The value with all bits set.
    const TRUE: Self;

    /// Name of the implementing type. This is only necessary until the compiler
    /// stabilizes `type_name()`.
    const TYPENAME: &'static str;

    type Access;
}

/// Batch implementation of `BitStore` for the appropriate fundamental integers.
macro_rules! bitstore {
	($($t:ty => $bits:literal , $atom:ty ;)*) => { $(
		impl BitStore for $t {
			const TYPENAME: &'static str = stringify!($t);

			const FALSE: Self = 0;
			const TRUE: Self = !0;

			#[cfg(feature = "atomic")]
			type Access = $atom;

			#[cfg(not(feature = "atomic"))]
			type Access = Cell<Self>;

			// #[inline(always)]
			// fn count_ones(self) -> usize {
			// 	Self::count_ones(self) as usize
			// }
		}
	)* };
}

bitstore! {
    u8 => 1, atomic::AtomicU8;
    u16 => 2, atomic::AtomicU16;
    u32 => 4, atomic::AtomicU32;
}

pub trait BitOrder {}

#[derive(Clone, Copy)]
#[doc(hidden)]
pub(crate) union Pointer<T>
where
    T: BitStore,
{
    /// A shareable pointer to some contended mutable data.
    a: *const <T as BitStore>::Access,
    /// A read pointer to some data.
    r: *const T,
    /// A write pointer to some data.
    w: *mut T,
    /// The pointer address as a bare integer.
    u: usize,
}

impl<T> From<&T> for Pointer<T>
where
    T: BitStore,
{
    fn from(r: &T) -> Self {
        Self { r }
    }
}

impl<T> From<*const T> for Pointer<T>
where
    T: BitStore,
{
    fn from(r: *const T) -> Self {
        Self { r }
    }
}

impl<T> From<&mut T> for Pointer<T>
where
    T: BitStore,
{
    fn from(w: &mut T) -> Self {
        Self { w }
    }
}

impl<T> From<*mut T> for Pointer<T>
where
    T: BitStore,
{
    fn from(w: *mut T) -> Self {
        Self { w }
    }
}

impl<T> From<usize> for Pointer<T>
where
    T: BitStore,
{
    fn from(u: usize) -> Self {
        Self { u }
    }
}

impl<T> Pointer<T>
where
    T: BitStore,
{
    pub(crate) fn w(self) -> *mut T {
        unsafe { self.w }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct BitPtr<T>
where
    T: BitStore,
{
    _ty: PhantomData<T>,
    ptr: NonNull<u8>,
    len: usize,
}

impl<T> BitPtr<T>
where
    T: BitStore,
{
    pub const PTR_DATA_MASK: usize = !Self::PTR_HEAD_MASK;
    pub const PTR_HEAD_BITS: usize = T::INDX as usize - Self::LEN_HEAD_BITS;
    pub const PTR_HEAD_MASK: usize = T::MASK as usize >> Self::LEN_HEAD_BITS;
    pub const LEN_HEAD_BITS: usize = 3;
    pub const LEN_HEAD_MASK: usize = 0b0111;
    pub const MAX_ELTS: usize = (Self::MAX_BITS >> 3) + 1;
    pub const MAX_BITS: usize = !0 >> Self::LEN_HEAD_BITS;

    pub(crate) fn pointer(&self) -> Pointer<T> {
        (self.ptr.as_ptr() as usize & Self::PTR_DATA_MASK).into()
    }

    #[inline]
    pub fn as_slice<'a>(&self) -> &'a [T] {
        //~^ ERROR: it usually isn't necessary to apply #[inline] to generic functions
        //~| HELP: See https://matklad.github.io/2021/07/09/inline-in-rust.html and https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
        //~| NOTE: generic functions are always `#[inline]` (monomorphization)
        //~| NOTE: `-D rpl::generic-function-marked-inline` implied by `-D warnings`
        //~| HELP: to override `-D warnings` add `#[allow(rpl::generic_function_marked_inline)]`
        unsafe { slice::from_raw_parts(self.pointer().r, self.elements()) }
    }

    /// Accesses the element slice behind the pointer as a Rust mutable slice.
    ///
    /// # Parameters
    ///
    /// - `&self`
    ///
    /// # Returns
    ///
    /// Standard Rust slice handle over the data governed by this pointer.
    ///
    /// # Lifetimes
    ///
    /// - `'a`: Lifetime for which the data behind the pointer is live.
    #[inline]
    pub fn as_mut_slice<'a>(&self) -> &'a mut [T] {
        //~^ ERROR: it usually isn't necessary to apply #[inline] to generic functions
        //~| HELP: See https://matklad.github.io/2021/07/09/inline-in-rust.html and https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
        //~| NOTE: generic functions are always `#[inline]` (monomorphization)
        //~| ERROR: mutable borrow from immutable input(s)
        //~| NOTE: immutable borrow here
        //~| NOTE: `-D rpl::mut-from-ref` implied by `-D warnings`
        //~| HELP: to override `-D warnings` add `#[allow(rpl::mut_from_ref)]`
        unsafe { slice::from_raw_parts_mut(self.pointer().w, self.elements()) }
    }

    pub fn elements(&self) -> usize {
        self.head().span(self.len()).0
    }

    #[inline]
    pub fn head(&self) -> BitIdx<T> {
        //~^ ERROR: it usually isn't necessary to apply #[inline] to generic functions
        //~| HELP: See https://matklad.github.io/2021/07/09/inline-in-rust.html and https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
        //~| NOTE: generic functions are always `#[inline]` (monomorphization)
        let ptr = self.ptr.as_ptr() as usize;
        let ptr_head = (ptr & Self::PTR_HEAD_MASK) << Self::LEN_HEAD_BITS;
        let len_head = self.len & Self::LEN_HEAD_MASK;
        let idx = (ptr_head | len_head) as u8;
        BitIdx {
            idx,
            _ty: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        //~^ ERROR: it usually isn't necessary to apply #[inline] to generic functions
        //~| HELP: See https://matklad.github.io/2021/07/09/inline-in-rust.html and https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
        //~| NOTE: generic functions are always `#[inline]` (monomorphization)
        self.len >> Self::LEN_HEAD_BITS
    }

    pub(crate) fn from_bitslice<O>(bs: &BitSlice<O, T>) -> Self
    where
        O: BitOrder,
    {
        let src = unsafe { &*(bs as *const BitSlice<O, T> as *const [()]) };
        let ptr = Pointer::from(src.as_ptr() as *const u8);
        let (ptr, len) = match (ptr.w(), src.len()) {
            (_, 0) => (NonNull::dangling(), 0),
            (p, _) if p.is_null() => unreachable!("Rust forbids null refs"),
            (p, l) => (unsafe { NonNull::new_unchecked(p) }, l),
        };
        Self {
            ptr,
            len,
            _ty: PhantomData,
        }
    }

    /// Cast a `*mut BitSlice<O, T>` raw pointer into an equivalent `BitPtr<T>`.
    pub(crate) fn from_mut_ptr<O>(ptr: *mut BitSlice<O, T>) -> Self
    where
        O: BitOrder,
    {
        unsafe { &*ptr }.bitptr()
    }

    pub(crate) fn into_bitslice_mut<'a, O>(self) -> &'a mut BitSlice<O, T>
    where
        O: BitOrder,
    {
        unsafe {
            &mut *(slice::from_raw_parts_mut(
                Pointer::from(self.ptr.as_ptr()).w() as *mut (),
                self.len,
            ) as *mut [()] as *mut BitSlice<O, T>)
        }
    }

    pub(crate) fn as_mut_ptr<O>(self) -> *mut BitSlice<O, T>
    where
        O: BitOrder,
    {
        self.into_bitslice_mut() as *mut BitSlice<O, T>
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct BitTail<T>
where
    T: BitStore,
{
    /// Semantic index *after* an element. Constrained to `0 ..= T::BITS`.
    end: u8,
    /// Marker for the tailed type.
    _ty: PhantomData<T>,
}

impl<T> BitTail<T>
where
    T: BitStore,
{
    /// Mark that `end` is a tail index for a type.
    ///
    /// # Parameters
    ///
    /// - `end` must be in the range `0 ..= T::BITS`.
    pub(crate) unsafe fn new_unchecked(end: u8) -> Self {
        debug_assert!(
            end <= T::BITS,
            "Bit tail {} cannot surpass type width {}",
            end,
            T::BITS,
        );
        Self {
            end,
            _ty: PhantomData,
        }
    }
    pub(crate) fn span(self, len: usize) -> (usize, Self) {
        let val = *self;
        debug_assert!(
            val <= T::BITS,
            "Tail out of range: {} overflows type width {}",
            val,
            T::BITS,
        );

        if len == 0 {
            return (0, self);
        }

        let head = val & T::MASK;

        let bits_in_head = (T::BITS - head) as usize;

        if len <= bits_in_head {
            let val = head + len as u8;
            return (0, unsafe { Self::new_unchecked(val) });
        }

        let bits_after_head = len - bits_in_head;

        let elts = bits_after_head >> T::INDX;
        let tail = bits_after_head as u8 & T::MASK;

        let is_zero = (tail == 0) as u8;
        let edges = 2 - is_zero as usize;
        let val = ((is_zero << T::INDX) | tail) as u8;
        return (elts + edges, unsafe { Self::new_unchecked(val) });

        /* The above expression is the branchless equivalent of this structure:

        if tail == 0 {
            (elts + 1, T::BITS.tail())
        }
        else {
            (elts + 2, tail.tail())
        }
        */
    }
}

impl<T: BitStore> Deref for BitTail<T> {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.end
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BitIdx<T>
where
    T: BitStore,
{
    /// Semantic index within an element. Constrained to `0 .. T::BITS`.
    idx: u8,
    /// Marker for the indexed type.
    _ty: PhantomData<T>,
}

impl<T: BitStore> Deref for BitIdx<T> {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.idx
    }
}

impl<T> BitIdx<T>
where
    T: BitStore,
{
    pub(crate) fn span(self, len: usize) -> (usize, BitTail<T>) {
        unsafe { BitTail::new_unchecked(*self) }.span(len)
    }
}

pub struct BitBox<O, T>
where
    O: BitOrder,
    T: BitStore,
{
    _order: PhantomData<O>,
    pointer: BitPtr<T>,
}

impl<O, T> Drop for BitBox<O, T>
where
    O: BitOrder,
    T: BitStore,
{
    fn drop(&mut self) {
        let ptr = self.as_mut_slice().as_mut_ptr();
        let len = self.as_slice().len();
        //  Run the `Box<[T]>` destructor.
        drop(unsafe { Vec::from_raw_parts(ptr, 0, len) }.into_boxed_slice());
    }
}

impl<O, T> BitBox<O, T>
where
    O: BitOrder,
    T: BitStore,
{
    pub unsafe fn from_raw(raw: *mut BitSlice<O, T>) -> Self {
        Self {
            _order: PhantomData,
            pointer: BitPtr::from_mut_ptr(raw),
        }
    }

    pub fn as_slice(&self) -> &[T] {
        self.bitptr().as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.bitptr().as_mut_slice()
    }

    /// Gives read access to the `BitPtr<T>` structure powering the box.
    ///
    /// # Parameters
    ///
    /// - `&self`
    ///
    /// # Returns
    ///
    /// A copy of the interior `BitPtr<T>`.
    pub(crate) fn bitptr(&self) -> BitPtr<T> {
        self.pointer
    }
}

#[repr(transparent)]
pub struct BitSlice<O, T>
where
    O: BitOrder,
    T: BitStore,
{
    /// BitOrder type for selecting bits inside an element.
    _kind: PhantomData<O>,
    /// Element type of the slice.
    ///
    /// eddyb recommends using `PhantomData<T>` and `[()]` instead of `[T]`
    /// alone.
    _type: PhantomData<T>,
    /// Slice of elements `T` over which the `BitSlice` has usage.
    _elts: [()],
}

impl<O, T> BitSlice<O, T>
where
    O: BitOrder,
    T: BitStore,
{
    pub(crate) fn bitptr(&self) -> BitPtr<T> {
        BitPtr::from_bitslice(self)
    }
}

#[repr(C)]
pub struct BitVec<O, T>
where
    O: BitOrder,
    T: BitStore,
{
    /// Phantom `BitOrder` member to satisfy the constraint checker.
    _order: PhantomData<O>,
    /// Slice pointer over the owned memory.
    pointer: BitPtr<T>,
    /// The number of *elements* this vector has allocated.
    capacity: usize,
}

impl<O, T> BitVec<O, T>
where
    O: BitOrder,
    T: BitStore,
{
    #[inline]
    pub unsafe fn from_raw_parts(pointer: BitPtr<T>, capacity: usize) -> Self {
        //~^ ERROR: it usually isn't necessary to apply #[inline] to generic functions
        //~| HELP: See https://matklad.github.io/2021/07/09/inline-in-rust.html and https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
        //~| NOTE: generic functions are always `#[inline]` (monomorphization)
        Self {
            _order: PhantomData,
            pointer,
            capacity,
        }
    }

    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    #[inline(always)]
    pub fn into_vec(self) -> Vec<T> {
        //~^ ERROR: it usually isn't necessary to apply #[inline] to generic functions
        //~| HELP: See https://matklad.github.io/2021/07/09/inline-in-rust.html and https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
        //~| NOTE: generic functions are always `#[inline]` (monomorphization)
        let slice = self.pointer.as_mut_slice();
        let out = unsafe { Vec::from_raw_parts(slice.as_mut_ptr(), slice.len(), self.capacity) };
        mem::forget(self);
        out
    }

    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    pub fn into_boxed_bitslice(self) -> BitBox<O, T> {
        let pointer = self.pointer;
        //  Convert the Vec allocation into a Box<[T]> allocation
        mem::forget(self.into_boxed_slice());
        //~^NOTE: the `std::vec::Vec<T>` value may be moved here
        unsafe { BitBox::from_raw(pointer.as_mut_ptr()) }
        //~^ERROR: use a pointer from `std::vec::Vec<T>` after it's moved
        //~|NOTE: `#[deny(rpl::use_after_move)]` on by default
    }

    #[inline]
    pub fn into_boxed_slice(self) -> Box<[T]> {
        //~^ ERROR: it usually isn't necessary to apply #[inline] to generic functions
        //~| HELP: See https://matklad.github.io/2021/07/09/inline-in-rust.html and https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
        //~| NOTE: generic functions are always `#[inline]` (monomorphization)
        self.into_vec().into_boxed_slice()
    }
}

fn main() {}
