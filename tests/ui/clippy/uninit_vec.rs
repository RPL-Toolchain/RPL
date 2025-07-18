//@compile-flags: -Z inline-mir=false
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;

#[derive(Default)]
struct MyVec {
    vec: Vec<u8>,
}

union MyOwnMaybeUninit {
    value: u8,
    uninit: (),
}

// https://github.com/rust-lang/rust/issues/119620
unsafe fn requires_paramenv<S>() {
    unsafe {
        let mut vec = Vec::<UnsafeCell<*mut S>>::with_capacity(1);
        //~^ uninit_vec
        vec.set_len(1);
    }

    let mut vec = Vec::<UnsafeCell<*mut S>>::with_capacity(2);
    //~^ uninit_vec
    unsafe {
        vec.set_len(2);
    }
}
// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn main() {
    // with_capacity() -> set_len() should be detected
    let mut vec: Vec<u8> = Vec::with_capacity(1000);
    //~^ uninit_vec
    //~| uninit_vec

    unsafe {
        vec.set_len(200);
        //~^ set_len_uninitialized
        //~| uninit_vec
    }

    // reserve() -> set_len() should be detected
    vec.reserve(1000);
    //~^ uninit_vec

    unsafe {
        vec.set_len(200);
        //~^ set_len_uninitialized
    }

    // new() -> set_len() should be detected
    let mut vec: Vec<u8> = Vec::new();
    //~^ uninit_vec

    unsafe {
        vec.set_len(200);
    }

    // default() -> set_len() should be detected
    let mut vec: Vec<u8> = Default::default();
    //~^ uninit_vec

    unsafe {
        vec.set_len(200);
    }

    let mut vec: Vec<u8> = Vec::default();
    //~^ uninit_vec

    unsafe {
        vec.set_len(200);
    }

    // test when both calls are enclosed in the same unsafe block
    unsafe {
        let mut vec: Vec<u8> = Vec::with_capacity(1000);
        //~^ uninit_vec
        //~| uninit_vec

        vec.set_len(200);
        //~^ set_len_uninitialized
        //~| uninit_vec

        vec.reserve(1000);
        //~^ uninit_vec

        vec.set_len(200);
        //~^ set_len_uninitialized
    }

    let mut vec: Vec<u8> = Vec::with_capacity(1000);
    //~^ uninit_vec

    unsafe {
        // test the case where there are other statements in the following unsafe block
        vec.set_len(200);
        //~^ set_len_uninitialized
        assert!(vec.len() == 200);
    }

    // handle vec stored in the field of a struct
    let mut my_vec = MyVec::default();
    my_vec.vec.reserve(1000);
    //~^ uninit_vec
    //~| uninit_vec

    unsafe {
        my_vec.vec.set_len(200);
        //~^ uninit_vec
        //~| uninit_vec
    }

    my_vec.vec = Vec::with_capacity(1000);
    //~^ uninit_vec
    //~| uninit_vec

    unsafe {
        my_vec.vec.set_len(200);
    }

    // Test `#[allow(...)]` attributes on inner unsafe block (shouldn't trigger)
    let mut vec: Vec<u8> = Vec::with_capacity(1000); // FIXME: false positive
    //~^ uninit_vec
    #[allow(rpl::uninit_vec)]
    unsafe {
        vec.set_len(200);
        //~^ set_len_uninitialized
    }

    // MaybeUninit-wrapped types should not be detected
    unsafe {
        let mut vec: Vec<MaybeUninit<u8>> = Vec::with_capacity(1000);
        vec.set_len(200);
        //~^ set_len_uninitialized

        let mut vec: Vec<(MaybeUninit<u8>, MaybeUninit<bool>)> = Vec::with_capacity(1000);
        vec.set_len(200);
        //~^ set_len_uninitialized

        let mut vec: Vec<(MaybeUninit<u8>, [MaybeUninit<bool>; 2])> = Vec::with_capacity(1000);
        vec.set_len(200);
        //~^ set_len_uninitialized
    }

    // Clippy's known false negative
    let mut vec1: Vec<u8> = Vec::with_capacity(1000); //~ uninit_vec
    let mut vec2: Vec<u8> = Vec::with_capacity(1000); //~ uninit_vec
    unsafe {
        vec1.set_len(200);
        //~^ set_len_uninitialized
        vec2.set_len(200);
        //~^ set_len_uninitialized
    }

    // set_len(0) should not be detected
    let mut vec: Vec<u8> = Vec::with_capacity(1000); // FIXME: false positive
    //~^ uninit_vec
    unsafe {
        vec.set_len(0);
        //~^ set_len_uninitialized
    }

    // ZSTs should not be detected
    let mut vec: Vec<()> = Vec::with_capacity(1000);
    unsafe {
        vec.set_len(10);
        //~^ set_len_uninitialized
    }

    // unions should not be detected
    let mut vec: Vec<MyOwnMaybeUninit> = Vec::with_capacity(1000);
    unsafe {
        vec.set_len(10);
        //~^ set_len_uninitialized
    }

    polymorphic::<()>();

    fn polymorphic<T>() {
        // We are conservative around polymorphic types.
        let mut vec: Vec<T> = Vec::with_capacity(1000);
        //~^ uninit_vec

        unsafe {
            vec.set_len(10);
            //~^ set_len_uninitialized
        }
    }

    fn poly_maybe_uninit<T>() {
        // We are conservative around polymorphic types.
        let mut vec: Vec<MaybeUninit<T>> = Vec::with_capacity(1000);
        unsafe {
            vec.set_len(10);
            //~^ set_len_uninitialized
        }
    }

    fn nested_union<T>() {
        let mut vec: Vec<UnsafeCell<MaybeUninit<T>>> = Vec::with_capacity(1);
        unsafe {
            vec.set_len(1);
            //~^ set_len_uninitialized
        }
    }

    struct Recursive<T>(*const Recursive<T>, MaybeUninit<T>);
    fn recursive_union<T>() {
        // Make sure we don't stack overflow on recursive types.
        // The pointer acts as the base case because it can't be uninit regardless of its pointee.

        let mut vec: Vec<Recursive<T>> = Vec::with_capacity(1);
        //~^ uninit_vec

        unsafe {
            vec.set_len(1);
            //~^ set_len_uninitialized
        }
    }

    #[repr(u8)]
    enum Enum<T> {
        Variant(T),
    }
    fn union_in_enum<T>() {
        // Enums can have a discriminant that can't be uninit, so this should still warn
        let mut vec: Vec<Enum<T>> = Vec::with_capacity(1);
        //~^ uninit_vec

        unsafe {
            vec.set_len(1);
            //~^ set_len_uninitialized
        }
    }
}
