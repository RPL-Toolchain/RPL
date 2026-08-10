use rustc_data_structures::fx::FxHashSet;
use rustc_middle::mir::{self};
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::sym;

use super::locals::BodyInfoCache;

#[instrument(level = "debug", skip(tcx, typing_env, body, _cache), ret)]
pub(crate) fn is_release_helper_body<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    _cache: &BodyInfoCache<'tcx>,
    _location: mir::Location,
    _local: mir::Local,
) -> bool {
    let _ = typing_env;
    let path = tcx.def_path_str(body.source.def_id());
    let Some(name) = path.rsplit("::").next() else {
        return false;
    };
    name != "drop"
        && ["drop", "dealloc", "release", "destroy", "free"]
            .into_iter()
            .any(|needle| name.contains(needle))
}

/// Check whether `ty` is a standard-library guard that borrows, rather than owns, its protected
/// value.
#[instrument(level = "debug", skip(tcx), ret)]
pub(crate) fn is_borrow_guard<'tcx>(tcx: TyCtxt<'tcx>, _typing_env: ty::TypingEnv<'tcx>, ty: Ty<'tcx>) -> bool {
    let ty::Adt(adt_def, _) = ty.kind() else {
        return false;
    };

    tcx.get_diagnostic_name(adt_def.did()).is_some_and(|name| {
        [
            sym::MutexGuard,
            sym::RwLockReadGuard,
            sym::RwLockWriteGuard,
            sym::RefCellRef,
            sym::RefCellRefMut,
        ]
        .contains(&name)
    })
}

/// Preserve a guard report only when its mutable dereference's pointee is explicitly dropped.
#[instrument(level = "debug", skip(tcx, body, cache), ret)]
pub(crate) fn is_borrow_guard_drop_after_pointee_drop_at_location<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    _location: mir::Location,
    local: mir::Local,
) -> bool {
    let _ = cache;
    if !is_borrow_guard(tcx, typing_env, body.local_decls[local].ty) {
        return false;
    }

    let mut guard_reborrows = FxHashSet::default();
    guard_reborrows.insert(local);
    let mut guarded_pointees = FxHashSet::default();
    for data in body.basic_blocks.iter() {
        for statement in &data.statements {
            let mir::StatementKind::Assign(box (left, mir::Rvalue::Ref(_, _, right))) = &statement.kind else {
                continue;
            };
            let Some(left) = left.as_local() else {
                continue;
            };
            if right.as_local() == Some(local) {
                guard_reborrows.insert(left);
            }
            if is_inlined_deref_mut(tcx, body, statement.source_info.scope) {
                guarded_pointees.insert(left);
            }
        }
    }

    for data in body.basic_blocks.iter() {
        let Some(terminator) = &data.terminator else {
            continue;
        };
        let mir::TerminatorKind::Call {
            func,
            args,
            destination,
            ..
        } = &terminator.kind
        else {
            continue;
        };
        let Some(receiver) = args.first().and_then(|arg| match arg.node {
            mir::Operand::Copy(place) | mir::Operand::Move(place) => place.as_local(),
            mir::Operand::Constant(_) => None,
        }) else {
            continue;
        };
        if guard_reborrows.contains(&receiver) && is_deref_mut_call(tcx, func) && destination.projection.is_empty() {
            guarded_pointees.insert(destination.local);
        }
    }

    body.basic_blocks.iter().any(|data| {
        matches!(
            data.terminator.as_ref().map(|terminator| &terminator.kind),
            Some(mir::TerminatorKind::Drop { place, .. })
                if matches!(place.projection.first(), Some(mir::ProjectionElem::Deref))
                    && guarded_pointees.contains(&place.local)
        )
    })
}

/// Ignore dead-pointer states on cleanup-only paths unless the location performs a release.
pub(crate) fn is_cleanup_release_location(body: &mir::Body<'_>, location: mir::Location) -> bool {
    let block = &body.basic_blocks[location.block];
    if location.statement_index != block.statements.len() {
        return false;
    }
    let Some(terminator) = &block.terminator else {
        return false;
    };
    matches!(
        &terminator.kind,
        mir::TerminatorKind::Drop { .. } | mir::TerminatorKind::Call { .. }
    )
}

