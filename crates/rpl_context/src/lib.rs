#![allow(internal_features)]
#![feature(rustc_private)]
#![feature(rustc_attrs)]
#![feature(debug_closure_helpers)]
#![feature(box_patterns)]
#![feature(let_chains)]

extern crate either;
extern crate rustc_abi;
extern crate rustc_arena;
extern crate rustc_ast;
extern crate rustc_ast_ir;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_target;
#[macro_use]
extern crate tracing;

mod arena;
mod context;
pub mod cvt_prim_ty;
pub mod pat;

pub(crate) use arena::Arena;
pub use context::{PatCtxt, PatternCtxt, PrimitiveTypes};
