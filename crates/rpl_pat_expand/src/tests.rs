use pretty_assertions::assert_eq;
use proc_macro2::TokenStream;
use quote::quote;

#[track_caller]
fn mir_test_case(input: TokenStream, output: TokenStream) {
    let pattern = syn::parse2(input).unwrap();
    let expanded = crate::expand_pattern(&pattern, None).unwrap_or_else(syn::Error::into_compile_error);
    assert_eq!(
        expanded.to_string().replace(";", ";\n"),
        output.to_string().replace(";", ";\n")
    );
}

macro_rules! mir_test_case {
    ( #[meta($($meta:tt)*)] pat!{ $( $tt:tt )* } => {
        meta! {$($meta_output:tt)*}
        $($output:tt)*
    } $(,)? ) => {
        mir_test_case(
            quote! {
                #[meta($($meta)*)]
                fn $pattern (..) -> _ = mir! {
                    $($tt)*
                }
            }, quote!{{
                let pattern = pcx.new_pattern();
                let pattern_fn = pattern.fns.new_fn_pat(::rustc_span::Symbol::intern("pattern"));
                $($meta_output)*
                pattern_fn.set_ret_ty(pcx.mk_any_ty());
                let mut mir_pat = ::rpl_context::pat::MirPattern::builder();
                $($output)*
                let mir_pat = mir_pat.build();
                let mir_pat = pcx.mk_mir_pattern(mir_pat);
                pattern_fn.set_body(::rpl_context::pat::FnBody::Mir(mir_pat));
                &*pattern
            }}
        )
    };
}

#[track_caller]
fn test_case(input: TokenStream, output: TokenStream) {
    let pattern = syn::parse2(input).unwrap();
    let expanded = crate::expand_pattern(&pattern, None).unwrap_or_else(syn::Error::into_compile_error);
    assert_eq!(
        expanded.to_string().replace(";", ";\n"),
        quote! {{
            let pattern = pcx.new_pattern();
            #output
            &*pattern
        }}
        .to_string()
        .replace(";", ";\n")
    );
}

macro_rules! test_case {
    (pat!{ $( $tt:tt )* } => $($output:tt)*) => {
        test_case(quote!($($tt)*), $($output)*)
    };
}

#[test]
fn test_ty_var() {
    mir_test_case!(
        #[meta($T:ty)]
        pat! {
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
            }
        }
    );
}

#[test]
fn test_ty_var_preds() {
    mir_test_case!(
        #[meta($T:ty where a || b)]
        pat! {
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[
                    &[
                        either::Either::Left(a),
                        either::Either::Left(b),
                    ],
                ]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
            }
        }
    );
}

#[test]
fn test_const_var() {
    mir_test_case!(
        #[meta($T:ty, $size: const(usize), $src: const($T))]
        pat! {
            let $array: [$T; $size] = _;
            let $slice: &[$T; $size] = &$array;
            let $buffer: std::alloc::Layout = std::alloc::Layout::new_unchecked(const $size, const $size);
            (*$buffer) = const $src;
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
                #[allow(non_snake_case)]
                let size_const_var = pattern_fn.meta.new_const_var(pcx.primitive_types.usize);
                #[allow(non_snake_case)]
                #[allow(unused_variables)]
                let size_const = pcx.mk_var_const(size_const_var);
                #[allow(non_snake_case)]
                let src_const_var = pattern_fn.meta.new_const_var(T_ty);
                #[allow(non_snake_case)]
                #[allow(unused_variables)]
                let src_const = pcx.mk_var_const(src_const_var);
            }
            let array_local = mir_pat.mk_local(pcx.mk_array_ty(T_ty, size_const));
            mir_pat.mk_assign(array_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let slice_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_array_ty(T_ty, size_const),
                ::rustc_middle::mir::Mutability::Not
            ));
            mir_pat.mk_assign(
                slice_local.into_place(),
                ::rpl_context::pat::Rvalue::Ref(
                    ::rpl_context::pat::RegionKind::ReAny,
                    ::rustc_middle::mir::BorrowKind::Shared,
                    array_local.into_place()
                )
            );
            let buffer_local =
                mir_pat.mk_local(pcx.mk_path_ty(pcx.mk_path_with_args(pcx.mk_item_path(&["std", "alloc", "Layout", ]), &[])));
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(
                    mir_pat
                        .mk_zeroed(
                            pcx.mk_path_with_args(pcx.mk_item_path(&["std", "alloc", "Layout", "new_unchecked", ]), &[])
                        )
                ),
                mir_pat.mk_list([
                    ::rpl_context::pat::Operand::Constant(mir_pat.mk_const_var(size_const_var)),
                    ::rpl_context::pat::Operand::Constant(mir_pat.mk_const_var(size_const_var))
                ]),
                Some(buffer_local.into_place())
            );
            mir_pat.mk_assign(
                ::rpl_context::pat::Place::new(buffer_local, pcx.mk_slice(&[::rpl_context::pat::PlaceElem::Deref, ])),
                ::rpl_context::pat::Rvalue::Use(
                    ::rpl_context::pat::Operand::Constant(
                        mir_pat.mk_const_var(src_const_var)
                    )
                )
            );
        }
    );
}

#[test]
fn test_place_var() {
    mir_test_case!(
        #[meta($T: ty, #[export(src)] $src: place($T))]
        pat! {
            let $reference: &$T = &$src;
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
                #[allow(non_snake_case)]
                let src_place_var = pattern_fn.meta.new_place_var(T_ty);
                #[allow(non_snake_case)]
                let src_local = pcx.mk_var_place(src_place_var);
                src = src_place_var;
            }
            let reference_local = mir_pat.mk_local(
                pcx
                    .mk_ref_ty(
                        ::rpl_context::pat::RegionKind::ReAny,
                        T_ty,
                        ::rustc_middle::mir::Mutability::Not
                    )
            );
            mir_pat.mk_assign(
                reference_local.into_place(),
                ::rpl_context::pat::Rvalue::Ref(
                    ::rpl_context::pat::RegionKind::ReAny,
                    ::rustc_middle::mir::BorrowKind::Shared,
                    src_local.into_place()
                )
            );
        }
    );
}

#[test]
fn test_coercion() {
    mir_test_case!(
        #[meta($T: ty)]
        pat! {
            let $reference: &[$T; 2] = _;
            let $reference: &[$T] = copy $reference as &[$T] (PointerCoercion(Unsize, Implicit));
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
            }
            let reference_local = mir_pat.mk_local(
                pcx.mk_ref_ty(
                    ::rpl_context::pat::RegionKind::ReAny,
                    pcx.mk_array_ty(
                        T_ty,
                        ::rpl_context::pat::ConstOperand::ScalarInt(2.into())),
                        ::rustc_middle::mir::Mutability::Not
                    )
                );
            mir_pat.mk_assign(
                reference_local.into_place(),
                ::rpl_context::pat::Rvalue::Any
            );
            let reference_local = mir_pat.mk_local(
                pcx.mk_ref_ty(
                    ::rpl_context::pat::RegionKind::ReAny,
                    pcx.mk_slice_ty(T_ty),
                    ::rustc_middle::mir::Mutability::Not
                )
            );
            mir_pat.mk_assign(
                reference_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::PointerCoercion(
                        rustc_middle::ty::adjustment::PointerCoercion::Unsize,
                        ::rustc_middle::mir::CoercionSource::Implicit
                    ),
                    ::rpl_context::pat::Operand::Copy(
                        reference_local.into_place()
                    ),
                    pcx.mk_ref_ty(
                        ::rpl_context::pat::RegionKind::ReAny,
                        pcx.mk_slice_ty(T_ty),
                        ::rustc_middle::mir::Mutability::Not
                    )
                )
            );
        }
    );
}

#[test]
fn test_cve_2020_25016() {
    mir_test_case!(
        #[meta( #[export(ty_var)] $T:ty where is_all_safe_trait)]
        pat! {
            type SliceT = [$T];
            type RefSliceT = &SliceT;
            type PtrSliceT = *const SliceT;
            type PtrU8 = *const u8;
            type SliceU8 = [u8];
            type PtrSliceU8 = *const SliceU8;
            type RefSliceU8 = &SliceU8;

            #[export(cast_from)]
            let $from_slice: SliceT = _;
            let $from_raw_slice: PtrSliceT = &raw const *$from_slice;
            let $from_len: usize = Len($from_slice);
            let $ty_size: usize = SizeOf($T);
            let $to_ptr: PtrU8 = copy $from_raw_slice as PtrU8 (PtrToPtr);
            let $to_len: usize = Mul(copy $from_len, copy $ty_size);
            let $to_raw_slice: PtrSliceU8 = *const SliceU8 from (copy $to_ptr, copy $to_len);
            #[export(cast_to)]
            let $to_slice: RefSliceU8 = &*$to_raw_slice;
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[
                    &[
                        either::Either::Left(is_all_safe_trait),
                    ],
                ]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
                ty_var = T_ty_var;
            }
            #[allow(non_snake_case)]
            let SliceT_ty = pcx.mk_slice_ty(T_ty);
            #[allow(non_snake_case)]
            let RefSliceT_ty = pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                SliceT_ty,
                ::rustc_middle::mir::Mutability::Not
            );
            #[allow(non_snake_case)]
            let PtrSliceT_ty = pcx.mk_raw_ptr_ty(SliceT_ty, ::rustc_middle::mir::Mutability::Not);
            #[allow(non_snake_case)]
            let PtrU8_ty = pcx.mk_raw_ptr_ty(
                pcx.primitive_types.u8, ::rustc_middle::mir::Mutability::Not
            );
            #[allow(non_snake_case)]
            let SliceU8_ty = pcx.mk_slice_ty(pcx.primitive_types.u8);
            #[allow(non_snake_case)]
            let PtrSliceU8_ty = pcx.mk_raw_ptr_ty(SliceU8_ty, ::rustc_middle::mir::Mutability::Not);
            #[allow(non_snake_case)]
            let RefSliceU8_ty = pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                SliceU8_ty,
                ::rustc_middle::mir::Mutability::Not
            );
            let from_slice_local = mir_pat.mk_local(SliceT_ty);
            cast_from = mir_pat.mk_assign(from_slice_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let from_raw_slice_local = mir_pat.mk_local(PtrSliceT_ty);
            mir_pat.mk_assign(
                from_raw_slice_local.into_place(),
                ::rpl_context::pat::Rvalue::RawPtr(
                    ::rustc_middle::mir::Mutability::Not,
                    ::rpl_context::pat::Place::new(
                        from_slice_local,
                        pcx.mk_slice(&[::rpl_context::pat::PlaceElem::Deref,])
                    )
                )
            );
            let from_len_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_assign(
                from_len_local.into_place(),
                ::rpl_context::pat::Rvalue::Len(from_slice_local.into_place())
            );
            let ty_size_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_assign(
                ty_size_local.into_place(),
                ::rpl_context::pat::Rvalue::NullaryOp(::rustc_middle::mir::NullOp::SizeOf, T_ty)
            );
            let to_ptr_local = mir_pat.mk_local(PtrU8_ty);
            mir_pat.mk_assign(
                to_ptr_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::PtrToPtr,
                    ::rpl_context::pat::Operand::Copy(from_raw_slice_local.into_place()),
                    PtrU8_ty
                )
            );
            let to_len_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_assign(
                to_len_local.into_place(),
                ::rpl_context::pat::Rvalue::BinaryOp(
                    ::rustc_middle::mir::BinOp::Mul,
                    Box::new([
                        ::rpl_context::pat::Operand::Copy(from_len_local.into_place()),
                        ::rpl_context::pat::Operand::Copy(ty_size_local.into_place())
                    ])
                )
            );
            let to_raw_slice_local = mir_pat.mk_local(PtrSliceU8_ty);
            mir_pat.mk_assign(
                to_raw_slice_local.into_place(),
                ::rpl_context::pat::Rvalue::Aggregate(
                    ::rpl_context::pat::AggKind::RawPtr(SliceU8_ty, ::rustc_middle::mir::Mutability::Not),
                    mir_pat.mk_list([
                        ::rpl_context::pat::Operand::Copy(to_ptr_local.into_place()),
                        ::rpl_context::pat::Operand::Copy(to_len_local.into_place())
                    ])
                )
            );
            let to_slice_local = mir_pat.mk_local(RefSliceU8_ty);
            cast_to = mir_pat.mk_assign(
                to_slice_local.into_place(),
                ::rpl_context::pat::Rvalue::Ref(
                    ::rpl_context::pat::RegionKind::ReAny,
                    ::rustc_middle::mir::BorrowKind::Shared,
                    ::rpl_context::pat::Place::new(
                        to_raw_slice_local,
                        pcx.mk_slice(&[::rpl_context::pat::PlaceElem::Deref,])
                    )
                )
            );
        }
    );
}

