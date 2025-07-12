#![feature(rustc_private)]

#[cfg(test)]
use rpl_parser::parser::{Grammar, pairs};

macro_rules! full_test {
    ($T:ident, $input:expr $(,)?) => {{
        use pest_typed::TypedParser as _;
        let input = $input;
        match Grammar::try_parse::<pairs::$T>(input) {
            Ok(_res) => {
                // println!("{:#?}", _res);
            },
            Err(e) => {
                eprintln!("Failed to parse input :\n{}", $input);
                // eprintln!("{}", e);
                panic!("\n{}\n", e);
            },
        }
    }};
}
#[test]
fn mir_rvalue_or_call() {
    full_test!(MirRvalueCast, "copy $x as isize (IntToInt)");
    full_test!(MirRvalue, "copy $x as isize (IntToInt)");
    full_test!(MirRvalueOrCall, "copy $x as isize (IntToInt)");
    full_test!(MirRvalue, "copy (*$from_vec_mut_borrow).buf.inner.ptr.pointer");
    full_test!(MirRvalue, "copy (((((*$from_vec_mut_borrow).buf).inner).ptr).pointer)");
    full_test!(MirRvalue, "copy ((((*$x).0).0).0)");
}

#[test]
fn predicates() {
    full_test!(Predicate, "p1(self)");
    full_test!(Predicate, "p1('a, 'b, std::mem::transmute)");
    full_test!(PredicateTerm, "!p($T)");
    full_test!(PredicateClause, "!p($T)");
    full_test!(PredicateClause, "(!p($T) || p($T))");
    full_test!(PredicateConjunction, "!p($T) && p($T)");
    full_test!(PredicateConjunction, "(a() || b()) && c()");
}

