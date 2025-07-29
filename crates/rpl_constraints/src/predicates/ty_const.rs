use rustc_middle::mir;
use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::Const;

pub type TyConstPredsFnPtr =
    for<'tcx> fn(TyCtxt<'tcx>, body: &mir::Body<'tcx>, ty::TypingEnv<'tcx>, Ty<'tcx>, Const<'tcx>) -> bool;

/// Check if `alignment` is enough for the given type `ty`.
#[instrument(level = "debug", skip(tcx), ret)]
pub fn maybe_misaligned<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    ty: Ty<'tcx>,
    alignment: Const<'tcx>,
) -> bool {
    let typing_env = ty::TypingEnv::post_analysis(tcx, body.source.def_id());
    match ty.kind() {
        // Param types can be anything, and we don't know the alignment.
        // Also, param types with unsafe traits have been filtered out in `is_all_safe_trait`.
        ty::TyKind::Param(_) => true,
        // foreign types are opaque to Rust
        ty::TyKind::Foreign(_) => true,
        _ => {
            let layout = tcx.layout_of(typing_env.as_query_input(ty)).unwrap();
            alignment
                .try_eval_target_usize(tcx, typing_env)
                .is_none_or(|alignment| alignment < layout.align.abi.bytes())
        },
    }
}