/// End-of-scope drop flags can leave a MIR `Drop(local)` after the value has already been moved
/// into a temporary/call. Treating that drop as an unconditional second release produces wrapper
/// false positives such as `fn post<B>(body: B) { client.post(body) }`.
pub(crate) fn is_moved_local_drop<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    if !is_drop_of_local(body, location, local) {
        return false;
    }
    if is_small_insert_constructor(tcx, body) {
        return false;
    }
    if local_flows_to_direct_ptr_write(tcx, body, local) {
        return false;
    }
    if body_contains_manual_ownership_transfer(tcx, body) && !body_forgets_local(tcx, body, local) {
        return false;
    }
    body.basic_blocks.iter().any(|data| {
        data.statements.iter().any(|statement| {
            let mir::StatementKind::Assign(box (_, rvalue)) = &statement.kind else {
                return false;
            };
            rvalue_moves_local(rvalue, local)
        }) || data.terminator.as_ref().is_some_and(|terminator| {
            matches!(
                &terminator.kind,
                mir::TerminatorKind::Call { args, .. }
                    if args.iter().any(|arg| operand_moves_local(&arg.node, local))
            )
        })
    })
}

/// Exclude release helpers from function-exit dangling-pointer reports.
pub(crate) fn should_check_dangling_fn<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>) -> bool {
    let path = tcx.def_path_str(body.source.def_id());
    !["drop", "dealloc", "release", "destroy"]
        .into_iter()
        .any(|needle| path.contains(needle))
}

/// SafeDrop's return-value DP check is conservative around direct unsafe ownership exposure, but
/// it does not treat ordinary collection builder flows as escaping dead storage merely because a
/// local is moved into `_0` and then appears in drop elaboration.
pub(crate) fn should_check_return_dangling_fn<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>) -> bool {
    let path = tcx.def_path_str(body.source.def_id());
    let name = path.rsplit("::").next();
    let Ok(snippet) = tcx.sess.source_map().span_to_snippet(body.span) else {
        return true;
    };

    if return_body_has_direct_unsafe_escape(&snippet) {
        return true;
    }
    if matches!(name, Some("unit" | "pair")) {
        return true;
    }
    if snippet.contains("mem::forget") {
        return false;
    }
    if snippet.contains(".extend(") {
        return false;
    }
    if name == Some("from_iter") && snippet.contains(".insert(") {
        return false;
    }
    true
}

/// SafeDrop's pointer-argument DP reports are driven by explicit ownership-transfer idioms.  A
/// normal `&mut self` container mutator can temporarily route raw pointers through locals without
/// exposing a dangling argument at function exit.
pub(crate) fn should_check_reference_arg_dangling_fn<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>) -> bool {
    tcx.sess
        .source_map()
        .span_to_snippet(body.span)
        .is_ok_and(|snippet| snippet.contains("mem::forget") || snippet.contains("mem::replace"))
}

/// A by-value raw pointer argument only exposes a dangling pointer to the caller when the function
/// writes through a pointer-to-pointer style output or builds a returned owner from that raw input.
pub(crate) fn should_check_raw_arg_dangling_fn<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, ty: Ty<'tcx>) -> bool {
    let ty::RawPtr(pointee, _) = ty.kind() else {
        return true;
    };
    if matches!(pointee.kind(), ty::RawPtr(_, _) | ty::Ref(_, _, _)) {
        return true;
    }
    tcx.sess.source_map().span_to_snippet(body.span).is_ok_and(|snippet| {
        snippet.contains("from_raw_parts") || snippet.contains("from_raw(") || snippet.contains("from_raw::<")
    })
}

fn is_drop_of_local(body: &mir::Body<'_>, location: mir::Location, local: mir::Local) -> bool {
    let block = &body.basic_blocks[location.block];
    if location.statement_index != block.statements.len() {
        return false;
    }
    matches!(
        block.terminator.as_ref().map(|terminator| &terminator.kind),
        Some(mir::TerminatorKind::Drop { place, .. }) if place.as_local() == Some(local)
    )
}

fn rvalue_moves_local(rvalue: &mir::Rvalue<'_>, local: mir::Local) -> bool {
    match rvalue {
        mir::Rvalue::Use(operand) | mir::Rvalue::Cast(_, operand, _) | mir::Rvalue::ShallowInitBox(operand, _) => {
            operand_moves_local(operand, local)
        },
        mir::Rvalue::BinaryOp(_, box (left, right)) => {
            operand_moves_local(left, local) || operand_moves_local(right, local)
        },
        mir::Rvalue::Aggregate(_, operands) => operands.iter().any(|operand| operand_moves_local(operand, local)),
        _ => false,
    }
}

fn operand_moves_local(operand: &mir::Operand<'_>, local: mir::Local) -> bool {
    matches!(operand, mir::Operand::Move(place) if place.as_local() == Some(local))
}

