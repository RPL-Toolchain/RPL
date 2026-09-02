// Predicate helpers mirror the evaluator's fixed argument list.
#![allow(clippy::too_many_arguments)]

use std::time::Duration;

use mirsa::analysis::combined::{AnalysisOptions, CombinedState, analyze_combined_with_config, state_before_location};
use mirsa::core::cfg::build_cfg;
use mirsa::core::mir::{collect_body_places, collect_interval_places, collect_ptr_places};
use mirsa::domains::interval::Interval;
use mirsa::framework::forward::{PathForwardAnalysisConfig, PathForwardAnalysisResult};
use rustc_middle::mir;
use rustc_middle::ty::{TyCtxt, TyKind, TypingEnv};
use rustc_span::Symbol;

use crate::Const;

#[instrument(level = "debug", skip(tcx, body), fields(n = body.local_decls.len()), ret)]
pub(crate) fn analyze_interval<'tcx>(
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

fn unsigned_bits_to_i128(bits: u128, bit_width: u64) -> i128 {
    if bit_width == 128 {
        if bits <= i128::MAX as u128 {
            bits as i128
        } else {
            i128::MAX
        }
    } else {
        let mask = (1u128 << bit_width) - 1;
        (bits & mask) as i128
    }
}

fn signed_bits_to_i128(bits: u128, bit_width: u64) -> i128 {
    if bit_width == 128 {
        bits as i128
    } else {
        let sign_bit = 1u128 << (bit_width - 1);
        let mask = (1u128 << bit_width) - 1;
        let x = bits & mask;
        if (x & sign_bit) != 0 {
            (x as i128) - ((1u128 << bit_width) as i128)
        } else {
            x as i128
        }
    }
}

fn const_to_i128<'tcx>(tcx: TyCtxt<'tcx>, typing_env: TypingEnv<'tcx>, konst: Const<'tcx>) -> Option<i128> {
    let scalar = konst.try_eval_scalar_int(tcx, typing_env)?;
    let ty = match konst {
        Const::MIR(konst) => konst.ty(),
        Const::Param(_) => return None,
    };
    let (bit_width, signed) = match ty.kind() {
        TyKind::Int(_) => (scalar.size().bits(), true),
        TyKind::Uint(_) => (scalar.size().bits(), false),
        TyKind::Bool => (1, false),
        TyKind::Char => (32, false),
        _ => return None,
    };
    let bits = scalar.to_bits_unchecked();
    Some(if signed {
        signed_bits_to_i128(bits, bit_width)
    } else {
        unsigned_bits_to_i128(bits, bit_width)
    })
}

fn is_interval_scalar_ty(ty: rustc_middle::ty::Ty<'_>) -> bool {
    matches!(
        ty.kind(),
        TyKind::Int(_) | TyKind::Uint(_) | TyKind::Bool | TyKind::Char
    )
}

fn is_core_num_nonzero<'tcx>(tcx: TyCtxt<'tcx>, ty: rustc_middle::ty::Ty<'tcx>) -> bool {
    let TyKind::Adt(adt_def, _) = ty.kind() else {
        return false;
    };
    matches!(
        tcx.def_path_str(adt_def.did()).as_str(),
        "core::num::nonzero::NonZero" | "std::num::NonZero"
    )
}

fn single_scalar_field_place<'tcx>(
    tcx: TyCtxt<'tcx>,
    place: mir::Place<'tcx>,
    ty: rustc_middle::ty::Ty<'tcx>,
) -> Option<mir::Place<'tcx>> {
    if is_interval_scalar_ty(ty) {
        return Some(place);
    }

    let TyKind::Adt(adt_def, args) = ty.kind() else {
        return None;
    };
    if adt_def.variants().len() != 1 {
        return None;
    }
    let variant = adt_def.variants().iter().next()?;
    if variant.fields.len() != 1 {
        return None;
    }

    let (idx, field) = variant.fields.iter().enumerate().next()?;
    let field_ty = field.ty(tcx, args);
    let field_place = place.project_deeper(&[mir::ProjectionElem::Field(idx.into(), field_ty)], tcx);
    single_scalar_field_place(tcx, field_place, field_ty)
}

fn scalar_field_place_by_name<'tcx>(
    tcx: TyCtxt<'tcx>,
    place: mir::Place<'tcx>,
    ty: rustc_middle::ty::Ty<'tcx>,
    field_name: &str,
) -> Option<mir::Place<'tcx>> {
    let TyKind::Adt(adt_def, args) = ty.kind() else {
        return None;
    };
    if adt_def.variants().len() != 1 {
        return None;
    }
    let variant = adt_def.variants().iter().next()?;
    let field_name = Symbol::intern(field_name);
    let (idx, field) = variant
        .fields
        .iter()
        .enumerate()
        .find(|(_, field)| field.name == field_name)?;
    let field_ty = field.ty(tcx, args);
    let field_place = place.project_deeper(&[mir::ProjectionElem::Field(idx.into(), field_ty)], tcx);
    single_scalar_field_place(tcx, field_place, field_ty)
}

fn nonzero_scalar_place<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    place: mir::Place<'tcx>,
) -> Option<mir::Place<'tcx>> {
    let ty = place.ty(&body.local_decls, tcx).ty;
    is_core_num_nonzero(tcx, ty).then(|| single_scalar_field_place(tcx, place, ty))?
}

