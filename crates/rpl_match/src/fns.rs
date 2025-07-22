use std::iter::zip;

use rpl_context::{PatCtxt, pat};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{self, TyCtxt};

use crate::MatchTyCtxt;
use crate::ty::MatchTy as _;

pub struct MatchFnCtxt<'a, 'pcx, 'tcx> {
    ty: MatchTyCtxt<'pcx, 'tcx>,
    fn_pat: &'a pat::FnPattern<'pcx>,
}

impl<'a, 'pcx, 'tcx> MatchFnCtxt<'a, 'pcx, 'tcx> {
    pub fn new(
        tcx: TyCtxt<'tcx>,
        pcx: PatCtxt<'pcx>,
        pat: &'pcx pat::RustItems<'pcx>,
        fn_pat: &'a pat::FnPattern<'pcx>,
    ) -> Self {
        // FIXME: `self_ty` should be passed from the caller.
        let ty = MatchTyCtxt::new(tcx, pcx, ty::TypingEnv::fully_monomorphized(), None, pat, &fn_pat.meta); // FIXME
        Self { ty, fn_pat }
    }

    #[instrument(level = "debug", skip_all, fields(fn_pat = %self.fn_pat, fn_did = ?fn_did.into()), ret)]
    pub fn match_fn(&self, fn_did: impl Into<DefId> + Copy) -> bool {
        let fn_did = fn_did.into();
        let poly_fn_sig = match self.ty.tcx.type_of(fn_did).instantiate_identity().kind() {
            ty::FnDef(..) => self.ty.tcx.fn_sig(fn_did).instantiate_identity(),
            ty::Closure(_, args) => args.as_closure().sig(),
            _ => unimplemented!(),
        };
        let fn_sig = self.ty.tcx.liberate_late_bound_regions(fn_did, poly_fn_sig);
        debug!(?fn_sig);
        (self.fn_pat.params.len() <= fn_sig.inputs().len() || self.fn_pat.params.non_exhaustive)
            && zip(self.fn_pat.params.iter(), fn_sig.inputs())
                .all(|(param_pat, &param_ty)| self.match_param(param_pat, param_ty))
            && self
                .ty
                .match_ty(self.fn_pat.ret.unwrap_or(self.ty.pcx.mk_unit_ty()), fn_sig.output())
    }

    fn match_param(&self, param_pat: &pat::Param<'pcx>, ty: ty::Ty<'tcx>) -> bool {
        self.ty.match_ty(param_pat.ty, ty)
    }
}
