#![feature(rustc_private)]
#![feature(map_try_insert)]
#![feature(box_patterns)]
#![feature(if_let_guard)]
#![feature(impl_trait_in_fn_trait_return)]
#![feature(let_chains)]
#![feature(macro_metavar_expr_concat)]

extern crate rpl_pat_syntax as syntax;

extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_span;

macro_rules! todo {
    () => {
        std::todo!("{}:{}", file!(), line!())
    };
    ($($arg:tt)*) => {
        std::todo!($($arg)*)
    };
}

mod check;
pub(crate) mod expand;
pub(crate) mod symbol_table;

#[cfg(test)]
mod tests;

pub(crate) use check::check_pattern;
pub use expand::{PatternDefFn, expand, expand_pattern};
pub(crate) use symbol_table::{SymbolTable, is_primitive};
