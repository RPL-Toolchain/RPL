#![allow(unused)]
#![feature(rustc_private)]
#![feature(macro_metavar_expr_concat)]
#![feature(generic_arg_infer)]
#![recursion_limit = "256"]

extern crate rustc_arena;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_middle;
extern crate rustc_span;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::LazyLock;

use pest_typed::ParsableTypedNode;
use pretty_assertions::assert_eq;
use quote::quote;
use rpl_context::pat::FnPattern;
use rpl_context::{PatCtxt, PatternCtxt, pat};
use rpl_graphviz::{pat_cfg_to_graphviz, pat_ddg_to_graphviz};
use rpl_match::mir::pat::FnPatternBody;
use rpl_meta::arena::Arena;
use rpl_meta::context::MetaContext;
use rpl_meta::symbol_table::{SymbolTable, WithPath};
use rpl_parser::pairs;
use rustc_data_structures::fx::FxHashMap;
use rustc_span::Symbol;

fn read_from_file(file: &str) -> std::io::Result<String> {
    let mut file = File::open(file)?;
    let mut expected = String::new();
    file.read_to_string(&mut expected)?;
    Ok(expected)
}

fn write_to_file(file: &str, content: &[u8]) -> std::io::Result<()> {
    File::create(file)?.write_all(content)?;
    Ok(())
}

