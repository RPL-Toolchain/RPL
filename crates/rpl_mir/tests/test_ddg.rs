#![allow(unused)]
#![feature(macro_metavar_expr_concat)]
#![feature(rustc_private)]

extern crate rustc_arena;
extern crate rustc_index;
extern crate rustc_middle;
extern crate rustc_span;

use std::fmt::Write;

use pretty_assertions::assert_eq;
use proc_macro2::TokenStream;
use quote::quote;
use rpl_context::{PatCtxt, PatternCtxt};
use rpl_mir::graph::{pat_control_flow_graph, pat_data_dep_graph};
use rpl_mir::pat::{FnPatternBody, Local};
use rustc_span::Symbol;

fn format_stmt_local((stmt, local): (usize, Local)) -> impl std::fmt::Debug {
    struct StmtLocal(usize, Local);
    impl std::fmt::Debug for StmtLocal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let &StmtLocal(stmt, local) = self;
            write!(f, "{stmt:?}/{local:?}")
        }
    }
    StmtLocal(stmt, local)
}

// macro_rules! test_case {
//     (fn $name:ident() { meta!($($rpl_meta:tt)*); $($input:tt)* } => { $($deps:tt)* }) => {
//         #[rpl_macros::pattern_def]
//         fn $name(pcx: PatCtxt<'_>) -> &MirPattern<'_> {
//             let pattern = rpl! {
//                 #[meta($($rpl_meta)*)]
//                 fn $pattern (..) -> _ = mir! {
//                     $($input)*
//                 }
//             };
//             pattern
//                 .fns
//                 .get_fn_pat(Symbol::intern("pattern"))
//                 .unwrap()
//                 .expect_mir_body()
//         }
//         #[test]
//         fn ${concat(test_, $name)}() {
//             PatternCtxt::entered_no_tcx(|pcx| {
//                 let pattern = $name(pcx);
//                 let cfg = pat_control_flow_graph(&pattern, (usize::BITS / u8::BITS).into());
//                 let graph = pat_data_dep_graph(&pattern, &cfg);
//                 let string = &mut String::new();
//                 for (bb, block) in graph.blocks() {
//                     write!(string, "{bb:?}: {{").unwrap();
//                     write!(string, "IN -> {:?},",
// block.rdep_start().map(format_stmt_local).collect::<Vec<_>>()).unwrap();                     for
// stmt in 0..block.num_statements() {                         write!(string, "{stmt} <- {:?},",
// block.deps(stmt).map(format_stmt_local).collect::<Vec<_>>()).unwrap();                         if
// stmt < pattern[bb].statements.len() {                             write!(string, " | {:?};",
// pattern[bb].statements[stmt]).unwrap();                         } else if let Some(terminator) =
// &pattern[bb].terminator {                             write!(string, " | {:?};",
// terminator).unwrap();                         }
//                     }
//                     write!(string, "OUT <- {:?};",
// block.dep_end().map(format_stmt_local).collect::<Vec<_>>()).unwrap();                     if
// block.full_rdep_start_end() {                         write!(string, "OUT <- IN / [*]").unwrap();
//                     } else {
//                         write!(string, "OUT <- IN / {:?}",
// block.rdep_start_end().collect::<Vec<_>>()).unwrap();                     }
//                     write!(string, "}}").unwrap();
//                 }
//                 assert_eq!(
//                     string
//                         .parse::<TokenStream>()
//                         .unwrap()
//                         .to_string()
//                         .replace(";", ";\n")
//                         .replace("{", "{\n")
//                         .replace("}", "}\n"),
//                     quote!($($deps)*)
//                         .to_string()
//                         .replace(";", ";\n")
//                         .replace("{", "{\n")
//                         .replace("}", "}\n"),
//                 );
//             });
//         }
//     };
// }
//
// test_case! {
//     fn cve_2020_25016() {
//         meta!($T:ty);
//
//         let $from_slice: &[$T] = _;
//         let $from_slice_mut: &mut [$T] = _;
//
//         let $from_raw: *const [$T] = &raw const *$from_slice;
//         let $from_raw_mut: *mut [$T] = &raw mut *$from_slice_mut;
//
//         let $to_ptr: *const u8 = copy $from_raw as *const u8 (PtrToPtr);
//         let $to_ptr_mut: *mut u8 = copy $from_raw_mut as *mut u8 (PtrToPtr);
//
//         let $from_len: usize = Len(*$from_slice);
//         let $from_mut_len: usize = Len(*$from_slice_mut);
//
//         let $ty_size: usize = SizeOf($T);
//
//         let $to_len: usize = Mul(move $from_len, move $ty_size);
//         let $to_mut_len: usize = Mul(move $from_mut_len, move $ty_size);
//
//         let $to_raw: *const [u8] = *const [u8] from (copy $to_ptr, copy $to_len);
//         let $to_raw_mut: *mut [u8] = *mut [u8] from (copy $to_ptr_mut, copy $to_mut_len);
//
//         let $to_slice: &[u8] = &*$to_raw;
//         let $to_slice_mut: &mut [u8] = &mut *$to_raw_mut;
//     } => {
//         ?bb0: {
//             IN  -> [],
//             0   <- [],              | _?0 = _;
//             1   <- [],              | _?1 = _;
//             2   <- [0/_?0],         | _?2 = &raw const (*_?0);
//             3   <- [1/_?1],         | _?3 = &raw mut (*_?1);
//             4   <- [2/_?2],         | _?4 = copy _?2 as *const u8 (PtrToPtr);
//             5   <- [3/_?3],         | _?5 = copy _?3 as *mut u8 (PtrToPtr);
//             6   <- [0/_?0],         | _?6 = Len((*_?0));
//             7   <- [1/_?1],         | _?7 = Len((*_?1));
//             8   <- [],              | _?8 = SizeOf(?T0);
//             9   <- [6/_?6, 8/_?8],  | _?9 = Mul(move _?6, move _?8);
//             10  <- [7/_?7, 8/_?8],  | _?10 = Mul(move _?7, move _?8);
//             11  <- [4/_?4, 9/_?9],  | _?11 = *const [u8] from (copy _?4, copy _?9);
//             12  <- [5/_?5, 10/_?10],| _?12 = *mut [u8] from (copy _?5, copy _?10);
//             13  <- [11/_?11],       | _?13 = &(*_?11);
//             14  <- [12/_?12],       | _?14 = &mut (*_?12);
//             15  <- [],      | end;
//             OUT <- [
//                 0/_?0, 1/_?1, 2/_?2, 3/_?3, 4/_?4, 5/_?5,
//                 9/_?9, 10/_?10, 11/_?11, 12/_?12, 13/_?13, 14/_?14
//             ];
//             OUT <- IN / []
//         }
//     }
// }
//
// test_case! {
//     fn cve_2020_35892_3_loop() {
//         meta!($T:ty, $SlabT:ty);
//
//         let $self: &mut $SlabT;
//         let $len: usize; // _2
//         let mut $x0: usize; // _17
//         let $x1: usize; // _14
//         let $x2: usize; // _15
//         let $x3: #[lang = "Option"]<usize>; // _3
//         let $x: usize; // _4
//         let mut $base: *mut $T; // _6
//         let $offset: isize; // _7
//         let $elem_ptr: *mut $T; // _5
//         let $x_cmp: usize; // _16
//         let $cmp: bool; // _13
//
//         $len = copy (*$self).len;
//         $x0 = const 0_usize;
//         loop {
//             $x_cmp = copy $x0;
//             $cmp = Lt(move $x_cmp, copy $len);
//             switchInt(move $cmp) {
//                 false => break,
//                 _ => {
//                     $x1 = copy $x0;
//                     $x2 = core::iter::Step::forward_unchecked(copy $x1, const 1_usize);
//                     $x0 = move $x2;
//                     $x3 = #[lang = "Some"](copy $x1);
//                     $x = copy ($x3 as Some).0;
//                     $base = copy (*$self).mem;
//                     $offset = copy $x as isize (IntToInt);
//                     $elem_ptr = Offset(copy $base, copy $offset);
//                     _ = core::ptr::drop_in_place(copy $elem_ptr);
//                 }
//             }
//         }
//     } => {
//         ?bb0: {
//             IN -> [0/_?0],
//             0 <- [], | _?1 = copy ((*_?0).len);
//             1 <- [], | _?2 = const 0_usize;
//             2 <- [], | goto ?bb1;
//             OUT <- [0/_?1, 1/_?2];
//             OUT <- IN / [_?0, _?3, _?4, _?5, _?6, _?7, _?8, _?9, _?10, _?11]
//         }
//         ?bb1: {
//             IN -> [0/_?2, 1/_?1],
//             0 <- [],  | _?10 = copy _?2;
//             1 <- [0/_?10], | _?11 = Lt(move _?10, copy _?1);
//             2 <- [1/_?11], | switchInt(move _?11) -> [false: ?bb4, otherwise: ?bb5];
//             OUT <- [];
//             OUT <- IN / [_?0, _?1, _?2, _?3, _?4, _?5, _?6, _?7, _?8, _?9]
//         }
//         ?bb2: { IN -> [], 0 <- [], | end;       OUT <- []; OUT <- IN / [*] }
//         ?bb3: { IN -> [], 0 <- [], | goto ?bb1; OUT <- []; OUT <- IN / [*] }
//         ?bb4: { IN -> [], 0 <- [], | goto ?bb2; OUT <- []; OUT <- IN / [*] }
//         ?bb5: {
//             IN -> [0/_?2],
//             0 <- [],      | _?3 = copy _?2;
//             1 <- [0/_?3], | _?4 = core::iter::Step::forward_unchecked(copy _?3, const 1_usize) ->
// ?bb6;             OUT <- [0/_?3, 1/_?4];
//             OUT <- IN / [_?0, _?1, _?2, _?5, _?6, _?7, _?8, _?9, _?10, _?11]
//         }
//         ?bb6: {
//             IN -> [0/_?4, 1/_?3, 3/_?0],
//             0 <- [],            | _?2 = move _?4;
//             1 <- [],            | _?5 = #[lang = "Some"](copy _?3);
//             2 <- [1/_?5],       | _?6 = copy ((_?5 as Some).0);
//             3 <- [],            | _?7 = copy ((*_?0).mem);
//             4 <- [2/_?6],       | _?8 = copy _?6 as isize (IntToInt);
//             5 <- [3/_?7, 4/_?8],| _?9 = Offset(copy _?7, copy _?8);
//             6 <- [5/_?9],       | core::ptr::drop_in_place(copy _?9) -> ?bb3;
//             OUT <- [0/_?2, 1/_?5, 2/_?6, 3/_?7, 4/_?8, 5/_?9];
//             OUT <- IN / [_?0, _?1, _?3, _?10, _?11]
//         }
//     }
// }
//
// test_case! {
//     fn cve_2020_35892_3_offset_by_one() {
//         meta!($T:ty, $SlabT:ty);
//         let $self: &mut $SlabT;
//         let $len: usize = copy (*$self).len;
//         let $len_isize: isize = move $len as isize (IntToInt);
//         let $base: *mut $T = copy (*$self).mem;
//         let $ptr_mut: *mut $T = Offset(copy $base, copy $len_isize);
//         let $ptr: *const $T = copy $ptr_mut as *const $T (PtrToPtr);
//         let $elem: $T = copy (*$ptr);
//     } => {
//         ?bb0: {
//             IN -> [0/_?0, 2/_?0],
//             0 <- [],            | _?1 = copy ((*_?0).len);
//             1 <- [0/_?1],       | _?2 = move _?1 as isize (IntToInt);
//             2 <- [] ,           | _?3 = copy ((*_?0).mem);
//             3 <- [1/_?2, 2/_?3],| _?4 = Offset (copy _?3, copy _?2);
//             4 <- [3/_?4],       | _?5 = copy _?4 as * const ?T0 (PtrToPtr);
//             5 <- [4/_?5],       | _?6 = copy (*_?5) ;
//             6 <- [],            | end ;
//             OUT <- [1/_?2, 2/_?3, 3/_?4, 4/_?5, 5/_?6];
//             OUT <- IN / [_?0]
//         }
//     }
// }
