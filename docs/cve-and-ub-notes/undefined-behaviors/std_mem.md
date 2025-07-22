# `std::mem`

## `std::mem::transmute`

### Signature

```rust
pub const unsafe fn transmute<Src, Dst>(_src: Src) -> Dst
```

Reinterprets the bits of a value of one type as another type. (Both types must have the same size. Compilation will fail if this is not guaranteed)

### UB1: Produce a value with an invalid state

Both the argument and the result must be [valid](https://doc.rust-lang.org/nomicon/what-unsafe-does.html) at their given type. Violating this condition leads to undefined behavior.

```rust
use std::mem::transmute;

// produce a value with an invalid state
fn invalid_value() -> bool {
    let x: u8 = 10;
    unsafe { transmute::<u8, bool>(x) }
}
```

### UB2: Transmutation between pointers and integers

**(Integer-To-Pointer)** Transmuting integers to pointers is a largely unspecified operation. It is likely not equivalent to an `as` cast. Doing non-zero-sized memory accesses with a pointer constructed this way is currently considered undefined behavior.

```rust
use std::mem::transmute;

pub fn transmute_int_to_ptr(x: usize) {
    let ptr: *const () = unsafe { transmute(x) };
    let ptr_usize = ptr as *const usize;
    println!("{}", unsafe { *ptr_usize });
}

fn main() {
    let x = 0x8;
    transmute_int_to_ptr(x);
}
```

**(Pointer-To-Integer)** Transmuting pointers to integers in a `const` context is undefined behavior, unless the pointer was originally created _from_ an integer. (That includes this function specifically, integer-to-pointer casts, and helpers like [`dangling`](https://doc.rust-lang.org/std/ptr/fn.dangling.html), but also semantically-equivalent conversions such as punning through `repr(C)` union fields.) Any attempt to use the resulting value for integer operations will abort const-evaluation.

## `std::mem::MaybeUninit<T>`

```rust
pub union MaybeUninit<T> {
    uninit: (),
    value: ManuallyDrop<T>,
}
```

A wrapper type to construct uninitialized instances of `T`.

### UB

```rust
pub const unsafe fn assume_init(self) -> T
```

Extracts the value from the `MaybeUninit<T>` container.

It is up to the caller to guarantee that the `MaybeUninit<T>` really is in an initialized state. Calling this when the content is not yet fully initialized causes immediate undefined behavior.

```Rust
fn main() {
    let b: bool = unsafe { MaybeUninit::uninit().assume_init() };
}
```

> We need to ensure that there is no write between the `MaybeUninit::uninit()` and `assume_init()`.
>
> This can be done with:
>
> (1)
>
> ```Rust
> let mut x = MaybeUninit::<&i32>::uninit();
> x.write(&0);
> let x = unsafe { x.assume_init() };
>
> ```
>
> (2)
>
> ```rust
> unsafe fn make_vec(out: *mut Vec<i32>) {
>     out.write(vec![1, 2, 3]);
> }
>
> let mut v = MaybeUninit::uninit();
> unsafe { make_vec(v.as_mut_ptr()); }
> let v = unsafe { v.assume_init() };
> ```

## `std::mem::uninitiallized`

### Signature

```rust
pub unsafe fn uninitialized<T>() -> T
```

Bypasses Rust’s normal memory-initialization checks by pretending to produce a value of type `T`, while doing nothing at all.

> Deprecated since 1.39.0: use `mem::MaybeUninit` instead.

### UB

The reason for deprecation is that the function basically cannot be used correctly: it has the same effect as `MaybeUninit::uninit().assume_init()`.

Therefore, it is immediate undefined behavior to call this function on nearly all types, including integer types and arrays of integer types, and even if the result is unused. (What type?)

```rust
use std::mem::uninitialized;

fn uninitialized_value<T>() -> T {
    unsafe {
        let x: T = uninitialized();
    }
}
```

## `std::mem::zeroed`

### Signature

```Rust
pub const unsafe fn zeroed<T>() -> T
```

Returns the value of type `T` represented by the all-zero byte-pattern.

### UB

The all-zero byte-pattern is not a valid value for reference types (`&T`, `&mut T`) and function pointers. Using `zeroed` on such types causes immediate undefined behavior because the Rust compiler assumes that there always is a valid value in a variable it considers initialized.

```rust
fn zeroed_inited_reference<T>() -> &'static T {
    let x: &'static T = unsafe { std::mem::zeroed() };
    x
}

fn zeroed_inited_reference_mut<T>() -> &'static mut T {
    let x: &'static mut T = unsafe { std::mem::zeroed() };
    x
}
```