macro_rules! test_case {
    ( $(#[$meta:meta])* fn $name:ident() {
        $(let imports = $imports:expr;)?
        let pattern = $pattern:expr;
    }) => {
        #[test]
        $(#[$meta])*
        fn $name() {
            rustc_span::create_session_if_not_set_then(rustc_span::edition::LATEST_STABLE_EDITION, |_| {
                static ARENA: LazyLock<Arena> = LazyLock::new(|| Arena::default());
                static MCTX: LazyLock<MetaContext> = LazyLock::new(|| MetaContext::new(&ARENA));

                let imports: [String; _] = [];
                $(let imports: [String; _] = $imports;)?
                let imports: Vec<&'static pairs::UsePath<'static>> = imports.iter().map(|import| &*Box::leak(Box::new(pairs::UsePath::try_parse(ARENA.alloc_str(import)).unwrap_or_else(|err| panic!("{err}"))))).collect();
                let pat: String = $pattern;
                let pat = pat.replace("$ ", "$");
                let pat = ARENA.alloc_str(&pat);
                let pat_item = pairs::RPLPatternItem::try_parse(pat).unwrap_or_else(|err| panic!("{err}"));
                let pat_item = &*Box::leak(Box::new(pat_item));
                let mut errors = Vec::new();
                let path = Path::new(file!());
                MCTX.set_active_path(Some(path));
                let symbol_tables = SymbolTable::collect_symbol_tables(
                    &MCTX,
                    &imports,
                    std::iter::once(pat_item), &mut errors
                );
                if !errors.is_empty() {
                    for error in &errors {
                        eprintln!("{error}");
                    }
                    panic!("Errors during symbol table collection");
                }
                let symbol_tables = &*Box::leak(Box::new(symbol_tables));
                PatternCtxt::entered_no_tcx(move |pcx| {
                    let pat = pcx.new_pattern();
                    let ident = Symbol::intern(pat_item.Identifier().span.as_str());
                    pat.add_pattern_item(
                        WithPath::new(path, pat_item),
                        &symbol_tables,
                        pat::PattOrUtil::Patt,
                    );
                    let patterns = pat.patt_block
                        .get(&ident)
                        .unwrap()
                        .expect_rust_items()
                        .fns
                        .iter()
                        .next()
                        .unwrap()
                        .body
                        .unwrap();
                    let mut cfg = Vec::new();
                    pat_cfg_to_graphviz(&patterns, &mut cfg, &Default::default()).unwrap();
                    let cfg = String::from_utf8(cfg).unwrap();
                    let mut ddg = Vec::new();
                    pat_ddg_to_graphviz(&patterns, &mut ddg, &Default::default()).unwrap();
                    let ddg = String::from_utf8(ddg).unwrap();

                    let cfg_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/graphs/", stringify!($name), "_pat_cfg.dot");
                    let ddg_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/graphs/", stringify!($name), "_pat_ddg.dot");

                    let cfg_expected = read_from_file(cfg_file).unwrap_or_default();
                    let ddg_expected = read_from_file(ddg_file).unwrap_or_default();

                    let cfg_expected_file = concat!(env!("CARGO_TARGET_TMPDIR"), "/", stringify!($name), "_pat_cfg.dot");
                    let ddg_expected_file = concat!(env!("CARGO_TARGET_TMPDIR"), "/", stringify!($name), "_pat_ddg.dot");

                    if cfg_expected != cfg {
                        write_to_file(cfg_expected_file, cfg.as_bytes()).unwrap();
                    }

                    if ddg_expected != ddg {
                        write_to_file(ddg_expected_file , ddg.as_bytes()).unwrap();
                    }
                    assert_eq!(cfg, cfg_expected, "CFG mismatch, see {cfg_file} and {cfg_expected_file}");
                    assert_eq!(ddg, ddg_expected, "DDG mismatch, see {ddg_file} and {ddg_expected_file}");
                });
            })
        }
    }
}

test_case! {
    fn cve_2020_35892_3() {
        let pattern =  quote!{p[$T: type, $SlabT: type] = fn _ () {
                let $self: &mut $SlabT;
                let $len: usize;
                let $x1: usize;
                let $x2: usize;
                let $opt: #[lang = "Option"]<usize>;
                let $discr: isize;
                let $x: usize;
                let $start_ref: &usize;
                let $end_ref: &usize;
                let $start: usize;
                let $end: usize;
                let $range: core::ops::range::Range<usize>;
                let mut $iter: core::ops::range::Range<usize>;
                let mut $iter_mut: &mut core::ops::range::Range<usize>;
                let mut $base: *mut $T;
                let $offset: isize;
                let $elem_ptr: *mut $T;
                let $cmp: bool;

                $len = copy (*$self).len;
                $range = core::ops::range::Range { start: const 0_usize, end: move $len };
                $iter = move $range;
                loop {
                    $iter_mut = &mut $iter;
                    $start_ref = &(*$iter_mut).start;
                    $start = copy *$start_ref;
                    $end_ref = &(*$iter_mut).end;
                    $end = copy *$end_ref;
                    $cmp = Lt(move $start, move $end);
                    switchInt(move $cmp) {
                        false => $opt = #[lang = "None"],
                        _ => {
                            $x1 = copy (*$iter_mut).start;
                            $x2 = core::iter::range::Step::forward_unchecked(copy $x1, const 1_usize);
                            (*$iter_mut).start = move $x2;
                            $opt = #[lang = "Some"](copy $x1);
                        }
                    }
                    $discr = discriminant($opt);
                    switchInt(move $discr) {
                        0_isize => break,
                        1_isize => {
                            $x = copy ($opt as Some).0;
                            $base = copy (*$self).mem;
                            $offset = copy $x as isize (IntToInt);
                            $elem_ptr = Offset(copy $base, copy $offset);
                            _ = core::ptr::drop_in_place(copy $elem_ptr);
                        }
                    }
                }
            }
        }.to_string();
    }
}

test_case! {
    fn cve_2020_35892_3_offset() {
        let pattern = quote!{
            p[$T:type, $SlabT:type] = fn _ () {
                let $self: &mut $SlabT;
                let $len: usize = copy (*$self).len;
                let $len_isize: isize = move $len as isize (IntToInt);
                let $base: *mut $T = copy (*$self).mem;
                let $ptr_mut: *mut $T = Offset(copy $base, copy $len_isize);
                let $ptr: *const $T = copy $ptr_mut as *const $T (PtrToPtr);
                let $elem: $T = copy (*$ptr);
            }
        }.to_string();
    }
}

test_case! {
    fn cve_2018_21000_const() {
        let pattern = quote!{
            p[$T:type] = fn _ () {
                let $from_slice: &[$T] = _;
                let $from_raw: *const [$T] = &raw const *$from_slice;
                let $from_len: usize = PtrMetadata(copy $from_slice);
                let $ty_size: usize = SizeOf($T);
                let $to_ptr_t: *const $T = move $from_raw as *const $T (PtrToPtr);
                let $to_ptr: *const u8 = move $to_ptr_t as *const u8 (PtrToPtr);
                let $to_len: usize = Mul(move $from_len, move $ty_size);
                let $to_raw: *const [u8] = *const [u8] from (copy $to_ptr, copy $to_len);
                let $to_slice: &[u8] = &*$to_raw;
            }
        }.to_string();
    }
}

test_case! {
    fn cve_2018_21000() {
        let pattern = quote!{
            p[$T:type] = fn _ () {
                let $from_slice_mut: &mut [$T] = _;
                let $from_raw_mut: *mut [$T] = &raw mut *$from_slice_mut;
                let $from_len_mut: usize = PtrMetadata(copy $from_slice_mut);
                let $ty_size_mut: usize = SizeOf($T);
                let $to_ptr_mut: *mut u8 = copy $from_raw_mut as *mut u8 (PtrToPtr);
                let $to_len_mut: usize = Mul(move $from_len_mut, move $ty_size_mut);
                let $to_raw_mut: *mut [u8] = *mut [u8] from (copy $to_ptr_mut, copy $to_len_mut);
                let $to_slice_mut: &mut [u8] = &mut *$to_raw_mut;
            }
        }.to_string();
    }
}

test_case! {
    fn cve_2020_35877() {
        let pattern = quote!{
            p[$T:type] = fn _ () {
                let $offset: usize = _; // _2
                let $offset_1: usize = copy $offset; // _3
                let $ptr_1: *const $T = _; // _4
                let $offset_2: usize = copy $offset_1; // _13
                let $flag: bool = Gt(move $offset_2, const 0usize); // _12
                let $ptr_3: *const $T; // _14
                let $ptr_4: *const $T; // _15
                let $reference: &$T; // _0
                loop {
                    $offset_2 = copy $offset_1; // _13
                    $flag = Gt(move $offset_2, const 0usize); // _12
                    switchInt(move $flag) {
                        0usize => {
                            $reference = &(*$ptr_1);
                            break;
                        }
                        _ => {
                            $offset_1 = Sub(copy $offset_1, const 1usize);
                            $ptr_4 = copy $ptr_1;
                            $ptr_3 = Offset(copy $ptr_4, _);
                            $ptr_1 = move $ptr_3;
                            continue;
                        }
                    }
                }
            }
        }.to_string();
    }
}

test_case! {
    fn cve_2021_27376() {
        let pattern = quote!{
            p[$T:type] = fn _ () {
                let $src: *const std::net::SocketAddrV4 = _;
                let $dst: *const libc::sockaddr = move $src as *const libc::sockaddr (PtrToPtr);
            }
        }.to_string();
    }
}

test_case! {
    fn cve_2021_29941_2() {
        let pattern = quote!{
            p[$T:type] = fn _ () {
                // type ExactSizeIterT = impl std::iter::ExactSizeIterator<Item = $T>;
                // let's use a std::ops::Range<$T> instead temporarily
                type RangeT = std::ops::Range<$T>;
                type Vec = std::vec::Vec;
                type VecT = std::vec::Vec<$T>;
                type RefMutVecT = &mut std::vec::Vec<$T>;
                type PtrMutT = *mut $T;
                type RefMutSliceT = &mut [$T];
                type EnumerateRangeT = std::iter::Enumerate<RangeT>;
                type RefMutEnumerateRangeT = &mut std::iter::Enumerate<RangeT>;
                type OptionUsizeT = std::option::Option<(usize, $T)>;

                let $iter: RangeT = _;
                // let len: usize = <RangeT as std::iter::ExactSizeIterator>::len(move iter);
                let $len: usize = RangeT::len(move $iter);
                let mut $vec: VecT = Vec::with_capacity(copy $len);
                let mut $ref_to_vec: RefMutVecT = &mut $vec;
                let mut $ptr_to_vec: PtrMutT = Vec::as_mut_ptr(move $ref_to_vec);
                let mut $slice: RefMutSliceT = std::slice::from_raw_parts_mut(copy $ptr_to_vec, copy $len);
                // let mut $enumerate: EnumerateRangeT = <RangeT as std::iter::Iterator>::enumerate(move $iter);
                let mut $enumerate: EnumerateRangeT = RangeT::enumerate(move $iter);
                let mut $ref_mut_enumerate: RefMutEnumerateRangeT = &mut $enumerate;
                let $next: OptionUsizeT;
                let $cmp: isize;
                let $first: usize;
                let $second_t: $T;
                let $second_usize: usize;
                let $_tmp: ();
                loop {
                    // next = <EnumerateRangeT as std::iter::Iterator>::next(move enumerate);
                    $next = EnumerateRangeT::next(move $ref_mut_enumerate);
                    // in `cmp = discriminant(copy next);`
                    // which discriminant should be used?
                    // $cmp = balabala::discriminant(copy $next);
                    $cmp = discriminant($next);
                    switchInt(move $cmp) {
                        // true or 1 here?
                        true => {
                            $first = copy ($next as Some).0;
                            $second_t = copy ($next as Some).1;
                            $second_usize = copy $second_t as usize (IntToInt);
                            (*$slice)[$second_usize] = copy $first as $T (IntToInt);
                        }
                        _ => break,
                    }
                }
                // variable shadowing?
                // There cannot be two mutable references to `vec` in the same scope
                $ref_to_vec = &mut $vec;
                $_tmp = Vec::set_len(move $ref_to_vec, copy $len);
            }
        }.to_string();
    }
}

test_case! {
    fn cve_2021_29941() {
        let imports = [
            quote!{ use std::iter::ExactSizeIterator; }.to_string(),
            quote!{ use alloc::vec::Vec; }.to_string(),
        ];
        let pattern = quote!{
            p[$I:type, $T:type] = fn _ () {
                let $iter: $I = _;
                let $len: usize = ExactSizeIterator::len(move $iter);
                let $vec: &mut Vec<$T> = _;
                _ = Vec::set_len(move $vec, copy $len);
            }
        }.to_string();
    }
}

test_case! {
    fn cve_2021_29941_uninitialized_slice() {
        let pattern = quote!{
            p[$T:type] = fn _ () {
                let $len: usize = _;
                let $vec: alloc::vec::Vec<$T> = alloc::vec::Vec::with_capacity(_);
                let $vec_ref: &alloc::vec::Vec<$T> = &$vec;
                let $ptr: *const $T = alloc::vec::Vec::as_ptr(move $vec_ref);
                let $slice: &[$T] = std::slice::from_raw_parts::<'_, $T>(move $ptr, copy $len);
            }
        }.to_string();
    }
}

test_case! {
    fn unsafe_cell_alias() {
        let pattern = quote!{
            p[$T:type] = fn _ () {
                type unsafe_cell_t = core::cell::UnsafeCell<$T>;

                let $a: &unsafe_cell_t = _;
                let $b: &unsafe_cell_t = _;
                let $raw_a: *const unsafe_cell_t = &raw const *$a;
                let $raw_b: *const unsafe_cell_t = &raw const *$b;
                let $raw_mut_a: *mut $T = copy $raw_a as *mut $T (PtrToPtr);
                let $raw_mut_b: *mut $T = copy $raw_b as *mut $T (PtrToPtr);
                let $mut_a: &mut $T = &mut *$raw_mut_a;
                let $mut_b: &mut $T = &mut *$raw_mut_b;
                (*$mut_a) = _;
                (*$mut_b) = _;
            }
        }.to_string();
    }
}

test_case! {
    fn control_flow() {
        let pattern = quote!{
            p[$T:type] = fn _ () {
                let $a: &mut i32 = _;
                let $f: bool = _;
                switchInt(copy $f) {
                    false => {}
                    _ => {
                        *$a = Add(copy (*$a), const 1_i32);
                    }
                }
            }
        }.to_string();
    }
}

test_case! {
    fn pattern_drop_unit_value()  {
        let pattern = quote!{
            p[$T:type] = fn _ () {
                let $raw_ptr: *mut $T = _;
                let $value: $T = _;
                drop((*$raw_ptr));
                (*$raw_ptr) = move $value;
            }
        }.to_string();
    }
}

test_case! {
    fn unchecked_ptr_offset_base() {
        let pattern = quote!{
            p[$T: type] = {
                fn $pattern(..) -> _ {
                    'ptr:
                    let $ptr: *const $T = _;
                    'offset:
                    let $ptr_1: *const $T = Offset(copy $ptr, _);
                }
            }
        }.to_string();
    }
}

test_case! {
    fn unchecked_ptr_offset_lt() {
        let pattern = quote!{
            p[$T: type, $U: type] = {
                fn $pattern(..) -> _ {
                    let $index: $U = _;
                    'ptr:
                    let $ptr: *const $T = _;
                    let $cmp: bool = Lt(copy $index, _);
                    'offset:
                    let $ptr_1: *const $T = Offset(copy $ptr, _);
                }
            }
        }.to_string();
    }
}

test_case! {
    fn unchecked_ptr_offset_le() {
        let pattern = quote!{
            p[$T: type, $U: type] = {
                fn $pattern(..) -> _ {
                    let $index: $U = _;
                    'ptr:
                    let $ptr: *const $T = _;
                    let $cmp: bool = Le(copy $index, _);
                    'offset:
                    let $ptr_1: *const $T = Offset(copy $ptr, _);
                }
            }
        }.to_string();
    }
}

test_case! {
    fn alloc_cast_write() {
        let pattern = quote!{
            p[$T:type] = fn _() {
                'alloc:
                let $ptr_1: *mut u8 = alloc::alloc::__rust_alloc(_, _); // _3
                let $ptr_2: *mut $T = move $ptr_1 as *mut $T (PtrToPtr); // _2
                'write:
                (*$ptr_2) = _;
            }
        }.to_string();
    }
}

test_case! {
    fn alloc_check_cast_write() {
        let pattern = quote!{
            p[$T:type] = fn _() {
                'alloc:
                let $ptr_1: *mut u8 = alloc::alloc::__rust_alloc(_, _); // _2
                let $const_ptr_1: *const u8 = copy $ptr_1 as *const u8 (PtrToPtr); // _19
                let $addr_1: usize = copy $const_ptr_1 as usize (Transmute); // _20
                // It's weird that `$ptr_2` can only be declared before `switchInt`
                // switchInt(move $addr_1) {
                //     0_usize => {}
                //     _ => {}
                // }
                let $ptr_2: *mut $T = copy $ptr_1 as *mut $T (PtrToPtr); // _4
                'write:
                (*$ptr_2) = _;
            }
        }.to_string();
    }
}

test_case! {
    fn alloc_cast_check_write() {
        let pattern = quote!{
            p[$T:type] = fn _() {
                'alloc:
                let $ptr_1: *mut u8 = alloc::alloc::__rust_alloc(_, _); // _3
                let $ptr_2: *mut $T = move $ptr_1 as *mut $T (PtrToPtr); // _2
                let $const_ptr_1: *const u8 = copy $ptr_2 as *const u8 (PtrToPtr); // _19
                let $addr_1: usize = copy $const_ptr_1 as usize (Transmute); // _20
                // switchInt(move $addr_1) {
                //     0_usize => {}
                //     _ => {}
                // }
                'write:
                (*$ptr_2) = _;
            }
        }.to_string();
    }
}

// macro_rules! test_case {
//     ( $(#[$meta:meta])* fn $name:ident() {
//         meta!($($rpl_meta:tt)*);
//         $($input:tt)*
//     }) => {
//         #[rpl_macros::pattern_def]
//         #[allow(unused_variables)]
//         fn $name(pcx: PatCtxt<'_>) -> &MirPattern<'_> {
//             let pattern = rpl! {
//                 #[meta($($rpl_meta)*)]
//                 fn $pattern (..) -> _ = mir! {
//                     $($input)*
//                 }
//             };
//             pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap().expect_mir_body()
//         }
//         #[test]
//         $(#[$meta])*
//         fn ${concat(test_, $name)}() {
//             PatternCtxt::entered_no_tcx(|pcx| {
//                 let patterns = $name(pcx);
//                 let mut cfg = Vec::new();
//                 pat_cfg_to_graphviz(&patterns, &mut cfg, &Default::default()).unwrap();
//                 let cfg = String::from_utf8(cfg).unwrap();
//                 let mut ddg = Vec::new();
//                 pat_ddg_to_graphviz(&patterns, &mut ddg, &Default::default()).unwrap();
//                 let ddg = String::from_utf8(ddg).unwrap();

//                 let cfg_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/graphs/",
// stringify!($name), "_pat_cfg.dot");                 let ddg_file =
// concat!(env!("CARGO_MANIFEST_DIR"), "/tests/graphs/", stringify!($name), "_pat_ddg.dot");

//                 let cfg_expected = read_from_file(cfg_file).unwrap_or_default();
//                 let ddg_expected = read_from_file(ddg_file).unwrap_or_default();

//                 let cfg_expected_file = concat!(env!("CARGO_TARGET_TMPDIR"), "/",
// stringify!($name), "_pat_cfg.dot");                 let ddg_expected_file =
// concat!(env!("CARGO_TARGET_TMPDIR"), "/", stringify!($name), "_pat_ddg.dot");

//                 if cfg_expected != cfg {
//                     write_to_file(cfg_expected_file, cfg.as_bytes()).unwrap();
//                 }

//                 if ddg_expected != ddg {
//                     write_to_file(ddg_expected_file , ddg.as_bytes()).unwrap();
//                 }
//                 assert_eq!(cfg, cfg_expected, "CFG mismatch, see {cfg_file} and
// {cfg_expected_file}");                 assert_eq!(ddg, ddg_expected, "DDG mismatch, see
// {ddg_file} and {ddg_expected_file}");             });
//         }
//     }
// }

// test_case! {
//     fn cve_2020_35892_3() {
//         meta!($T:ty, $SlabT:ty);
//
//         let $self: &mut $SlabT;
//         let $len: usize;
//         let $x1: usize;
//         let $x2: usize;
//         let $opt: #[lang = "Option"]<usize>;
//         let $discr: isize;
//         let $x: usize;
//         let $start_ref: &usize;
//         let $end_ref: &usize;
//         let $start: usize;
//         let $end: usize;
//         let $range: core::ops::range::Range<usize>;
//         let mut $iter: core::ops::range::Range<usize>;
//         let mut $iter_mut: &mut core::ops::range::Range<usize>;
//         let mut $base: *mut $T;
//         let $offset: isize;
//         let $elem_ptr: *mut $T;
//         let $cmp: bool;
//
//         $len = copy (*$self).len;
//         $range = core::ops::range::Range { start: const 0_usize, end: move $len };
//         $iter = move $range;
//         loop {
//             $iter_mut = &mut $iter;
//             $start_ref = &(*$iter_mut).start;
//             $start = copy *$start_ref;
//             $end_ref = &(*$iter_mut).end;
//             $end = copy *$end_ref;
//             $cmp = Lt(move $start, move $end);
//             switchInt(move $cmp) {
//                 false => $opt = #[lang = "None"],
//                 _ => {
//                     $x1 = copy (*$iter_mut).start;
//                     $x2 = core::iter::range::Step::forward_unchecked(copy $x1, const 1_usize);
//                     (*$iter_mut).start = move $x2;
//                     $opt = #[lang = "Some"](copy $x1);
//                 }
//             }
//             $discr = discriminant($opt);
//             switchInt(move $discr) {
//                 0_isize => break,
//                 1_isize => {
//                     $x = copy ($opt as Some).0;
//                     $base = copy (*$self).mem;
//                     $offset = copy $x as isize (IntToInt);
//                     $elem_ptr = Offset(copy $base, copy $offset);
//                     _ = core::ptr::drop_in_place(copy $elem_ptr);
//                 }
//             }
//         }
//     }
// }
//
// test_case! {
//     fn cve_2020_35892_3_offset() {
//         meta!($T:ty, $SlabT:ty);
//         let $self: &mut $SlabT;
//         let $len: usize = copy (*$self).len;
//         let $len_isize: isize = move $len as isize (IntToInt);
//         let $base: *mut $T = copy (*$self).mem;
//         let $ptr_mut: *mut $T = Offset(copy $base, copy $len_isize);
//         let $ptr: *const $T = copy $ptr_mut as *const $T (PtrToPtr);
//         let $elem: $T = copy (*$ptr);
//     }
// }
//
// test_case! {
//     fn cve_2018_21000_const() {
//         meta!($T:ty);
//
//         let $from_slice: &[$T] = _;
//         let $from_raw: *const [$T] = &raw const *$from_slice;
//         let $from_len: usize = PtrMetadata(copy $from_slice);
//         let $ty_size: usize = SizeOf($T);
//         let $to_ptr_t: *const T = move $from_raw as *const $T (PtrToPtr);
//         let $to_ptr: *const u8 = move $to_ptr_t as *const u8 (PtrToPtr);
//         let $to_len: usize = Mul(move $from_len, move $ty_size);
//         let $to_raw: *const [u8] = *const [u8] from (copy $to_ptr, copy $to_len);
//         let $to_slice: &[u8] = &*$to_raw;
//     }
// }
//
// test_case! {
//     fn cve_2018_21000() {
//         meta!($T:ty);
//
//         let $from_slice_mut: &mut [$T] = _;
//         let $from_raw_mut: *mut [$T] = &raw mut *$from_slice_mut;
//         let $from_len_mut: usize = PtrMetadata(copy $from_slice_mut);
//         let $ty_size_mut: usize = SizeOf($T);
//         let $to_ptr_mut: *mut u8 = copy $from_raw_mut as *mut u8 (PtrToPtr);
//         let $to_len_mut: usize = Mul(move $from_len_mut, move $ty_size_mut);
//         let $to_raw_mut: *mut [u8] = *mut [u8] from (copy $to_ptr_mut, copy $to_len_mut);
//         let $to_slice_mut: &mut [u8] = &mut *$to_raw_mut;
//     }
// }
//
// test_case! {
//     fn cve_2020_35877() {
//         meta!($T:ty);
//
//         let $offset: usize = _; // _2
//         let $offset_1: usize = copy $offset; // _3
//         let $ptr_1: *const $T = _; // _4
//         let $offset_2: usize = copy $offset_1; // _13
//         let $flag: bool = Gt(move $offset_2, const 0usize); // _12
//         let $ptr_3: *const $T; // _14
//         let $ptr_4: *const $T; // _15
//         let $reference: &$T; // _0
//         loop {
//             $offset_2 = copy $offset_1; // _13
//             $flag = Gt(move $offset_2, const 0usize); // _12
//             switchInt(move $flag) {
//                 0usize => {
//                     $reference = &(*$ptr_1);
//                     break;
//                 }
//                 _ => {
//                     $offset_1 = Sub(copy $offset_1, const 1usize);
//                     $ptr_4 = copy $ptr_1;
//                     $ptr_3 = Offset(copy $ptr_4, _);
//                     $ptr_1 = move $ptr_3;
//                     continue;
//                 }
//             }
//         }
//     }
// }
//
// test_case! {
//     fn cve_2021_27376() {
//         meta!();
//
//         let $src: *const std::net::SocketAddrV4 = _;
//         let $dst: *const libc::sockaddr = move $src as *const libc::sockaddr (PtrToPtr);
//     }
// }
//
// test_case! {
//     fn cve_2021_29941_2() {
//         meta!($T:ty);
//         // type ExactSizeIterT = impl std::iter::ExactSizeIterator<Item = $T>;
//         // let's use a std::ops::Range<$T> instead temporarily
//         type RangeT = std::ops::Range<$T>;
//         type VecT = std::vec::Vec<$T>;
//         type RefMutVecT = &mut std::vec::Vec<$T>;
//         type PtrMutT = *mut $T;
//         type RefMutSliceT = &mut [$T];
//         type EnumerateRangeT = std::iter::Enumerate<RangeT>;
//         type RefMutEnumerateRangeT = &mut std::iter::Enumerate<RangeT>;
//         type OptionUsizeT = std::option::Option<(usize, $T)>;
//
//         let $iter: RangeT = _;
//         // let len: usize = <RangeT as std::iter::ExactSizeIterator>::len(move iter);
//         let $len: usize = RangeT::len(move $iter);
//         let mut $vec: VecT = std::vec::Vec::with_capacity(copy $len);
//         let mut $ref_to_vec: RefMutVecT = &mut $vec;
//         let mut $ptr_to_vec: PtrMutT = Vec::as_mut_ptr(move $ref_to_vec);
//         let mut $slice: RefMutSliceT = std::slice::from_raw_parts_mut(copy $ptr_to_vec, copy
// $len);         // let mut enumerate: EnumerateRangeT = <RangeT as
// std::iter::Iterator>::enumerate(move iter);         let mut $enumerate: EnumerateRangeT =
// RangeT::enumerate(move $iter);         let mut $enumerate: RefMutEnumerateRangeT = &mut
// $enumerate;         let $next: OptionUsizeT;
//         let $cmp: isize;
//         let $first: usize;
//         let $second_t: $T;
//         let $second_usize: usize;
//         let $_tmp: ();
//         loop {
//             // next = <EnumerateRangeT as std::iter::Iterator>::next(move enumerate);
//             $next = EnumerateRangeT::next(move $enumerate);
//             // in `cmp = discriminant(copy next);`
//             // which discriminant should be used?
//             $cmp = balabala::discriminant(copy $next);
//             switchInt(move $cmp) {
//                 // true or 1 here?
//                 true => {
//                     $first = copy ($next as Some).0;
//                     $second_t = copy ($next as Some).1;
//                     $second_usize = copy $second_t as usize (IntToInt);
//                     (*$slice)[$second_usize] = copy $first as $T (IntToInt);
//                 }
//                 _ => break,
//             }
//         }
//         // variable shadowing?
//         // There cannot be two mutable references to `vec` in the same scope
//         $ref_to_vec = &mut $vec;
//         $_tmp = Vec::set_len(move $ref_to_vec, copy $len);
//     }
//
// }
//
// test_case! {
//     fn cve_2021_29941() {
//         meta!(
//             $T:ty,
//             $I:ty,
//         );
//
//         let $iter: $I = _;
//         let $len: usize = std::iter::ExactSizeIterator::len(move $iter);
//         let $vec: &mut alloc::vec::Vec<$T> = _;
//         _ = alloc::vec::Vec::set_len(move $vec, copy $len);
//     }
// }
//
// test_case! {
//     fn cve_2021_29941_uninitialized_slice() {
//         meta!(
//             $T:ty,
//         );
//
//         let $len: usize = _;
//         let $vec: alloc::vec::Vec<$T> = alloc::vec::Vec::with_capacity(_);
//         let $vec_ref: &alloc::vec::Vec<$T> = &$vec;
//         let $ptr: *const $T = alloc::vec::Vec::as_ptr(move $vec_ref);
//         let $slice: &[$T] = std::slice::from_raw_parts::<'_, $T>(move $ptr, copy $len);
//     }
// }
//
// test_case! {
//     fn unsafe_cell_alias() {
//         meta!($T:ty);
//
//         type unsafe_cell_t = core::cell::UnsafeCell<$T>;
//
//         let $a: &unsafe_cell_t = _;
//         let $b: &unsafe_cell_t = _;
//         let $raw_a: *const unsafe_cell_t = &raw const *$a;
//         let $raw_b: *const unsafe_cell_t = &raw const *$b;
//         let $raw_mut_a: *mut $T = copy $raw_a as *mut $T (PtrToPtr);
//         let $raw_mut_b: *mut $T = copy $raw_b as *mut $T (PtrToPtr);
//         let $mut_a: &mut $T = &mut *$raw_mut_a;
//         let $mut_b: &mut $T = &mut *$raw_mut_b;
//         (*$mut_a) = _;
//         (*$mut_b) = _;
//     }
// }
//
// test_case! {
//     fn control_flow() {
//         meta!();
//
//         let $a: &mut i32 = _;
//         let $f: bool = _;
//         switchInt(copy $f) {
//             false => {}
//             _ => {
//                 *$a = Add(copy (*$a), const 1_i32);
//             }
//         }
//     }
// }
//
// test_case! {
//     fn pattern_drop_unit_value()  {
//         meta!($T:ty);
//         let $raw_ptr: *mut $T = _;
//         let $value: $T = _;
//         drop((*$raw_ptr));
//         (*$raw_ptr) = move $value;
//     }
// }
