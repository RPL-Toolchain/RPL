use rustc_hir as hir;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::mir;
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::{Span, Symbol};

use rpl_context::{PatCtxt, pat};
use rpl_mir::CheckMirCtxt;

use crate::lints::{ALLOC_MAYBE_ZERO, MISALIGNED_POINTER, USE_AFTER_REALLOC};

#[instrument(level = "info", skip_all)]
pub fn check_item(tcx: TyCtxt<'_>, pcx: PatCtxt<'_>, item_id: hir::ItemId) {
    let item = tcx.hir().item(item_id);
    // let def_id = item_id.owner_id.def_id;
    let mut check_ctxt = CheckFnCtxt { tcx, pcx };
    check_ctxt.visit_item(item);
}

struct CheckFnCtxt<'pcx, 'tcx> {
    tcx: TyCtxt<'tcx>,
    pcx: PatCtxt<'pcx>,
}

impl<'tcx> Visitor<'tcx> for CheckFnCtxt<'_, 'tcx> {
    type NestedFilter = All;
    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) -> Self::Result {
        match item.kind {
            hir::ItemKind::Trait(hir::IsAuto::No, hir::Safety::Safe, ..)
            | hir::ItemKind::Impl(_)
            | hir::ItemKind::Fn { .. } => {},
            _ => return,
        }
        intravisit::walk_item(self, item);
    }

    #[instrument(level = "info", skip(self, kind, decl, _span))]
    fn visit_fn(
        &mut self,
        kind: intravisit::FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body_id: hir::BodyId,
        _span: Span,
        def_id: LocalDefId,
    ) -> Self::Result {
        // let attrs: Vec<_> = self
        //     .tcx
        //     .get_attrs_by_path(def_id.to_def_id(), &[Symbol::intern("rpl"), Symbol::intern("check")])
        //     .collect();
        // info!("attrs: {:?}", attrs);
        // if attrs.is_empty() {
        //     return;
        // }

        if self.tcx.is_mir_available(def_id) {
            let body = self.tcx.optimized_mir(def_id);

            if kind.header().is_none_or(|header| !header.is_unsafe()) {
                let pattern = alloc_misaligned_cast(self.pcx);

                for matches in CheckMirCtxt::new(self.tcx, self.pcx, body, pattern.pattern, pattern.fn_pat).check() {
                    let alloc = matches[pattern.alloc].span_no_inline(body);
                    let write = matches[pattern.cast].span_no_inline(body);
                    let ty = matches[pattern.ty.idx];
                    let alignment = matches[pattern.alignment.idx];

                    if maybe_misaligned(self.tcx, body, ty, alignment) {
                        self.tcx.emit_node_span_lint(
                            MISALIGNED_POINTER,
                            self.tcx.local_def_id_to_hir_id(def_id),
                            write,
                            crate::errors::MisalignedPointer { alloc, write, ty },
                        );
                    }
                }
            }

            for pattern in [
                use_after_realloc_deref_const(self.pcx),
                use_after_realloc_deref_mut(self.pcx),
                use_after_realloc_read_const(self.pcx),
                use_after_realloc_read_mut(self.pcx),
                use_after_realloc_write_const(self.pcx),
                use_after_realloc_write_mut(self.pcx),
            ] {
                for matches in CheckMirCtxt::new(self.tcx, self.pcx, body, pattern.pattern, pattern.fn_pat).check() {
                    let realloc = matches[pattern.realloc].span_no_inline(body);
                    let deref = matches[pattern.deref].span_no_inline(body);
                    let ty = matches[pattern.ty.idx];

                    self.tcx.emit_node_span_lint(
                        USE_AFTER_REALLOC,
                        self.tcx.local_def_id_to_hir_id(def_id),
                        deref,
                        crate::errors::UseAfterRealloc {
                            realloc,
                            r#use: deref,
                            ty,
                        },
                    );
                }
            }

            // For `unsafe` functions, it's the caller's responsibility to ensure that the allocation is safe.
            // So we only check `alloc_maybe_zero` for safe functions.
            if kind.header().is_none_or(|header| !header.is_unsafe()) && self.tcx.visibility(def_id).is_public() {
                for pattern in [alloc_maybe_zero_mul(self.pcx), alloc_zeroed_maybe_zero_mul(self.pcx)] {
                    for matches in CheckMirCtxt::new(self.tcx, self.pcx, body, pattern.pattern, pattern.fn_pat).check()
                    {
                        let alloc = matches[pattern.alloc].span_no_inline(body);
                        let size_arg = matches[pattern.size];
                        let fn_name = self.tcx.item_name(def_id.to_def_id());
                        let alloc_fn = pattern.alloc_fn;

                        // debug!("{:?} {}", size_arg, body.arg_count);

                        // Check if `size` is an argument.
                        if size_arg.as_usize() < 1 + body.arg_count {
                            let size = body.local_decls[size_arg].source_info.span;
                            self.tcx.emit_node_span_lint(
                                ALLOC_MAYBE_ZERO,
                                self.tcx.local_def_id_to_hir_id(def_id),
                                alloc,
                                crate::errors::AllocMaybeZero {
                                    alloc,
                                    size,
                                    fn_name,
                                    alloc_fn,
                                },
                            );
                        }
                    }
                }
            }
        }
        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}