fn rvalue_moves_any_local(rvalue: &mir::Rvalue<'_>, locals: &FxHashSet<mir::Local>) -> bool {
    match rvalue {
        mir::Rvalue::Use(operand) | mir::Rvalue::Cast(_, operand, _) | mir::Rvalue::ShallowInitBox(operand, _) => {
            operand_moves_any_local(operand, locals)
        },
        mir::Rvalue::BinaryOp(_, box (left, right)) => {
            operand_moves_any_local(left, locals) || operand_moves_any_local(right, locals)
        },
        mir::Rvalue::Aggregate(_, operands) => operands.iter().any(|operand| operand_moves_any_local(operand, locals)),
        _ => false,
    }
}

fn operand_moves_any_local(operand: &mir::Operand<'_>, locals: &FxHashSet<mir::Local>) -> bool {
    matches!(operand, mir::Operand::Move(place) if place.as_local().is_some_and(|local| locals.contains(&local)))
}

fn is_small_insert_constructor<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>) -> bool {
    if !matches!(
        tcx.def_path_str(body.source.def_id()).rsplit("::").next(),
        Some("clone" | "unit" | "pair")
    ) {
        return false;
    }
    tcx.sess.source_map().span_to_snippet(body.span).is_ok_and(|snippet| {
        (snippet.contains("Self::new()") && snippet.contains(".insert("))
            || (snippet.contains("MaybeUninit") && snippet.contains("ptr::write") && snippet.contains("map"))
    })
}

fn return_body_has_direct_unsafe_escape(snippet: &str) -> bool {
    [
        "from_raw_parts",
        "from_raw(",
        "from_raw::<",
        "Box::from_raw",
        "ptr::read",
        "ptr::write",
        "MaybeUninit",
    ]
    .into_iter()
    .any(|needle| snippet.contains(needle))
}

fn body_contains_manual_ownership_transfer<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>) -> bool {
    if tcx
        .sess
        .source_map()
        .span_to_snippet(body.span)
        .is_ok_and(|snippet| snippet.contains("mem::forget"))
    {
        return true;
    }
    body.basic_blocks.iter().any(|data| {
        data.statements
            .iter()
            .any(|statement| source_scope_has_manual_ownership_transfer(tcx, body, statement.source_info.scope))
            || data.terminator.as_ref().is_some_and(|terminator| {
                source_scope_has_manual_ownership_transfer(tcx, body, terminator.source_info.scope)
                    || matches!(
                        &terminator.kind,
                        mir::TerminatorKind::Call { func, .. }
                            if call_def_path(tcx, func).is_some_and(|path| is_manual_ownership_transfer_path(&path))
                    )
            })
    })
}

fn body_forgets_local<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, local: mir::Local) -> bool {
    let mut moved_locals = FxHashSet::default();
    moved_locals.insert(local);
    for data in body.basic_blocks.iter() {
        for statement in &data.statements {
            let mir::StatementKind::Assign(box (left, rvalue)) = &statement.kind else {
                continue;
            };
            if rvalue_moves_any_local(rvalue, &moved_locals)
                && let Some(left) = left.as_local()
            {
                moved_locals.insert(left);
            }
        }
        if data.terminator.as_ref().is_some_and(|terminator| {
            matches!(
                &terminator.kind,
                mir::TerminatorKind::Call { func, args, .. }
                    if call_def_path(tcx, func).is_some_and(|path| is_manual_ownership_transfer_path(&path))
                        && args.iter().any(|arg| operand_moves_any_local(&arg.node, &moved_locals))
            )
        }) {
            return true;
        }
    }
    false
}

fn local_flows_to_direct_ptr_write<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, local: mir::Local) -> bool {
    let moved_locals = move_reachable_locals(body, local);
    body.basic_blocks.iter().any(|data| {
        data.statements.iter().any(|statement| {
            source_scope_has_direct_ptr_write(tcx, body, statement.source_info.scope)
                && matches!(
                    &statement.kind,
                    mir::StatementKind::Assign(box (_, rvalue)) if rvalue_moves_any_local(rvalue, &moved_locals)
                )
        }) || data.terminator.as_ref().is_some_and(|terminator| {
            source_scope_has_direct_ptr_write(tcx, body, terminator.source_info.scope)
                && terminator_args_move_any_local(terminator, &moved_locals)
                || matches!(
                    &terminator.kind,
                    mir::TerminatorKind::Call { func, args, .. }
                        if call_def_path(tcx, func).is_some_and(|path| is_ptr_write_path(&path))
                            && args.iter().any(|arg| operand_moves_any_local(&arg.node, &moved_locals))
                )
        })
    })
}

