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
        Self::with_typing_env(tcx, pcx, pat, fn_pat, ty::TypingEnv::fully_monomorphized(), None)
    }

    pub fn with_typing_env(
        tcx: TyCtxt<'tcx>,
        pcx: PatCtxt<'pcx>,
        pat: &'pcx pat::RustItems<'pcx>,
        fn_pat: &'a pat::FnPattern<'pcx>,
        typing_env: ty::TypingEnv<'tcx>,
        self_ty: Option<ty::Ty<'tcx>>,
    ) -> Self {
        let ty = MatchTyCtxt::new(tcx, pcx, typing_env, self_ty, pat, &fn_pat.meta);
        Self { ty, fn_pat }
    }

    pub fn ty(&self) -> &MatchTyCtxt<'pcx, 'tcx> {
        &self.ty
    }

    #[instrument(level = "debug", skip_all, fields(fn_pat = %self.fn_pat, fn_did = ?fn_did.into()), ret)]
    pub fn match_fn(&self, fn_did: impl Into<DefId> + Copy) -> bool {
        let fn_did = fn_did.into();
        let poly_fn_sig = match self.ty.tcx.type_of(fn_did).instantiate_identity().kind() {
            ty::FnDef(..) => self.ty.tcx.fn_sig(fn_did).instantiate_identity(),
            ty::Closure(_, args) => args.as_closure().sig(),
            kind => {
                // The item index can also contain items whose type is not a function
                // signature (e.g. async blocks lower to coroutines). They cannot match a
                // `fn` slot; skip them instead of panicking with an ICE.
                debug!(
                    kind = ?kind,
                    def_id = ?fn_did,
                    path = %self.ty.tcx.def_path_str(fn_did),
                    "skipping non-function item in function slot"
                );
                return false;
            },
        };
        let fn_sig = self.ty.tcx.liberate_late_bound_regions(fn_did, poly_fn_sig);
        debug!(?fn_sig);
        let params_ok = (self.fn_pat.params.len() <= fn_sig.inputs().len() || self.fn_pat.params.non_exhaustive)
            && zip(self.fn_pat.params.iter(), fn_sig.inputs())
                .all(|(param_pat, &param_ty)| self.match_param(param_pat, param_ty));
        if !params_ok {
            return false;
        }
        // Omitted `ret` is unconstrained (`fn $f(..) {}` must not require `()`), so
        // signature-only patterns like generic `#[inline]` still match `-> T`.
        match self.fn_pat.ret {
            Some(ret_pat) => self.ty.match_ty(ret_pat, fn_sig.output()),
            None => true,
        }
    }

    fn match_param(&self, param_pat: &pat::Param<'pcx>, ty: ty::Ty<'tcx>) -> bool {
        self.ty.match_ty(param_pat.ty, ty)
    }
}
