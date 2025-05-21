use std::ops::Not;

use rpl_context::PatCtxt;
use rpl_mir::{CheckMirCtxt, pat};
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{self as hir};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::ty::TyCtxt;
use rustc_span::{Span, Symbol};

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
            let pattern_transmute_to_bool = pattern_transmute_to_bool(self.pcx);
            for matches in CheckMirCtxt::new(
                self.tcx,
                self.pcx,
                body,
                pattern_transmute_to_bool.pattern,
                pattern_transmute_to_bool.fn_pat,
            )
            .check()
            {
                let transmute_from = matches[pattern_transmute_to_bool.transmute_from].span_no_inline(body);
                let transmute_to = matches[pattern_transmute_to_bool.transmute_to].span_no_inline(body);
                let ty_var = matches[pattern_transmute_to_bool.ty_var.idx];
                debug!(?transmute_from, ?transmute_to, ?ty_var);

                let translate_to_stmt = matches[pattern_transmute_to_bool.transmute_to];
                if let rpl_mir::StatementMatch::Location(loc) = translate_to_stmt {
                    if rpl_predicates::translate_from_hir_function(self.tcx, loc, body, "std::mem::transmute") {
                        self.tcx.emit_node_span_lint(
                            crate::lints::TRANSMUTING_TYPE_TO_BOOL,
                            self.tcx.local_def_id_to_hir_id(def_id),
                            transmute_from,
                            crate::errors::TransmutingTypeToBool {
                                from: transmute_from,
                                to: transmute_to,
                                ty: ty_var,
                            },
                        );
                    }
                }
            }
        }
        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}

struct PatternTransmute<'pcx> {
    pattern: &'pcx pat::Pattern<'pcx>,
    fn_pat: &'pcx pat::Fn<'pcx>,
    transmute_from: pat::Location,
    transmute_to: pat::Location,
    ty_var: pat::TyVar<'pcx>,
}

// The pattern for transmuting a type to a boolean
#[rpl_macros::pattern_def]
fn pattern_transmute_to_bool(pcx: PatCtxt<'_>) -> PatternTransmute<'_> {
    let ty_var;
    let transmute_from;
    let transmute_to;
    let pattern = rpl! {
        #[meta( #[export(ty_var)] $T:ty)]
        fn $pattern (..) -> _ = mir! {
            #[export(transmute_from)]
            let $transmute_from: $T = _;
            #[export(transmute_to)]
            // FIXME: move and copy are both allowed here
            let $transmute_to: bool = move $transmute_from as bool (Transmute);
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    PatternTransmute {
        pattern,
        fn_pat,
        transmute_from,
        transmute_to,
        ty_var,
    }
}