#[test]
fn test_cve_2020_35877_unchecked_offset_const() {
    mir_test_case!(
        #[meta($T:ty, $size: const(usize), $offset: const(usize))]
        pat! {
            let $array: &[$T; $size] = _; // _1
            let $slice_ref: &[$T] = copy $array as &[$T] (PointerCoercion(Unsize, Implicit)); // _3 bb0[0]
            let $slice_ptr: *const [$T] = &raw const (*$slice_ref); // _5 bb0[1]
            #[export(ptr)]
            let $ptr: *const $T = move $slice_ptr as *const $T (PtrToPtr); // _2 bb0[2]
            let $offset: usize = const $offset; // _6 bb0[3]
            #[export(offset)]
            let $ptr_1: *const $T = Offset(copy $ptr, copy $offset); // _4 bb0[4]
            let $value: &$T = &(*$ptr_1); // _0 bb0[5]
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
                #[allow(non_snake_case)]
                let size_const_var = pattern_fn.meta.new_const_var(pcx.primitive_types.usize);
                #[allow(non_snake_case)]
                #[allow (unused_variables)]
                let size_const = pcx.mk_var_const(size_const_var);
                #[allow(non_snake_case)]
                let offset_const_var = pattern_fn.meta.new_const_var(pcx.primitive_types.usize);
                #[allow(non_snake_case)]
                #[allow(unused_variables)]
                let offset_const = pcx.mk_var_const(offset_const_var);
            }
            let array_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_array_ty(T_ty, size_const),
                ::rustc_middle::mir::Mutability::Not
            ));
            mir_pat.mk_assign(array_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let slice_ref_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_slice_ty(T_ty),
                ::rustc_middle::mir::Mutability::Not
            ));
            mir_pat.mk_assign(
                slice_ref_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::PointerCoercion(
                        rustc_middle::ty::adjustment::PointerCoercion::Unsize,
                        ::rustc_middle::mir::CoercionSource::Implicit
                    ),
                    ::rpl_context::pat::Operand::Copy(array_local.into_place()),
                    pcx.mk_ref_ty(
                        ::rpl_context::pat::RegionKind::ReAny,
                        pcx.mk_slice_ty(T_ty),
                        ::rustc_middle::mir::Mutability::Not
                    )
                )
            );
            let slice_ptr_local =
                mir_pat.mk_local(pcx.mk_raw_ptr_ty(pcx.mk_slice_ty(T_ty), ::rustc_middle::mir::Mutability::Not));
            mir_pat.mk_assign(
                slice_ptr_local.into_place(),
                ::rpl_context::pat::Rvalue::RawPtr(
                    ::rustc_middle::mir::Mutability::Not,
                    ::rpl_context::pat::Place::new(
                        slice_ref_local,
                        pcx.mk_slice(&[
                            ::rpl_context::pat::PlaceElem::Deref,
                        ])
                    )
                )
            );
            let ptr_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Not));
            ptr = mir_pat.mk_assign(
                ptr_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::PtrToPtr,
                    ::rpl_context::pat::Operand::Move(slice_ptr_local.into_place()),
                    pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Not)
                )
            );
            let offset_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_assign(
                offset_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Constant(
                    mir_pat.mk_const_var(offset_const_var)
                ))
            );
            let ptr_1_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Not));
            offset = mir_pat.mk_assign(
                ptr_1_local.into_place(),
                ::rpl_context::pat::Rvalue::BinaryOp(
                    ::rustc_middle::mir::BinOp::Offset,
                    Box::new([
                        ::rpl_context::pat::Operand::Copy(ptr_local.into_place()),
                        ::rpl_context::pat::Operand::Copy(offset_local.into_place())
                    ])
                )
            );
            let value_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                T_ty,
                ::rustc_middle::mir::Mutability::Not
            ));
            mir_pat.mk_assign(
                value_local.into_place(),
                ::rpl_context::pat::Rvalue::Ref(
                    ::rpl_context::pat::RegionKind::ReAny,
                    ::rustc_middle::mir::BorrowKind::Shared,
                    ::rpl_context::pat::Place::new(ptr_1_local, pcx.mk_slice(&[::rpl_context::pat::PlaceElem::Deref, ]))
                )
            );
        }
    );
}

#[test]
fn test_cve_2020_35892_revised() {
    mir_test_case!(
        #[meta($T:ty, $SlabT:ty)]
        pat! {
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
                $end = copy *$end;
                $cmp = Lt(move $start, copy $end);
                switchInt(move $cmp) {
                    false => $opt = #[lang = "None"],
                    _ => {
                        $x1 = copy (*$iter_mut).start;
                        $x2 = core::iter::range::Step::forward_unchecked(copy $x1, const 1_usize);
                        (*$iter_mut).start = copy $x2;
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
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);

                #[allow(non_snake_case)]
                let SlabT_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let SlabT_ty = pcx.mk_var_ty(SlabT_ty_var);
            }

            let self_local = mir_pat.mk_self(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                SlabT_ty,
                ::rustc_middle::mir::Mutability::Mut
            ));
            let len_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let x1_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let x2_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let opt_local = mir_pat.mk_local(pcx.mk_adt_ty(pcx.mk_path_with_args(
                pcx.mk_lang_item("Option"),
                &[pcx.primitive_types.usize.into()]
            )));
            let discr_local = mir_pat.mk_local(pcx.primitive_types.isize);
            let x_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let start_ref_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.primitive_types.usize,
                ::rustc_middle::mir::Mutability::Not
            ));
            let end_ref_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.primitive_types.usize,
                ::rustc_middle::mir::Mutability::Not
            ));
            let start_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let end_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let range_local = mir_pat.mk_local(pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["core", "ops", "range", "Range",]),
                &[pcx.primitive_types.usize.into(),]
            )));
            let iter_local = mir_pat.mk_local(pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["core", "ops", "range", "Range",]),
                &[pcx.primitive_types.usize.into(),]
            )));
            let iter_mut_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_path_ty(pcx.mk_path_with_args(
                    pcx.mk_item_path(&["core", "ops", "range", "Range",]),
                    &[pcx.primitive_types.usize.into(),]
                )),
                ::rustc_middle::mir::Mutability::Mut
            ));
            let base_local =
                mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Mut));
            let offset_local = mir_pat.mk_local(pcx.primitive_types.isize);
            let elem_ptr_local =
                mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Mut));
            let cmp_local = mir_pat.mk_local(pcx.primitive_types.bool);
            mir_pat.mk_assign(
                len_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(::rpl_context::pat::Place::new(
                    self_local,
                    pcx.mk_slice(&[
                        ::rpl_context::pat::PlaceElem::Deref,
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(
                            ::rustc_span::Symbol::intern("len")
                        )),
                    ])
                )))
            );
            mir_pat.mk_assign(
                range_local.into_place(),
                ::rpl_context::pat::Rvalue::Aggregate(
                    ::rpl_context::pat::AggKind::Adt(
                        pcx.mk_path_with_args(
                            pcx.mk_item_path(&["core", "ops", "range", "Range",]),
                            &[]
                        ),
                        mir_pat.mk_list([
                            ::rustc_span::Symbol::intern("start"),
                            ::rustc_span::Symbol::intern("end")
                        ]).into()
                    ),
                    mir_pat.mk_list([
                        ::rpl_context::pat::Operand::Constant(
                            ::rpl_context::pat::ConstOperand::ScalarInt(0_usize.into())
                        ),
                        ::rpl_context::pat::Operand::Move(len_local.into_place())
                    ]),
                )
            );
            mir_pat.mk_assign(
                iter_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Move(range_local.into_place()))
            );
            mir_pat.mk_loop(|mir_pat| {
                mir_pat.mk_assign(
                    iter_mut_local.into_place(),
                    ::rpl_context::pat::Rvalue::Ref(
                        ::rpl_context::pat::RegionKind::ReAny,
                        ::rustc_middle::mir::BorrowKind::Mut {
                            kind: ::rustc_middle::mir::MutBorrowKind::Default
                        },
                        iter_local.into_place()
                    ));
                mir_pat.mk_assign(
                    start_ref_local.into_place(),
                    ::rpl_context::pat::Rvalue::Ref(
                        ::rpl_context::pat::RegionKind::ReAny,
                        ::rustc_middle::mir::BorrowKind::Shared,
                        ::rpl_context::pat::Place::new(iter_mut_local, pcx.mk_slice(&[
                            ::rpl_context::pat::PlaceElem::Deref,
                            ::rpl_context::pat::PlaceElem::Field(
                                ::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("start"))
                            ),
                        ]))
                    )
                );
                mir_pat.mk_assign(
                    start_local.into_place(),
                    ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(
                        ::rpl_context::pat::Place::new(
                            start_ref_local,
                            pcx.mk_slice(&[::rpl_context::pat::PlaceElem::Deref,])
                        )
                    ))
                );
                mir_pat.mk_assign(
                    end_ref_local.into_place(),
                    ::rpl_context::pat::Rvalue::Ref(
                        ::rpl_context::pat::RegionKind::ReAny,
                        ::rustc_middle::mir::BorrowKind::Shared,
                        ::rpl_context::pat::Place::new(
                            iter_mut_local,
                            pcx.mk_slice(&[
                                ::rpl_context::pat::PlaceElem::Deref,
                                ::rpl_context::pat::PlaceElem::Field(
                                    ::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("end"))
                                ),
                            ])
                        )
                    )
                );
                mir_pat.mk_assign(
                    end_local.into_place(),
                    ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(
                        ::rpl_context::pat::Place::new(
                            end_local,
                            pcx.mk_slice(&[::rpl_context::pat::PlaceElem::Deref,])
                        )
                    ))
                );
                mir_pat.mk_assign(
                    cmp_local.into_place(),
                    ::rpl_context::pat::Rvalue::BinaryOp(
                        ::rustc_middle::mir::BinOp::Lt,
                        Box::new([
                            ::rpl_context::pat::Operand::Move(start_local.into_place()),
                            ::rpl_context::pat::Operand::Copy(end_local.into_place())
                        ])
                    )
                );
                mir_pat.mk_switch_int(::rpl_context::pat::Operand::Move(cmp_local.into_place()), |mut mir_pat| {
                    mir_pat.mk_switch_target(false, |mir_pat| {
                        mir_pat.mk_assign(
                            opt_local.into_place(),
                            ::rpl_context::pat::Rvalue::Aggregate(
                                ::rpl_context::pat::AggKind::Adt(
                                    pcx.mk_path_with_args(pcx.mk_lang_item("None"), &[]),
                                    ::rpl_context::pat::AggAdtKind::Unit
                                ),
                                Box::new([])
                            )
                        );
                    });
                    mir_pat.mk_otherwise(|mir_pat| {
                        mir_pat.mk_assign(
                            x1_local.into_place(),
                            ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(
                                ::rpl_context::pat::Place::new(
                                    iter_mut_local,
                                    pcx.mk_slice(&[
                                        ::rpl_context::pat::PlaceElem::Deref,
                                        ::rpl_context::pat::PlaceElem::Field(
                                            ::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("start"))
                                        ),
                                    ])
                                )
                            ))
                        );
                        mir_pat.mk_fn_call(
                            ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(pcx.mk_path_with_args(
                                pcx.mk_item_path(&["core", "iter", "range", "Step", "forward_unchecked",]),
                                &[]
                            ))),
                            mir_pat.mk_list([
                                ::rpl_context::pat::Operand::Copy(x1_local.into_place()),
                                ::rpl_context::pat::Operand::Constant(
                                    ::rpl_context::pat::ConstOperand::ScalarInt(1_usize.into())
                                )
                            ]),
                            Some(x2_local.into_place())
                        );
                        mir_pat.mk_assign(
                            ::rpl_context::pat::Place::new(
                                iter_mut_local,
                                pcx.mk_slice(&[
                                    ::rpl_context::pat::PlaceElem::Deref,
                                    ::rpl_context::pat::PlaceElem::Field(
                                        ::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("start"))
                                    ),
                                ])
                            ),
                            ::rpl_context::pat::Rvalue::Use(
                                ::rpl_context::pat::Operand::Copy(x2_local.into_place())
                            )
                        );
                        mir_pat.mk_fn_call(
                            ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                                pcx.mk_path_with_args(pcx.mk_lang_item("Some"), &[])
                            )),
                            mir_pat.mk_list([
                                ::rpl_context::pat::Operand::Copy(x1_local.into_place())
                            ]),
                            Some(opt_local.into_place())
                        );
                    });
                });
                mir_pat.mk_assign(
                    discr_local.into_place(),
                    ::rpl_context::pat::Rvalue::Discriminant(opt_local.into_place())
                );
                mir_pat.mk_switch_int(
                    ::rpl_context::pat::Operand::Move(discr_local.into_place()),
                    |mut mir_pat| {
                        mir_pat.mk_switch_target(0_isize, |mir_pat| { mir_pat.mk_break(); });
                        mir_pat.mk_switch_target(1_isize, |mir_pat| {
                            mir_pat.mk_assign(
                                x_local.into_place(),
                                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(
                                    ::rpl_context::pat::Place::new(
                                        opt_local,
                                        pcx.mk_slice(&[
                                            ::rpl_context::pat::PlaceElem::Downcast(
                                                ::rustc_span::Symbol::intern("Some")
                                            ),
                                            ::rpl_context::pat::PlaceElem::Field(
                                                ::rpl_context::pat::FieldAcc::Unnamed(0u32.into())
                                            ),
                                        ])
                                    )
                                ))
                            );
                            mir_pat.mk_assign(
                                base_local.into_place(),
                                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(
                                    ::rpl_context::pat::Place::new(
                                        self_local,
                                        pcx.mk_slice(&[
                                            ::rpl_context::pat::PlaceElem::Deref,
                                            ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(
                                                ::rustc_span::Symbol::intern("mem")
                                            )),
                                        ])
                                    )
                                ))
                            );
                            mir_pat.mk_assign(
                                offset_local.into_place(),
                                ::rpl_context::pat::Rvalue::Cast(
                                    ::rustc_middle::mir::CastKind::IntToInt,
                                    ::rpl_context::pat::Operand::Copy(x_local.into_place()),
                                    pcx.primitive_types.isize
                                )
                            );
                            mir_pat.mk_assign(
                                elem_ptr_local.into_place(),
                                ::rpl_context::pat::Rvalue::BinaryOp(
                                    ::rustc_middle::mir::BinOp::Offset,
                                    Box::new([
                                        ::rpl_context::pat::Operand::Copy(base_local.into_place()),
                                        ::rpl_context::pat::Operand::Copy(offset_local.into_place())
                                    ])
                                )
                            );
                            mir_pat.mk_fn_call(
                                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                                    pcx.mk_path_with_args(
                                        pcx.mk_item_path(&["core", "ptr", "drop_in_place",]),
                                        &[]
                                    )
                                )),
                                mir_pat.mk_list([::rpl_context::pat::Operand::Copy(
                                    elem_ptr_local.into_place()
                                )]),
                                None
                            );
                        });
                    }
                );
            });
        }
    );
}

