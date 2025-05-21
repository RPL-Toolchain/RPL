use std::collections::BTreeSet;

use rustc_hir as hir;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::mir::Body;
use rustc_middle::ty::TyCtxt;
use rustc_span::{Span, Symbol};

use rpl_context::pat::Location;
use rpl_context::{PatCtxt, pat};
use rpl_mir::{CheckMirCtxt, Matched};

use crate::lints::UNCHECKED_ALLOCATED_POINTER;

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

            let pattern = alloc_cast_write(self.pcx);

            let pattern_2 = alloc_cast_check_write(self.pcx);
            let matches_2 = CheckMirCtxt::new(self.tcx, self.pcx, body, pattern_2.pattern, pattern_2.fn_pat).check();
            let pattern_3 = alloc_check_cast_write(self.pcx);
            let matches_3 = CheckMirCtxt::new(self.tcx, self.pcx, body, pattern_3.pattern, pattern_3.fn_pat).check();
            let pattern_4 = alloc_cast_check_as_write(self.pcx);
            let matches_4 = CheckMirCtxt::new(self.tcx, self.pcx, body, pattern_4.pattern, pattern_4.fn_pat).check();

            fn collect_matched(matched: &Matched<'_>, ptr: Location, write: Location, body: &Body<'_>) -> (Span, Span) {
                let span_alloc = matched[ptr].span_no_inline(body);
                let span_write = matched[write].span_no_inline(body);
                trace!(?span_alloc, ?span_write, "checked write found");
                (span_alloc, span_write)
            }
            let locations: BTreeSet<_> = matches_2
                .iter()
                .map(|matches| collect_matched(matches, pattern_2.alloc, pattern_2.write, body))
                .chain(
                    matches_3
                        .iter()
                        .map(|matches| collect_matched(matches, pattern_3.alloc, pattern_3.write, body)),
                )
                .chain(
                    matches_4
                        .iter()
                        .map(|matches| collect_matched(matches, pattern_4.alloc, pattern_4.write, body)),
                )
                .collect();

            for matches in CheckMirCtxt::new(self.tcx, self.pcx, body, pattern.pattern, pattern.fn_pat).check() {
                let alloc = matches[pattern.alloc].span_no_inline(body);
                let write = matches[pattern.write].span_no_inline(body);
                let ty = matches[pattern.ty.idx];

                if locations.contains(&(alloc, write)) {
                    // The returned pointer is checked, so don't emit an error
                    continue;
                }
                // let global = self.tcx.type_of(global_did).instantiate_identity();
                self.tcx.emit_node_span_lint(
                    UNCHECKED_ALLOCATED_POINTER,
                    self.tcx.local_def_id_to_hir_id(def_id),
                    write,
                    crate::errors::UncheckedAllocatedPointer { alloc, write, ty },
                );
            }

            // let pattern = alloc_misaligned_cast(self.pcx);

            // for matches in CheckMirCtxt::new(self.tcx, self.pcx, body, pattern.pattern,
            // pattern.fn_pat).check() {     let alloc =
            // matches[pattern.alloc].span_no_inline(body);     let write =
            // matches[pattern.cast].span_no_inline(body);

            //     let ty = matches[T];
            //     // let global = self.tcx.type_of(global_did).instantiate_identity();
            //     self.tcx.emit_node_span_lint(
            //         UNCHECKED_ALLOCATED_POINTER,
            //         self.tcx.local_def_id_to_hir_id(def_id),
            //         write,
            //         crate::errors::UncheckedAllocatedPointer { alloc, write, ty },
            //     );
            // }
        }
        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}

struct Pattern<'pcx> {
    pattern: &'pcx pat::Pattern<'pcx>,
    fn_pat: &'pcx pat::Fn<'pcx>,
    alloc: pat::Location,
    write: pat::Location,
    ty: pat::TyVar<'pcx>,
}

#[rpl_macros::pattern_def]
fn alloc_cast_write(pcx: PatCtxt<'_>) -> Pattern<'_> {
    let alloc;
    let write;
    let ty;
    let pattern = rpl! {
        #[meta(#[export(ty)] $T:ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(alloc)]
            let $ptr_1: *mut u8 = alloc::alloc::__rust_alloc(_, _); // _3
            let $ptr_2: *mut $T = move $ptr_1 as *mut $T (PtrToPtr); // _2
            #[export(write)]
            (*$ptr_2) = _;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern {
        pattern,
        fn_pat,
        alloc,
        write,
        ty,
    }
}

#[rpl_macros::pattern_def]
fn alloc_check_cast_write(pcx: PatCtxt<'_>) -> Pattern<'_> {
    let alloc;
    let write;
    let ty;
    let pattern = rpl! {
        #[meta(#[export(ty)] $T:ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(alloc)]
            let $ptr_1: *mut u8 = alloc::alloc::__rust_alloc(_, _); // _2
            let $const_ptr_1: *const u8 = copy $ptr_1 as *const u8 (PtrToPtr); // _19
            let $addr_1: usize = copy $const_ptr_1 as usize (Transmute); // _20
            // It's weird that `$ptr_2` can only be declared before `switchInt`
            // switchInt(move $addr_1) {
            //     0_usize => {}
            //     _ => {}
            // }
            let $ptr_2: *mut $T = copy $ptr_1 as *mut $T (PtrToPtr); // _4
            #[export(write)]
            (*$ptr_2) = _;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern {
        pattern,
        fn_pat,
        alloc,
        write,
        ty,
    }
}

#[rpl_macros::pattern_def]
fn alloc_cast_check_write(pcx: PatCtxt<'_>) -> Pattern<'_> {
    let alloc;
    let write;
    let ty;
    let pattern = rpl! {
        #[meta(#[export(ty)] $T:ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(alloc)]
            let $ptr_1: *mut u8 = alloc::alloc::__rust_alloc(_, _); // _3
            let $ptr_2: *mut $T = move $ptr_1 as *mut $T (PtrToPtr); // _2
            let $const_ptr_1: *const u8 = copy $ptr_2 as *const u8 (PtrToPtr); // _19
            let $addr_1: usize = copy $const_ptr_1 as usize (Transmute); // _20
            // switchInt(move $addr_1) {
            //     0_usize => {}
            //     _ => {}
            // }
            #[export(write)]
            (*$ptr_2) = _;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern {
        pattern,
        fn_pat,
        alloc,
        write,
        ty,
    }
}

#[rpl_macros::pattern_def]
fn alloc_cast_check_as_write(pcx: PatCtxt<'_>) -> Pattern<'_> {
    let alloc;
    let write;
    let ty;
    let pattern = rpl! {
        #[meta(#[export(ty)] $T:ty)]
        fn $pattern(..) -> _ = mir! {
            #[export(alloc)]
            let $ptr_1: *mut u8 = alloc::alloc::__rust_alloc(_, _); // _3
            let $ptr_2: *mut $T = move $ptr_1 as *mut $T (PtrToPtr); // _2
            let $addr_1: usize = copy $ptr_2 as usize (PointerExposeProvenance); // _6
            // switchInt(move $addr_1) {
            //     0_usize => {}
            //     _ => {}
            // }
            #[export(write)]
            (*$ptr_2) = _;
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();

    Pattern {
        pattern,
        fn_pat,
        alloc,
        write,
        ty,
    }
}