fn tracked_interval<'tcx>(state: &CombinedState<'tcx>, place: mir::Place<'tcx>) -> Option<(i128, i128)> {
    let interval = state.interval.tracked_interval_resolved(&state.symbolic, &place)?;
    (!interval.is_empty()).then_some((interval.low, interval.high))
}

fn tracked_len<'tcx>(state: &CombinedState<'tcx>, place: mir::Place<'tcx>) -> Option<Interval> {
    state.interval.get_len(&place).or_else(|| {
        state.interval.all_fact_places().into_iter().find_map(|candidate| {
            state
                .symbolic
                .equiv_places_readonly(place, candidate)
                .then(|| state.interval.get_len(&candidate))
                .flatten()
        })
    })
}

fn place_known_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    place: mir::Place<'tcx>,
) -> Option<Interval> {
    let ty = place.ty(&body.local_decls, tcx).ty;
    match ty.kind() {
        TyKind::Array(_, len) => len
            .try_to_target_usize(tcx)
            .map(|len| Interval::new(len as i128, len as i128)),
        TyKind::Slice(_) | TyKind::Str => tracked_len(state, place).or_else(|| {
            matches!(place.as_ref().projection.first(), Some(mir::ProjectionElem::Deref))
                .then(|| place_known_len(tcx, body, state, mir::Place::from(place.local)))
                .flatten()
        }),
        TyKind::Ref(_, inner, _) => match inner.kind() {
            TyKind::Array(_, len) => len
                .try_to_target_usize(tcx)
                .map(|len| Interval::new(len as i128, len as i128)),
            TyKind::Slice(_) | TyKind::Str => tracked_len(state, place),
            _ => None,
        },
        TyKind::RawPtr(inner, _) => match inner.kind() {
            TyKind::Array(_, len) => len
                .try_to_target_usize(tcx)
                .map(|len| Interval::new(len as i128, len as i128)),
            TyKind::Slice(_) | TyKind::Str => tracked_len(state, place),
            _ => None,
        },
        _ => None,
    }
}

fn raw_slice_ptr_len_from_rvalue<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    rvalue: &mir::Rvalue<'tcx>,
    depth: usize,
) -> Option<Interval> {
    match rvalue {
        mir::Rvalue::Use(mir::Operand::Copy(place) | mir::Operand::Move(place))
        | mir::Rvalue::Cast(_, mir::Operand::Copy(place) | mir::Operand::Move(place), _) => {
            place_known_len(tcx, body, state, *place)
                .or_else(|| raw_slice_ptr_len_from_defs(tcx, body, state, location, *place, depth - 1))
        },
        mir::Rvalue::RawPtr(_, place) => place_known_len(tcx, body, state, *place),
        mir::Rvalue::Aggregate(kind, operands) if matches!(kind.as_ref(), mir::AggregateKind::RawPtr(_, _)) => operands
            .iter()
            .nth(1)
            .and_then(|operand| operand_interval(tcx, body, state, operand))
            .map(|(low, high)| Interval::new(low, high)),
        _ => None,
    }
}

fn raw_slice_ptr_len_from_defs<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    target: mir::Place<'tcx>,
    depth: usize,
) -> Option<Interval> {
    if depth == 0 {
        return None;
    }
    let mut exact_rhs = None;
    let mut equiv_rhs = None;
    for (block, data) in body.basic_blocks.iter_enumerated() {
        for (statement_index, statement) in data.statements.iter().enumerate() {
            let statement_location = mir::Location { block, statement_index };
            if !location_precedes(statement_location, location) {
                continue;
            }
            let mir::StatementKind::Assign(box (place, rvalue)) = &statement.kind else {
                continue;
            };
            if *place == target {
                exact_rhs = Some(rvalue);
            } else if state.symbolic.equiv_places_readonly(*place, target) {
                equiv_rhs = Some(rvalue);
            }
        }
    }

    let Some(rhs) = exact_rhs else {
        return raw_slice_ptr_len_from_calls(tcx, body, state, location, target).or_else(|| {
            let rhs = equiv_rhs?;
            raw_slice_ptr_len_from_rvalue(tcx, body, state, location, rhs, depth)
        });
    };
    raw_slice_ptr_len_from_rvalue(tcx, body, state, location, rhs, depth)
}

fn call_func_path<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, func: &mir::Operand<'tcx>) -> Option<String> {
    let TyKind::FnDef(def_id, _) = func.ty(&body.local_decls, tcx).kind() else {
        return None;
    };
    Some(tcx.def_path_str(*def_id))
}

fn raw_slice_ptr_len_from_calls<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    target: mir::Place<'tcx>,
) -> Option<Interval> {
    let mut exact_len = None;
    let mut equiv_len = None;
    for (block, data) in body.basic_blocks.iter_enumerated() {
        let call_location = mir::Location {
            block,
            statement_index: data.statements.len(),
        };
        if !location_precedes(call_location, location) {
            continue;
        }
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
        let Some(path) = call_func_path(tcx, body, func) else {
            continue;
        };
        if !path.ends_with("::slice_from_raw_parts") && !path.ends_with("::slice_from_raw_parts_mut") {
            continue;
        }
        let Some(len) = args
            .get(1)
            .and_then(|arg| operand_interval(tcx, body, state, &arg.node))
            .map(|(low, high)| Interval::new(low, high))
        else {
            continue;
        };
        if *destination == target {
            exact_len = Some(len);
        } else if state.symbolic.equiv_places_readonly(*destination, target) {
            equiv_len = Some(len);
        }
    }
    exact_len.or(equiv_len)
}