#[test]
fn test_cve_2020_35892() {
    mir_test_case!(
        #[meta($T:ty, $SlabT:ty)]
        pat! {
            let $self: &mut $SlabT;
            let $len: usize; // _2
            let mut $x0: usize; // _17
            let $x1: usize; // _14
            let $x2: usize; // _15
            let $x3: #[lang = "Option"]<usize>; // _3
            let $x: usize; // _4
            let mut $base: *mut T; // _6
            let $offset: isize; // _7
            let $elem_ptr: *mut T; // _5
            let $x_cmp: usize; // _16
            let $cmp: bool; // _13

            $len = copy (*$self).len;
            $x0 = const 0_usize;
            loop {
                $x_cmp = copy $x0;
                $cmp = Lt(move $x_cmp, copy $len);
                switchInt(move $cmp) {
                    false => break,
                    _ => {
                        $x1 = copy $x0;
                        $x2 = core::iter::range::Step::forward_unchecked(copy $x1, const 1_usize);
                        $x0 = move $x2;
                        $x3 = #[lang = "Some"](copy $x1);
                        $x = copy ($x3 as Some).0;
                        $base = copy (*$self).mem;
                        $offset = copy $x as isize (IntToInt);
                        $elem_ptr = Offset(copy $base, copy $offset);
                        _ = core::ptr::drop_in_place(copy $elem_ptr);
                    }
                }
            }
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
                #[allow(non_snake_case)]
                let SlabT_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let SlabT_ty = pcx.mk_var_ty(SlabT_ty_var);
            }

            let self_local = mir_pat.mk_self(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny, SlabT_ty, ::rustc_middle::mir::Mutability::Mut
            ));
            let len_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let x0_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let x1_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let x2_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let x3_local = mir_pat.mk_local(pcx.mk_adt_ty(pcx.mk_path_with_args(
                pcx.mk_lang_item("Option"),
                &[pcx.primitive_types.usize.into()]
            )));
            let x_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let base_local =
                mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Mut));
            let offset_local = mir_pat.mk_local(pcx.primitive_types.isize);
            let elem_ptr_local =
                mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Mut));
            let x_cmp_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let cmp_local = mir_pat.mk_local(pcx.primitive_types.bool);
            mir_pat.mk_assign(
                len_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(::rpl_context::pat::Place::new(
                    self_local,
                    pcx.mk_slice(&[
                        ::rpl_context::pat::PlaceElem::Deref,
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(
                            ::rustc_span::Symbol::intern("len")
                        )),
                    ])
                )))
            );
            mir_pat.mk_assign(
                x0_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Constant(
                    ::rpl_context::pat::ConstOperand::ScalarInt(0_usize.into())
                ))
            );
            mir_pat.mk_loop(|mir_pat| {
                mir_pat.mk_assign(
                    x_cmp_local.into_place(),
                    ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(x0_local.into_place()))
                );
                mir_pat.mk_assign(
                    cmp_local.into_place(),
                    ::rpl_context::pat::Rvalue::BinaryOp(
                        ::rustc_middle::mir::BinOp::Lt,
                        Box::new([
                            ::rpl_context::pat::Operand::Move(x_cmp_local.into_place()),
                            ::rpl_context::pat::Operand::Copy(len_local.into_place())
                        ])
                    )
                );
                mir_pat.mk_switch_int(
                    ::rpl_context::pat::Operand::Move(cmp_local.into_place()),
                    |mut mir_pat| {
                        mir_pat.mk_switch_target(false, |mir_pat| {
                            mir_pat.mk_break();
                        });
                        mir_pat.mk_otherwise(|mir_pat| {
                            mir_pat.mk_assign(
                                x1_local.into_place(),
                                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(
                                    x0_local.into_place()
                                ))
                            );
                            mir_pat.mk_fn_call(
                                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                                    pcx.mk_path_with_args(
                                        pcx.mk_item_path(&["core", "iter", "range", "Step", "forward_unchecked",]),
                                        &[]
                                    )
                                )),
                                mir_pat.mk_list([
                                    ::rpl_context::pat::Operand::Copy(x1_local.into_place()),
                                    ::rpl_context::pat::Operand::Constant(
                                        ::rpl_context::pat::ConstOperand::ScalarInt(1_usize.into())
                                    )
                                ]),
                                Some(x2_local.into_place())
                            );
                            mir_pat.mk_assign(
                                x0_local.into_place(),
                                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Move(
                                    x2_local.into_place()
                                ))
                            );
                            mir_pat.mk_fn_call(
                                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                                    pcx.mk_path_with_args(pcx.mk_lang_item("Some"), &[])
                                )),
                                mir_pat.mk_list([::rpl_context::pat::Operand::Copy(
                                    x1_local.into_place()
                                )]),
                                Some(x3_local.into_place())
                            );
                            mir_pat.mk_assign(
                                x_local.into_place(),
                                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(
                                    ::rpl_context::pat::Place::new(
                                        x3_local,
                                        pcx.mk_slice(&[
                                            ::rpl_context::pat::PlaceElem::Downcast(
                                                ::rustc_span::Symbol::intern("Some")
                                            ),
                                            ::rpl_context::pat::PlaceElem::Field(
                                                ::rpl_context::pat::FieldAcc::Unnamed(0u32.into())
                                            ),
                                        ])
                                    )
                                ))
                            );
                            mir_pat.mk_assign(
                                base_local.into_place(),
                                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(
                                    ::rpl_context::pat::Place::new(
                                        self_local,
                                        pcx.mk_slice(&[
                                            ::rpl_context::pat::PlaceElem::Deref,
                                            ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(
                                                ::rustc_span::Symbol::intern("mem")
                                            )),
                                        ])
                                    )
                                ))
                            );
                            mir_pat.mk_assign(
                                offset_local.into_place(),
                                ::rpl_context::pat::Rvalue::Cast(
                                    ::rustc_middle::mir::CastKind::IntToInt,
                                    ::rpl_context::pat::Operand::Copy(x_local.into_place()),
                                    pcx.primitive_types.isize
                                )
                            );
                            mir_pat.mk_assign(
                                elem_ptr_local.into_place(),
                                ::rpl_context::pat::Rvalue::BinaryOp(
                                    ::rustc_middle::mir::BinOp::Offset,
                                    Box::new([
                                        ::rpl_context::pat::Operand::Copy(base_local.into_place()),
                                        ::rpl_context::pat::Operand::Copy(offset_local.into_place())
                                    ])
                                )
                            );
                            mir_pat.mk_fn_call(
                                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                                    pcx.mk_path_with_args(
                                        pcx.mk_item_path(&["core", "ptr", "drop_in_place",]),
                                        &[]
                                    )
                                )),
                                mir_pat.mk_list([::rpl_context::pat::Operand::Copy(
                                    elem_ptr_local.into_place()
                                )]),
                                None
                            );
                        });
                    }
                );
            });
        }
    );
}