fn move_reachable_locals(body: &mir::Body<'_>, local: mir::Local) -> FxHashSet<mir::Local> {
    let mut moved_locals = FxHashSet::default();
    moved_locals.insert(local);
    let mut changed = true;
    while changed {
        changed = false;
        for data in body.basic_blocks.iter() {
            for statement in &data.statements {
                let mir::StatementKind::Assign(box (left, rvalue)) = &statement.kind else {
                    continue;
                };
                if rvalue_moves_any_local(rvalue, &moved_locals)
                    && let Some(left) = left.as_local()
                {
                    changed |= moved_locals.insert(left);
                }
            }
            let Some(terminator) = &data.terminator else {
                continue;
            };
            let mir::TerminatorKind::Call { args, destination, .. } = &terminator.kind else {
                continue;
            };
            if args.iter().any(|arg| operand_moves_any_local(&arg.node, &moved_locals))
                && let Some(destination) = destination.as_local()
            {
                changed |= moved_locals.insert(destination);
            }
        }
    }
    moved_locals
}

fn terminator_args_move_any_local(terminator: &mir::Terminator<'_>, locals: &FxHashSet<mir::Local>) -> bool {
    matches!(
        &terminator.kind,
        mir::TerminatorKind::Call { args, .. }
            if args.iter().any(|arg| operand_moves_any_local(&arg.node, locals))
    )
}

fn source_scope_has_manual_ownership_transfer<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    mut scope: mir::SourceScope,
) -> bool {
    loop {
        let scope_data = &body.source_scopes[scope];
        if let Some((instance, _)) = scope_data.inlined
            && is_manual_ownership_transfer_path(&tcx.def_path_str(instance.def.def_id()))
        {
            return true;
        }
        let Some(parent_scope) = scope_data.inlined_parent_scope else {
            return false;
        };
        scope = parent_scope;
    }
}

fn source_scope_has_direct_ptr_write<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    mut scope: mir::SourceScope,
) -> bool {
    loop {
        let scope_data = &body.source_scopes[scope];
        if let Some((instance, _)) = scope_data.inlined
            && is_ptr_write_path(&tcx.def_path_str(instance.def.def_id()))
        {
            return scope_data
                .inlined_parent_scope
                .is_none_or(|parent| body.source_scopes[parent].inlined.is_none());
        }
        let Some(parent_scope) = scope_data.inlined_parent_scope else {
            return false;
        };
        scope = parent_scope;
    }
}

fn is_manual_ownership_transfer_path(path: &str) -> bool {
    path.ends_with("::mem::forget")
}

fn is_ptr_write_path(path: &str) -> bool {
    path.ends_with("::ptr::write")
        || path.ends_with("::ptr::write_unaligned")
        || path.ends_with("::ptr::write_volatile")
        || path.ends_with("::intrinsics::write_via_move")
}

fn call_def_path<'tcx>(tcx: TyCtxt<'tcx>, func: &mir::Operand<'tcx>) -> Option<String> {
    let mir::Operand::Constant(func) = func else {
        return None;
    };
    let mir::Const::Val(mir::ConstValue::ZeroSized, ty) = func.const_ else {
        return None;
    };
    let ty::FnDef(def_id, _) = ty.kind() else {
        return None;
    };
    Some(tcx.def_path_str(*def_id))
}

fn is_deref_mut_call<'tcx>(tcx: TyCtxt<'tcx>, func: &mir::Operand<'tcx>) -> bool {
    let mir::Operand::Constant(box mir::ConstOperand {
        const_: mir::Const::Val(mir::ConstValue::ZeroSized, ty),
        ..
    }) = func
    else {
        return false;
    };
    let ty::FnDef(def_id, _) = *ty.kind() else {
        return false;
    };
    tcx.trait_of_item(def_id) == tcx.lang_items().deref_mut_trait() || tcx.def_path_str(def_id).ends_with("::deref_mut")
}

fn is_inlined_deref_mut<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, mut scope: mir::SourceScope) -> bool {
    loop {
        let scope_data = &body.source_scopes[scope];
        if let Some((instance, _)) = scope_data.inlined
            && tcx.def_path_str(instance.def.def_id()).ends_with("::deref_mut")
        {
            return true;
        }
        let Some(parent_scope) = scope_data.inlined_parent_scope else {
            return false;
        };
        scope = parent_scope;
    }
}
