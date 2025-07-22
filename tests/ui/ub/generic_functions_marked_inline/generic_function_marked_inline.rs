#[inline]
pub fn foo1<T>(x: T) -> T {
    //~^ERROR: it usually isn't necessary to apply #[inline] to generic functions
    x
}

#[inline]
pub fn foo2<'a>(x: &'a i32) -> &'a i32 {
    x
}

struct Struct<S> {
    field: S,
}

impl<S> Struct<S> {
    #[inline]
    pub fn foo3<T>(x: T) -> T {
        //~^ERROR: it usually isn't necessary to apply #[inline] to generic functions
        x
    }

    #[inline]
    pub fn foo4(self) -> S {
        //~^ERROR: it usually isn't necessary to apply #[inline] to generic functions
        self.field
    }
}

#[inline]
pub fn parse_str(wat: impl AsRef<str>) -> Result<Vec<u8>, ()> {
    //~^ERROR: it usually isn't necessary to apply #[inline] to generic functions
    _parse_str(wat.as_ref())
}

fn _parse_str(wat: &str) -> Result<Vec<u8>, ()> {
    Ok(wat.as_bytes().to_vec())
}

fn main() {}
