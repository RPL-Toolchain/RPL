use crate::lints::UNCHECKED_POINTER_OFFSET;
use rpl_context::PatCtxt;
use rpl_mir::{CheckMirCtxt, pat};
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{self as hir};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::ty::TyCtxt;
use rustc_span::{Span, Symbol};
use std::ops::Not;

#[instrument(level = "info", skip_all)]
pub fn check_item(tcx: TyCtxt<'_>, pcx: PatCtxt<'_>, item_id: hir::ItemId) {
    let item = tcx.hir().item(item_id);
    // let def_id = item_id.owner_id.def_id;
    let mut check_ctxt = CheckFnCtxt::new(tcx, pcx);
    check_ctxt.visit_item(item);
}

struct CheckFnCtxt<'pcx, 'tcx> {
    tcx: TyCtxt<'tcx>,
    pcx: PatCtxt<'pcx>,
}

impl<'pcx, 'tcx> CheckFnCtxt<'pcx, 'tcx> {
    fn new(tcx: TyCtxt<'tcx>, pcx: PatCtxt<'pcx>) -> Self {
        Self { tcx, pcx }
    }
}

impl<'tcx> Visitor<'tcx> for CheckFnCtxt<'_, 'tcx> {
    type NestedFilter = All;
    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    #[instrument(level = "debug", skip_all, fields(?item.owner_id))]
    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) -> Self::Result {
        match item.kind {
            hir::ItemKind::Trait(hir::IsAuto::No, ..) | hir::ItemKind::Impl(_) | hir::ItemKind::Fn { .. } => {},
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
        // let attrs: Vec<_> = self
        //     .tcx
        //     .get_attrs_by_path(def_id.to_def_id(), &[Symbol::intern("rpl"), Symbol::intern("check")])
        //     .collect();
        // info!("attrs: {:?}", attrs);
        // if attrs.is_empty() {
        //     return;
        // }

        if kind.header().is_none_or(|header| header.is_unsafe().not())
            && self.tcx.visibility(def_id).is_public()
            && self.tcx.is_mir_available(def_id)
        {
            let body = self.tcx.optimized_mir(def_id);

            for pattern in [
                pattern_unchecked_ptr_offset(self.pcx),
                pattern_unchecked_mut_ptr_offset(self.pcx),
                pattern_unchecked_ptr_casted_offset(self.pcx),
                pattern_unchecked_mut_ptr_casted_offset(self.pcx),
                pattern_unchecked_ptr_arith_offset(self.pcx),
                pattern_unchecked_mut_ptr_arith_offset(self.pcx),
                pattern_unchecked_ptr_casted_arith_offset(self.pcx),
                pattern_unchecked_mut_ptr_casted_arith_offset(self.pcx),
            ] {
                let matches = CheckMirCtxt::new(self.tcx, self.pcx, body, pattern.pattern, pattern.fn_pat).check();
                for matches in matches {
                    let len = matches[pattern.len];
                    if !len.is_arg(body) {
                        trace!(?len, "not an argument, skipping");
                        continue;
                    }
                    let ptr = matches[pattern.ptr];
                    let offset = matches[pattern.offset];
                    let len = matches[pattern.len];
                    debug!(?ptr, ?offset, ?len, ?pattern.ptr, ?pattern.offset, ?pattern.len, "unchecked offset found");
                    let ptr = ptr.span_no_inline(body);
                    let offset = offset.span_no_inline(body);
                    let len = len.span_no_inline(body);
                    self.tcx.emit_node_span_lint(
                        UNCHECKED_POINTER_OFFSET,
                        self.tcx.local_def_id_to_hir_id(def_id),
                        offset,
                        crate::errors::UncheckedPtrPublicOffset { ptr, offset, len },
                    );
                }
            }
        }
        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}

struct PatternUncheckedPtrOffsetGeneral<'pcx> {
    pattern: &'pcx pat::Pattern<'pcx>,
    fn_pat: &'pcx pat::Fn<'pcx>,
    len: pat::Location,
    ptr: pat::Location,
    offset: pat::Location,
}

