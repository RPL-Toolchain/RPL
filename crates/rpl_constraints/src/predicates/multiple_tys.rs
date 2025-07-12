use rustc_middle::ty::{self, Ty, TyCtxt};

pub type MultipleTysPredsFnPtr = for<'tcx> fn(TyCtxt<'tcx>, ty::TypingEnv<'tcx>, Vec<Ty<'tcx>>) -> bool;

/// Check if all tys' sizes are the same
pub fn same_size<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, tys: Vec<Ty<'tcx>>) -> bool {
    let mut layout_res = tys.iter().map(|ty| tcx.layout_of(typing_env.as_query_input(*ty)));
    if layout_res.any(|layout| layout.is_err()) {
        return false;
    }
    // if all layouts are ok, check if all sizes are the same
    let layouts = layout_res
        .map(|layout| layout.unwrap().layout.size())
        .collect::<Vec<_>>();
    layouts.windows(2).all(|w| w[0] == w[1])
}

/// Check if all tys' alignments are the same
pub fn same_abi_and_pref_align<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, tys: Vec<Ty<'tcx>>) -> bool {
    let mut layout_res = tys.iter().map(|ty| tcx.layout_of(typing_env.as_query_input(*ty)));
    if layout_res.any(|layout| layout.is_err()) {
        return false;
    }
    // if all layouts are ok, check if all alignments are the same
    let layouts = layout_res
        .map(|layout| layout.unwrap().layout.align())
        .collect::<Vec<_>>();
    layouts.windows(2).all(|w| w[0] == w[1])
}

/// Check if the first type's layout is compatible with the rest of the types' layouts.
#[instrument(level = "debug", skip(tcx, typing_env), ret)]
pub fn compatible_layout<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, tys: Vec<Ty<'tcx>>) -> bool {
    fn compatible_layout<'tcx>(
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        from: Ty<'tcx>,
        to: Ty<'tcx>,
    ) -> bool {
        if let Ok(from) = tcx.try_normalize_erasing_regions(typing_env, from)
            && let Ok(to) = tcx.try_normalize_erasing_regions(typing_env, to)
            && let Ok(from_layout) = tcx.layout_of(typing_env.as_query_input(from))
            && let Ok(to_layout) = tcx.layout_of(typing_env.as_query_input(to))
        {
            from_layout.size == to_layout.size && from_layout.align.abi == to_layout.align.abi
        } else {
            // no idea about layout, so don't lint
            true
        }
    }

    if let Some((first_ty, remained_tys)) = tys.split_first() {
        // Check if all types have the same layout as the first type
        return remained_tys
            .iter()
            .all(|ty| compatible_layout(tcx, typing_env, *first_ty, *ty));
    }
    true
}