fn const_operand_interval(konst: &mir::ConstOperand<'_>) -> Option<(i128, i128)> {
    let scalar = konst.const_.try_to_scalar_int()?;
    let (bit_width, signed) = match konst.ty().kind() {
        TyKind::Int(_) => (scalar.size().bits(), true),
        TyKind::Uint(_) => (scalar.size().bits(), false),
        TyKind::Bool => (1, false),
        TyKind::Char => (32, false),
        _ => return None,
    };
    let bits = scalar.to_bits_unchecked();
    let value = if signed {
        signed_bits_to_i128(bits, bit_width)
    } else {
        unsigned_bits_to_i128(bits, bit_width)
    };
    Some((value, value))
}

fn wrapped_scalar_field_interval<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    place: mir::Place<'tcx>,
    field_name: &str,
    depth: usize,
) -> Option<(i128, i128)> {
    let ty = place.ty(&body.local_decls, tcx).ty;
    scalar_field_place_by_name(tcx, place, ty, field_name)
        .and_then(|field_place| tracked_interval(state, field_place))
        .or_else(|| wrapped_scalar_field_interval_from_defs(tcx, body, state, location, place, field_name, depth))
}

fn wrapped_scalar_field_interval_from_rvalue<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    target: mir::Place<'tcx>,
    field_name: &str,
    rvalue: &mir::Rvalue<'tcx>,
    depth: usize,
) -> Option<(i128, i128)> {
    match rvalue {
        mir::Rvalue::Use(mir::Operand::Copy(place) | mir::Operand::Move(place))
        | mir::Rvalue::Cast(_, mir::Operand::Copy(place) | mir::Operand::Move(place), _) => {
            wrapped_scalar_field_interval(tcx, body, state, location, *place, field_name, depth - 1)
        },
        mir::Rvalue::Aggregate(kind, operands) if matches!(kind.as_ref(), mir::AggregateKind::Adt(..)) => {
            let ty = target.ty(&body.local_decls, tcx).ty;
            let TyKind::Adt(adt_def, args) = ty.kind() else {
                return None;
            };
            if adt_def.variants().len() != 1 {
                return None;
            }
            let variant = adt_def.variants().iter().next()?;
            let field_name = Symbol::intern(field_name);
            let (idx, field) = variant
                .fields
                .iter()
                .enumerate()
                .find(|(_, field)| field.name == field_name)?;
            let field_ty = field.ty(tcx, args);
            if !is_interval_scalar_ty(field_ty) {
                return None;
            }
            operands
                .get(idx.into())
                .and_then(|operand| operand_interval(tcx, body, state, operand))
        },
        _ => None,
    }
}

fn wrapped_scalar_field_interval_from_defs<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    target: mir::Place<'tcx>,
    field_name: &str,
    depth: usize,
) -> Option<(i128, i128)> {
    if depth == 0 {
        return None;
    }

    let mut exact_rhs = None;
    let mut equiv_rhs = None;
    for (block, data) in body.basic_blocks.iter_enumerated() {
        for (statement_index, statement) in data.statements.iter().enumerate() {
            let statement_location = mir::Location { block, statement_index };
            if !location_precedes(statement_location, location) {
                continue;
            }
            let mir::StatementKind::Assign(box (place, rvalue)) = &statement.kind else {
                continue;
            };
            if *place == target {
                exact_rhs = Some(rvalue);
            } else if state.symbolic.equiv_places_readonly(*place, target) {
                equiv_rhs = Some(rvalue);
            }
        }
    }

    if let Some(rhs) = exact_rhs.or(equiv_rhs) {
        let directly_reuses_target = matches!(
            rhs,
            mir::Rvalue::Use(mir::Operand::Copy(place) | mir::Operand::Move(place))
                | mir::Rvalue::Cast(_, mir::Operand::Copy(place) | mir::Operand::Move(place), _)
                if *place == target
        );
        if !directly_reuses_target
            && let Some(interval) =
                wrapped_scalar_field_interval_from_rvalue(tcx, body, state, location, target, field_name, rhs, depth)
        {
            return Some(interval);
        }
    }

    range_inclusive_constructor_field_interval(tcx, body, state, location, target, field_name)
}

fn range_inclusive_constructor_field_interval<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    target: mir::Place<'tcx>,
    field_name: &str,
) -> Option<(i128, i128)> {
    let TyKind::Adt(adt_def, _) = target.ty(&body.local_decls, tcx).ty.kind() else {
        return None;
    };
    if tcx.item_name(adt_def.did()).as_str() != "RangeInclusive" {
        return None;
    }
    let arg_index = match field_name {
        "start" => 0,
        "end" => 1,
        _ => return None,
    };

    let mut exact = None;
    let mut equivalent = None;
    for (block, data) in body.basic_blocks.iter_enumerated() {
        let call_location = mir::Location {
            block,
            statement_index: data.statements.len(),
        };
        if !location_precedes(call_location, location) {
            continue;
        }
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
        let TyKind::FnDef(def_id, _) = func.ty(&body.local_decls, tcx).kind() else {
            continue;
        };
        if tcx.item_name(*def_id).as_str() != "new" {
            continue;
        }
        let interval = args
            .get(arg_index)
            .and_then(|arg| operand_interval(tcx, body, state, &arg.node));
        if *destination == target {
            exact = interval;
        } else if state.symbolic.equiv_places_readonly(*destination, target) {
            equivalent = interval;
        }
    }
    exact.or(equivalent)
}