#[test]
fn test_cve_2018_21000() {
    mir_test_case!(
        #[meta($T1:ty, $T2:ty, $T3:ty)]
        pat! {
            type VecT1 = std::vec::Vec<$T1>;
            type VecT2 = std::vec::Vec<$T2>;
            type VecT3 = std::vec::Vec<$T3>;
            type PtrT1 = *mut $T1;
            type PtrT3 = *mut $T3;

            let $from_vec: VecT1 = _;
            let $size: usize = SizeOf($T2);
            let $from_cap: usize = Vec::capacity(move $from_vec);
            let $to_cap: usize = Mul(copy $from_cap, copy $size);
            let $from_len: usize = Len($from_vec);
            let $to_len: usize = Mul(copy $from_len, copy $size);
            let $from_vec_ptr: PtrT1 = Vec::as_mut_ptr(move $from_vec);
            let $to_vec_ptr: PtrT3 = copy $from_vec_ptr as PtrT3 (PtrToPtr);
            let $_tmp: () = std::mem::forget(move $from_vec);
            let $res: VecT3 = Vec::from_raw_parts(copy $to_vec_ptr, copy $to_cap, copy $to_len);
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T1_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T1_ty = pcx.mk_var_ty(T1_ty_var);
                #[allow(non_snake_case)]
                let T2_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T2_ty = pcx.mk_var_ty(T2_ty_var);
                #[allow(non_snake_case)]
                let T3_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T3_ty = pcx.mk_var_ty(T3_ty_var);
            }
            #[allow(non_snake_case)]
            let VecT1_ty = pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["std", "vec", "Vec",]),
                &[T1_ty.into(),]
            ));
            #[allow(non_snake_case)]
            let VecT2_ty = pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["std", "vec", "Vec",]),
                &[T2_ty.into(),]
            ));
            #[allow(non_snake_case)]
            let VecT3_ty = pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["std", "vec", "Vec",]),
                &[T3_ty.into(),]
            ));
            #[allow(non_snake_case)]
            let PtrT1_ty = pcx.mk_raw_ptr_ty(T1_ty, ::rustc_middle::mir::Mutability::Mut);
            #[allow(non_snake_case)]
            let PtrT3_ty = pcx.mk_raw_ptr_ty(T3_ty, ::rustc_middle::mir::Mutability::Mut);
            let from_vec_local = mir_pat.mk_local(VecT1_ty);
            mir_pat.mk_assign(from_vec_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let size_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_assign(
                size_local.into_place(),
                ::rpl_context::pat::Rvalue::NullaryOp(::rustc_middle::mir::NullOp::SizeOf, T2_ty)
            );
            let from_cap_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                    pcx.mk_path_with_args(
                        pcx.mk_item_path(&["Vec", "capacity",]),
                        &[]
                    )
                )),
                mir_pat.mk_list([::rpl_context::pat::Operand::Move(from_vec_local.into_place())]),
                Some(from_cap_local.into_place())
            );
            let to_cap_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_assign(
                to_cap_local.into_place(),
                ::rpl_context::pat::Rvalue::BinaryOp(
                    ::rustc_middle::mir::BinOp::Mul,
                    Box::new([
                        ::rpl_context::pat::Operand::Copy(from_cap_local.into_place()),
                        ::rpl_context::pat::Operand::Copy(size_local.into_place())
                    ])
                )
            );
            let from_len_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_assign(
                from_len_local.into_place(),
                ::rpl_context::pat::Rvalue::Len(from_vec_local.into_place())
            );
            let to_len_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_assign(
                to_len_local.into_place(),
                ::rpl_context::pat::Rvalue::BinaryOp(
                    ::rustc_middle::mir::BinOp::Mul,
                    Box::new([
                        ::rpl_context::pat::Operand::Copy(from_len_local.into_place()),
                        ::rpl_context::pat::Operand::Copy(size_local.into_place())
                    ])
                )
            );
            let from_vec_ptr_local = mir_pat.mk_local(PtrT1_ty);
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                    pcx.mk_path_with_args(
                        pcx.mk_item_path(&["Vec", "as_mut_ptr",]),
                        &[]
                    )
                )),
                mir_pat.mk_list([::rpl_context::pat::Operand::Move(from_vec_local.into_place())]),
                Some(from_vec_ptr_local.into_place())
            );
            let to_vec_ptr_local = mir_pat.mk_local(PtrT3_ty);
            mir_pat.mk_assign(
                to_vec_ptr_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::PtrToPtr,
                    ::rpl_context::pat::Operand::Copy(from_vec_ptr_local.into_place()),
                    PtrT3_ty
                )
            );
            let _tmp_local = mir_pat.mk_local(pcx.mk_tuple_ty(&[]));
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(pcx.mk_path_with_args(
                    pcx.mk_item_path(&["std", "mem", "forget",]),
                    &[]
                ))),
                mir_pat.mk_list([
                    ::rpl_context::pat::Operand::Move(from_vec_local.into_place())
                ]),
                Some(_tmp_local.into_place())
            );
            let res_local = mir_pat.mk_local(VecT3_ty);
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                    pcx.mk_path_with_args(
                        pcx.mk_item_path(&["Vec", "from_raw_parts",]),
                        &[]
                    )
                )),
                mir_pat.mk_list([
                    ::rpl_context::pat::Operand::Copy(to_vec_ptr_local.into_place()),
                    ::rpl_context::pat::Operand::Copy(to_cap_local.into_place()),
                    ::rpl_context::pat::Operand::Copy(to_len_local.into_place())
                ]),
                Some(res_local.into_place())
            );
        }
    );
}

#[test]
fn test_cve_2020_35881_const() {
    mir_test_case!(
        #[meta($T1:ty)]
        pat! {
            type PtrT1 = *const $T1;
            type PtrPtrT1 = *const *const $T1;
            type DerefPtrT1 = &*const $T1;
            type PtrT2 = *const ();
            type PtrPtrT2 = *const *const ();

            let $ptr_to_data: PtrT1 = _;
            let $data: DerefPtrT1 = &$ptr_to_data;
            let $ptr_to_ptr_to_data: PtrPtrT1 = &raw const (*$data);
            let $ptr_to_ptr_to_res: PtrPtrT2 = move $ptr_to_ptr_to_data as *const *const () (Transmute);
            let $ptr_to_res: PtrT2 = copy* $ptr_to_ptr_to_res;
            // neglected the type-size-equivalence check
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T1_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T1_ty = pcx.mk_var_ty(T1_ty_var);
            }

            #[allow(non_snake_case)]
            let PtrT1_ty = pcx.mk_raw_ptr_ty(
                T1_ty,
                ::rustc_middle::mir::Mutability::Not
            );
            #[allow(non_snake_case)]
            let PtrPtrT1_ty = pcx.mk_raw_ptr_ty(
                pcx.mk_raw_ptr_ty(
                    T1_ty,
                    ::rustc_middle::mir::Mutability::Not
                ),
                ::rustc_middle::mir::Mutability::Not
            );
            #[allow(non_snake_case)]
            let DerefPtrT1_ty = pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_raw_ptr_ty(
                    T1_ty,
                    ::rustc_middle::mir::Mutability::Not
                ),
                ::rustc_middle::mir::Mutability::Not
            );
            #[allow(non_snake_case)]
            let PtrT2_ty = pcx.mk_raw_ptr_ty(
                pcx.mk_tuple_ty(&[]),
                ::rustc_middle::mir::Mutability::Not
            );
            #[allow(non_snake_case)]
            let PtrPtrT2_ty = pcx.mk_raw_ptr_ty(
                pcx.mk_raw_ptr_ty(
                    pcx.mk_tuple_ty(&[]),
                    ::rustc_middle::mir::Mutability::Not
                ),
                ::rustc_middle::mir::Mutability::Not
            );
            let ptr_to_data_local = mir_pat.mk_local(PtrT1_ty);
            mir_pat.mk_assign(ptr_to_data_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let data_local = mir_pat.mk_local(DerefPtrT1_ty);
            mir_pat.mk_assign(
                data_local.into_place(),
                ::rpl_context::pat::Rvalue::Ref(
                    ::rpl_context::pat::RegionKind::ReAny,
                    ::rustc_middle::mir::BorrowKind::Shared,
                    ptr_to_data_local.into_place()
                )
            );
            let ptr_to_ptr_to_data_local = mir_pat.mk_local(PtrPtrT1_ty);
            mir_pat.mk_assign(
                ptr_to_ptr_to_data_local.into_place(),
                ::rpl_context::pat::Rvalue::RawPtr(
                    ::rustc_middle::mir::Mutability::Not,
                    ::rpl_context::pat::Place::new(
                        data_local,
                        pcx.mk_slice(
                            &[::rpl_context::pat::PlaceElem::Deref,]
                        )
                    )
                )
            );
            let ptr_to_ptr_to_res_local = mir_pat.mk_local(PtrPtrT2_ty);
            mir_pat.mk_assign(
                ptr_to_ptr_to_res_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::Transmute,
                    ::rpl_context::pat::Operand::Move(
                        ptr_to_ptr_to_data_local.into_place()
                    ),
                    pcx.mk_raw_ptr_ty(
                        pcx.mk_raw_ptr_ty(
                            pcx.mk_tuple_ty(&[]),
                            ::rustc_middle::mir::Mutability::Not
                        ),
                        ::rustc_middle::mir::Mutability::Not
                    )
                )
            );
            let ptr_to_res_local = mir_pat.mk_local(PtrT2_ty);
            mir_pat.mk_assign(
                ptr_to_res_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(
                    ::rpl_context::pat::Operand::Copy(
                        ::rpl_context::pat::Place::new(
                            ptr_to_ptr_to_res_local,
                            pcx.mk_slice(
                                &[::rpl_context::pat::PlaceElem::Deref,]
                            )
                        )
                    )
                )
            );
        }
    )
}

#[test]
fn test_cve_2020_35881_mut() {
    mir_test_case!(
        #[meta($T1:ty)]
        pat! {
            type PtrT1 = *mut $T1;
            type PtrPtrT1 = *mut *mut $T1;
            type DerefPtrT1 = &mut *mut $T1;
            type PtrT2 = *mut ();
            type PtrPtrT2 = *mut *mut ();

            let $ptr_to_data: PtrT1 = _;
            let $data: DerefPtrT1 = &mut $ptr_to_data;
            let $ptr_to_ptr_to_data: PtrPtrT1 = &raw mut (*$data);
            let $ptr_to_ptr_to_res: PtrPtrT2 = move $ptr_to_ptr_to_data as *mut *mut () (Transmute);
            let $ptr_to_res: PtrT2 = copy *$ptr_to_ptr_to_res;
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T1_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T1_ty = pcx.mk_var_ty(T1_ty_var);
            }
            #[allow(non_snake_case)]
            let PtrT1_ty = pcx.mk_raw_ptr_ty(
                T1_ty,
                ::rustc_middle::mir::Mutability::Mut
            );
            #[allow(non_snake_case)]
            let PtrPtrT1_ty = pcx.mk_raw_ptr_ty(
                pcx.mk_raw_ptr_ty(
                    T1_ty,
                    ::rustc_middle::mir::Mutability::Mut
                ),
                ::rustc_middle::mir::Mutability::Mut
            );
            #[allow(non_snake_case)]
            let DerefPtrT1_ty = pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_raw_ptr_ty(
                    T1_ty,
                    ::rustc_middle::mir::Mutability::Mut
                ),
                ::rustc_middle::mir::Mutability::Mut
            );
            #[allow(non_snake_case)]
            let PtrT2_ty = pcx.mk_raw_ptr_ty(
                pcx.mk_tuple_ty(&[]),
                ::rustc_middle::mir::Mutability::Mut
            );
            #[allow(non_snake_case)]
            let PtrPtrT2_ty = pcx.mk_raw_ptr_ty(
                pcx.mk_raw_ptr_ty(
                    pcx.mk_tuple_ty(&[]),
                    ::rustc_middle::mir::Mutability::Mut
                ),
                ::rustc_middle::mir::Mutability::Mut
            );
            let ptr_to_data_local = mir_pat.mk_local(PtrT1_ty);
            mir_pat.mk_assign(ptr_to_data_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let data_local = mir_pat.mk_local(DerefPtrT1_ty);
            mir_pat.mk_assign(
                data_local.into_place(),
                ::rpl_context::pat::Rvalue::Ref(
                    ::rpl_context::pat::RegionKind::ReAny,
                    ::rustc_middle::mir::BorrowKind::Mut {
                        kind: ::rustc_middle::mir::MutBorrowKind::Default
                    },
                    ptr_to_data_local.into_place()
                )
            );
            let ptr_to_ptr_to_data_local = mir_pat.mk_local(PtrPtrT1_ty);
            mir_pat.mk_assign(
                ptr_to_ptr_to_data_local.into_place(),
                ::rpl_context::pat::Rvalue::RawPtr(
                    ::rustc_middle::mir::Mutability::Mut,
                    ::rpl_context::pat::Place::new(
                        data_local,
                        pcx.mk_slice(
                            &[::rpl_context::pat::PlaceElem::Deref,]
                        )
                    )
                )
            );
            let ptr_to_ptr_to_res_local = mir_pat.mk_local(PtrPtrT2_ty);
            mir_pat.mk_assign(
                ptr_to_ptr_to_res_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::Transmute,
                    ::rpl_context::pat::Operand::Move(
                        ptr_to_ptr_to_data_local.into_place()
                    ),
                    pcx.mk_raw_ptr_ty(
                        pcx.mk_raw_ptr_ty(
                            pcx.mk_tuple_ty(&[]),
                            ::rustc_middle::mir::Mutability::Mut
                        ),
                        ::rustc_middle::mir::Mutability::Mut
                    )
                )
            );
            let ptr_to_res_local = mir_pat.mk_local(PtrT2_ty);
            mir_pat.mk_assign(
                ptr_to_res_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(
                    ::rpl_context::pat::Operand::Copy(
                        ::rpl_context::pat::Place::new(
                            ptr_to_ptr_to_res_local,
                            pcx.mk_slice(
                                &[::rpl_context::pat::PlaceElem::Deref,]
                            )
                        )
                    )
                )
            );
        }
    )
}

