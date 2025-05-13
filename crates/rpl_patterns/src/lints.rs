use rustc_lint_defs::declare_tool_lint;

declare_tool_lint! {
    /// The `rpl::lengthless_buffer_passed_to_extern_function` lint detects a buffer
    /// pointer passed to an extern function without specifying its length.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use libc::c_char;
    /// extern fn gets(c: *const c_char) -> i32 {
    ///     0
    /// }
    ///
    /// fn pass_buffer_pointer_without_length() {
    ///     let mut p = [8u8; 64];
    ///     unsafe {
    ///         gets(&p as *const u8 as *const c_char);
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// When you pass a lengthless buffer to an extern function, the most probable
    /// situation is that you are using some old style C API which fills the buffer
    /// as much as it has, and it's never safe to use.
    ///
    /// However, in some cases, the size of the buffer may be fixed, and this lint
    /// can be suppressed then.
    pub rpl::LENGTHLESS_BUFFER_PASSED_TO_EXTERN_FUNCTION,
    Warn,
    "detects a lengthless buffer passed to extern function"
}

declare_tool_lint! {
    /// The `rpl::rust_string_pointer_as_c_string_pointer` lint detects a Rust string pointer
    /// used as a C string pointer directly, for example, using `as` or `std::mem::transmute`
    ///
    /// ### Example
    ///
    /// ```rust
    /// use libc::c_char;
    /// extern fn gets(c: *const c_char) -> i32 {
    ///     0
    /// }
    ///
    /// fn cast_rust_string_pointer_to_c_string_pointer() {
    ///     let mut p = String::from("hello");
    ///     let p = p.as_bytes().as_ptr();
    ///     unsafe {
    ///         gets(p as *const c_char);
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// C strings normally end with a `\0`, while Rust strings do not. And
    /// Rust strings must contain valid UTF-8.
    pub rpl::RUST_STRING_POINTER_AS_C_STRING_POINTER,
    Deny,
    "detects a Rust string pointer used as a C string pointer directly"
}

declare_tool_lint! {
    /// The `rpl::unchecked_pointer_offset` lint detects a pointer that is offset using an unchecked integer.
    /// This is a common source of undefined behavior.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn index(p: *const u8, index: usize) -> *const u8 {
    ///     unsafe {
    ///         p.add(index)
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The `add` method is used to offset a pointer by a given number of elements.
    /// However, if the index is not checked, it can lead to undefined behavior.
    /// To avoid this, you should always check the index before offsetting the pointer,
    /// unless both the pointer and the index are guaranteed to be valid, for example,
    /// when the index is calculated from the length of the slice, or both are constants.
    pub rpl::UNCHECKED_POINTER_OFFSET,
    Warn,
    "detects a pointer that is offset using an unchecked integer"
}

declare_tool_lint! {
    /// The `rpl::cassandra_iter_next_ptr_passed_to_cass_iter_get` lint detects a pointer returned by
    /// `cassandra_iterator_next` that is utilized in `cass_iterator_get_*`.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// extern crate cassandra_cpp_sys;
    /// use cassandra_cpp_sys::CassIterator as _CassIterator;
    /// use cassandra_cpp_sys::{
    ///     cass_false, cass_iterator_get_aggregate_meta, cass_iterator_next, cass_true,
    ///     CassAggregateMeta as _CassAggregateMeta,
    /// };
    /// use std::iter::Iterator;
    /// pub struct AggregateMeta(*const _CassAggregateMeta);
    /// impl AggregateMeta {
    ///     fn build(inner: *const _CassAggregateMeta) -> Self {
    ///         if inner.is_null() {
    ///             panic!("Unexpected null pointer")
    ///         };
    ///         AggregateMeta(inner)
    ///     }
    /// }
    /// #[derive(Debug)]
    /// pub struct AggregateIterator(*mut _CassIterator);
    /// impl Iterator for AggregateIterator {
    ///     type Item = AggregateMeta;
    ///     #[deny(rpl::cassandra_iter_next_ptr_passed_to_cass_iter_get)]
    ///     fn next(&mut self) -> Option<<Self as Iterator>::Item> {
    ///         unsafe {
    ///             match cass_iterator_next(self.0) {
    ///                 cass_false => None,
    ///                 cass_true => {
    ///                     let field_value = cass_iterator_get_aggregate_meta(self.0);
    ///                     Some(AggregateMeta::build(field_value))
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Code that attempts to **use an item returned by an iterator
    /// after the iterator has advanced to the next item** will be accessing freed memory,
    /// which caused by the underlying Cassandra driver which invalidates the current item when `next()` is called,
    /// leading to a **use-after-free** vulnerability.
    pub rpl::CASSANDRA_ITER_NEXT_PTR_PASSED_TO_CASS_ITER_GET,
    Deny,
    "detects a pointer returned by `cassandra_iterator_next` that is utilized in `cass_iterator_get_*`"
}

