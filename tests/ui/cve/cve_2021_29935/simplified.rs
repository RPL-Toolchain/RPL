//@ revisions: inline regular
//@[inline] compile-flags: -Z inline-mir=true
//@[regular] compile-flags: -Z inline-mir=false
//@[regular] check-pass

use std::fmt::{self, Write};
use std::marker::PhantomData;

use smallvec::SmallVec;

mod private {
    pub trait Sealed {}
}

pub trait UriPart: private::Sealed {
    const DELIMITER: char;
}

struct Query {}

impl private::Sealed for Query {}

impl UriPart for Query {
    const DELIMITER: char = '&';
}

/// A struct used to format strings for [`UriDisplay`].
///
/// # Marker Generic: `Formatter<Path>` vs. `Formatter<Query>`
///
/// Like [`UriDisplay`], the [`UriPart`] parameter `P` in `Formatter<P>` must be
/// either [`Path`] or [`Query`] resulting in either `Formatter<Path>` or
/// `Formatter<Query>`. The `Path` version is used when formatting parameters
/// in the path part of the URI while the `Query` version is used when
/// formatting parameters in the query part of the URI. The
/// [`write_named_value()`] method is only available to `UriDisplay<Query>`.
///
/// [`UriPart`]: crate::uri::UriPart
/// [`Path`]: crate::uri::Path
/// [`Query`]: crate::uri::Query
///
/// # Overview
///
/// A mutable version of this struct is passed to [`UriDisplay::fmt()`]. This
/// struct properly formats series of values for use in URIs. In particular,
/// this struct applies the following transformations:
///
///   * When **multiple values** are written, they are separated by `/` for
///     `Path` types and `&` for `Query` types.
///
/// Additionally, for `Formatter<Query>`:
///
///   * When a **named value** is written with [`write_named_value()`], the name
///     is written out, followed by a `=`, followed by the value.
///
///   * When **nested named values** are written, typically by passing a value
///     to [`write_named_value()`] whose implementation of `UriDisplay` also
///     calls `write_named_vlaue()`, the nested names are joined by a `.`,
///     written out followed by a `=`, followed by the value.
///
/// [`UriDisplay`]: crate::uri::UriDisplay
/// [`UriDisplay::fmt()`]: crate::uri::UriDisplay::fmt()
/// [`write_named_value()`]: crate::uri::Formatter::write_named_value()
///
/// # Usage
///
/// Usage is fairly straightforward:
///
///   * For every _named value_ you wish to emit, call [`write_named_value()`].
///   * For every _unnamed value_ you wish to emit, call [`write_value()`].
///   * To write a string directly, call [`write_raw()`].
///
/// The `write_named_value` method automatically prefixes the `name` to the
/// written value and, along with `write_value` and `write_raw`, handles nested
/// calls to `write_named_value` automatically, prefixing names when necessary.
/// Unlike the other methods, `write_raw` does _not_ prefix any nested names
/// every time it is called. Instead, it only prefixes the _first_ time it is
/// called, after a call to `write_named_value` or `write_value`, or after a
/// call to [`refresh()`].
///
/// [`refresh()`]: crate::uri::Formatter::refresh()
///
/// # Example
///
/// The following example uses all of the `write` methods in a varied order to
/// display the semantics of `Formatter<Query>`. Note that `UriDisplay` should
/// rarely be implemented manually, preferring to use the derive, and that this
/// implementation is purely demonstrative.
///
/// ```rust
/// # extern crate rocket;
/// use std::fmt;
///
/// use rocket::http::uri::{Formatter, UriDisplay, Query};
///
/// struct Outer {
///     value: Inner,
///     another: usize,
///     extra: usize
/// }
///
/// struct Inner {
///     value: usize,
///     extra: usize
/// }
///
/// impl UriDisplay<Query> for Outer {
///     fn fmt(&self, f: &mut Formatter<Query>) -> fmt::Result {
///         f.write_named_value("outer_field", &self.value)?;
///         f.write_named_value("another", &self.another)?;
///         f.write_raw("out")?;
///         f.write_raw("side")?;
///         f.write_value(&self.extra)
///     }
/// }
///
/// impl UriDisplay<Query> for Inner {
///     fn fmt(&self, f: &mut Formatter<Query>) -> fmt::Result {
///         f.write_named_value("inner_field", &self.value)?;
///         f.write_value(&self.extra)?;
///         f.write_raw("inside")
///     }
/// }
///
/// let inner = Inner { value: 0, extra: 1 };
/// let outer = Outer { value: inner, another: 2, extra: 3 };
/// let uri_string = format!("{}", &outer as &UriDisplay<Query>);
/// assert_eq!(uri_string, "outer_field.inner_field=0&\
///                         outer_field=1&\
///                         outer_field=inside&\
///                         another=2&\
///                         outside&\
///                         3");
/// ```
///
/// Note that you can also use the `write!` macro to write directly to the
/// formatter as long as the [`std::fmt::Write`] trait is in scope. Internally,
/// the `write!` macro calls [`write_raw()`], so care must be taken to ensure
/// that the written string is URI-safe.
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// use std::fmt::{self, Write};
///
/// use rocket::http::uri::{UriDisplay, Formatter, UriPart, Path, Query};
///
/// pub struct Complex(u8, u8);
///
/// impl<P: UriPart> UriDisplay<P> for Complex {
///     fn fmt(&self, f: &mut Formatter<P>) -> fmt::Result {
///         write!(f, "{}+{}", self.0, self.1)
///     }
/// }
///
/// let uri_string = format!("{}", &Complex(42, 231) as &UriDisplay<Path>);
/// assert_eq!(uri_string, "42+231");
///
/// #[derive(UriDisplayQuery)]
/// struct Message {
///     number: Complex,
/// }
///
/// let message = Message { number: Complex(42, 47) };
/// let uri_string = format!("{}", &message as &UriDisplay<Query>);
/// assert_eq!(uri_string, "number=42+47");
/// ```
///
/// [`write_value()`]: crate::uri::Formatter::write_value()
/// [`write_raw()`]: crate::uri::Formatter::write_raw()
pub struct Formatter<'i, P> {
    prefixes: SmallVec<[&'static str; 3]>,
    inner: &'i mut (dyn Write + 'i),
    previous: bool,
    fresh: bool,
    _marker: PhantomData<P>,
}