fn is_layout_ty<'tcx>(tcx: TyCtxt<'tcx>, ty: rustc_middle::ty::Ty<'tcx>) -> bool {
    let TyKind::Adt(adt_def, _) = ty.kind() else {
        return false;
    };
    matches!(
        tcx.def_path_str(adt_def.did()).as_str(),
        "core::alloc::layout::Layout" | "std::alloc::Layout"
    )
}

fn type_size<'tcx>(tcx: TyCtxt<'tcx>, ty: rustc_middle::ty::Ty<'tcx>) -> Option<i128> {
    tcx.layout_of(TypingEnv::fully_monomorphized().as_query_input(ty))
        .ok()
        .map(|layout| i128::from(layout.size.bytes()))
}

fn layout_constructor_size_interval<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    target: mir::Place<'tcx>,
    depth: usize,
) -> Option<(i128, i128)> {
    if depth == 0 {
        return None;
    }

    let mut exact = None;
    let mut equivalent = None;
    for (block, data) in body.basic_blocks.iter_enumerated() {
        let call_location = mir::Location {
            block,
            statement_index: data.statements.len(),
        };
        if !location_precedes(call_location, location) {
            continue;
        }
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
        let is_exact = *destination == target;
        let is_equivalent = state.symbolic.equiv_places_readonly(*destination, target);
        if !is_exact && !is_equivalent {
            continue;
        }
        let Some(path) = call_func_path(tcx, body, func) else {
            continue;
        };
        let interval = if path.ends_with("Layout::from_size_align_unchecked") {
            args.first()
                .and_then(|arg| operand_interval(tcx, body, state, &arg.node))
        } else if path.ends_with("Layout::new") {
            let TyKind::FnDef(_, generic_args) = func.ty(&body.local_decls, tcx).kind() else {
                continue;
            };
            generic_args.types().next().and_then(|ty| {
                let size = type_size(tcx, ty)?;
                Some((size, size))
            })
        } else if path.ends_with("::unwrap") {
            args.first().and_then(|arg| match arg.node {
                mir::Operand::Copy(source) | mir::Operand::Move(source) => {
                    layout_constructor_size_interval(tcx, body, state, location, source, depth - 1)
                },
                mir::Operand::Constant(_) => None,
            })
        } else if path.ends_with("Layout::from_size_align") {
            args.first()
                .and_then(|arg| operand_interval(tcx, body, state, &arg.node))
        } else if path.ends_with("Layout::array") {
            let count = args
                .first()
                .and_then(|arg| operand_interval(tcx, body, state, &arg.node));
            let TyKind::FnDef(_, generic_args) = func.ty(&body.local_decls, tcx).kind() else {
                continue;
            };
            let element_size = generic_args.types().next().and_then(|ty| type_size(tcx, ty));
            match (count, element_size) {
                (Some(count), Some(element_size)) => mul_interval(count, (element_size, element_size)),
                _ => None,
            }
        } else {
            continue;
        };
        if is_exact {
            exact = interval;
        } else {
            equivalent = interval;
        }
    }
    exact.or(equivalent)
}

fn layout_size_interval<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    layout: mir::Local,
) -> Option<(i128, i128)> {
    let place = mir::Place::from(layout);
    if !is_layout_ty(tcx, place.ty(&body.local_decls, tcx).ty) {
        return None;
    }
    wrapped_scalar_field_interval(tcx, body, state, location, place, "size", 8)
        .or_else(|| layout_constructor_size_interval(tcx, body, state, location, place, 2))
}

fn operand_interval<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    operand: &mir::Operand<'tcx>,
) -> Option<(i128, i128)> {
    match operand {
        mir::Operand::Copy(place) | mir::Operand::Move(place) => {
            let ty = place.ty(&body.local_decls, tcx).ty;
            is_interval_scalar_ty(ty).then(|| tracked_interval(state, *place))?
        },
        mir::Operand::Constant(konst) => const_operand_interval(konst),
    }
}

fn is_nonzero_new_unchecked_call<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    func: &mir::Operand<'tcx>,
    destination: mir::Place<'tcx>,
) -> bool {
    if !is_core_num_nonzero(tcx, destination.ty(&body.local_decls, tcx).ty) {
        return false;
    }
    let TyKind::FnDef(def_id, _) = func.ty(&body.local_decls, tcx).kind() else {
        return false;
    };
    tcx.item_name(*def_id).as_str() == "new_unchecked"
}

fn location_precedes(left: mir::Location, right: mir::Location) -> bool {
    left.block < right.block || (left.block == right.block && left.statement_index < right.statement_index)
}

fn nonzero_constructor_interval<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    local: mir::Local,
) -> Option<(i128, i128)> {
    let target = mir::Place::from(local);
    if !is_core_num_nonzero(tcx, target.ty(&body.local_decls, tcx).ty) {
        return None;
    }

    let mut interval = None;
    for (block, data) in body.basic_blocks.iter_enumerated() {
        let call_location = mir::Location {
            block,
            statement_index: data.statements.len(),
        };
        if !location_precedes(call_location, location) {
            continue;
        }
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
        if !is_nonzero_new_unchecked_call(tcx, body, func, *destination) {
            continue;
        }
        if !state.symbolic.equiv_places_readonly(target, *destination) {
            continue;
        }
        let arg = &args.first()?.node;
        interval = operand_interval(tcx, body, state, arg);
    }

    interval
}

