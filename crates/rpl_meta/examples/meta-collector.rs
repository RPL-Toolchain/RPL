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
#![recursion_limit = "256"]

extern crate rustc_data_structures;
extern crate rustc_span;

use std::num::NonZero;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use rpl_meta::RPLMetaError;
use rpl_meta::cli::{read_file_from_path_buf, traverse_rpl};
use rustc_data_structures::sync::Registry;

pub fn collect_file_from_args_for_test() -> Vec<(PathBuf, String)> {
    let args = std::env::args();
    let args = args.skip(1);
    if args.len() == 0 {
        eprintln!("Usage: cargo run --package rpl_meta --example meta-collector <file1> <file2> ...");
        vec![]
    } else {
        let mut res = vec![];
        for arg in args {
            traverse_rpl(arg.into(), |path| {
                let buf = read_file_from_path_buf(&path);
                let buf = match buf {
                    Ok(buf) => buf,
                    Err(err) => {
                        eprintln!(
                            "{}",
                            RPLMetaError::FileError {
                                path,
                                error: Arc::new(err)
                            }
                        );
                        return;
                    },
                };
                res.push((path, buf));
            });
        }
        res.sort_by(|(p1, _), (p2, _)| p1.cmp(p2));
        res
    }
}

fn main() {
    // Only for testing purposes
    Registry::new(NonZero::new(1).unwrap()).register();
    rustc_span::create_session_if_not_set_then(rustc_span::edition::LATEST_STABLE_EDITION, |_| {
        static MCTX_ARENA: OnceLock<rpl_meta::arena::Arena<'_>> = OnceLock::new();
        static MCTX: OnceLock<rpl_meta::context::MetaContext<'_>> = OnceLock::new();
        let mctx_arena = MCTX_ARENA.get_or_init(rpl_meta::arena::Arena::default);
        let patterns_and_paths = mctx_arena.alloc(collect_file_from_args_for_test());
        let _mctx = MCTX.get_or_init(|| rpl_meta::parse_and_collect(mctx_arena, patterns_and_paths, |_| ()));
    });
}
