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
mod rudra_paths;
pub mod session;
mod statement;
mod ty;

pub use adt::{
    AdtFieldMap, AdtMatch, Candidates, MatchAdtCtxt, all_adt_fields_resolved, collect_adt_def_bindings,
    collect_adt_field_bindings, collect_used_field_pats, reset_adt_field_bindings_after_probe,
    seed_ty_vars_from_adt_field_candidates,
};
pub use counted::CountedMatch;
pub use fns::MatchFnCtxt;
pub use place::MatchPlaceCtxt;
pub use session::{
    BindingSnapshot, CrateItemIndex, FnSlotCandidate, MatchCollectCtxt, MatchSession, MatchSlot, MetaBindings,
    MultiMatched, OwnedLintMatch, SessionConfig, SessionLintTarget, SessionOutcome, SessionResult,
};
pub use ty::{MatchTyCtxt, TryCmpAs};