pub(crate) fn local_interval<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
) -> Option<(i128, i128)> {
    let state = state_before_location(tcx, body, result, location)?;
    let place = mir::Place::from(local);
    tracked_interval(&state, place)
        .or_else(|| {
            let place = nonzero_scalar_place(tcx, body, place)?;
            tracked_interval(&state, place)
        })
        .or_else(|| nonzero_constructor_interval(tcx, body, &state, location, local))
}

fn local_or_pointee_interval<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    local: mir::Local,
) -> Option<((i128, i128), rustc_middle::ty::Ty<'tcx>)> {
    let place = mir::Place::from(local);
    let ty = place.ty(&body.local_decls, tcx).ty;
    if is_interval_scalar_ty(ty) {
        return tracked_interval(state, place).map(|interval| (interval, ty));
    }
    let TyKind::Ref(_, inner, _) = ty.kind() else {
        return None;
    };
    if !is_interval_scalar_ty(*inner) {
        return None;
    }
    let pointee = place.project_deeper(&[mir::ProjectionElem::Deref], tcx);
    tracked_interval(state, pointee).map(|interval| (interval, *inner))
}

fn scalar_bits<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    ty: rustc_middle::ty::Ty<'tcx>,
    value: i128,
) -> Option<u128> {
    let bit_width = tcx.layout_of(typing_env.as_query_input(ty)).ok()?.size.bits();
    if bit_width == 0 || bit_width > 64 {
        return None;
    }
    let mask = (1u128 << bit_width) - 1;
    Some((value as u128) & mask)
}

pub(crate) fn value_invalid_for_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
    target_ty: rustc_middle::ty::Ty<'tcx>,
) -> bool {
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let Some(((low, high), source_ty)) = local_or_pointee_interval(tcx, body, &state, local) else {
        return false;
    };
    if low != high {
        return false;
    }
    let Some(bits) = scalar_bits(tcx, typing_env, source_ty, low) else {
        return false;
    };

    match target_ty.kind() {
        TyKind::Bool => bits > 1,
        TyKind::Char => bits > 0x10_FFFF || (0xD800..=0xDFFF).contains(&bits),
        TyKind::Adt(_, _) if is_core_num_nonzero(tcx, target_ty) => bits == 0,
        _ => false,
    }
}

pub(crate) fn interval_eq_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(expected) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some((low, high)) = local_interval(tcx, body, result, location, local) else {
        return false;
    };
    low == expected && high == expected
}

pub(crate) fn interval_ne_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(expected) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let local_ty = body.local_decls[local].ty;
    if is_interval_scalar_ty(local_ty) || is_core_num_nonzero(tcx, local_ty) {
        return local_interval(tcx, body, result, location, local)
            .is_some_and(|interval| interval_excludes_const(interval, expected));
    }
    if !(-128..=255).contains(&expected) {
        return false;
    }
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let Some((len_low, len_high)) = local_len(tcx, body, &state, location, local) else {
        return false;
    };
    if len_low != len_high || !(0..=MAX_EXPLICIT_BYTE_SLICE_LEN).contains(&len_low) {
        return false;
    }
    let len = len_low as u64;
    let Some(source) = byte_sequence_source(tcx, body, &state, location, mir::Place::from(local), 16) else {
        return false;
    };

    (0..len).all(|index| {
        array_element_interval(tcx, body, &state, source, len, index)
            .is_some_and(|element| interval_excludes_const((element.low, element.high), expected))
    })
}

fn interval_excludes_const((low, high): (i128, i128), expected: i128) -> bool {
    high < expected || low > expected
}

pub(crate) fn interval_lt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(expected) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some((_low, high)) = local_interval(tcx, body, result, location, local) else {
        return false;
    };
    high < expected
}

pub(crate) fn interval_le_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(expected) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some((_low, high)) = local_interval(tcx, body, result, location, local) else {
        return false;
    };
    high <= expected
}

pub(crate) fn interval_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(expected) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some((low, _high)) = local_interval(tcx, body, result, location, local) else {
        return false;
    };
    low > expected
}

pub(crate) fn interval_ge_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(expected) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some((low, _high)) = local_interval(tcx, body, result, location, local) else {
        return false;
    };
    low >= expected
}

pub(crate) fn intervals_equal<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
) -> bool {
    let Some((left_low, left_high)) = local_interval(tcx, body, result, location, left) else {
        return false;
    };
    let Some((right_low, right_high)) = local_interval(tcx, body, result, location, right) else {
        return false;
    };
    left_low == left_high && right_low == right_high && left_low == right_low
}

pub(crate) fn interval_less_than<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
) -> bool {
    let Some((_left_low, left_high)) = local_interval(tcx, body, result, location, left) else {
        return false;
    };
    let Some((right_low, _right_high)) = local_interval(tcx, body, result, location, right) else {
        return false;
    };
    left_high < right_low
}

pub(crate) fn interval_greater_than<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
) -> bool {
    let Some((left_low, _left_high)) = local_interval(tcx, body, result, location, left) else {
        return false;
    };
    let Some((_right_low, right_high)) = local_interval(tcx, body, result, location, right) else {
        return false;
    };
    left_low > right_high
}

pub(crate) fn layout_size_eq_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    layout: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(expected) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    layout_size_interval(tcx, body, &state, location, layout)
        .is_some_and(|(low, high)| low == expected && high == expected)
}

