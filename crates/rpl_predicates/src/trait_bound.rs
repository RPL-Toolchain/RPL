use rustc_hir as hir;
use rustc_middle::ty::{self, Ty, TyCtxt, TypingMode};
use rustc_span::{Symbol, sym};

/// Check if self_ty's trait bounds are all safe.
#[instrument(level = "debug", skip(tcx), ret)]
pub fn is_all_safe_trait<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, self_ty: Ty<'tcx>) -> bool {
    const EXCLUDED_DIAG_ITEMS: &[Symbol] = &[sym::Send, sym::Sync];
    typing_env
        .param_env
        .caller_bounds()
        .iter()
        .filter_map(|clause| clause.as_trait_clause())
        .filter(|clause| clause.self_ty().no_bound_vars().expect("Unhandled bound vars") == self_ty)
        .map(|clause| clause.def_id())
        .filter(|&def_id| {
            tcx.get_diagnostic_name(def_id)
                .is_none_or(|name| !EXCLUDED_DIAG_ITEMS.contains(&name))
        })
        .map(|def_id| tcx.trait_def(def_id))
        .inspect(|trait_def| debug!(?trait_def))
        .all(|trait_def| matches!(trait_def.safety, hir::Safety::Safe))
}

/// Check if ty is not unpin.
#[instrument(level = "debug", skip(tcx), ret)]
pub fn is_not_unpin<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    !ty.is_unpin(tcx, typing_env)
}

/// Check if ty is sync.
#[instrument(level = "debug", skip(tcx), ret)]
pub fn is_sync<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    use rustc_infer::infer::TyCtxtInferExt;
    let infcx = tcx.infer_ctxt().build(TypingMode::PostAnalysis);
    let trait_def_id = tcx.require_lang_item(hir::LangItem::Sync, None);
    rustc_trait_selection::traits::type_known_to_meet_bound_modulo_regions(
        &infcx,
        typing_env.param_env,
        ty,
        trait_def_id,
    )
}
