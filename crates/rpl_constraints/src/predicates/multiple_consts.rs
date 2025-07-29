use rustc_middle::ty::{self, TyCtxt};

use crate::Const;

// FIX: consider a more general way for error handling
pub type MultipleConstsPredsFnPtr = for<'tcx> fn(TyCtxt<'tcx>, ty::TypingEnv<'tcx>, Vec<Const<'tcx>>) -> bool;

/// Check if those constants are in a strictly increasing order
#[instrument(level = "debug", skip(tcx), ret)]
pub fn usize_lt<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, consts: Vec<Const<'tcx>>) -> bool {
    consts.windows(2).all(|w| {
        if let (Some(c1), Some(c2)) = (
            w[0].try_eval_scalar_int(tcx, typing_env),
            w[1].try_eval_scalar_int(tcx, typing_env),
        ) {
            let c1 = c1.to_target_usize(tcx);
            let c2 = c2.to_target_usize(tcx);
            c1 < c2
        } else {
            warn!("Encountered non-integer constants in usize_lt predicate");
            false // Non-integer constants or compile-time unknown values are not considered strictly increasing
        }
    })
}