pub(crate) fn layout_size_gt<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
) -> bool {
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let Some((left_low, _)) = layout_size_interval(tcx, body, &state, location, left) else {
        return false;
    };
    let Some((_, right_high)) = layout_size_interval(tcx, body, &state, location, right) else {
        return false;
    };
    left_low > right_high
}

pub(crate) fn range_start_gt_end<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    range: mir::Local,
) -> bool {
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let range = mir::Place::from(range);
    let Some((start_low, _start_high)) = wrapped_scalar_field_interval(tcx, body, &state, location, range, "start", 8)
    else {
        return false;
    };
    let Some((_end_low, end_high)) = wrapped_scalar_field_interval(tcx, body, &state, location, range, "end", 8) else {
        return false;
    };
    start_low > end_high
}

pub(crate) fn range_start_gt_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    arr_local: mir::Local,
    range: mir::Local,
) -> bool {
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let range = mir::Place::from(range);
    let Some((start_low, _start_high)) = wrapped_scalar_field_interval(tcx, body, &state, location, range, "start", 8)
    else {
        return false;
    };
    let Some((_len_low, len_high)) = local_len(tcx, body, &state, location, arr_local) else {
        return false;
    };
    start_low > len_high
}

pub(crate) fn range_end_gt_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    arr_local: mir::Local,
    range: mir::Local,
) -> bool {
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let range = mir::Place::from(range);
    let Some((end_low, _end_high)) = wrapped_scalar_field_interval(tcx, body, &state, location, range, "end", 8) else {
        return false;
    };
    let Some((_len_low, len_high)) = local_len(tcx, body, &state, location, arr_local) else {
        return false;
    };
    end_low > len_high
}

pub(crate) fn range_end_ge_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    arr_local: mir::Local,
    range: mir::Local,
) -> bool {
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let range = mir::Place::from(range);
    let Some((end_low, _end_high)) = wrapped_scalar_field_interval(tcx, body, &state, location, range, "end", 8) else {
        return false;
    };
    let Some((_len_low, len_high)) = local_len(tcx, body, &state, location, arr_local) else {
        return false;
    };
    end_low >= len_high
}

pub(crate) fn add_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(limit) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some((left_low, _left_high)) = local_interval(tcx, body, result, location, left) else {
        return false;
    };
    let Some((right_low, _right_high)) = local_interval(tcx, body, result, location, right) else {
        return false;
    };
    left_low.checked_add(right_low).is_some_and(|sum| sum > limit)
}

pub(crate) fn add_lt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(limit) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some((_left_low, left_high)) = local_interval(tcx, body, result, location, left) else {
        return false;
    };
    let Some((_right_low, right_high)) = local_interval(tcx, body, result, location, right) else {
        return false;
    };
    left_high.checked_add(right_high).is_some_and(|sum| sum < limit)
}

pub(crate) fn sub_lt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(limit) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some((_left_low, left_high)) = local_interval(tcx, body, result, location, left) else {
        return false;
    };
    let Some((right_low, _right_high)) = local_interval(tcx, body, result, location, right) else {
        return false;
    };
    left_high.checked_sub(right_low).is_some_and(|diff| diff < limit)
}

pub(crate) fn sub_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(limit) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some((left_low, _left_high)) = local_interval(tcx, body, result, location, left) else {
        return false;
    };
    let Some((_right_low, right_high)) = local_interval(tcx, body, result, location, right) else {
        return false;
    };
    left_low.checked_sub(right_high).is_some_and(|diff| diff > limit)
}

fn mul_interval(left: (i128, i128), right: (i128, i128)) -> Option<(i128, i128)> {
    let products = [
        left.0.checked_mul(right.0)?,
        left.0.checked_mul(right.1)?,
        left.1.checked_mul(right.0)?,
        left.1.checked_mul(right.1)?,
    ];
    Some((
        *products.iter().min().expect("non-empty product list"),
        *products.iter().max().expect("non-empty product list"),
    ))
}

pub(crate) fn mul_lt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(limit) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some(left_interval) = local_interval(tcx, body, result, location, left) else {
        return false;
    };
    let Some(right_interval) = local_interval(tcx, body, result, location, right) else {
        return false;
    };
    let Some((_product_low, product_high)) = mul_interval(left_interval, right_interval) else {
        return false;
    };
    product_high < limit
}

pub(crate) fn mul_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(limit) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some(left_interval) = local_interval(tcx, body, result, location, left) else {
        return false;
    };
    let Some(right_interval) = local_interval(tcx, body, result, location, right) else {
        return false;
    };
    let Some((product_low, _product_high)) = mul_interval(left_interval, right_interval) else {
        return false;
    };
    product_low > limit
}

pub(crate) fn size_mul_lt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    value: mir::Local,
    ty: rustc_middle::ty::Ty<'tcx>,
    konst: Const<'tcx>,
) -> bool {
    let Some(limit) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some(value_interval) = local_interval(tcx, body, result, location, value) else {
        return false;
    };
    let Ok(layout) = tcx.layout_of(typing_env.as_query_input(ty)) else {
        return false;
    };
    let size = i128::from(layout.size.bytes());
    mul_interval(value_interval, (size, size)).is_some_and(|(_, product_high)| product_high < limit)
}