declare_tool_lint! {
    /// The `rpl::set_len_to_extend` lint detects using `Vec::set_len` to extend the length of a `Vec`
    /// without initializing the new elements.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn set_len_to_extend() {
    ///     let mut v = vec![1, 2, 3];
    ///     unsafe {
    ///         v.set_len(5);
    ///     }
    ///     v[4]; // undefined behavior
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The `set_len` method is used to change the length of a `Vec` without changing its capacity.
    /// However, it does not initialize the new elements, which can lead to undefined behavior,
    /// even if the `Vec` is not used after the `set_len` call,
    /// as the `Vec` may be dropped, and the destructor may access the uninitialized memory.
    /// To avoid this, you should always use the `resize` method to extend the length of a `Vec`.
    pub rpl::SET_LEN_TO_EXTEND,
    Deny,
    "detects using `Vec::set_len` to extend the length of a `Vec` without initializing the new elements"
}

declare_tool_lint! {
    /// The `rpl::set_len_to_truncate` lint detects using `Vec::set_len` to truncate the length of a `Vec`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn set_len_to_truncate() {
    ///     let mut v = vec![Box::new(1), Box::new(2), Box::new(3)];
    ///     unsafe {
    ///         v.set_len(2); // memory leak
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The `set_len` method is used to change the length of a `Vec` without changing its capacity.
    /// However, it does not drop the elements that are removed, which can lead to memory leaks.
    /// To avoid this, you should always use the `truncate` method to truncate the length of a `Vec`.
    pub rpl::SET_LEN_TO_TRUNCATE,
    Warn,
    "detects using `Vec::set_len` to truncate the length of a `Vec` without dropping the elements"
}

declare_tool_lint! {
    /// The `rpl::set_len_to_uninitialized` lint detects using `Vec::set_len` to truncate the length of a `Vec`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn set_len_to_uninitialized() {
    ///     let mut v: Vec<i32> = Vec::with_capacity(3);
    ///     unsafe {
    ///         v.set_len(3);
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The `set_len` method is used to change the length of a `Vec` without changing its capacity.
    /// However, it does not initialize the new elements, which can lead to undefined behavior.
    /// To avoid this, you should always use the `resize` method to initialize the new elements.
    pub rpl::SET_LEN_UNINITIALIZED,
    Deny,
    "detects calling `Vec::set_len` without initializing the new elements in advance"
}

declare_tool_lint! {
    /// The `rpl::unsound_slice_cast` lint detects a slice cast that can lead to undefined behavior.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use core::{mem::size_of, slice::from_raw_parts};
    ///
    /// fn unsound_slice_cast() {
    ///     let v: Vec<usize> = vec![1, 2, 3];
    ///     let slice: &[usize] = v.as_slice();
    ///     let slice: &[u8] = unsafe { from_raw_parts(
    ///         slice.as_ptr() as *const u8,
    ///         slice.len() * size_of::<usize>()
    ///     ) };
    ///     // undefined behavior
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// A reference to a slice must has suitable alignment and size for the type it points to.
    pub rpl::UNSOUND_SLICE_CAST,
    Deny,
    "detects a slice cast that can lead to undefined behavior"
}

declare_tool_lint! {
    /// The `rpl::use_after_drop` lint detects using a value after it has been dropped.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn use_after_drop() {
    ///     let x: Box<i32> = Box::new(42);
    ///     let y: *const i32 = Box::as_ref(&x) as *const i32;
    ///     drop(x);
    ///     unsafe {
    ///         *y; // undefined behavior
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Using a value after it has been dropped is undefined behavior.
    pub rpl::USE_AFTER_DROP,
    Deny,
    "detects using a value after it has been dropped"
}

declare_tool_lint! {
    /// The `rpl::offset_by_one` lint detects reading or writing a value at an offset outside the bounds of a buffer by one.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn out_of_bounds() {
    ///     let mut v = vec![1, 2, 3];
    ///     let p = v.as_mut_ptr();
    ///     unsafe {
    ///         *p.offset(3) = 4; // undefined behavior
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Reading or writing a value at an offset outside the bounds of a buffer by one is undefined behavior.
    pub rpl::OFFSET_BY_ONE,
    Deny,
    "detects reading or writing a value at an offset outside the bounds of a buffer by one"
}

