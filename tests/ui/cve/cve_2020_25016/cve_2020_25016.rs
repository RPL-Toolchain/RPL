pub trait ComponentSlice<T> {
    /// The components interpreted as an array, e.g. one `RGB` expands to 3 elements.
    ///
    /// It's implemented for individual pixels as well as slices of pixels.
    fn as_slice(&self) -> &[T];
    /// The components interpreted as a mutable array, e.g. one `RGB` expands to 3 elements.
    ///
    /// It's implemented for individual pixels as well as slices of pixels.
    fn as_mut_slice(&mut self) -> &mut [T];
}

pub trait ComponentBytes<T: Copy + Send + Sync + 'static>
where
    Self: ComponentSlice<T>,
{
    /// The components interpreted as raw bytes, in machine's native endian. In `RGB` bytes of the red component are first.
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        //~^ ERROR: it usually isn't necessary to apply #[inline] to generic functions
        //~| HELP: See https://matklad.github.io/2021/07/09/inline-in-rust.html and https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
        //~| NOTE: generic functions are always `#[inline]` (monomorphization)
        //~| NOTE: `-D rpl::generic-function-marked-inline` implied by `-D warnings`
        //~| HELP: to override `-D warnings` add `#[allow(rpl::generic_function_marked_inline)]`
        let slice = self.as_slice();
        //~^ NOTE: trying to cast from this value of `&[T]` type
        unsafe {
            core::slice::from_raw_parts(
                //~^ ERROR: it is unsound to cast any slice `&[T]` to a byte slice `&[u8]`
                //~| NOTE: `#[deny(rpl::unsound_slice_cast)]` on by default
                slice.as_ptr() as *const _,
                slice.len() * core::mem::size_of::<T>(),
            )
        }
    }
    /// The components interpreted as raw bytes, in machine's native endian. In `RGB` bytes of the red component are first.
    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        //~^ ERROR: it usually isn't necessary to apply #[inline] to generic functions
        //~| HELP: See https://matklad.github.io/2021/07/09/inline-in-rust.html and https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
        //~| NOTE: generic functions are always `#[inline]` (monomorphization)
        let slice = self.as_mut_slice();
        //~^ NOTE: trying to cast from this value of `&mut [T]` type
        unsafe {
            core::slice::from_raw_parts_mut(
                //~^ ERROR: it is unsound to cast any slice `&mut [T]` to a byte slice `&mut [u8]`
                slice.as_mut_ptr() as *mut _,
                slice.len() * core::mem::size_of::<T>(),
            )
        }
    }
}

fn main() {}