pub(crate) fn size_mul_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    value: mir::Local,
    ty: rustc_middle::ty::Ty<'tcx>,
    konst: Const<'tcx>,
) -> bool {
    let Some(limit) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some(value_interval) = local_interval(tcx, body, result, location, value) else {
        return false;
    };
    let Ok(layout) = tcx.layout_of(typing_env.as_query_input(ty)) else {
        return false;
    };
    let size = i128::from(layout.size.bytes());
    mul_interval(value_interval, (size, size)).is_some_and(|(product_low, _)| product_low > limit)
}

pub(crate) fn slice_size_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    slice: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(limit) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let Some(len) = local_len(tcx, body, &state, location, slice) else {
        return false;
    };
    let mut ty = body.local_decls[slice].ty;
    if let TyKind::Ref(_, inner, _) | TyKind::RawPtr(inner, _) = ty.kind() {
        ty = *inner;
    }
    let (TyKind::Slice(element) | TyKind::Array(element, _)) = ty.kind() else {
        return false;
    };
    let Ok(layout) = tcx.layout_of(typing_env.as_query_input(*element)) else {
        return false;
    };
    let size = i128::from(layout.size.bytes());
    mul_interval(len, (size, size)).is_some_and(|(low, _)| low > limit)
}

fn local_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    local: mir::Local,
) -> Option<(i128, i128)> {
    let place = mir::Place::from(local);
    let ty = place.ty(&body.local_decls, tcx).ty;
    let len_iv = match ty.kind() {
        TyKind::Array(_, len) => len
            .try_to_target_usize(tcx)
            .map(|len| Interval::new(len as i128, len as i128)),
        TyKind::Slice(_) => tracked_len(state, place),
        TyKind::Ref(_, inner, _) => match inner.kind() {
            TyKind::Array(_, len) => len
                .try_to_target_usize(tcx)
                .map(|len| Interval::new(len as i128, len as i128)),
            TyKind::Slice(_) => tracked_len(state, place),
            _ => None,
        },
        TyKind::RawPtr(inner, _) => match inner.kind() {
            TyKind::Array(_, len) => len
                .try_to_target_usize(tcx)
                .map(|len| Interval::new(len as i128, len as i128)),
            TyKind::Slice(_) | TyKind::Str => {
                tracked_len(state, place).or_else(|| raw_slice_ptr_len_from_defs(tcx, body, state, location, place, 8))
            },
            _ => tracked_len(state, place),
        },
        _ => None,
    }?;
    if len_iv.is_empty() {
        return None;
    }
    Some((len_iv.low, len_iv.high))
}

const MAX_EXPLICIT_BYTE_SLICE_LEN: i128 = 32;

fn is_byte_array_like<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, place: mir::Place<'tcx>) -> bool {
    let ty = place.ty(&body.local_decls, tcx).ty;
    let TyKind::Array(element, _) = ty.kind() else {
        return false;
    };
    matches!(
        element.kind(),
        TyKind::Int(rustc_middle::ty::IntTy::I8) | TyKind::Uint(rustc_middle::ty::UintTy::U8)
    )
}

fn direct_byte_array_source<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    place: mir::Place<'tcx>,
) -> Option<mir::Place<'tcx>> {
    if is_byte_array_like(tcx, body, place) {
        return Some(place);
    }

    state.interval.all_fact_places().into_iter().find(|candidate| {
        is_byte_array_like(tcx, body, *candidate) && state.symbolic.equiv_places_readonly(place, *candidate)
    })
}

fn byte_sequence_source_from_rvalue<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    rvalue: &mir::Rvalue<'tcx>,
    depth: usize,
) -> Option<mir::Place<'tcx>> {
    match rvalue {
        mir::Rvalue::Use(mir::Operand::Copy(place) | mir::Operand::Move(place))
        | mir::Rvalue::Cast(_, mir::Operand::Copy(place) | mir::Operand::Move(place), _) => {
            byte_sequence_source(tcx, body, state, location, *place, depth - 1)
        },
        mir::Rvalue::Ref(_, _, place) | mir::Rvalue::RawPtr(_, place) => {
            byte_sequence_source(tcx, body, state, location, *place, depth - 1)
        },
        _ => None,
    }
}

fn byte_sequence_source<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    location: mir::Location,
    target: mir::Place<'tcx>,
    depth: usize,
) -> Option<mir::Place<'tcx>> {
    if depth == 0 {
        return None;
    }
    if let Some(source) = direct_byte_array_source(tcx, body, state, target) {
        return Some(source);
    }

    let mut definition = None;
    for (block, data) in body.basic_blocks.iter_enumerated() {
        for (statement_index, statement) in data.statements.iter().enumerate() {
            let statement_location = mir::Location { block, statement_index };
            if !location_precedes(statement_location, location) {
                continue;
            }
            let mir::StatementKind::Assign(box (place, rvalue)) = &statement.kind else {
                continue;
            };
            if *place == target {
                if definition.is_some() {
                    return None;
                }
                definition = Some(rvalue);
            }
        }
    }
    if let Some(rvalue) = definition {
        return byte_sequence_source_from_rvalue(tcx, body, state, location, rvalue, depth);
    }

    let mut receiver = None;
    for (block, data) in body.basic_blocks.iter_enumerated() {
        let call_location = mir::Location {
            block,
            statement_index: data.statements.len(),
        };
        if !location_precedes(call_location, location) {
            continue;
        }
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
        if *destination != target {
            continue;
        }
        let Some(path) = call_func_path(tcx, body, func) else {
            continue;
        };
        if !path.ends_with("::as_ptr")
            && !path.ends_with("::as_mut_ptr")
            && !path.ends_with("::cast")
            && !path.ends_with("::cast_const")
            && !path.ends_with("::cast_mut")
        {
            return None;
        }
        let Some(mir::Operand::Copy(place) | mir::Operand::Move(place)) = args.first().map(|arg| &arg.node) else {
            return None;
        };
        if receiver.is_some() {
            return None;
        }
        receiver = Some(*place);
    }

    byte_sequence_source(tcx, body, state, location, receiver?, depth - 1)
}