declare_tool_lint! {
    /// The `rpl::misordered_parameters` lint detects misordered parameters in a function call.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use std::mem::forget;
    ///
    /// fn from_raw_parts_misordered() {
    ///     let mut v = vec![1, 2, 3];
    ///     let ptr = v.as_mut_ptr();
    ///     let len = v.len();
    ///     let cap = v.capacity();
    ///     forget(v);
    ///     let v = unsafe {
    ///         Vec::from_raw_parts(ptr, cap, len) // misordered parameters
    ///     };
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Misordered parameters in a unsafe function call can lead to undefined behavior.
    pub rpl::MISORDERED_PARAMETERS,
    Deny,
    "detects misordered parameters in a function call"
}

declare_tool_lint! {
    /// The `rpl::wrong_assumption_of_fat_pointer_layout` lint detects casting a fat pointer
    /// to a thin pointer using `as` or `std::mem::transmute`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn cast_fat_pointer() {
    ///     let p = &mut [1, 2, 3] as *mut [i32];
    ///     let p = p as *mut i32; // undefined behavior
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// It's not documented that the data pointer part of a fat pointer is always at the beginning of the fat pointer.
    pub rpl::WRONG_ASSUMPTION_OF_FAT_POINTER_LAYOUT,
    Deny,
    "detects casting a fat pointer to a thin pointer using `as` or `std::mem::transmute`"
}

declare_tool_lint! {
    /// The `rpl::wrong_assumption_of_layout_compatibility` lint detects a wrong assumption of layout compatibility.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use core::net::{SocketAddrV4, Ipv4Addr};
    /// use libc::sockaddr;
    ///
    /// fn cast_to_sockaddr() {
    ///     let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    ///     let p: *const sockaddr = &socket as *const SocketAddrV4 as *const sockaddr;
    ///     // p may not be a valid sockaddr pointer
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The layout of `SocketAddrV4` and `sockaddr` may not be compatible.
    /// See <https://github.com/rust-lang/rust/pull/78802> for more information.
    pub rpl::WRONG_ASSUMPTION_OF_LAYOUT_COMPATIBILITY,
    Deny,
    "detects casting a fat pointer to a thin pointer using `as` or `std::mem::transmute`"
}

declare_tool_lint! {
    /// The `rpl::trust_exact_size_iterator` lint detects some codes, whose safety depends on the correctness of
    /// the implementation of [`core::iter::ExactSizeIterator`].
    ///
    /// ### Example
    ///
    /// ```rust
    /// use core::iter::ExactSizeIterator;
    /// fn foo<T, I: Iterator<Item = T> + ExactSizeIterator>(iter: I) {
    ///     let len = iter.len();
    ///     let mut v: Vec<T> = Vec::with_capacity(len);
    ///     let p = v.as_mut_ptr();
    ///     for x in iter {
    ///         unsafe {
    ///             p.write(x);
    ///         }
    ///     }
    ///     unsafe {
    ///         v.set_len(len);
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The safety of the code depends on the correctness of the implementation of `ExactSizeIterator`.
    pub rpl::TRUST_EXACT_SIZE_ITERATOR,
    Deny,
    "detects some codes, whose safety depends the correctness of the implementation of `core::iter::ExactSizeIterator`"
}

declare_tool_lint! {
    /// The `rpl::slice_from_raw_parts_uninitialized` lint detects calling `std::slice::from_raw_parts` or
    /// `std::slice::from_raw_parts_mut` with uninitialized memory.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn slice_from_raw_parts_uninitialized() {
    ///     let mut v: Vec<i32> = Vec::with_capacity(3);
    ///     let p = v.as_ptr();
    ///     let cap = v.capacity();
    ///     let slice = unsafe {
    ///         std::slice::from_raw_parts(p, cap)
    ///     };
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The `from_raw_parts` and `from_raw_parts_mut` functions requires that the memory is properly initialized.
    pub rpl::SLICE_FROM_RAW_PARTS_UNINITIALIZED,
    Deny,
    "detects calling `std::slice::from_raw_parts` or `std::slice::from_raw_parts_mut` with uninitialized memory"
}

