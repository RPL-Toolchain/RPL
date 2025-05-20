use rpl_context::PatCtxt;
use rpl_mir::{CheckMirCtxt, pat};
use rustc_hir as hir;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::ty::TyCtxt;
use rustc_span::{Span, Symbol};
use std::ops::Not;

use crate::lints::UNVALIDATED_SLICE_FROM_RAW_PARTS;

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
            let pattern = pattern_from_raw_parts_iter(self.pcx);
            for matches in CheckMirCtxt::new(self.tcx, self.pcx, body, pattern.pattern, pattern.fn_pat).check() {
                let slice = matches[pattern.slice].span_no_inline(body);
                let src = matches[pattern.src].span_no_inline(body);
                let ptr = matches[pattern.ptr].span_no_inline(body);
                self.tcx.emit_node_span_lint(
                    UNVALIDATED_SLICE_FROM_RAW_PARTS,
                    self.tcx.local_def_id_to_hir_id(def_id),
                    src,
                    crate::errors::UnvalidatedSliceFromRawParts { src, ptr, slice },
                );
            }
        }
        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}

struct PatternFromRawParts<'pcx> {
    pattern: &'pcx pat::Pattern<'pcx>,
    fn_pat: &'pcx pat::Fn<'pcx>,
    slice: pat::Location,
    src: pat::Location,
    ptr: pat::Location,
}

#[rpl_macros::pattern_def]
fn pattern_from_raw_parts_iter(pcx: PatCtxt<'_>) -> PatternFromRawParts<'_> {
    let src;
    let ptr;
    let slice;
    let pattern = rpl! {
        #[meta($I:ty, $T:ty)]
        fn $pattern (..) -> _ = mir! {
            #[export(src)]
            let $src: $I = _;
            let $src_ref: &mut $I = &mut $src;
            let $next: std::option::Option<*const T> = std::iter::Iterator::next(move $src_ref);
            #[export(ptr)]
            let $ptr: *const T = std::option::Option::unwrap(move $next);
            #[export(slice)]
            let $slice: &[T] = std::slice::from_raw_parts::<'_, $T>(copy $ptr, _);
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    PatternFromRawParts {
        pattern,
        fn_pat,
        src,
        ptr,
        slice,
    }
}