macro_rules! template {
    ($name:ident -> $ret:ident { $($fields:ident),* $(,)? } {$($inner:tt)*}) => {
        #[rpl_macros::pattern_def]
        fn $name(pcx: PatCtxt<'_>) -> $ret<'_> {
            $(
                let $fields;
            )*
            let pattern = rpl! {
                $($inner)*
            };
            let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

            $ret { pattern, fn_pat, $($fields),* }
        }
    };
}

template! {
    pattern_unchecked_ptr_offset -> PatternUncheckedPtrOffsetGeneral { len, ptr, offset } {
        #[meta($T: ty, $U: ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(len)]
            let $len: $U = _;
            #[export(ptr)]
            let $ptr: *const $T = _;
            #[export(offset)]
            let $ptr_1: *const $T = Offset(copy $ptr, copy $len);
        }
    }
}

template! {
    pattern_unchecked_mut_ptr_offset -> PatternUncheckedPtrOffsetGeneral { len, ptr, offset } {
        #[meta($T: ty, $U: ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(len)]
            let $len: $U = _;
            #[export(ptr)]
            let $ptr: *mut $T = _;
            #[export(offset)]
            let $ptr_1: *mut $T = Offset(copy $ptr, copy $len);
        }
    }
}

template! {
    pattern_unchecked_ptr_casted_offset -> PatternUncheckedPtrOffsetGeneral { len, ptr, offset } {
        #[meta($T: ty, $U1: ty, $U2: ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(len)]
            let $len1: $U1 = _;
            let $len2: $U2 = copy $len1 as $U2 (IntToInt);
            #[export(ptr)]
            let $ptr: *const $T = _;
            #[export(offset)]
            let $ptr_1: *const $T = Offset(copy $ptr, copy $len2);
        }
    }
}

template! {
    pattern_unchecked_mut_ptr_casted_offset -> PatternUncheckedPtrOffsetGeneral { len, ptr, offset } {
        #[meta($T: ty, $U1: ty, $U2: ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(len)]
            let $len1: $U1 = _;
            let $len2: $U2 = copy $len1 as $U2 (IntToInt);
            #[export(ptr)]
            let $ptr: *mut $T = _;
            #[export(offset)]
            let $ptr_1: *mut $T = Offset(copy $ptr, copy $len2);
        }
    }
}

template! {
    pattern_unchecked_ptr_arith_offset -> PatternUncheckedPtrOffsetGeneral { len, ptr, offset } {
        #[meta($T: ty, $U: ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(len)]
            let $len: $U = _; // _6
            #[export(ptr)]
            let $ptr: *const $T = _; // _8
            #[export(offset)]
            let $ptr_1: *const $T = std::intrinsics::arith_offset::<$T>(copy $ptr, copy $len); // _7
        }
    }
}

template! {
    pattern_unchecked_mut_ptr_arith_offset -> PatternUncheckedPtrOffsetGeneral { len, ptr, offset } {
        #[meta($T: ty, $U: ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(len)]
            let $len: $U = _; // _6
            #[export(ptr)]
            let $ptr: *mut $T = _; // _8
            #[export(offset)]
            let $ptr_1: *mut $T = std::intrinsics::arith_offset::<$T>(copy $ptr, copy $len); // _7
        }
    }
}

template! {
    pattern_unchecked_ptr_casted_arith_offset -> PatternUncheckedPtrOffsetGeneral { len, ptr, offset } {
        #[meta($T: ty, $U1: ty, $U2: ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(len)]
            let $len1: $U1 = _; // _2
            let $len2: $U2 = copy $len1 as $U2 (IntToInt); // _6
            #[export(ptr)]
            let $ptr: *const $T = _; // _8
            #[export(offset)]
            let $ptr_1: *const $T = std::intrinsics::arith_offset::<$T>(copy $ptr, copy $len2); // _7
        }
    }
}

template! {
    pattern_unchecked_mut_ptr_casted_arith_offset -> PatternUncheckedPtrOffsetGeneral { len, ptr, offset } {
        #[meta($T: ty, $U1: ty, $U2: ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(len)]
            let $len1: $U1 = _; // _2
            let $len2: $U2 = copy $len1 as $U2 (IntToInt); // _6
            #[export(ptr)]
            let $ptr: *mut $T = _; // _8
            #[export(offset)]
            let $ptr_1: *mut $T = std::intrinsics::arith_offset::<$T>(copy $ptr, copy $len2); // _7
        }
    }
}
