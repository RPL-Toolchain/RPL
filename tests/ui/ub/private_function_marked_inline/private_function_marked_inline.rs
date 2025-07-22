#[inline]
fn foo11() {}
//~^ERROR: it usually isn't necessary to apply #[inline] to private functions

#[inline]
pub(crate) fn foo12() {}
//~^ERROR: it usually isn't necessary to apply #[inline] to private functions
#[inline]
pub fn foo13() {}

#[inline(always)]
fn foo21() {}
//~^ERROR: it usually isn't necessary to apply #[inline] to private functions

#[inline(always)]
pub(crate) fn foo22() {}
//~^ERROR: it usually isn't necessary to apply #[inline] to private functions

#[inline(always)]
pub fn foo23() {}

#[inline(never)]
fn foo31() {}

#[inline(never)]
pub(crate) fn foo32() {}

#[inline(never)]
pub fn foo33() {}

trait Trait {
    #[inline]
    fn foo_in_trait() {}
    //~^ERROR: it usually isn't necessary to apply #[inline] to private functions
    //~|ERROR: it usually isn't necessary to apply #[inline] to generic functions
}

struct Struct;

impl Struct {
    #[inline]
    fn foo_in_impl() {}
    //~^ERROR: it usually isn't necessary to apply #[inline] to private functions
}

macro_rules! private_inline {
    ($ident:ident) => {
        #[inline]
        fn $ident() {}
        //~^ERROR: it usually isn't necessary to apply #[inline] to private functions
    };
    (#[$meta:meta] $ident:ident) => {
        #[$meta]
        fn $ident() {}
        //~^ERROR: it usually isn't necessary to apply #[inline] to private functions
    };
}

private_inline!(bar1);
private_inline!(#[inline] bar2);

fn main() {}
