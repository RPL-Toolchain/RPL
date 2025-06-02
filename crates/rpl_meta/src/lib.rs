#![feature(rustc_private)]
#![feature(map_try_insert)]
#![feature(box_patterns)]
#![feature(if_let_guard)]
#![feature(impl_trait_in_fn_trait_return)]
#![feature(let_chains)]
#![feature(macro_metavar_expr_concat)]

extern crate rpl_parser as parser;
extern crate rustc_arena;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_log;
extern crate rustc_span;
#[macro_use]
extern crate tracing;

pub mod arena;
pub mod check;
pub mod cli;
pub mod context;
pub mod error;
pub mod idx;
pub mod meta;
pub mod symbol_table;
pub mod utils;

use arena::Arena;
use context::MetaContext;
pub use error::RPLMetaError;
use meta::SymbolTables;
use std::path::PathBuf;

#[instrument(level = "info", skip_all, fields(patterns = ?path_and_content))]
pub fn parse_and_collect<'mcx>(
    arena: &'mcx Arena<'mcx>,
    path_and_content: &'mcx Vec<(PathBuf, String)>,
) -> (MetaContext<'mcx>, bool) {
    let mut mctx = MetaContext::new(arena);
    let mut has_error = false;
    for (path, content) in path_and_content {
        let idx = mctx.request_rpl_idx(path);
        let content = mctx.alloc_str(content);
        mctx.contents.insert(idx, content);
    }

    for (idx, content) in &mctx.contents {
        let path = mctx.id2path.get(idx).unwrap(); // safe unwrap
        mctx.set_active_path(Some(path));
        let parse_res = parser::parse_main(content, path);
        match parse_res {
            Ok(main) => {
                // Cache the syntax tree
                let main = mctx.alloc_ast(main);
                mctx.syntax_trees.insert(*idx, main);
                // Perform meta collection
                let meta = SymbolTables::collect(path, main, *idx, &mctx);
                has_error |= meta.show_error();
                mctx.symbol_tables.insert(*idx, meta);
            },
            Err(err) => {
                warn!("{}", RPLMetaError::from(err));
                has_error = true;
                continue;
            },
        }
        // Seems unnecessary.
        // mctx.set_active_path(None);
    }
    (mctx, has_error)
}