#[test]
fn test_cve_2021_29941_2() {
    mir_test_case!(
        #[meta($T:ty)]
        pat! {
            // type ExactSizeIterT = impl std::iter::ExactSizeIterator<Item = $T>;
            // let's use a std::ops::Range<$T> instead temporarily
            type RangeT = std::ops::Range<$T>;
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
            let mut $vec: VecT = std::vec::Vec::with_capacity(copy $len);
            let mut $ref_to_vec: RefMutVecT = &mut $vec;
            let mut $ptr_to_vec: PtrMutT = Vec::as_mut_ptr(move $ref_to_vec);
            let mut $slice: RefMutSliceT = std::slice::from_raw_parts_mut(copy $ptr_to_vec, copy $len);
            // let mut enumerate: EnumerateRangeT = <RangeT as std::iter::Iterator>::enumerate(move iter);
            let mut $enumerate: EnumerateRangeT = RangeT::enumerate(move $iter);
            let mut $enumerate: RefMutEnumerateRangeT = &mut $enumerate;
            let $next: OptionUsizeT;
            let $cmp: isize;
            let $first: usize;
            let $second_t: $T;
            let $second_usize: usize;
            loop {
                // next = <EnumerateRangeT as std::iter::Iterator>::next(move enumerate);
                $next = EnumerateRangeT::next(move $enumerate);
                // in `cmp = discriminant(copy next);`
                // which discriminant should be used?
                $cmp = balabala::discriminant(copy $next);
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
            _ = Vec::set_len(move $ref_to_vec, copy $len);
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
            }

            #[allow(non_snake_case)]
            let RangeT_ty = pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["std", "ops", "Range",]),
                &[T_ty.into(),]
            ));
            #[allow(non_snake_case)]
            let VecT_ty = pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["std", "vec", "Vec",]),
                &[T_ty.into(),]
            ));
            #[allow(non_snake_case)]
            let RefMutVecT_ty = pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_path_ty(pcx.mk_path_with_args(
                    pcx.mk_item_path(&["std", "vec", "Vec",]),
                    &[T_ty.into(),]
                )),
                ::rustc_middle::mir::Mutability::Mut
            );
            #[allow(non_snake_case)]
            let PtrMutT_ty = pcx.mk_raw_ptr_ty(
                T_ty,::rustc_middle::mir::Mutability::Mut
            );
            #[allow(non_snake_case)]
            let RefMutSliceT_ty = pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_slice_ty(T_ty),
                ::rustc_middle::mir::Mutability::Mut
            );
            #[allow(non_snake_case)]
            let EnumerateRangeT_ty = pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["std", "iter", "Enumerate",]),
                &[RangeT_ty.into(),]
            ));
            #[allow(non_snake_case)]
            let RefMutEnumerateRangeT_ty = pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_path_ty(pcx.mk_path_with_args(
                    pcx.mk_item_path(&["std", "iter", "Enumerate",]),
                    &[RangeT_ty.into(),]
                )),
                ::rustc_middle::mir::Mutability::Mut
            );
            #[allow(non_snake_case)]
            let OptionUsizeT_ty = pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["std", "option", "Option",]),
                &[pcx.mk_tuple_ty(&[pcx.primitive_types.usize, T_ty]).into(),]
            ));
            let iter_local = mir_pat.mk_local(RangeT_ty);
            mir_pat.mk_assign(iter_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let len_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(
                    mir_pat.mk_zeroed(
                        pcx.mk_path_with_args(pcx.mk_item_path(&["RangeT", "len",]), &[])
                    )
                ),
                mir_pat.mk_list([
                    ::rpl_context::pat::Operand::Move(iter_local.into_place())
                ]),
                Some(len_local.into_place())
            );
            let vec_local = mir_pat.mk_local(VecT_ty);
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                    pcx.mk_path_with_args(
                        pcx.mk_item_path(&["std", "vec", "Vec", "with_capacity",]),
                        &[]
                    )
                )),
                mir_pat.mk_list([
                    ::rpl_context::pat::Operand::Copy(len_local.into_place())
                ]),
                Some(vec_local.into_place())
            );
            let ref_to_vec_local = mir_pat.mk_local(RefMutVecT_ty);
            mir_pat.mk_assign(
                ref_to_vec_local.into_place(),
                ::rpl_context::pat::Rvalue::Ref(
                    ::rpl_context::pat::RegionKind::ReAny,
                    ::rustc_middle::mir::BorrowKind::Mut{
                        kind : ::rustc_middle::mir::MutBorrowKind::Default
                    },
                    vec_local.into_place()
                )
            );
            let ptr_to_vec_local = mir_pat.mk_local(PtrMutT_ty);
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                    pcx.mk_path_with_args(
                        pcx.mk_item_path(&["Vec", "as_mut_ptr",]),
                        &[]
                    )
                )),
                mir_pat.mk_list([
                    :: rpl_context::pat::Operand::Move(ref_to_vec_local.into_place())
                ]),
                Some(ptr_to_vec_local.into_place())
            );
            let slice_local = mir_pat.mk_local(RefMutSliceT_ty);
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                    pcx.mk_path_with_args(
                        pcx.mk_item_path(&["std", "slice", "from_raw_parts_mut",]),
                        &[]
                    )
                )),
                mir_pat.mk_list([
                    :: rpl_context::pat::Operand::Copy(ptr_to_vec_local.into_place()),
                    ::rpl_context::pat::Operand::Copy(len_local.into_place())
                ]),
                Some(slice_local.into_place())
            );
            let enumerate_local = mir_pat.mk_local(EnumerateRangeT_ty);
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                    pcx.mk_path_with_args(
                        pcx.mk_item_path(&["RangeT", "enumerate" ,]),
                        &[]
                    )
                )),
                mir_pat.mk_list([
                    ::rpl_context::pat::Operand::Move(iter_local.into_place())
                ]),
                Some(enumerate_local.into_place())
            );
            let enumerate_local = mir_pat.mk_local(RefMutEnumerateRangeT_ty);
            mir_pat.mk_assign(
                enumerate_local.into_place(),
                ::rpl_context::pat::Rvalue::Ref(
                    ::rpl_context::pat::RegionKind::ReAny,
                    ::rustc_middle::mir::BorrowKind::Mut{
                        kind : ::rustc_middle::mir::MutBorrowKind::Default
                    },
                    enumerate_local.into_place()
                )
            );
            let next_local = mir_pat.mk_local(OptionUsizeT_ty);
            let cmp_local = mir_pat.mk_local(pcx.primitive_types.isize);
            let first_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let second_t_local = mir_pat.mk_local(T_ty);
            let second_usize_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_loop(
                |mir_pat| {
                    mir_pat.mk_fn_call(
                        ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                            pcx.mk_path_with_args(
                                pcx.mk_item_path(&["EnumerateRangeT", "next",]),
                                &[]
                            )
                        )),
                        mir_pat.mk_list([
                            :: rpl_context::pat::Operand::Move(enumerate_local.into_place())
                        ]),
                        Some(next_local.into_place())
                    );
                    mir_pat.mk_fn_call(
                        ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                            pcx.mk_path_with_args(
                                pcx.mk_item_path(&["balabala", "discriminant",]),
                                &[]
                            )
                        )),
                        mir_pat.mk_list([
                            ::rpl_context::pat::Operand::Copy(next_local.into_place())
                        ]),
                        Some(cmp_local.into_place())
                    );
                    mir_pat.mk_switch_int(
                        ::rpl_context::pat::Operand::Move(cmp_local.into_place()),
                        |mut mir_pat| {
                            mir_pat.mk_switch_target(
                                true, |mir_pat| {
                                    mir_pat.mk_assign(
                                        first_local.into_place(),
                                        ::rpl_context::pat::Rvalue::Use(
                                            ::rpl_context::pat::Operand::Copy(
                                                ::rpl_context::pat::Place::new(
                                                    next_local,
                                                    pcx.mk_slice(&[
                                                        ::rpl_context::pat::PlaceElem::Downcast(
                                                            ::rustc_span::Symbol::intern("Some")
                                                        ),
                                                        ::rpl_context::pat::PlaceElem::Field(
                                                            ::rpl_context::pat::FieldAcc::Unnamed(0u32.into())
                                                        ),
                                                    ])
                                                )
                                            )
                                        )
                                    );
                                    mir_pat.mk_assign(
                                        second_t_local.into_place(),
                                        ::rpl_context::pat::Rvalue::Use(
                                            ::rpl_context::pat::Operand::Copy(
                                                ::rpl_context::pat::Place::new(
                                                    next_local,
                                                    pcx.mk_slice(&[
                                                        ::rpl_context::pat::PlaceElem::Downcast(
                                                            ::rustc_span::Symbol::intern("Some")
                                                        ),
                                                        ::rpl_context::pat::PlaceElem::Field(
                                                            ::rpl_context::pat::FieldAcc::Unnamed(1u32.into())
                                                        ),
                                                    ])
                                                )
                                            )
                                        )
                                    );
                                    mir_pat.mk_assign(
                                        second_usize_local.into_place(),
                                        ::rpl_context::pat::Rvalue::Cast(
                                            ::rustc_middle::mir::CastKind::IntToInt,
                                            ::rpl_context::pat::Operand::Copy(
                                                second_t_local.into_place()
                                            ),
                                            pcx.primitive_types.usize
                                        )
                                    );
                                    mir_pat.mk_assign(
                                        ::rpl_context::pat::Place::new(
                                            slice_local,
                                            pcx.mk_slice(&[
                                                ::rpl_context::pat::PlaceElem::Deref,
                                                ::rpl_context::pat::PlaceElem::Index(second_usize_local),
                                            ])
                                        ),
                                        ::rpl_context::pat::Rvalue::Cast(
                                            ::rustc_middle::mir::CastKind::IntToInt,
                                            ::rpl_context::pat::Operand::Copy(first_local.into_place()),
                                            T_ty
                                        )
                                    );
                                }
                            );
                            mir_pat.mk_otherwise(|mir_pat| { mir_pat.mk_break(); });
                        }
                    );
                }
            );
            mir_pat.mk_assign(
                ref_to_vec_local.into_place(),
                ::rpl_context::pat::Rvalue::Ref(
                    ::rpl_context::pat::RegionKind::ReAny,
                    ::rustc_middle::mir::BorrowKind::Mut {
                        kind: ::rustc_middle::mir::MutBorrowKind::Default
                    },
                    vec_local.into_place()
                )
            );
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
                    pcx.mk_path_with_args(
                        pcx.mk_item_path(&["Vec", "set_len",]),
                        &[]
                    )
                )),
                mir_pat.mk_list([
                    ::rpl_context::pat::Operand::Move(ref_to_vec_local.into_place()),
                    ::rpl_context::pat::Operand::Copy(len_local.into_place())
                ]),
                None
            );
        }
    );
}

