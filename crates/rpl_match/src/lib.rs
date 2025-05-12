#![feature(rustc_private)]
#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(cell_update)]

extern crate rustc_abi;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_middle;
extern crate rustc_span;
#[macro_use]
extern crate tracing;

mod adt;
mod counted;
mod fns;
mod place;
pub mod resolve;
mod ty;

pub use adt::{AdtMatch, Candidates, MatchAdtCtxt};
pub use counted::CountedMatch;
pub use fns::MatchFnCtxt;
pub use place::MatchPlaceCtxt;
pub use ty::MatchTyCtxt;