struct Pattern2<'pcx> {
    pattern: &'pcx pat::Pattern<'pcx>,
    fn_pat: &'pcx pat::Fn<'pcx>,
    alloc: pat::Location,
    cast: pat::Location,
    ty: pat::TyVar<'pcx>,
    alignment: pat::ConstVar<'pcx>,
}

#[rpl_macros::pattern_def]
fn alloc_misaligned_cast(pcx: PatCtxt<'_>) -> Pattern2<'_> {
    let alloc;
    let cast;
    let ty;
    let alignment;
    let pattern = rpl! {
        #[meta(
            #[export(ty)] $T:ty
                where rpl_predicates::is_all_safe_trait,
            #[export(alignment)] $alignment: const(usize)
        )]
        fn $pattern(..) -> _ = mir! {
            let $layout_result: core::result::Result<core::alloc::Layout, _> = alloc::alloc::Layout::from_size_align(
                _,
                const $alignment
            );
            let $layout: core::alloc::Layout = core::result::Result::unwrap(move $layout_result);
            #[export(alloc)]
            let $ptr_1: *mut u8 = alloc::alloc::alloc(copy $layout);
            #[export(cast)]
            let $ptr_2: *mut $T = move $ptr_1 as *mut $T (PtrToPtr);
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern2 {
        pattern,
        fn_pat,
        alloc,
        cast,
        ty,
        alignment,
    }
}

#[instrument(level = "debug", skip(tcx), ret)]
fn maybe_misaligned<'tcx>(
    tcx: ty::TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    ty: Ty<'tcx>,
    alignment: mir::Const<'tcx>,
) -> bool {
    let typing_env = ty::TypingEnv::post_analysis(tcx, body.source.def_id());
    match ty.kind() {
        // Param types can be anything, and we don't know the alignment.
        // Also, param types with unsafe traits have been filtered out in `is_all_safe_trait`.
        ty::TyKind::Param(_) => true,
        // foreign types are opaque to Rust
        ty::TyKind::Foreign(_) => true,
        _ => {
            let layout = tcx.layout_of(typing_env.as_query_input(ty)).unwrap();
            alignment.eval_target_usize(tcx, typing_env) < layout.align.pref.bytes()
        },
    }
}

struct Pattern3<'pcx> {
    pattern: &'pcx pat::Pattern<'pcx>,
    fn_pat: &'pcx pat::Fn<'pcx>,
    realloc: pat::Location,
    deref: pat::Location,
    ty: pat::TyVar<'pcx>,
}