#[test]
fn test_cve_2018_21000_inlined() {
    mir_test_case!(
        #[meta($T:ty)]
        pat! {
            type Global = alloc::alloc::Global;

            let $from_vec: alloc::vec::Vec<u8, Global> = _;
            let $to_vec: alloc::vec::Vec<$T, Global>;
            let $to_vec_cap: usize;
            let mut $from_vec_cap: usize;
            let mut $tsize: usize;
            let $to_vec_len: usize;
            let mut $from_vec_len: usize;
            let mut $from_vec_ptr: core::ptr::non_null::NonNull<u8>;
            let mut $to_raw_vec: alloc::raw_vec::RawVec<$T, Global>;
            let mut $to_raw_vec_inner: alloc::raw_vec::RawVecInner<Global>;
            let mut $to_vec_wrapped_len: alloc::raw_vec::Cap;
            let mut $from_vec_unique_ptr: core::ptr::unique::Unique<u8>;

            $from_vec_ptr = copy $from_vec.buf.inner.ptr.pointer;
            $from_vec_cap = copy $from_vec.buf.inner.cap.0;
            $tsize = SizeOf($T);
            $to_vec_cap = Div(move $from_vec_cap, copy $tsize);
            $from_vec_len = copy $from_vec.len;
            $to_vec_len = Div(move $from_vec_len, copy $tsize);
            $to_vec_wrapped_len = #[ctor] alloc::raw_vec::Cap(copy $to_vec_len);
            $from_vec_unique_ptr = core::ptr::unique::Unique::<u8> {
                pointer: copy $from_vec_ptr,
                _marker: const core::marker::PhantomData::<u8>,
            };
            $to_raw_vec_inner = alloc::raw_vec::RawVecInner::<Global> {
                ptr: move $from_vec_unique_ptr,
                cap: copy $to_vec_wrapped_len,
                alloc: const alloc::alloc::Global,
            };
            $to_raw_vec = alloc::raw_vec::RawVec::<$T, Global> {
                inner: move $to_raw_vec_inner,
                _marker: const core::marker::PhantomData::<$T>,
            };
            #[export(from_raw_parts)]
            $to_vec = alloc::vec::Vec::<$T, Global> {
                buf: move $to_raw_vec,
                len: copy $to_vec_cap,
            };
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
            }
            #[allow(non_snake_case)]
            let Global_ty = pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["alloc", "alloc", "Global",]),
                &[]
            ));
            let from_vec_local = mir_pat.mk_local(pcx.mk_path_ty(
                pcx.mk_path_with_args(
                    pcx.mk_item_path(&["alloc", "vec", "Vec",]),
                    &[pcx.primitive_types.u8.into(), Global_ty.into(),]
                )
            ));
            mir_pat.mk_assign(from_vec_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let to_vec_local = mir_pat.mk_local(pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["alloc", "vec", "Vec",]),
                &[T_ty.into(), Global_ty.into(),]
            )));
            let to_vec_cap_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let from_vec_cap_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let tsize_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let to_vec_len_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let from_vec_len_local = mir_pat.mk_local(pcx.primitive_types.usize);
            let from_vec_ptr_local = mir_pat.mk_local(pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["core", "ptr", "non_null", "NonNull",]),
                &[pcx.primitive_types.u8.into(),]
            )));
            let to_raw_vec_local = mir_pat.mk_local(pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["alloc", "raw_vec", "RawVec",]),
                &[T_ty.into(), Global_ty.into(),]
            )));
            let to_raw_vec_inner_local = mir_pat.mk_local(pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["alloc", "raw_vec", "RawVecInner",]),
                &[Global_ty.into(),]
            )));
            let to_vec_wrapped_len_local = mir_pat.mk_local(pcx.mk_path_ty(
                pcx.mk_path_with_args(pcx.mk_item_path(&["alloc", "raw_vec", "Cap",]), &[])
            ));
            let from_vec_unique_ptr_local = mir_pat.mk_local(pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["core", "ptr", "unique", "Unique",]),
                &[pcx.primitive_types.u8.into(),]
            )));
            mir_pat.mk_assign(
                from_vec_ptr_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(
                    ::rpl_context::pat::Place::new(from_vec_local, pcx.mk_slice(&[
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("buf"))),
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("inner"))),
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("ptr"))),
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("pointer"))),
                    ]))
                ))
            );
            mir_pat.mk_assign(
                from_vec_cap_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(
                    ::rpl_context::pat::Place::new(from_vec_local, pcx.mk_slice(&[
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("buf"))),
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("inner"))),
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("cap"))),
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Unnamed(0u32.into())),
                    ]))
                ))
            );
            mir_pat.mk_assign(
                tsize_local.into_place(),
                ::rpl_context::pat::Rvalue::NullaryOp(::rustc_middle::mir::NullOp::SizeOf, T_ty)
            );
            mir_pat.mk_assign(
                to_vec_cap_local.into_place(),
                ::rpl_context::pat::Rvalue::BinaryOp(
                    ::rustc_middle::mir::BinOp::Div, Box::new([
                        ::rpl_context::pat::Operand::Move(from_vec_cap_local.into_place()),
                        ::rpl_context::pat::Operand::Copy(tsize_local.into_place())
                    ])
                )
            );
            mir_pat.mk_assign(
                from_vec_len_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(
                    ::rpl_context::pat::Place::new(from_vec_local, pcx.mk_slice(&[
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(::rustc_span::Symbol::intern("len"))),
                    ]))
                ))
            );
            mir_pat.mk_assign(
                to_vec_len_local.into_place(),
                ::rpl_context::pat::Rvalue::BinaryOp(
                    ::rustc_middle::mir::BinOp::Div,
                    Box::new([
                        ::rpl_context::pat::Operand::Move(from_vec_len_local.into_place()),
                        ::rpl_context::pat::Operand::Copy(tsize_local.into_place())
                    ])
                )
            );
            mir_pat.mk_assign(
                to_vec_wrapped_len_local.into_place(),
                ::rpl_context::pat::Rvalue::Aggregate(
                    ::rpl_context::pat::AggKind::Adt(
                        pcx.mk_path_with_args(
                            pcx.mk_item_path(&["alloc", "raw_vec", "Cap",]),
                            &[]
                        ),
                        ::rpl_context::pat::AggAdtKind::Tuple
                    ),
                    mir_pat.mk_list([::rpl_context::pat::Operand::Copy(to_vec_len_local.into_place())]),
                )
            );
            mir_pat.mk_assign(
                from_vec_unique_ptr_local.into_place(),
                ::rpl_context::pat::Rvalue::Aggregate(
                    ::rpl_context::pat::AggKind::Adt(
                        pcx.mk_path_with_args(
                            pcx.mk_item_path(&["core", "ptr", "unique", "Unique",]),
                            &[pcx.primitive_types.u8.into(),]
                        ),
                        mir_pat.mk_list([
                            ::rustc_span::Symbol::intern("pointer"),
                            ::rustc_span::Symbol::intern("_marker"),
                        ])
                        .into()
                    ),
                    mir_pat.mk_list([
                        ::rpl_context::pat::Operand::Copy(from_vec_ptr_local.into_place()),
                        ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(pcx.mk_path_with_args(
                            pcx.mk_item_path(&["core", "marker", "PhantomData",]),
                            &[pcx.primitive_types.u8.into(),]
                        ))),
                    ]),
                )
            );
            mir_pat.mk_assign(
                to_raw_vec_inner_local.into_place(),
                ::rpl_context::pat::Rvalue::Aggregate(
                    ::rpl_context::pat::AggKind::Adt(
                        pcx.mk_path_with_args(
                            pcx.mk_item_path(&["alloc", "raw_vec", "RawVecInner",]),
                            &[Global_ty.into(),]
                        ),
                        mir_pat.mk_list([
                            ::rustc_span::Symbol::intern("ptr"),
                            ::rustc_span::Symbol::intern("cap"),
                            ::rustc_span::Symbol::intern("alloc"),
                        ]).into()
                    ),
                    mir_pat.mk_list([
                        ::rpl_context::pat::Operand::Move(from_vec_unique_ptr_local.into_place()),
                        ::rpl_context::pat::Operand::Copy(to_vec_wrapped_len_local.into_place()),
                        ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(pcx.mk_path_with_args(
                            pcx.mk_item_path(&["alloc", "alloc", "Global",]),
                            &[]
                        ))),
                    ]),
                )
            );
            mir_pat.mk_assign(
                to_raw_vec_local.into_place(),
                ::rpl_context::pat::Rvalue::Aggregate(
                    ::rpl_context::pat::AggKind::Adt(
                        pcx.mk_path_with_args(
                            pcx.mk_item_path(&["alloc", "raw_vec", "RawVec",]),
                            &[T_ty.into(), Global_ty.into(),]
                        ),
                        mir_pat.mk_list([
                            ::rustc_span::Symbol::intern("inner"),
                            ::rustc_span::Symbol::intern("_marker"),
                        ]).into()
                    ),
                    mir_pat.mk_list([
                        ::rpl_context::pat::Operand::Move(to_raw_vec_inner_local.into_place()),
                        ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(pcx.mk_path_with_args(
                            pcx.mk_item_path(&["core", "marker", "PhantomData",]),
                            &[T_ty.into(),]
                        ))),
                    ]),
                )
            );
            from_raw_parts = mir_pat.mk_assign(
                to_vec_local.into_place(),
                ::rpl_context::pat::Rvalue::Aggregate(
                    ::rpl_context::pat::AggKind::Adt(
                        pcx.mk_path_with_args(
                            pcx.mk_item_path(&["alloc", "vec", "Vec",]),
                            &[T_ty.into(), Global_ty.into(),]
                        ),
                        mir_pat.mk_list([
                            ::rustc_span::Symbol::intern("buf"),
                            ::rustc_span::Symbol::intern("len"),
                        ]).into()
                    ),
                    mir_pat.mk_list([
                        ::rpl_context::pat::Operand::Move(to_raw_vec_local.into_place()),
                        ::rpl_context::pat::Operand::Copy(to_vec_cap_local.into_place()),
                    ]),
                )
            );
        }
    );
}

