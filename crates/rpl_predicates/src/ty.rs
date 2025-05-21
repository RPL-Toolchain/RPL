use rustc_middle::ty::{self, Ty, TyCtxt};

#[instrument(level = "debug", skip(tcx), ret)]
#[allow(unused_variables)]
pub fn is_integral<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    ty.is_integral()
}

#[instrument(level = "debug", skip(tcx), ret)]
#[allow(unused_variables)]
pub fn is_ptr<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    ty.is_any_ptr()
}

#[allow(unused_variables)]
pub fn is_primitive<'tcx>(_tcx: TyCtxt<'tcx>, _typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    ty.is_primitive()
}