#[test]
fn cve_2018_21000() {
    full_test!(
        main,
        "\
pattern CVE-2018-21000
util {
    use alloc::vec::Vec;
    use core::ptr::non_null::NonNull;
    use core::ptr::unique::Unique;
    use alloc::raw_vec::Cap;
    use alloc::raw_vec::RawVecInner;
    use alloc::raw_vec::RawVec;
    use alloc::alloc::Global;
    use core::marker::PhantomData;
    p_misordered_para[
        $T1: type,
        $T2: type,
        $T3: type,
    ] = fn _ (..) -> _ {
        let $from_vec: Vec::<$T1> = _;
        let mut $from_vec_mut_borrow: &mut Vec::<$T1> = &mut $from_vec;
        let mut $from_vec_non_null: NonNull::<u8> = copy (*$from_vec_mut_borrow).buf.inner.ptr.pointer;
        let mut $from_vec_const_ptr: *const u8 = copy ($from_vec_non_null.pointer);
        let mut $from_vec_mut_ptr: *mut u8 =copy $from_vec_const_ptr as *mut u8 (PtrToPtr);
        let mut $from_vec_inmutable_borrow: &Vec::<$T1> = &$from_vec;
        let mut $from_vec_cap: usize = copy (*$from_vec_inmutable_borrow).buf.inner.cap.0;
        let mut $from_vec_len: usize = copy (*$from_vec_inmutable_borrow).len;
        let mut $tsize: usize = SizeOf($T2);
        let mut $to_vec_cap: usize = $Op(move $from_vec_cap, copy $tsize);
        let mut $to_vec_len: usize = $Op(move $from_vec_len, copy $tsize);
        let mut $to_vec_wrong_cap: Cap = #[Ctor] Cap(copy $to_vec_len);
        let mut $to_vec_mut_ptr: *mut $T3 = copy $from_vec_mut_ptr as *mut $T3 (PtrToPtr);
        let mut $to_vec_const_ptr: *const u8 = copy $to_vec_mut_ptr as *const u8 (PtrToPtr);
        let mut $to_vec_non_null: NonNull::<u8> = NonNull::<u8> { 
            pointer: copy $to_vec_const_ptr 
        };
        let mut $to_vec_unique: Unique::<u8> = Unique::<u8> { 
            pointer: move $to_vec_non_null, 
            _marker: const PhantomData::<u8>
        };
        let mut $to_vec_raw_inner: RawVecInner = RawVecInner { 
            ptr: move $to_vec_unique, 
            cap: copy $to_vec_wrong_cap, 
            alloc: const Global
        };
        let mut $to_vec_raw: RawVec::<$T3> = RawVec::<$T3> {
            inner: move $to_vec_raw_inner, 
            _marker: const PhantomData::<$T3> 
        };
        let mut $to_vec: Vec::<$T3> = Vec::<$T3> { 
            buf: move $to_vec_raw, 
            len: copy $to_vec_cap 
        };
    }
}
patt {
    p1[$T: type] = p_reversed_para[
        $T1 = u8,
        $T2 = $T,
        $T3 = $T,
        // $Op = Div
    ]
    p2[$T: type] = p_reversed_para[
        $T1 = $T,
        $T2 = $T,
        $T3 = u8,
        //  $Op = Mul
    ]
}
"
    );
}
#[test]
fn cve_2019_15548() {
    full_test!(
        main,
        "\
pattern CVE-2019-15548-MIR
patt {
    use ncurses::instr;
    use libc::c_char;
    use std::string::String;
    // Only work for crate::ll::instr or ncures::instr
    p1[
        $T: type,
    ] = fn _ (..) -> _ {
        let $src: &String = _;
        let $bytes: &[u8] = String::as_bytes(move $src);
        let $ptr: *const u8 = slice::as_ptr(copy $bytes);
        let $dst: *const c_char = copy $ptr as *const c_char (Transmute);
        let $ret: $T = $crate::ll::instr(move $dst);
    }
    // Pass a string ptr to $c_func
    p2 = fn _ (..) -> _ {
        let $ptr: *const c_char = _;
        _ = $c_func(move $ptr);
    }
}
"
    );
}
#[test]
fn cve_2019_16138() {
    full_test!(
        main,
        "\
pattern CVE-2019-16138
patt {
    use std::vec::Vec;
    p[
        $T: type
    ] = pub fn _ (..) -> _ {
        let $vec: Vec<$T> = std::vec::Vec::with_capacity(_);
        let $vec_ref: &mut Vec<$T> = &mut $vec;
        _ = std::vec::Vec::set_len(move $vec_ref, _);
    }
}
"
    );
}
#[test]
fn cve_2020_25016() {
    full_test!(
        main,
        "\
pattern CVE-2020-25016
patt {
    p_unsound_cast_const[
        $T: type
    ] = fn _ (..) -> _ {
        let $from_slice: &[$T] = _;
        let $from_raw: *const [$T] = &raw const *$from_slice;
        let $from_len: usize = PtrMetadata(copy $from_slice);
        let $ty_size: usize = SizeOf($T);
        let $to_ptr: *const u8 = copy $from_raw as *const u8 (PtrToPtr);
        let $to_len: usize = Mul(move $from_len, move $ty_size);
        let $to_raw: *const [u8] = *const [u8] from (copy $to_ptr, copy $to_len);
        let $to_slice: &[u8] = &*$to_raw;
    } where {
        safety = safe
    }
    p_unsound_cast_mut[
        $T: type
    ] = fn _ (..) -> _ {
        let $from_slice_mut: &mut [$T] = _;
        let $from_raw_mut: *mut [$T] = &raw mut *$from_slice_mut;
        let $from_len_mut: usize = PtrMetadata(copy $from_slice_mut);
        let $ty_size_mut: usize = SizeOf($T);
        let $to_ptr_mut: *mut u8 = copy $from_raw_mut as *mut u8 (PtrToPtr);
        let $to_len_mut: usize = Mul(move $from_len_mut, move $ty_size_mut);
        let $to_raw_mut: *mut [u8] = *mut [u8] from (copy $to_ptr_mut, copy $to_len_mut);
        let $to_slice_mut: &mut [u8] = &mut *$to_raw_mut;
    } where {
        safety = safe
    }
}
"
    );
}
#[test]
fn cve_2020_35881() {
    full_test!(
        main,
        "\
pattern CVE-2020-35881
patt {
    p_wrong_assumption_of_fat_pointer_layout_const_const = fn _ (..) -> _ {
        let $ptr: *const $T = _;
        let $ref_to_ptr: &*const $T = &$ptr;
        let $ptr_to_ptr_t: *const *const $T = &raw const (*$ref_to_ptr);
        let $ptr_to_ptr: *const *const () = move $ptr_to_ptr_t as *const *const () (Transmute);
        let $data_ptr: *const () = _;
    }
    p_wrong_assumption_of_fat_pointer_layout_const_mut = fn _ (..) -> _ {
        let $ptr: *const $T = _;
        let $ref_to_ptr: &mut *const $T = &mut $ptr;
        let $ptr_to_ptr_t: *mut *const $T = &raw mut (*$ref_to_ptr);
        let $ptr_to_ptr: *mut *mut () = move $ptr_to_ptr_t as *mut *mut () (Transmute);
        let $data_ptr: *mut () = _;
    }
    
    p_wrong_assumption_of_fat_pointer_layout_mut_const = fn _ (..) -> _ {
        let $ptr: *mut $T = _;
        let $ref_to_ptr: &*mut $T = &$ptr;
        let $ptr_to_ptr_t: *const *mut $T = &raw const (*$ref_to_ptr);
        let $ptr_to_ptr: *const *mut () = move $ptr_to_ptr_t as *const *mut () (Transmute);
        let $data_ptr: *const () = _;
    }
    
    p_wrong_assumption_of_fat_pointer_layout_mut_mut = fn _ (..) -> _ {
        let $ptr: *mut $T = _;
        let $ref_to_ptr: &mut *mut $T = &mut $ptr;
        let $ptr_to_ptr_t: *mut *mut $T = &raw mut (*$ref_to_ptr);
        let $ptr_to_ptr: *mut *mut () = move $ptr_to_ptr_t as *mut *mut () (Transmute);
        let $data_ptr: *mut () = _;
    }
}
"
    );
}
#[test]
fn cve_2020_35888() {
    full_test!(
        main,
        "\
pattern CVE-2020-35888
patt {
    p_move[
        $T: type,
    ] = pub fn _ (..) -> _ {
        let $raw_ptr: *mut $T = _;
        let $value: $T = _;
        drop((*$raw_ptr));
        (*$raw_ptr) = move $value;
    }
}
"
    );
}
#[test]
fn cve_2020_35892_35893() {
    full_test!(
        main,
        r#"
pattern CVE-2020-35892-3
patt {
    use core::ops::range::Range<usize>;
    // how to express constraints on meta variables?
    // 1. use a where clause (how to define a where clause?)
    // 2. use a predicate (how to define a predicate?)
    p[
        $T: type,
        $SlabT: type
    ]
    = fn _ (..) -> _ {
        let $self: &mut $SlabT;
        let $len: usize = copy (*$self).len;
        let $range: Range<usize> = Range { start: const 0_usize, end: move $len };
        let $iter: Range<usize> = move $range;
        let $iter_mut: &mut Range<usize> = &mut $iter;
        let $start_ref: &usize = &(*$iter_mut).start;
        let $start: usize = copy *$start_ref;
        let $end_ref: &usize = &(*$iter_mut).end;
        let $end: usize = copy *$end_ref;
        let $cmp: bool = Lt(move $start, move $end);
        let $opt: Option<usize>;
        let $x1: usize;
        let $x2: usize;
        let $x: usize;
        let $base: *mut $T;
        let $offset: isize;
        let $elem_ptr: *mut $T;
        let $discr: isize;
        loop {
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
}
"#
    );
}
#[test]
fn cve_2020_35898_9() {
    full_test!(
        main,
        r#"
pattern CVE-2020-35898-9
patt {
    use std::cell::UnsafeCell;
    use std::rc::Rc;
    use std::rc::RcInner;
    p_rc_unsafe_cell[
        $T: type
    ] = {           
        pub struct $Cell {
            $inner: Rc<UnsafeCell<$T>>,
        }
        impl $Cell {
            fn $get_mut(..) -> _ {
                let $self: &mut $Cell = _;
                let $inner_ref: &Rc<UnsafeCell<$T>> = &(*$self).$inner;
                let $inner_ptr: RcInner<UnsafeCell<$T>> = copy (*$inner_ref).ptr;
                let $const_ptr: *const RcInner<UnsafeCell<$T>> = copy $inner_ptr.pointer;
                let $unsafe_cell: &UnsafeCell<$T> = &(*$const_ptr).value;
                let $unsafe_cell_ptr: *const UnsafeCell<$T> = &raw const (*$unsafe_cell);
                let $value_ptr: *mut $T = copy $unsafe_cell_ptr as *mut $T (PtrToPtr);
                let $value_mut_ref: &mut $T = &mut (*$value_ptr);
            }
        }
    }
}
// detection after monomorpization?
"#
    );
}
#[test]
fn cve_2021_27376() {
    full_test!(
        main,
        r#"
pattern CVE-2021-27376
patt {
    use std::net::SocketAddrV6;
    use libc::socketaddr;
    p_const_const_ver = fn _ (..) -> _ {
        let $src: *const SocketAddrV6 = _;
        let $dst: *const socketaddr = move $src as *const socketaddr (PtrToPtr);
    }
    p_mut_const_ver = fn _ (..) -> _ {
        let $src: *mut SocketAddrV6 = _;
        let $dst: *const socketaddr = move $src as *const socketaddr (PtrToPtr);
    }
    
    p_const_mut_ver = fn _ (..) -> _ {
        let $src: *const SocketAddrV6 = _;
        let $dst: *mut socketaddr = move $src as *mut socketaddr (PtrToPtr);
    }
    
    p_mut_mut_ver = fn _ (..) -> _ {
        let $src: *mut SocketAddrV6 = _;
        let $dst: *mut socketaddr = move $src as *mut socketaddr (PtrToPtr);
    }
}
"#
    );
}
#[test]
fn cve_2020_35873() {
    full_test!(
        main,
        r#"
pattern CVE-2021-35873
patt {
    use alloc::ffi::c_str::CString;
    use core::ffi::c_str::CStr;
    use core::ptr::non_null::NonNull;
    p = pub fn _ (..) -> _ {
        let $cstring: CString = _;
        let $cstring_ref: &CString = &$cstring;
        let $non_null: NonNull<[u8]> = copy ((((*$cstring_ref).inner).0).pointer);
        let $uslice_ptr: *const [u8] = copy $non_null.pointer;
        let $cstr_ptr: *const CStr = copy $uslice_ptr as *const CStr (PtrToPtr);
        let $cstr: &CStr = &(*$cstr_ptr);
        let $islice: *const [i8] = &raw const ((*$cstr).inner);
        let $iptr: *const i8 = move $islice as *const i8 (PtrToPtr);
        let $iptr_arg: *const i8;
        let $s: i32;
        drop($cstring);
        $s = _;
        $iptr_arg = copy $iptr;
        _ = $crate::ffi::sqlite3session_attach(move $s, move $iptr_arg);
    }
}
"#
    );
}
#[test]
fn cve_2024_27284() {
    full_test!(
        main,
        r#"
pattern CVE-2024-27284
patt {
    use std::iter::Iterator;
    use cassandra_cpp_sys::CassIterator;
    use cassandra_cpp_sys::cassandra::case_iterator_next;
    use cassandra_cpp_sys::cassandra::cass_iterator_get_row;
    use cassandra_cpp_sys::cassandra::case_bool_t;
    p_incorrect_iterator_impl[
        $T: type,
        $Item: type
    ] = impl Iterator for $T {
        fn next(..) -> _ {
            let $mut_iter: *mut CassIterator = _;
            let $next_res: cass_bool_t = case_iterator_next(copy $mut_iter);
            let $discr: u32 = discriminant($next_res);
            let $const_iter: *const CassIterator;
            let $item: $Item;
            switchInt(move $discr) {
                1 => {
                    $const_iter = move $mut_iter as *const CassIterator(PtrToPtr);
                    $item = cass_iterator_get_row(move $const_iter);
                }
                _ => {
                    // how to express I don't care about the statements in this block?
                }
            }
        }
    }
}
"#
    );
}

#[test]
fn cve_2020_35860() {
    full_test!(
        main,
        r#"
pattern CVE-2020-35860

diag {
    p_deref = {
        help = "The public struct $CBox contains a raw pointer ($ptr) to a type $T.",
        help = "Its `Deref` implementation dereferences the pointer without null checks.",
        help = "Specifically, the `Deref` implementation calls `CStr::from_ptr(self.$ptr)`",
        help = "whose safety requirements include that the pointer must be non-null.",
        label(CBox) = "$CBox is defined here",
        label(ptr)  = "$ptr is defined here",
    }
}
"#
    );
}

#[test]
fn mir_cast_kind() {
    full_test!(MirCastKind, "Transmute")
}

#[test]
fn mir_operand() {
    full_test!(MirOperand, "move $p");
    full_test!(MirOperand, "copy $p");
    full_test!(MirOperand, "copy (*$p)");
    full_test!(MirOperand, "move $p as DstVec");
}

#[test]
fn ty() {
    full_test!(Type, "DstVec");
    full_test!(Type, "$T");
}

#[test]
fn rvalue_cast() {
    full_test!(MirRvalueCast, "move $p as *const u8 (Transmute)");
    full_test!(MirRvalueCast, "move $p as DstVec (Transmute)");
    full_test!(MirRvalueCast, "move $p as $T (Transmute)");
}

#[test]
fn mir_local_decl() {
    full_test!(MirLocalDecl, "let $q: DstVec = move $p as DstVec (Transmute);");
}
