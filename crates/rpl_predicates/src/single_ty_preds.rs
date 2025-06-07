use rustc_hir as hir;
use rustc_middle::ty::{self, Ty, TyCtxt, TypingMode};
use rustc_span::{Symbol, sym};

pub type SingleTyPredsFnPtr = for<'tcx> fn(TyCtxt<'tcx>, ty::TypingEnv<'tcx>, Ty<'tcx>) -> bool;

/// Check if self_ty's trait bounds are all safe.
#[instrument(level = "debug", skip(tcx), ret)]
pub fn is_all_safe_trait<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    const EXCLUDED_DIAG_ITEMS: &[Symbol] = &[sym::Send, sym::Sync];
    typing_env
        .param_env
        .caller_bounds()
        .iter()
        .filter_map(|clause| clause.as_trait_clause())
        .filter(|clause| clause.self_ty().no_bound_vars().expect("Unhandled bound vars") == ty)
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
#[allow(unused_variables)]
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

/// Check if ty is integral.
#[instrument(level = "debug", skip(tcx), ret)]
#[allow(unused_variables)]
pub fn is_integral<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    ty.is_integral()
}

/// Check if ty is a pointer.
#[instrument(level = "debug", skip(tcx), ret)]
#[allow(unused_variables)]
pub fn is_ptr<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    ty.is_any_ptr()
}

/// Check if ty needs to be dropped.
#[instrument(level = "debug", skip(tcx), ret)]
pub fn needs_drop<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    ty.needs_drop(tcx, typing_env)
}

/// Check if ty is a primitive type.
#[allow(unused_variables)]
pub fn is_primitive<'tcx>(_tcx: TyCtxt<'tcx>, _typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    ty.is_primitive()
}

/// Check if ty is a ZST.
pub fn is_zst<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    if let Ok(layout) = tcx.layout_of(typing_env.as_query_input(ty)) {
        layout.layout.is_zst()
    } else {
        false
    }
}