fn array_element_interval<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    state: &CombinedState<'tcx>,
    source: mir::Place<'tcx>,
    len: u64,
    index: u64,
) -> Option<Interval> {
    if index >= len {
        return None;
    }

    let index_projection = mir::ProjectionElem::ConstantIndex {
        offset: index,
        min_length: len,
        from_end: false,
    };
    let source_is_reference = matches!(source.ty(&body.local_decls, tcx).ty.kind(), TyKind::Ref(_, _, _));
    let element_place = if source_is_reference {
        source.project_deeper(&[mir::ProjectionElem::Deref, index_projection], tcx)
    } else {
        source.project_deeper(&[index_projection], tcx)
    };

    state
        .interval
        .tracked_interval_resolved(&state.symbolic, &element_place)
        .filter(|element| !element.is_empty())
}

pub(crate) fn byte_slice_is_empty<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let Some((len_low, len_high)) = local_len(tcx, body, &state, location, local) else {
        return false;
    };
    if len_low != len_high || !(0..=MAX_EXPLICIT_BYTE_SLICE_LEN).contains(&len_low) {
        return false;
    }

    len_low == 0
}

pub(crate) fn byte_slice_last_byte_not_nul<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let Some((len_low, len_high)) = local_len(tcx, body, &state, location, local) else {
        return false;
    };
    if len_low != len_high || !(0..=MAX_EXPLICIT_BYTE_SLICE_LEN).contains(&len_low) || len_low == 0 {
        return false;
    }
    let len = len_low as u64;

    let Some(source) = byte_sequence_source(tcx, body, &state, location, mir::Place::from(local), 16) else {
        return false;
    };
    let Some(last_byte) = array_element_interval(tcx, body, &state, source, len, len - 1) else {
        return false;
    };

    last_byte.low > 0 || last_byte.high < 0
}

pub(crate) fn byte_slice_contains_interior_nul<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let Some((len_low, len_high)) = local_len(tcx, body, &state, location, local) else {
        return false;
    };
    if len_low != len_high || !(0..=MAX_EXPLICIT_BYTE_SLICE_LEN).contains(&len_low) || len_low <= 1 {
        return false;
    };
    let len = len_low as u64;

    let Some(source) = byte_sequence_source(tcx, body, &state, location, mir::Place::from(local), 16) else {
        return false;
    };
    for index in 0..len - 1 {
        let Some(element) = array_element_interval(tcx, body, &state, source, len, index) else {
            continue;
        };
        if element.low == 0 && element.high == 0 {
            return true;
        }
    }

    false
}

fn index_violates_bound<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    arr_local: mir::Local,
    idx_local: mir::Local,
    allow_equal: bool,
) -> bool {
    let Some(state) = state_before_location(tcx, body, result, location) else {
        return false;
    };
    let idx_place = mir::Place::from(idx_local);
    let Some(idx_iv) = state.interval.tracked_interval_resolved(&state.symbolic, &idx_place) else {
        return false;
    };
    if idx_iv.is_empty() || idx_iv.low < 0 {
        return false;
    }
    let Some((_len_low, len_high)) = local_len(tcx, body, &state, location, arr_local) else {
        return false;
    };
    if allow_equal {
        idx_iv.low > len_high
    } else {
        idx_iv.low >= len_high
    }
}

pub(crate) fn index_ge_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    arr_local: mir::Local,
    idx_local: mir::Local,
) -> bool {
    index_violates_bound(tcx, body, result, location, arr_local, idx_local, false)
}

pub(crate) fn index_gt_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    arr_local: mir::Local,
    idx_local: mir::Local,
) -> bool {
    index_violates_bound(tcx, body, result, location, arr_local, idx_local, true)
}

pub(crate) fn not_power_of_two<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    let Some((low, high)) = local_interval(tcx, body, result, location, local) else {
        return false;
    };
    if low != high {
        return false;
    }
    low < 0 || !(low as u128).is_power_of_two()
}

pub(crate) fn rounded_up_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    result: &PathForwardAnalysisResult<CombinedState<'tcx>>,
    location: mir::Location,
    size_local: mir::Local,
    align_local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    let Some(limit) = const_to_i128(tcx, typing_env, konst) else {
        return false;
    };
    if limit < 0 {
        return true;
    }
    let Some((size_low, _size_high)) = local_interval(tcx, body, result, location, size_local) else {
        return false;
    };
    let Some((align_low, align_high)) = local_interval(tcx, body, result, location, align_local) else {
        return false;
    };
    if align_low != align_high || align_low <= 0 {
        return false;
    }
    let align = align_low as u128;
    if !align.is_power_of_two() {
        return false;
    }
    if size_low < 0 {
        return false;
    }

    let max_size_for_align = (limit as u128).saturating_sub(align - 1);
    size_low as u128 > max_size_for_align
}
