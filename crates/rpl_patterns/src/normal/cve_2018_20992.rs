use std::collections::HashSet;

use rpl_context::PatCtxt;
use rustc_hir as hir;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::ty::TyCtxt;
use rustc_span::{Span, Symbol};

use rpl_mir::{CheckMirCtxt, pat};

use crate::lints::SET_LEN_TO_EXTEND;

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
            hir::ItemKind::Trait(hir::IsAuto::No, ..) | hir::ItemKind::Impl(_) | hir::ItemKind::Fn { .. } => {},
            _ => return,
        }
        intravisit::walk_item(self, item);
    }

    fn visit_fn(
        &mut self,
        _kind: intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        _body_id: hir::BodyId,
        _span: Span,
        def_id: LocalDefId,
    ) -> Self::Result {
        if self.tcx.is_mir_available(def_id) {
            let body = self.tcx.optimized_mir(def_id);
            let pattern = pattern_vec_set_len_to_extend(self.pcx);
            let mut records = HashSet::new();
            for matches in CheckMirCtxt::new(self.tcx, self.pcx, body, pattern.pattern, pattern.fn_pat).check() {
                let record = (pattern.vec, pattern.set_len);
                if records.contains(&record) {
                    continue;
                }
                records.insert(record);
                let set_len = matches[pattern.set_len].span_no_inline(body);
                let vec = matches[pattern.vec].span_no_inline(body);
                self.tcx.emit_node_span_lint(
                    SET_LEN_TO_EXTEND,
                    self.tcx.local_def_id_to_hir_id(def_id),
                    set_len,
                    crate::errors::VecSetLenToExtend { set_len, vec },
                );
            }
        }
    }
}

struct VecSetLenToExtend<'pcx> {
    pattern: &'pcx pat::Pattern<'pcx>,
    fn_pat: &'pcx pat::Fn<'pcx>,
    vec: pat::Location,
    set_len: pat::Location,
}

#[rpl_macros::pattern_def]
fn pattern_vec_set_len_to_extend(pcx: PatCtxt<'_>) -> VecSetLenToExtend<'_> {
    let vec;
    let set_len;
    let pattern = rpl! {
        #[meta($T:ty)]
        fn $pattern(..) -> _ = mir! {
            type VecT = alloc::vec::Vec::<$T>;

            #[export(vec)]
            let $vec: VecT = _;
            let $new_len: usize = _;
            let $vec_ref: &VecT = &$vec;
            let $old_len: usize = std::vec::Vec::len(move $vec_ref);
            let $cmp: bool = Lt(move $old_len, copy $new_len);
            let $vec_mut_ref: &mut VecT;
            let $set_len_res: ();
            #[export(set_len)]
            switchInt(move $cmp) {
                false => {}
                _ => {
                    $vec_mut_ref = &mut $vec;
                    $set_len_res = std::vec::Vec::set_len(move $vec_mut_ref, copy $new_len);
                }
            }
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    VecSetLenToExtend {
        pattern,
        fn_pat,
        vec,
        set_len,
    }
}