impl<'i, P> Formatter<'i, P> {
    // #[inline(always)]
    pub(crate) fn new(inner: &'i mut (dyn Write + 'i)) -> Self {
        Formatter {
            inner,
            prefixes: SmallVec::new(),
            previous: false,
            fresh: true,
            _marker: PhantomData,
        }
    }
}

impl<P> Formatter<'_, P> {
    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    fn with_prefix<F>(&mut self, prefix: &str, f: F) -> fmt::Result
    where
        F: FnOnce(&mut Self) -> fmt::Result,
    {
        // The `prefix` string is pushed in a `StackVec` for use by recursive
        // (nested) calls to `write_raw`. The string is pushed here and then
        // popped here. `self.prefixes` is modified nowhere else, and no strings
        // leak from the the vector. As a result, it is impossible for a
        // `prefix` to be accessed incorrectly as:
        //
        //   * Rust _guarantees_ it exists for the lifetime of this method
        //   * it is only reachable while this method's stack is active because
        //     it is popped before this method returns
        //   * thus, at any point that it's reachable, it's valid
        //
        // Said succinctly: this `prefixes` stack shadows a subset of the
        // `with_prefix` stack precisely, making it reachable to other code.
        let prefix: &'static str = unsafe { std::mem::transmute(prefix) };

        self.prefixes.push(prefix);
        let result = f(self);
        self.prefixes.pop();

        result
    }
}

impl<P: UriPart> Formatter<'_, P> {
    pub fn write_raw<S: AsRef<str>>(&mut self, string: S) -> fmt::Result {
        // This implementation is a bit of a lie to the type system. Instead of
        // implementing this twice, one for <Path> and again for <Query>, we do
        // this once here. This is okay since we know that this handles the
        // cases for both Path and Query, and doing it this way allows us to
        // keep the uri part generic _generic_ in other implementations that use
        // `write_raw`.
        if self.fresh && P::DELIMITER == '/' {
            if self.previous {
                self.inner.write_char(P::DELIMITER)?;
            }
        } else if self.fresh && P::DELIMITER == '&' {
            if self.previous {
                self.inner.write_char(P::DELIMITER)?;
            }

            if !self.prefixes.is_empty() {
                for (i, prefix) in self.prefixes.iter().enumerate() {
                    //~[inline]^ERROR: casting the result of `from_raw_parts` to *const [&str]
                    self.inner.write_str(prefix)?;
                    if i < self.prefixes.len() - 1 {
                        self.inner.write_str(".")?;
                    }
                }

                self.inner.write_str("=")?;
            }
        }

        self.fresh = false;
        self.previous = true;
        self.inner.write_str(string.as_ref())
    }
}

fn main() {
    struct Wrapper<'a>(Formatter<'a, Query>);

    impl Drop for Wrapper<'_> {
        fn drop(&mut self) {
            self.0.write_raw("world").ok();
        }
    }
    let mut buffer = String::new();
    let mut wrapper = Wrapper(Formatter::new(&mut buffer));
    let tmp = String::from("hello");
    wrapper.0.with_prefix(tmp.as_str(), |_| panic!()).unwrap();
    println!("{:?}", wrapper.0.prefixes);
}