#[test]
fn test_cve_2019_15548() {
    mir_test_case!(
        #[meta($T:ty)]
        pat! {
            type c_char = libc::c_char;

            let $src: &alloc::string::String = _;
            let $bytes: &[u8] = alloc::string::String::as_bytes(move $src);
            let $ptr: *const u8 = core::slice::as_ptr(copy $bytes);
            let $dst: *const c_char = copy $ptr as *const c_char (Transmute);
            let $ret: $T = $crate::ll::instr(move $dst);
        } => {
            meta!{
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
            }
            #[allow(non_snake_case)]
            let c_char_ty = pcx.mk_path_ty(pcx.mk_path_with_args(pcx.mk_item_path(&["libc", "c_char", ]), &[]));
            let src_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_path_ty(pcx.mk_path_with_args(pcx.mk_item_path(&["alloc", "string", "String", ]), &[])),
                ::rustc_middle::mir::Mutability::Not
            ));
            mir_pat.mk_assign(src_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let bytes_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_slice_ty(pcx.primitive_types.u8),
                ::rustc_middle::mir::Mutability::Not
            ));
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(
                    mir_pat.mk_zeroed(pcx.mk_path_with_args(pcx.mk_item_path(&["alloc", "string", "String", "as_bytes", ]), &[]))
                ),
                mir_pat.mk_list([::rpl_context::pat::Operand::Move(src_local.into_place())]),
                Some(bytes_local.into_place())
            );
            let ptr_local = mir_pat.mk_local(
                pcx.mk_raw_ptr_ty(pcx.primitive_types.u8, ::rustc_middle::mir::Mutability::Not)
            );
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(
                    mir_pat.mk_zeroed(pcx.mk_path_with_args(pcx.mk_item_path(&["core", "slice", "as_ptr", ]), &[]))
                ),
                mir_pat.mk_list([::rpl_context::pat::Operand::Copy(bytes_local.into_place())]),
                Some(ptr_local.into_place())
            );
            let dst_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(c_char_ty, ::rustc_middle::mir::Mutability::Not));
            mir_pat.mk_assign(
                dst_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::Transmute,
                    ::rpl_context::pat::Operand::Copy(ptr_local.into_place()),
                    pcx.mk_raw_ptr_ty(c_char_ty, ::rustc_middle::mir::Mutability::Not)
                )
            );
            let ret_local = mir_pat.mk_local(T_ty);
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(
                    mir_pat.mk_zeroed(pcx.mk_path_with_args(pcx.mk_item_path(&["crate", "ll", "instr", ]), &[]))
                ),
                mir_pat.mk_list([::rpl_context::pat::Operand::Move(dst_local.into_place())]),
                Some(ret_local.into_place())
            );
        }
    );
}

#[test]
fn test_cve_2019_15548_2() {
    mir_test_case!(
        #[meta($T:ty)]
        pat! {
            type c_char = libc::c_char;

            let $src: &alloc::string::String = _;
            let $bytes: &[u8] = alloc::string::String::as_bytes(move $src);
            let $ptr: *const u8 = core::slice::as_ptr(copy $bytes);
            let $dst: *const c_char = copy $ptr as *const c_char (Transmute);
            let $ret: $T = $crate::ll::instr(move $dst);
        } => {
            meta!{
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
            }
            #[allow(non_snake_case)]
            let c_char_ty = pcx.mk_path_ty(pcx.mk_path_with_args(pcx.mk_item_path(&["libc", "c_char", ]), &[]));
            let src_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_path_ty(pcx.mk_path_with_args(pcx.mk_item_path(&["alloc", "string", "String", ]), &[])),
                ::rustc_middle::mir::Mutability::Not
            ));
            mir_pat.mk_assign(src_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let bytes_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                pcx.mk_slice_ty(pcx.primitive_types.u8),
                ::rustc_middle::mir::Mutability::Not
            ));
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(
                    mir_pat.mk_zeroed(pcx.mk_path_with_args(pcx.mk_item_path(&["alloc", "string", "String", "as_bytes", ]), &[]))
                ),
                mir_pat.mk_list([::rpl_context::pat::Operand::Move(src_local.into_place())]),
                Some(bytes_local.into_place())
            );
            let ptr_local = mir_pat.mk_local(
                pcx.mk_raw_ptr_ty(pcx.primitive_types.u8, ::rustc_middle::mir::Mutability::Not)
            );
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(
                    mir_pat.mk_zeroed(pcx.mk_path_with_args(pcx.mk_item_path(&["core", "slice", "as_ptr", ]), &[]))
                ),
                mir_pat.mk_list([::rpl_context::pat::Operand::Copy(bytes_local.into_place())]),
                Some(ptr_local.into_place())
            );
            let dst_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(c_char_ty, ::rustc_middle::mir::Mutability::Not));
            mir_pat.mk_assign(
                dst_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::Transmute,
                    ::rpl_context::pat::Operand::Copy(ptr_local.into_place()),
                    pcx.mk_raw_ptr_ty(c_char_ty, ::rustc_middle::mir::Mutability::Not)
                )
            );
            let ret_local = mir_pat.mk_local(T_ty);
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(
                    mir_pat.mk_zeroed(pcx.mk_path_with_args(pcx.mk_item_path(&["crate", "ll", "instr", ]), &[]))
                ),
                mir_pat.mk_list([::rpl_context::pat::Operand::Move(dst_local.into_place())]),
                Some(ret_local.into_place())
            );
        }
    );
}

#[test]
fn test_cve_2019_15548_2_i8() {
    mir_test_case!(
        #[meta()]
        pat! {
            type c_char = libc::c_char;

            let $ptr: *const c_char = _;
            _ = $crate::ll::instr(move $ptr);
        } => {
            meta! {}
            #[allow(non_snake_case)]
            let c_char_ty = pcx.mk_path_ty(pcx.mk_path_with_args(
                pcx.mk_item_path(&["libc", "c_char",]),
                &[]
            ));
            let ptr_local = mir_pat.mk_local(
                pcx.mk_raw_ptr_ty(c_char_ty, ::rustc_middle::mir::Mutability::Not)
            );
            mir_pat.mk_assign(ptr_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(
                    mir_pat.mk_zeroed(
                        pcx.mk_path_with_args(
                            pcx.mk_item_path(&["crate", "ll", "instr",]),
                            &[]
                        )
                    )
                ),
                mir_pat.mk_list([::rpl_context::pat::Operand::Move(ptr_local.into_place())]),
                None
            );
        }
    )
}

// #[test]
// fn test_cve_2019_15548_2_i8() {
//     mir_test_case!(
//         #[meta()]
//         pat! {
//             type c_char = i8;

//             let ptr: *const c_char = _;
//             _ = $crate::ll::instr(move ptr);
//         } => {
//             meta! {}
//             #[allow(non_snake_case)]
//             let c_char_ty = pcx.primitive_types.i8;
//             let ptr_local = mir_pat.mk_local(
//                 pcx.mk_raw_ptr_ty(c_char_ty, ::rustc_middle::mir::Mutability::Not)
//             );
//             mir_pat.mk_assign(ptr_local.into_place(), ::rpl_context::pat::Rvalue::Any);
//             mir_pat.mk_fn_call(
//                 ::rpl_context::pat::Operand::Constant(mir_pat.mk_zeroed(
//                     pcx.mk_path_with_args(
//                         pcx.mk_item_path(&["crate", "ll", "instr", ]),
//                         &[]
//                     ))
//                 ),
//                 mir_pat.mk_list([::rpl_context::pat::Operand::Move(ptr_local.into_place())]),
//                 None
//             );
//         }
//     )
// }

#[test]
fn test_cve_2021_27376() {
    mir_test_case!(
        #[meta()]
        pat! {
            let $src: *const std::net::SocketAddrV4 = _;
            let $dst: *const libc::sockaddr = move $src as *const libc::sockaddr (PtrToPtr);
        } => {
            meta! {}
            let src_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(
                pcx.mk_path_ty(pcx.mk_path_with_args(pcx.mk_item_path(&["std", "net", "SocketAddrV4", ]), &[])),
                ::rustc_middle::mir::Mutability::Not
            ));
            mir_pat.mk_assign(src_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let dst_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(
                pcx.mk_path_ty(pcx.mk_path_with_args(pcx.mk_item_path(&["libc", "sockaddr", ]), &[])),
                ::rustc_middle::mir::Mutability::Not
            ));
            mir_pat.mk_assign(
                dst_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::PtrToPtr,
                    ::rpl_context::pat::Operand::Move(src_local.into_place()),
                    pcx.mk_raw_ptr_ty(
                        pcx.mk_path_ty(pcx.mk_path_with_args(pcx.mk_item_path(&["libc", "sockaddr", ]), &[])),
                        ::rustc_middle::mir::Mutability::Not
                    )
                )
            );
        }
    );
}

#[test]
fn test_cve_2020_35877() {
    mir_test_case! {
        #[meta($T:ty)]
        pat! {
            let $offset: usize = _; // _?0 <-> _2
            let $offset_1: usize = copy $offset; // _?1 <-> _3
            let $ptr_1: *const $T = _; // _?2 <-> _4
            let $offset_2: usize = copy $offset_1; // _?3 <-> _13
            let $flag: bool = Gt(move $offset_2, const 0usize); // _?4 <-> _12
            let $ptr_3: *const $T; // _?5 <-> _14
            let $ptr_4: *const $T; // _?6 <-> _15
            let $reference: &$T; // _?7 <-> _0
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
                    }
                }
            }
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);
            }
            let offset_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_assign(offset_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let offset_1_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_assign(
                offset_1_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(offset_local.into_place()))
            );
            let ptr_1_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Not));
            mir_pat.mk_assign(ptr_1_local.into_place(), ::rpl_context::pat::Rvalue::Any);
            let offset_2_local = mir_pat.mk_local(pcx.primitive_types.usize);
            mir_pat.mk_assign(
                offset_2_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(offset_1_local.into_place()))
            );
            let flag_local = mir_pat.mk_local(pcx.primitive_types.bool);
            mir_pat.mk_assign(
                flag_local.into_place(),
                ::rpl_context::pat::Rvalue::BinaryOp(
                    ::rustc_middle::mir::BinOp::Gt,
                    Box::new([
                        ::rpl_context::pat::Operand::Move(offset_2_local.into_place()),
                        ::rpl_context::pat::Operand::Constant(
                            ::rpl_context::pat::ConstOperand::ScalarInt(0usize.into())
                        )
                    ])
                )
            );
            let ptr_3_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Not));
            let ptr_4_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Not));
            let reference_local = mir_pat.mk_local(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                T_ty,
                ::rustc_middle::mir::Mutability::Not
            ));
            mir_pat.mk_loop(|mir_pat| {
                mir_pat.mk_assign(
                    offset_2_local.into_place(),
                    ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(offset_1_local.into_place()))
                );
                mir_pat.mk_assign(
                    flag_local.into_place(),
                    ::rpl_context::pat::Rvalue::BinaryOp(
                        ::rustc_middle::mir::BinOp::Gt,
                        Box::new([
                            ::rpl_context::pat::Operand::Move(offset_2_local.into_place()),
                            ::rpl_context::pat::Operand::Constant(
                                ::rpl_context::pat::ConstOperand::ScalarInt(0usize.into())
                            )
                        ])
                    )
                );
                mir_pat.mk_switch_int(
                    ::rpl_context::pat::Operand::Move(flag_local.into_place()),
                    |mut mir_pat| {
                        mir_pat.mk_switch_target(0usize, |mir_pat| {
                            mir_pat.mk_assign(
                                reference_local.into_place(),
                                ::rpl_context::pat::Rvalue::Ref(
                                    ::rpl_context::pat::RegionKind::ReAny,
                                    ::rustc_middle::mir::BorrowKind::Shared,
                                    ::rpl_context::pat::Place::new(
                                        ptr_1_local,
                                        pcx.mk_slice(&[::rpl_context::pat::PlaceElem::Deref, ])
                                    )
                                )
                            );
                            mir_pat.mk_break();
                        });
                        mir_pat.mk_otherwise(|mir_pat| {
                            mir_pat.mk_assign(
                                offset_1_local.into_place(),
                                ::rpl_context::pat::Rvalue::BinaryOp(
                                    ::rustc_middle::mir::BinOp::Sub,
                                    Box::new([
                                        ::rpl_context::pat::Operand::Copy(offset_1_local.into_place()),
                                        ::rpl_context::pat::Operand::Constant(
                                            ::rpl_context::pat::ConstOperand::ScalarInt(
                                                1usize.into()
                                            )
                                        )
                                    ])
                                )
                            );
                            mir_pat.mk_assign(
                                ptr_4_local.into_place(),
                                ::rpl_context::pat::Rvalue::Use(
                                    ::rpl_context::pat::Operand::Copy(ptr_1_local.into_place())
                                )
                            );
                            mir_pat.mk_assign(
                                ptr_3_local.into_place(),
                                ::rpl_context::pat::Rvalue::BinaryOp(
                                    ::rustc_middle::mir::BinOp::Offset,
                                    Box::new([
                                        ::rpl_context::pat::Operand::Copy(ptr_4_local.into_place()),
                                        ::rpl_context::pat::Operand::Any
                                    ])
                                )
                            );
                            mir_pat.mk_assign(
                                ptr_1_local.into_place(),
                                ::rpl_context::pat::Rvalue::Use(
                                    ::rpl_context::pat::Operand::Move(ptr_3_local.into_place())
                                )
                            );
                        });
                    }
                );
            });
        }
    }
}

