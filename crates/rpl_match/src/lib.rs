#![allow(internal_features)]
#![feature(rustc_private)]
#![feature(rustc_attrs)]
#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(box_patterns)]
#![feature(try_trait_v2)]
#![feature(debug_closure_helpers)]
#![feature(iter_chain)]
#![feature(iterator_try_collect)]
#![feature(cell_update)]
#![warn(unused_qualifications)]

extern crate either;
extern crate rustc_abi;
extern crate rustc_arena;
extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_fluent_macro;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_macros;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_target;
extern crate rustc_type_ir;
extern crate smallvec;
#[macro_use]
extern crate tracing;

mod adt;
mod counted;
mod fns;
pub mod graph; // FIXME: visibility
pub mod matches; // FIXME: visibility
pub mod mir; // FIXME: visibility
mod place;
pub mod predicate_evaluator;
pub mod resolve;
mod statement;
mod ty;

pub use adt::{AdtMatch, Candidates, MatchAdtCtxt};
pub use counted::CountedMatch;
pub use fns::MatchFnCtxt;
pub use place::MatchPlaceCtxt;
pub use ty::{MatchTyCtxt, TryCmpAs};
