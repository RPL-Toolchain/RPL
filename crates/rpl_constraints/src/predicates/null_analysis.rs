use std::time::Duration;

use mirsa::analysis::combined::{AnalysisOptions, CombinedState, analyze_combined_with_config, state_before_location};
use mirsa::core::cfg::build_cfg;
use mirsa::core::mir::{collect_body_places, collect_interval_places, collect_ptr_places};
use mirsa::domains::nullptr::NullPtr;
use mirsa::framework::forward::{PathForwardAnalysisConfig, PathForwardAnalysisResult};
use rustc_middle::mir;
use rustc_middle::ty::TyCtxt;

#[instrument(level = "debug", skip(tcx, body), fields(n = body.local_decls.len()), ret)]
pub(crate) fn analyze_null<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
) -> PathForwardAnalysisResult<CombinedState<'tcx>> {
    let cfg = build_cfg(body);
    let all_places = collect_body_places(tcx, body);
    let places = collect_interval_places(tcx, body);
    let ptr_places = collect_ptr_places(tcx, body);
    analyze_combined_with_config(
        tcx,
        body,
        &cfg,
        &places,
        &all_places,
        &ptr_places,
        PathForwardAnalysisConfig {
            max_paths: 8,
            widen_after_iterations: Some(8),
            max_duration: Some(Duration::from_secs(5)),
        },
        AnalysisOptions::default(),
    )
}

pub(crate) fn is_null_at<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let place = mir::Place::from(local);
    let value = state
        .nullptr
        .access_path_for_place_resolved(&state.symbolic, place)
        .map(|path| state.nullptr.value_or_maybe(&path))
        .unwrap_or(NullPtr::Bot);
    matches!(value, NullPtr::Null)
}