#[test]
fn test_cve_2020_35873() {
    test_case! {
        pat! {
            fn $pattern(i32, *const std::ffi::CStr) -> i32;
        } => quote! {
            let pattern_fn = pattern.fns.new_fn_pat(::rustc_span::Symbol::intern("pattern"));
            pattern_fn.set_ret_ty(pcx.primitive_types.i32);
            pattern_fn.params.add_param(
                ::rustc_span::symbol::kw::Empty,
                ::rustc_middle::mir::Mutability::Not,
                pcx.primitive_types.i32
            );
            pattern_fn.params.add_param(
                ::rustc_span::symbol::kw::Empty,
                ::rustc_middle::mir::Mutability::Not,
                pcx.mk_raw_ptr_ty(
                    pcx.mk_path_ty(pcx.mk_path_with_args(
                        pcx.mk_item_path(&["std", "ffi", "CStr",]),
                        &[]
                    )),
                    ::rustc_middle::mir::Mutability::Not
                )
            );
        }
    }
    test_case! {
        pat! {
            #[meta($SessT:ty)]
            fn $ffi_call(*mut $SessT, *const std::ffi::c_char) -> i32;
        } => quote! {
            let ffi_call_fn = pattern.fns.new_fn_pat(::rustc_span::Symbol::intern("ffi_call"));
            #[allow(non_snake_case)]
            let SessT_ty_var = ffi_call_fn.meta.new_ty_var(&[]);
            #[allow(non_snake_case)]
            let SessT_ty = pcx.mk_var_ty(SessT_ty_var );
            ffi_call_fn.set_ret_ty(pcx.primitive_types.i32);

            ffi_call_fn.params.add_param(
                ::rustc_span::symbol::kw::Empty,
                ::rustc_middle::mir::Mutability::Not,
                pcx.mk_raw_ptr_ty(
                    SessT_ty,
                    ::rustc_middle::mir::Mutability::Mut
                )
            );
            ffi_call_fn.params.add_param(
                ::rustc_span::symbol::kw::Empty,
                ::rustc_middle::mir::Mutability::Not,
                pcx.mk_raw_ptr_ty(
                    pcx.mk_path_ty(pcx.mk_path_with_args(
                        pcx.mk_item_path(&["std", "ffi", "c_char",]),
                        &[]
                    )),
                    ::rustc_middle::mir::Mutability::Not
                )
            );
        }
    }
}

#[test]
fn test_cve_2020_35892_3() {
    test_case!(
        pat! {
            #[meta($T:ty)]
            struct $SlabT {
                $mem: *mut $T,
                $len: usize,
            }
        } => quote! {
            #[allow(non_snake_case)]
            let SlabT_adt = pattern.new_struct(::rustc_span::Symbol::intern("SlabT"));

            #[allow(non_snake_case)]
            let T_ty_var = SlabT_adt.meta.new_ty_var(&[]);
            #[allow(non_snake_case)]
            let T_ty = pcx.mk_var_ty(T_ty_var);

            #[allow(non_snake_case)]
            let SlabT_adt = SlabT_adt.non_enum_variant_mut();

            SlabT_adt.add_field(::rustc_span::Symbol::intern("mem"), pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Mut));
            SlabT_adt.add_field(::rustc_span::Symbol::intern("len"), pcx.primitive_types.usize);
        }
    );
    mir_test_case!(
        #[meta($T:ty, $SlabT:ty)]
        pat! {
            let $self: &mut $SlabT;
            #[export(len)]
            let $len: usize = copy (*$self).len;
            let $len_isize: isize = move $len as isize (IntToInt);
            let $base: *mut $T = copy (*$self).mem;
            #[export(ptr)]
            let $ptr_mut: *mut $T = Offset(copy $base, copy $len_isize);
            let $ptr: *const $T = copy $ptr_mut as *const $T (PtrToPtr);
            #[export(read)]
            let $elem: $T = copy (*$ptr);
        } => {
            meta! {
                #[allow(non_snake_case)]
                let T_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let T_ty = pcx.mk_var_ty(T_ty_var);

                #[allow(non_snake_case)]
                let SlabT_ty_var = pattern_fn.meta.new_ty_var(&[]);
                #[allow(non_snake_case)]
                let SlabT_ty = pcx.mk_var_ty(SlabT_ty_var);
            }
            let self_local = mir_pat.mk_self(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReAny,
                SlabT_ty,
                ::rustc_middle::mir::Mutability::Mut
            ));
            let len_local = mir_pat.mk_local(pcx.primitive_types.usize);
            len = mir_pat.mk_assign(
                len_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(::rpl_context::pat::Place::new(
                    self_local,
                    pcx.mk_slice(&[
                        ::rpl_context::pat::PlaceElem::Deref,
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(
                            ::rustc_span::Symbol::intern("len")
                        )),
                    ])
                )))
            );
            let len_isize_local = mir_pat.mk_local(pcx.primitive_types.isize);
            mir_pat.mk_assign(
                len_isize_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::IntToInt,
                    ::rpl_context::pat::Operand::Move(len_local.into_place()),
                    pcx.primitive_types.isize
                )
            );
            let base_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Mut));
            mir_pat.mk_assign(
                base_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(::rpl_context::pat::Place::new(
                    self_local,
                    pcx.mk_slice(&[
                        ::rpl_context::pat::PlaceElem::Deref,
                        ::rpl_context::pat::PlaceElem::Field(::rpl_context::pat::FieldAcc::Named(
                            ::rustc_span::Symbol::intern("mem")
                        )),
                    ])
                )))
            );
            let ptr_mut_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Mut));
            ptr = mir_pat.mk_assign(
                ptr_mut_local.into_place(),
                ::rpl_context::pat::Rvalue::BinaryOp(
                    ::rustc_middle::mir::BinOp::Offset,
                    Box::new([
                        ::rpl_context::pat::Operand::Copy(base_local.into_place()),
                        ::rpl_context::pat::Operand::Copy(len_isize_local.into_place())
                    ])
                )
            );
            let ptr_local = mir_pat.mk_local(pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Not));
            mir_pat.mk_assign(
                ptr_local.into_place(),
                ::rpl_context::pat::Rvalue::Cast(
                    ::rustc_middle::mir::CastKind::PtrToPtr,
                    ::rpl_context::pat::Operand::Copy(ptr_mut_local.into_place()),
                    pcx.mk_raw_ptr_ty(T_ty, ::rustc_middle::mir::Mutability::Not)
                )
            );
            let elem_local = mir_pat.mk_local(T_ty);
            read = mir_pat.mk_assign(
                elem_local.into_place(),
                ::rpl_context::pat::Rvalue::Use(::rpl_context::pat::Operand::Copy(::rpl_context::pat::Place::new(
                    ptr_local,
                    pcx.mk_slice(&[::rpl_context::pat::PlaceElem::Deref,])
                )))
            );
        }
    );
}

#[test]
fn test_cve_2020_35907() {
    test_case! {
        pat! {
            #[meta($T:ty where is_sync)]
            fn $pattern(..) -> &'static $T = mir! {
                let $result: core::result::Result<&'static $T, _> =
                    std::thread::LocalKey::<std::cell::UnsafeCell<std::task::Waker>>::try_with::<_, _>(_, _);
                #[export(ret)]
                let $RET: &'static $T = core::result::Result::expect(move $result, _);
            }
        } => quote! {
            let pattern_fn = pattern.fns.new_fn_pat(::rustc_span::Symbol::intern("pattern"));
            #[allow(non_snake_case)]
            let T_ty_var = pattern_fn.meta.new_ty_var(&[
                &[
                    either::Either::Left(is_sync),
                ],
            ]);
            #[allow(non_snake_case)]
            let T_ty = pcx.mk_var_ty(T_ty_var);
            pattern_fn.set_ret_ty(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReStatic,
                T_ty,
                ::rustc_middle::mir::Mutability::Not
            ));
            let mut mir_pat = ::rpl_context::pat::MirPattern::builder();
            let result_local = mir_pat.mk_local(
                pcx.mk_path_ty(
                    pcx.mk_path_with_args(
                        pcx.mk_item_path(&["core", "result", "Result",]),
                        &[
                            pcx.mk_ref_ty(
                                ::rpl_context::pat::RegionKind::ReStatic,
                                T_ty,
                                ::rustc_middle::mir::Mutability::Not
                            )
                            .into(),
                            pcx.mk_any_ty().into(),
                        ]
                    )
                )
            );
            mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(
                    mir_pat.mk_zeroed(
                        pcx.mk_path_with_args(
                            pcx.mk_item_path(&["std", "thread", "LocalKey", "try_with",]),
                            &[
                                pcx.mk_path_ty(
                                    pcx.mk_path_with_args(
                                        pcx.mk_item_path(&["std", "cell", "UnsafeCell",]),
                                        &[pcx.mk_path_ty(pcx.mk_path_with_args(pcx.mk_item_path(&["std", "task", "Waker",]), &[])).into(),]
                                    )
                                )
                                .into(),
                                pcx.mk_any_ty().into(),
                                pcx.mk_any_ty().into(),
                            ]
                        )
                    )
                ),
                mir_pat.mk_list([::rpl_context::pat::Operand::Any, ::rpl_context::pat::Operand::Any]),
                Some(result_local.into_place())
            );
            let RET_local = mir_pat.mk_return(pcx.mk_ref_ty(
                ::rpl_context::pat::RegionKind::ReStatic,
                T_ty,
                ::rustc_middle::mir::Mutability::Not
            ));
            ret = mir_pat.mk_fn_call(
                ::rpl_context::pat::Operand::Constant(
                    mir_pat.mk_zeroed(pcx.mk_path_with_args(pcx.mk_item_path(&["core", "result", "Result", "expect",]), &[]))
                ),
                mir_pat.mk_list([
                    ::rpl_context::pat::Operand::Move(result_local.into_place()),
                    ::rpl_context::pat::Operand::Any
                ]),
                Some(RET_local.into_place())
            );
            let mir_pat = mir_pat.build();
            let mir_pat = pcx.mk_mir_pattern(mir_pat);
            pattern_fn.set_body(::rpl_context::pat::FnBody::Mir(mir_pat));
        }
    }
}
