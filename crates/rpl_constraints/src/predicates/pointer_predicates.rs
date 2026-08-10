use rustc_middle::mir::{self};
use rustc_middle::ty::{self, TyCtxt};

use super::locals::BodyInfoCache;

/// Check if a local has a dead alias according to the intra-procedural pointer-state analysis
/// before the matched location.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn is_freed_at_location<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    if body.basic_blocks[location.block].is_cleanup {
        if !super::is_cleanup_release_location(body, location)
            || has_dead_normal_drop_at_same_source(tcx, typing_env, body, cache, location)
        {
            return false;
        }
    }
    if super::is_moved_local_drop(tcx, body, location, local) {
        return false;
    }
    cache.is_dead_before_local(tcx, typing_env, body, location, local, false)
}

fn has_dead_normal_drop_at_same_source<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    cleanup_location: mir::Location,
) -> bool {
    let Some(cleanup_terminator) = &body.basic_blocks[cleanup_location.block].terminator else {
        return false;
    };
    if !matches!(cleanup_terminator.kind, mir::TerminatorKind::Drop { .. }) {
        return false;
    }

    body.basic_blocks.iter_enumerated().any(|(block, data)| {
        if data.is_cleanup {
            return false;
        }
        let Some(terminator) = &data.terminator else {
            return false;
        };
        let mir::TerminatorKind::Drop { place, .. } = terminator.kind else {
            return false;
        };
        terminator.source_info.span == cleanup_terminator.source_info.span
            && cache.is_dead_before_local(
                tcx,
                typing_env,
                body,
                mir::Location {
                    block,
                    statement_index: data.statements.len(),
                },
                place.local,
                false,
            )
    })
}

/// Check if a local can reach a dead raw pointer according to the intra-procedural pointer-state
/// analysis before the matched location. This matches SafeDrop's dangling-pointer check for
/// pointer arguments.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn is_dangling_ptr_at_location<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    cache.is_dead_before_local(tcx, typing_env, body, location, local, true)
}

/// Check a dead argument only when the local callee actually uses the corresponding parameter.
/// Known unsafe APIs remain expressed by their dedicated RPL patterns.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn is_uaf_call_arg_at_location<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    cache.is_call_argument_used(tcx, typing_env, body, location, local)
        && cache.is_dead_before_local(tcx, typing_env, body, location, local, false)
}

fn is_dead_at_exit<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    local: mir::Local,
    pointer_only: bool,
    cleanup: bool,
) -> bool {
    body.basic_blocks.iter_enumerated().any(|(block, data)| {
        data.is_cleanup == cleanup
            && data
                .terminator
                .as_ref()
                .is_some_and(|terminator| cleanup || matches!(terminator.kind, mir::TerminatorKind::Return))
            && cache.is_dead_before_local(
                tcx,
                typing_env,
                body,
                mir::Location {
                    block,
                    statement_index: data.statements.len(),
                },
                local,
                pointer_only,
            )
    })
}

/// Check whether a local can expose a dangling pointer at a function exit. This mirrors SafeDrop's
/// `dp_check`: `_0` is checked at normal returns without raw-pointer filtering, while pointer
/// arguments are checked at normal and cleanup exits with raw-pointer filtering.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn has_dangling_ptr_at_exit<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    local: mir::Local,
) -> bool {
    if !super::should_check_dangling_fn(tcx, body) {
        return false;
    }
    if local == mir::RETURN_PLACE && !super::should_check_return_dangling_fn(tcx, body) {
        return false;
    }
    if local == mir::RETURN_PLACE {
        is_dead_at_exit(tcx, typing_env, body, cache, local, false, false)
    } else if !is_arg_local(local, body) {
        false
    } else if matches!(body.local_decls[local].ty.kind(), ty::Ref(_, _, _))
        && !super::should_check_reference_arg_dangling_fn(tcx, body)
    {
        false
    } else if matches!(body.local_decls[local].ty.kind(), ty::RawPtr(_, _))
        && !super::should_check_raw_arg_dangling_fn(tcx, body, body.local_decls[local].ty)
    {
        false
    } else {
        is_dead_at_exit(tcx, typing_env, body, cache, local, true, false)
            || is_dead_at_exit(tcx, typing_env, body, cache, local, true, true)
    }
}

fn is_arg_local(local: mir::Local, body: &mir::Body<'_>) -> bool {
    local.as_usize() > 0 && local.as_usize() <= body.arg_count
}