#[rpl_macros::pattern_def]
fn use_after_realloc_deref_const(pcx: PatCtxt<'_>) -> Pattern3<'_> {
    let realloc;
    let deref;
    let ty;
    let pattern = rpl! {
        #[meta(#[export(ty)] $T:ty)]
        fn $pattern(..) -> _ = mir! {
            let $old_ptr: *const $T = _;
            let $old_ptr_u8: *mut u8 = copy $old_ptr as *mut u8 (PtrToPtr);
            #[export(realloc)]
            let $new_ptr_u8: *mut u8 = alloc::alloc::realloc(move $old_ptr_u8, _, _);
            #[export(deref)]
            let $ref_old: &$T = &*$old_ptr;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern3 {
        pattern,
        fn_pat,
        realloc,
        deref,
        ty,
    }
}

#[rpl_macros::pattern_def]
fn use_after_realloc_deref_mut(pcx: PatCtxt<'_>) -> Pattern3<'_> {
    let realloc;
    let deref;
    let ty;
    let pattern = rpl! {
        #[meta(#[export(ty)] $T:ty)]
        fn $pattern(..) -> _ = mir! {
            let $old_ptr: *mut $T = _;
            let $old_ptr_u8: *mut u8 = copy $old_ptr as *mut u8 (PtrToPtr);
            #[export(realloc)]
            let $new_ptr_u8: *mut u8 = alloc::alloc::realloc(move $old_ptr_u8, _, _);
            #[export(deref)]
            let $ref_old: &mut $T = &mut *$old_ptr;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern3 {
        pattern,
        fn_pat,
        realloc,
        deref,
        ty,
    }
}

#[rpl_macros::pattern_def]
fn use_after_realloc_read_const(pcx: PatCtxt<'_>) -> Pattern3<'_> {
    let realloc;
    let deref;
    let ty;
    let pattern = rpl! {
        #[meta(#[export(ty)] $T:ty)]
        fn $pattern(..) -> _ = mir! {
            let $old_ptr: *const $T = _;
            let $old_ptr_u8: *mut u8 = copy $old_ptr as *mut u8 (PtrToPtr);
            #[export(realloc)]
            let $new_ptr_u8: *mut u8 = alloc::alloc::realloc(move $old_ptr_u8, _, _);
            #[export(deref)]
            let $ref_old: $T = copy *$old_ptr;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern3 {
        pattern,
        fn_pat,
        realloc,
        deref,
        ty,
    }
}

#[rpl_macros::pattern_def]
fn use_after_realloc_read_mut(pcx: PatCtxt<'_>) -> Pattern3<'_> {
    let realloc;
    let deref;
    let ty;
    let pattern = rpl! {
        #[meta(#[export(ty)] $T:ty)]
        fn $pattern(..) -> _ = mir! {
            let $old_ptr: *mut $T = _;
            let $old_ptr_u8: *mut u8 = copy $old_ptr as *mut u8 (PtrToPtr);
            #[export(realloc)]
            let $new_ptr_u8: *mut u8 = alloc::alloc::realloc(move $old_ptr_u8, _, _);
            #[export(deref)]
            let $ref_old: $T = copy *$old_ptr;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern3 {
        pattern,
        fn_pat,
        realloc,
        deref,
        ty,
    }
}

#[rpl_macros::pattern_def]
fn use_after_realloc_write_const(pcx: PatCtxt<'_>) -> Pattern3<'_> {
    let realloc;
    let deref;
    let ty;
    let pattern = rpl! {
        #[meta(#[export(ty)] $T:ty)]
        fn $pattern(..) -> _ = mir! {
            let $old_ptr: *const $T = _;
            let $old_ptr_u8: *mut u8 = copy $old_ptr as *mut u8 (PtrToPtr);
            #[export(realloc)]
            let $new_ptr_u8: *mut u8 = alloc::alloc::realloc(move $old_ptr_u8, _, _);
            #[export(deref)]
            *$old_ptr = _;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern3 {
        pattern,
        fn_pat,
        realloc,
        deref,
        ty,
    }
}

#[rpl_macros::pattern_def]
fn use_after_realloc_write_mut(pcx: PatCtxt<'_>) -> Pattern3<'_> {
    let realloc;
    let deref;
    let ty;
    let pattern = rpl! {
        #[meta(#[export(ty)] $T:ty)]
        fn $pattern(..) -> _ = mir! {
            let $old_ptr: *mut $T = _;
            let $old_ptr_u8: *mut u8 = copy $old_ptr as *mut u8 (PtrToPtr);
            #[export(realloc)]
            let $new_ptr_u8: *mut u8 = alloc::alloc::realloc(move $old_ptr_u8, _, _);
            #[export(deref)]
            *$old_ptr = _;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern3 {
        pattern,
        fn_pat,
        realloc,
        deref,
        ty,
    }
}

struct Pattern4<'pcx> {
    pattern: &'pcx pat::Pattern<'pcx>,
    fn_pat: &'pcx pat::Fn<'pcx>,
    alloc: pat::Location,
    size: pat::Local,
    alloc_fn: &'static str,
}

#[rpl_macros::pattern_def]
fn alloc_zeroed_maybe_zero_mul(pcx: PatCtxt<'_>) -> Pattern4<'_> {
    let size;
    let alloc;
    let pattern = rpl! {
        fn $pattern(..) -> _ = mir! {
            #[export(size)]
            let $count: usize;
            let $size: usize = Mul(copy $count, _);
            let $result: core::result::Result<alloc::alloc::Layout, alloc::alloc::LayoutError> =
                alloc::alloc::Layout::from_size_align(
                    copy $size,
                    _
                );
            let $layout : alloc::alloc::Layout =
                core::result::Result<alloc::alloc::Layout, alloc::alloc::LayoutError>::unwrap(
                    move $result
                );
            #[export(alloc)]
            _ = alloc::alloc::alloc_zeroed(copy $layout);
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern4 {
        pattern,
        fn_pat,
        alloc,
        size,
        alloc_fn: "alloc_zeroed",
    }
}

#[rpl_macros::pattern_def]
fn alloc_maybe_zero_mul(pcx: PatCtxt<'_>) -> Pattern4<'_> {
    let size;
    let alloc;
    let pattern = rpl! {
        fn $pattern(..) -> _ = mir! {
            #[export(size)]
            let $count: usize;
            let $size: usize = Mul(copy $count, _);
            let $result: core::result::Result<alloc::alloc::Layout, alloc::alloc::LayoutError> =
                alloc::alloc::Layout::from_size_align(
                    copy $size,
                    _
                );
            let $layout : alloc::alloc::Layout =
                core::result::Result<alloc::alloc::Layout, alloc::alloc::LayoutError>::unwrap(
                    move $result
                );
            #[export(alloc)]
            _ = alloc::alloc::alloc(copy $layout);
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern4 {
        pattern,
        fn_pat,
        alloc,
        size,
        alloc_fn: "alloc",
    }
}
