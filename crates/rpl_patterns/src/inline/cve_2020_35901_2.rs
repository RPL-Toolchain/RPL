use rpl_context::PatCtxt;
use rpl_mir::{CheckMirCtxt, pat};
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{self as hir};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::ty::TyCtxt;
use rustc_span::{Span, Symbol};

use crate::lints::UNSOUND_PIN_PROJECT;

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
        // let vis = self.tcx.local_visibility(def_id);
        // FIXME: should check accessibility of trait methods
        if self.tcx.is_mir_available(def_id)
        // && (vis == ty::Visibility::Public || vis == ty::Visibility::Restricted(CRATE_DEF_ID))
        {
            let body = self.tcx.optimized_mir(def_id);

            let pattern = pattern_pin(self.pcx);
            for matches in CheckMirCtxt::new(self.tcx, self.pcx, body, pattern.pattern, pattern.fn_pat).check() {
                let span = matches[pattern.pin_new].span_no_inline(body);
                let mut_self = body.local_decls[matches[pattern.mut_self]].source_info.span;
                let ty = matches[pattern.ty_var.idx];
                debug!(?span, ?mut_self, ?ty);
                self.tcx.emit_node_span_lint(
                    UNSOUND_PIN_PROJECT,
                    self.tcx.local_def_id_to_hir_id(def_id),
                    span,
                    crate::errors::UnsoundPinNewUnchecked { span, mut_self, ty },
                );
            }
            let pattern = pattern_pin_field(self.pcx);
            for matches in CheckMirCtxt::new(self.tcx, self.pcx, body, pattern.pattern, pattern.fn_pat).check() {
                let span = matches[pattern.pin_new].span_no_inline(body);
                let mut_self = body.local_decls[matches[pattern.mut_self]].source_info.span;
                let ty = matches[pattern.ty_var.idx];
                debug!(?span, ?mut_self, ?ty);
                self.tcx.emit_node_span_lint(
                    UNSOUND_PIN_PROJECT,
                    self.tcx.local_def_id_to_hir_id(def_id),
                    span,
                    crate::errors::UnsoundPinNewUnchecked { span, mut_self, ty },
                );
            }
        }
        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}

struct PatternPin<'pcx> {
    pattern: &'pcx pat::Pattern<'pcx>,
    fn_pat: &'pcx pat::Fn<'pcx>,
    mut_self: pat::Local,
    pin_new: pat::Location,
    ty_var: pat::TyVar<'pcx>,
}

#[rpl_macros::pattern_def]
fn pattern_pin(pcx: PatCtxt<'_>) -> PatternPin<'_> {
    let ty_var;
    let mut_self;
    let pin_new;
    #[allow(non_snake_case)]
    let pattern = rpl! {
        #[meta($S:ty)]
        struct $SizedStream {
            $field: $S,
        }

        #[meta(#[export(ty_var)] $S:ty where rpl_predicates::is_not_unpin)]
        fn $pattern(..) -> _ = mir! {
            #[export(mut_self)]
            let $self: &mut $SizedStream;
            #[export(pin_new)]
            let mut $pin_mut_struct: std::pin::Pin<&mut $SizedStream> = std::pin::Pin::<_> { __pointer: copy $self };
            let mut $mut_struct: &mut $SizedStream = copy ($pin_mut_struct.__pointer);
            let $mut_field: &mut $S = &mut ((*$mut_struct).$field);
            let mut $pin_mut_field: std::pin::Pin<&mut $S> = std::pin::Pin::<_> { __pointer: copy $mut_field };
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    PatternPin {
        pattern,
        fn_pat,
        mut_self,
        pin_new,
        ty_var,
    }
}

#[rpl_macros::pattern_def]
fn pattern_pin_field(pcx: PatCtxt<'_>) -> PatternPin<'_> {
    let ty_var;
    let mut_self;
    let pin_new;
    #[allow(non_snake_case)]
    let pattern = rpl! {
        #[meta($T:ty)]
        struct $Framed {
            $field: $T,
        }

        #[meta(#[export(ty_var)] $T:ty where rpl_predicates::is_not_unpin)]
        fn $pattern(..) -> _ = mir! {
            #[export(mut_self)]
            let $self: &mut $Framed;
            let mut $self_field: &mut $T = &mut (*$self).$field;
            #[export(pin_new)]
            let mut $pin_mut_struct: std::pin::Pin<&mut $T> = std::pin::Pin::<_> { __pointer: copy $self_field };
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    PatternPin {
        pattern,
        fn_pat,
        mut_self,
        pin_new,
        ty_var,
    }
}