declare_tool_lint! {
    /// The `rpl::get_mut_in_rc_unsafecell` lint detects calling [`std::cell::UnsafeCell::get_mut`] on an [`Rc<UnsafeCell<T>>`].
    ///
    /// ### Example
    ///
    /// ```rust
    /// use std::cell::UnsafeCell;
    /// use std::rc::Rc;
    ///
    /// fn aliasing_mut() {
    ///     let rc = Rc::new(UnsafeCell::new(42));
    ///
    ///     let p1: &mut i32 = unsafe { &mut *rc.as_ref().get() };
    ///     let p2: &mut i32 = unsafe { &mut *rc.as_ref().get() };
    ///
    ///     // p1 and p2 may point to the same memory
    ///     println!("{:p} {:p}", p1, p2);
    ///     assert_eq!(p1, p2);
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The `get_mut` method is used to get a mutable reference to the value in the `UnsafeCell`.
    pub rpl::GET_MUT_IN_RC_UNSAFECELL,
    Deny,
    "detects calling `std::cell::UnsafeCell::get_mut` on an `Rc<UnsafeCell<T>>`"
}

declare_tool_lint! {
    /// The `rpl::drop_uninit_value` lint detects dropping an uninitialized value.
    ///
    /// ### Example
    ///
    /// ```rust
    /// #![feature(maybe_uninit_array_assume_init)]
    /// use std::mem::MaybeUninit;
    ///
    /// fn write_many<T: Clone>(value: T) -> [T; 3] {
    ///   let mut x = [const { MaybeUninit::uninit() }; 3];
    ///   for i in 0..3 {
    ///     unsafe {
    ///       *x[i].as_mut_ptr() = value.clone(); // May drop uninitialized value pointed by y.add(i)
    ///     }
    ///   }
    ///   unsafe { MaybeUninit::array_assume_init(x) }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Dropping an uninitialized value is undefined behavior.
    pub rpl::DROP_UNINIT_VALUE,
    Deny,
    "detects dropping an uninitialized value"
}

declare_tool_lint! {
    /// The `rpl::thread_local_static_ref` lint detects casting a reference to a thread-local static variable (which implements `Sync`) to a static reference.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use std::cell::UnsafeCell;
    ///
    /// thread_local! {
    ///   static THREAD_LOCAL: UnsafeCell<i32> = UnsafeCell::new(0);
    /// }
    ///
    /// pub fn static_ref() -> &'static i32 {
    ///   THREAD_LOCAL.with(|l| unsafe { &*l.get() })
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// It is unsound to expose a `&'static T` from a thread-local where `T` is `Sync`.
    pub rpl::THREAD_LOCAL_STATIC_REF,
    Deny,
    "detects casting a reference to a thread-local static variable (which implements `Sync`) to a static reference"
}

declare_tool_lint! {
    /// The `rpl::unvalidated_slice_from_raw_parts` lint detects calling `std::slice::from_raw_parts` or
    /// `std::slice::from_raw_parts_mut` with a pointer that is not a valid pointer to the slice.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn to_slice<'a, T>(p: *const T) -> &'a [T] {
    ///   unsafe {
    ///     std::slice::from_raw_parts(p, 1)
    ///   }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The `from_raw_parts` and `from_raw_parts_mut` functions require that the pointer is a valid pointer to the slice.
    pub rpl::UNVALIDATED_SLICE_FROM_RAW_PARTS,
    Deny,
    "detects calling `std::slice::from_raw_parts` or `std::slice::from_raw_parts_mut` with a pointer that is not a valid pointer to the slice"
}

declare_tool_lint! {
    /// The `rpl::unsound_cast_between_u64_and_atomic_u64` lint detects an unsound cast between [`u64`] and [`AtomicU64`].
    ///
    /// ### Example
    ///
    /// ```rust
    /// use std::sync::atomic::AtomicU64;
    ///
    /// fn cast_between_u64_and_atomic_u64() {
    ///     let x = &1u64;
    ///     let y = x as *const u64 as *const AtomicU64; // may cause undefined behavior if AtomicU64 has larger alignment
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The alignment of [`AtomicU64`] may be larger than [`u64`], which can cause undefined behavior.
    ///
    /// [`AtomicU64`]: std::sync::atomic::AtomicU64
    pub rpl::UNSOUND_CAST_BETWEEN_U64_AND_ATOMIC_U64,
    Deny,
    "detects an unsound cast between `u64` and `AtomicU64`"
}

