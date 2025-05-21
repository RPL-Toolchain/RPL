use std::ops::Not;

use rpl_context::PatCtxt;
use rpl_mir::{CheckMirCtxt, pat};
use rustc_hir as hir;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::{Span, Symbol};

use crate::lints::UNSOUND_SLICE_CAST;

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

    #[instrument(level = "debug", skip_all, fields(?item.owner_id))]
    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) -> Self::Result {
        match item.kind {
            hir::ItemKind::Trait(hir::IsAuto::No, hir::Safety::Safe, ..)
            | hir::ItemKind::Impl(_)
            | hir::ItemKind::Fn { .. } => {},
            _ => return,
        }
        intravisit::walk_item(self, item);
    }

    #[instrument(level = "info", skip_all, fields(?def_id))]
    fn visit_fn(
        &mut self,
        kind: intravisit::FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body_id: hir::BodyId,
        _span: Span,
        def_id: LocalDefId,
    ) -> Self::Result {
        if self.tcx.visibility(def_id).is_public()
            && kind.header().is_none_or(|header| header.is_unsafe().not())
            && self.tcx.is_mir_available(def_id)
        {
            let body = self.tcx.optimized_mir(def_id);
            let pattern_cast = pattern_cast(self.pcx);
            for matches in
                CheckMirCtxt::new(self.tcx, self.pcx, body, pattern_cast.pattern, pattern_cast.fn_pat).check()
            {
                let cast_from = matches[pattern_cast.cast_from].span_no_inline(body);
                let cast_to = matches[pattern_cast.cast_to].span_no_inline(body);
                let ty = matches[pattern_cast.ty_var];
                debug!(?cast_from, ?cast_to, ?ty);
                self.tcx.emit_node_span_lint(
                    UNSOUND_SLICE_CAST,
                    self.tcx.local_def_id_to_hir_id(def_id),
                    cast_to,
                    crate::errors::UnsoundSliceCast {
                        cast_from,
                        cast_to,
                        ty,
                        mutability: ty::Mutability::Not.into(),
                    },
                );
            }

            let pattern_cast_mut = pattern_cast_mut(self.pcx);
            for matches in CheckMirCtxt::new(
                self.tcx,
                self.pcx,
                body,
                pattern_cast_mut.pattern,
                pattern_cast_mut.fn_pat,
            )
            .check()
            {
                let cast_from = matches[pattern_cast_mut.cast_from].span_no_inline(body);
                let cast_to = matches[pattern_cast_mut.cast_to].span_no_inline(body);
                let ty = matches[pattern_cast_mut.ty_var];
                debug!(?cast_from, ?cast_to, ?ty);
                self.tcx.emit_node_span_lint(
                    UNSOUND_SLICE_CAST,
                    self.tcx.local_def_id_to_hir_id(def_id),
                    cast_to,
                    crate::errors::UnsoundSliceCast {
                        cast_from,
                        cast_to,
                        ty,
                        mutability: ty::Mutability::Mut.into(),
                    },
                );
            }
        }
        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}

struct PatternCast<'pcx> {
    pattern: &'pcx pat::Pattern<'pcx>,
    fn_pat: &'pcx pat::Fn<'pcx>,
    ty_var: pat::TyVarIdx,
    cast_from: pat::Location,
    cast_to: pat::Location,
}

#[rpl_macros::pattern_def]
fn pattern_cast(pcx: PatCtxt<'_>) -> PatternCast<'_> {
    let ty_var;
    let cast_from;
    let cast_to;
    let pattern = rpl! {
        #[meta( #[export(ty_var)] $T:ty
            where rpl_predicates::is_all_safe_trait && !rpl_predicates::is_primitive)]
        fn $pattern (..) -> _ = mir! {
            #[export(cast_from)]
            let $from_slice: &[$T] = _;
            let $from_raw: *const [$T] = &raw const *$from_slice;
            let $from_len: usize = PtrMetadata(copy $from_slice);
            let $ty_size: usize = SizeOf($T);
            let $to_ptr_t: *const T = move $from_raw as *const $T (PtrToPtr);
            let $to_ptr: *const u8 = move $to_ptr_t as *const u8 (PtrToPtr);
            let $to_len: usize = Mul(move $from_len, move $ty_size);
            let $to_raw: *const [u8] = *const [u8] from (copy $to_ptr, copy $to_len);
            #[export(cast_to)]
            let $to_slice: &[u8] = &*$to_raw;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    PatternCast {
        pattern,
        fn_pat,

        ty_var: ty_var.idx,
        cast_from,
        cast_to,
    }
}

#[rpl_macros::pattern_def]
fn pattern_cast_mut(pcx: PatCtxt<'_>) -> PatternCast<'_> {
    let ty_var;
    let cast_from;
    let cast_to;
    let pattern = rpl! {
        #[meta( #[export(ty_var)] $T:ty
            where rpl_predicates::is_all_safe_trait && !rpl_predicates::is_primitive)]
        fn $pattern (..) -> _ = mir! {

            #[export(cast_from)]
            let $from_slice_mut: &mut [$T] = _;
            let $from_slice_ref: &[$T] = &*$from_slice_mut;
            let $from_raw_mut: *mut [$T] = &raw mut *$from_slice_mut;
            let $from_len_mut: usize = PtrMetadata(move $from_slice_ref);
            let $ty_size_mut: usize = SizeOf($T);
            let $to_ptr_mut_t: *mut $T = move $from_raw_mut as *mut $T (PtrToPtr);
            let $to_ptr_mut: *mut u8 = move $to_ptr_mut_t as *mut u8 (PtrToPtr);
            let $to_len_mut: usize = Mul(move $from_len_mut, move $ty_size_mut);
            let $to_raw_mut: *mut [u8] = *mut [u8] from (copy $to_ptr_mut, copy $to_len_mut);
            #[export(cast_to)]
            let $to_slice_mut: &mut [u8] = &mut *$to_raw_mut;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    PatternCast {
        pattern,
        fn_pat,
        ty_var: ty_var.idx,
        cast_from,
        cast_to,
    }
}
