use derive_more::{Debug, Display};
use rustc_const_eval::interpret::Scalar;
use rustc_middle::mir;
use rustc_middle::ty::{self, ScalarInt, TyCtxt, TypingEnv};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Const<'tcx> {
    #[debug("{_0:?}")]
    #[display("{_0}")]
    MIR(mir::Const<'tcx>),
    #[debug("{_0:?}")]
    #[display("{_0}")]
    Param(ty::ParamConst),
}

impl<'tcx> Const<'tcx> {
    pub fn try_eval_target_usize(self, tcx: TyCtxt<'tcx>, typing_env: TypingEnv<'tcx>) -> Option<u64> {
        match self {
            Self::MIR(konst) => Some(konst.eval_target_usize(tcx, typing_env)),
            Self::Param(_) => None,
        }
    }
    pub fn try_eval_scalar(self, tcx: TyCtxt<'tcx>, typing_env: TypingEnv<'tcx>) -> Option<Scalar> {
        match self {
            Self::MIR(konst) => konst.try_eval_scalar(tcx, typing_env),
            Self::Param(_) => None,
        }
    }
    pub fn try_eval_scalar_int(self, tcx: TyCtxt<'tcx>, typing_env: TypingEnv<'tcx>) -> Option<ScalarInt> {
        match self {
            Self::MIR(konst) => konst.try_eval_scalar_int(tcx, typing_env),
            Self::Param(_) => None,
        }
    }
}