declare_tool_lint! {
    /// The `rpl::deref_null_pointer` lint detects dereferencing a null pointer.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use std::ops::Deref;
    ///
    /// struct CBox<T: ?Sized> {
    ///   pub ptr: *mut T, // `ptr` may be assigned to as it's public
    /// }
    ///
    /// impl<T: ?Sized> CBox<T> {
    ///   #[inline]
    ///   /// Wrap the pointer in a `CBox`.
    ///   pub fn new(ptr: *mut T) -> Self {
    ///     CBox { ptr }
    ///   }
    ///   #[inline]
    ///   /// Returns the internal pointer.
    ///   pub unsafe fn as_ptr(&self) -> *mut T {
    ///     self.ptr
    ///   }
    /// }
    ///
    /// impl<T: ?Sized> Deref for CBox<T> {
    ///   type Target = T;
    ///
    ///   fn deref(&self) -> &T {
    ///     unsafe {
    ///       &*self.ptr // undefined behavior
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Dereferencing a null pointer is undefined behavior.
    pub rpl::DEREF_NULL_POINTER,
    Deny,
    "detects dereferencing a null pointer"
}

declare_tool_lint! {
    /// The `rpl::deref_unchecked_ptr_offset` lint detects dereferencing a pointer that is offset using an unchecked integer.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn index(p: *const u8, index: usize) -> *const u8 {
    ///   unsafe {
    ///     p.add(index)
    ///   }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Dereferencing a pointer that is offset using an unchecked integer is undefined behavior.
    pub rpl::DEREF_UNCHECKED_PTR_OFFSET,
    Deny,
    "detects dereferencing a pointer that is offset using an unchecked integer"
}

declare_tool_lint! {
    /// The `rpl::unsound_pin_project` lint detects unsound usage of `#[pin_project]`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use std::pin::Pin;
    /// use pin_project::pin_project;
    ///
    /// #[pin_project]
    /// pub struct Stream<S> {
    ///     #[pin]
    ///     stream: S,
    /// }
    ///
    /// impl<S> Stream<S> {
    ///     fn as_pin_mut(&mut self) -> Pin<&mut S> {
    ///         unsafe {
    ///             Pin::new_unchecked(self).project().stream
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// It is unsound to call `Pin::new_unchecked` on a mutable reference that can be freely moved.
    pub rpl::UNSOUND_PIN_PROJECT,
    Deny,
    "detects unsound usage of `#[pin_project]`"
}

declare_tool_lint! {
    /// The `rpl::use_after_move` lint detects using a value after it has been moved to another location.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn use_after_move() {
    ///     let mut v = vec![1, 2, 3];
    ///     v.reserve(10);
    ///     let p = v.as_mut_ptr();
    ///     let b = v.into_boxed_slice();
    ///     unsafe {
    ///         *p = 4; // undefined behavior
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Using a value after it has been moved to another location is undefined behavior.
    pub rpl::USE_AFTER_MOVE,
    Deny,
    "detects using a value after it has been moved to another location"
}

declare_tool_lint! {
    /// The `rpl::private_function_marked_inline` lint detects private functions that are marked with `#[inline]`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// #[inline]
    /// fn foo() {
    ///     println!("foo");
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// It is not necessary to apply `#[inline]` to private functions.
    pub rpl::PRIVATE_FUNCTION_MARKED_INLINE,
    Warn,
    "detects private functions that are marked with `#[inline]`"
}

declare_tool_lint! {
    /// The `rpl::generic_function_marked_inline` lint detects generic functions that are marked with `#[inline]`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// #[inline]
    /// pub fn foo<T>(x: T) -> T {
    ///     x
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// `#[inline]` has no effect because generic functions are always `#[inline]` (monomorphization).
    pub rpl::GENERIC_FUNCTION_MARKED_INLINE,
    Warn,
    "detects generic functions that are marked with `#[inline]`"
}
declare_tool_lint! {
    /// The `rpl::transmuting_type_to_bool` lint detects a transmute from a type to a boolean.
    ///
    /// ### Example
    ///
    /// ```rust
    ///
    /// use std::mem::transmute;
    ///
    /// fn invalid_value() -> bool {
    ///     let x: u8 = 10;
    ///     unsafe { transmute::<u8, bool>(x) }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Transmuting integers to booleans is highly unsound,
    /// probably producing a boolean value with an invalid state.
    pub rpl::TRANSMUTING_TYPE_TO_BOOL,
    Warn,
    "detects a transmute from a type to a boolean"
}

