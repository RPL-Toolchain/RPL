use rustc_hir as hir;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::ty::TyCtxt;
use rustc_span::{Span, Symbol};

use rpl_context::{PatCtxt, pat};
use rpl_mir::CheckMirCtxt;

use crate::lints::USE_AFTER_MOVE;

/// - `INLINE_MIR_THRESHOLD = 200`
/// - `INLINE_MIR_FORWARDER_THRESHOLD = 200`
/// - `INLINE_MIR_HINT_THRESHOLD = 200`
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
            let pattern = pattern(self.pcx);
            for matches in CheckMirCtxt::new(self.tcx, self.pcx, body, pattern.pattern, pattern.fn_pat).check() {
                let use_span = matches[pattern.ptr_use].span_no_inline(body);
                let move_span = matches[pattern.vec_move].span_no_inline(body);
                let ty = matches[pattern.ty.idx];
                // let global = self.tcx.type_of(global_did).instantiate_identity();
                self.tcx.emit_node_span_lint(
                    USE_AFTER_MOVE,
                    self.tcx.local_def_id_to_hir_id(def_id),
                    use_span,
                    crate::errors::UseAfterMove {
                        use_span,
                        move_span,
                        ty,
                    },
                );
            }
        }
        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}

struct Pattern<'pcx> {
    pattern: &'pcx pat::Pattern<'pcx>,
    fn_pat: &'pcx pat::Fn<'pcx>,
    vec_move: pat::Location,
    ptr_use: pat::Location,
    ty: pat::TyVar<'pcx>,
}

#[rpl_macros::pattern_def]
fn pattern(pcx: PatCtxt<'_>) -> Pattern<'_> {
    let vec_move;
    let ptr_use;
    let ty;
    let pattern = rpl! {
        #[meta($T:ty)]
        struct $Ptr {
            $ptr: core::ptr::NonNull<$T>,
        }

        struct $BitVec {
            $ptr: $Ptr,
        }

        #[meta($T:ty, #[export(ty)] $Vec:ty)]
        fn $pattern(..) -> _ = mir! {
            let $bit_vec_1: $BitVec = _; // _1
            let $bit_ptr: $Ptr = copy ($bit_vec_1.$ptr); // _2 bb0[0]
            let $bit_vec_2: $BitVec = move $bit_vec_1; // _6 bb0[1]
            let $bit_ptr_1: &$Ptr = &($bit_vec_2.$ptr); // _8 bb0[2]
            let $non_null_1: core::ptr::NonNull<u8> = copy ((*$bit_ptr_1).$ptr); // _19 bb0[3]
            let $ptr_1: *mut u8 = copy $non_null_1 as *mut u8 (Transmute); // _18 bb0[4]
            let $addr_1: usize = move $ptr_1 as usize (PointerExposeProvenance); // _17 bb0[5]
            let $addr_2: usize = BitAnd(move $addr_1, _); // _16 bb0[6]
            // let $pointer_1: Pointer<$T> = move $ptr_1 as usize (PointerExposeProvenance); // _14
            let $ptr_2: *mut $T = _; // _13 bb0[8]
            let $ptr_mut_slice: *mut [$T] = *mut [$T] from (copy $ptr_2, _); // _20 bb2[0]
            let $ref_mut_slice: &mut [$T] = &mut (*$ptr_mut_slice); // _7 bb2[1]
            let $ptr_slice: *mut [$T] = &raw mut (*$ref_mut_slice); // _21 bb2[2]
            let $ptr_2: *mut $T = move $ptr_slice as *mut $T (PtrToPtr); // _9 bb2[3]
            let $ptr_3: *const u8 = copy $ptr_2 as *const u8 (PtrToPtr); // _28 bb3[1]
            let $non_null: core::ptr::NonNull<u8> = core::ptr::NonNull<u8> { pointer: copy $ptr_3 }; // _27 bb3[2]
            // let $non_null: core::ptr::NonNull<u8> = _; // _65
            let $unique: core::ptr::Unique<u8> = core::ptr::Unique<u8> { pointer: move $non_null, _marker: const core::marker::PhantomData::<u8> }; // _26 bb3[3]
            let $raw_vec_inner: alloc::raw_vec::RawVecInner = alloc::raw_vec::RawVecInner { ptr: move $unique, cap: _, alloc: _ }; // _24 bb3[4]
            let $raw_vec: alloc::raw_vec::RawVec<$T> = alloc::raw_vec::RawVec<$T> { inner: move $raw_vec_inner, _marker: _ }; // _22 bb3[5]
            let $vec: $Vec /* alloc::vec::Vec<$T> */ = alloc::vec::Vec<$T> { buf: move $raw_vec, len: _ }; // _5 bb3[6]
            #[export(vec_move)]
            let $boxed_slice: alloc::boxed::Box<[$T]> = alloc::vec::Vec::into_boxed_slice(move $vec); // _3 bb3[7]
            #[export(ptr_use)]
            let $non_null_2: core::ptr::NonNull<u8> = copy ($bit_ptr.$ptr); // _37 bb1[0]
        }
    };
    let fn_pat = pattern.fns.get_fn_pat(Symbol::intern("pattern")).unwrap();
    // let ptr = pattern.get_adt(Symbol::intern("Ptr")).unwrap();

    Pattern {
        pattern,
        fn_pat,
        vec_move,
        ptr_use,
        ty,
    }
}
