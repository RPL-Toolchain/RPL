//@ignore-on-host
//@compile-flags: -Z inline-mir
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::de::StrRead;
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

/// Dim of dynamically-sized algebraic entities.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Dynamic {
    value: usize,
}

impl Dynamic {
    /// A dynamic size equal to `value`.
    #[inline]
    pub fn new(value: usize) -> Self {
        Self { value }
    }
}

impl Serialize for Dynamic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Dynamic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        usize::deserialize(deserializer).map(|x| Dynamic { value: x })
    }
}

/// Trait implemented by any type that can be used as a dimension. This includes type-level
/// integers and `Dynamic` (for dimensions not known at compile-time).
pub trait Dim: Any + Debug + Copy + PartialEq + Send + Sync {
    #[inline(always)]
    fn is<D: Dim>() -> bool {
        TypeId::of::<Self>() == TypeId::of::<D>()
    }

    /// Gets the compile-time value of `Self`. Returns `None` if it is not known, i.e., if `Self =
    /// Dynamic`.
    fn try_to_usize() -> Option<usize>;

    /// Gets the run-time value of `self`. For type-level integers, this is the same as
    /// `Self::try_to_usize().unwrap()`.
    fn value(&self) -> usize;

    /// Builds an instance of `Self` from a run-time value. Panics if `Self` is a type-level
    /// integer and `dim != Self::try_to_usize().unwrap()`.
    fn from_usize(dim: usize) -> Self;
}

impl Dim for Dynamic {
    #[inline]
    fn try_to_usize() -> Option<usize> {
        None
    }

    #[inline]
    fn from_usize(dim: usize) -> Self {
        Self::new(dim)
    }

    #[inline]
    fn value(&self) -> usize {
        self.value
    }
}

/// A Vec-based matrix data storage. It may be dynamically-sized.
#[repr(C)]
#[derive(Eq, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VecStorage<T, R: Dim, C: Dim> {
    data: Vec<T>,
    nrows: R,
    ncols: C,
}

fn main() {
    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    let _ = <VecStorage<f64, Dynamic, Dynamic> as Deserialize>::deserialize::<
        &mut serde_json::Deserializer<StrRead>,
    >;
}