declare_tool_lint! {
    /// The `rpl::transmuting_int_to_ptr` lint detects a transmute from an integer type to a pointer type.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use std::mem::transmute;
    ///
    /// fn transmute_int_to_ptr(x: usize) {
    ///     let ptr: *const () = unsafe { transmute(x) };
    ///     let ptr_usize = ptr as *const usize;
    ///     println!("{}", unsafe { *ptr_usize });
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Transmuting integers to pointers is a largely unspecified operation.
    /// It is likely not equivalent to an as cast.
    pub rpl::TRANSMUTING_INT_TO_PTR,
    Warn,
    "detects a transmute from an integer type to a pointer type"
}

declare_tool_lint! {
    /// The `rpl::bad_manually_drop_operation_sequence` lint detects a sequence of operations
    /// that are not allowed on a `ManuallyDrop` type.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use std::mem::ManuallyDrop;
    ///
    /// fn double_drop<T>(x: T) {
    ///     let mut s = ManuallyDrop::new(x);
    ///     unsafe {
    ///         ManuallyDrop::drop(&mut s);
    ///         ManuallyDrop::drop(&mut s); // double drop
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The `ManuallyDrop` type is used to prevent the automatic dropping of a value.
    /// However, it is not allowed to drop a `ManuallyDrop` value twice,
    /// as it can lead to undefined behavior.
    ///
    /// The same applies to calling `ManuallyDrop::take` after dropping the value.
    pub rpl::BAD_MANUALLY_DROP_OPERATION_SEQUENCE,
    Deny,
    "detects a sequence of operations that are not allowed on a `ManuallyDrop` type"
}

declare_tool_lint! {
    /// The `rpl::unchecked_allocated_pointer` lint detects that a pointer allocated through [`std::alloc::alloc`] is not checked for null.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use std::alloc::{alloc, dealloc, Layout};
    ///
    /// fn alloc_unchecked() {
    ///     let layout = Layout::new::<u8>();
    ///     unsafe {
    ///         let ptr = alloc(layout) as *mut u8;
    ///         ptr.write(42);
    ///         dealloc(ptr as *mut u8, layout);
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The `alloc` function returns a pointer to the allocated memory,
    /// but it does not check if the allocation was successful.
    /// If the allocation fails, it returns a null pointer,
    /// which can lead to undefined behavior if dereferenced.
    /// To avoid this, you should always check the pointer for null before using it.
    pub rpl::UNCHECKED_ALLOCATED_POINTER,
    Warn,
    "detects that a pointer allocated through `std::alloc::alloc` is not checked for null"
}

declare_tool_lint! {
    /// The `rpl::misaligned_pointer` lint detects that a pointer allocated with an alignment that is not suitable for the type.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use std::{alloc::{alloc, dealloc, Layout}, mem::size_of};
    ///
    /// fn alloc_unchecked<T>() {
    ///     let layout = Layout::from_size_align(size_of::<T>(), 8).unwrap();
    ///     unsafe {
    ///         let ptr = alloc(layout) as *mut T;
    ///         dealloc(ptr as *mut u8, layout);
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The `alloc` function returns a pointer to the allocated memory,
    /// whose alignment is from the `layout`,
    /// and please check the alignment of the pointer before using it.
    pub rpl::MISALIGNED_POINTER,
    Warn,
    "detects that a pointer allocated with an alignment that is not suitable for the type"
}

declare_tool_lint! {
    /// The `rpl::use_after_realloc` lint detects using a pointer after it has been reallocated.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use std::alloc::{Layout, alloc, dealloc, realloc};
    ///
    /// fn use_after_realloc() {
    ///     let layout = Layout::array::<u8>(16).unwrap();
    ///     unsafe {
    ///         let ptr = alloc(layout) as *mut u8;
    ///         assert!(!ptr.is_null());
    ///         ptr.write(42);
    ///         let new_ptr = realloc(ptr as *mut u8, layout, layout.size() * 2) as *mut u8;
    ///         assert!(!new_ptr.is_null());
    ///         // use the old pointer after realloc
    ///         let x = *ptr;
    ///         assert_eq!(x, 42);
    ///         dealloc(ptr as *mut u8, layout);
    ///     }
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// The `realloc` function may move the memory to a new location,
    /// and the old pointer may no longer be valid.
    pub rpl::USE_AFTER_REALLOC,
    Deny,
    "detects using a pointer after it has been reallocated"
}
