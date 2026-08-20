use std::cell::{OnceCell, RefCell};
use std::fmt;
use std::rc::Rc;

use mirsa::analysis::combined::CombinedState;
use mirsa::framework::forward::PathForwardAnalysisResult;
use mirsa::framework::printer::StateEntries;
use rustc_data_structures::fx::FxHashMap;
use rustc_index::IndexVec;
use rustc_middle::mir::{self};
use rustc_middle::ty::{self, TyCtxt};

use super::internval_analysis::analyze_interval;
use super::null_analysis::analyze_null;
use super::ptr_state_analysis::{PointerSummaryCache, analyze_pointer_state_query, call_argument_is_used};
use crate::Const;

#[derive(Default)]
pub struct CrateAnalysisCache {
    pointer_summaries: Rc<PointerSummaryCache>,
}

pub struct BodyInfoCache<'tcx> {
    null_analysis: OnceCell<PathForwardAnalysisResult<CombinedState<'tcx>>>,
    interval_analysis: OnceCell<PathForwardAnalysisResult<CombinedState<'tcx>>>,
    pointer_state_queries: RefCell<FxHashMap<((usize, usize), usize, bool), bool>>,
    pointer_summaries: Rc<PointerSummaryCache>,
    /// `product_of[i][j]` is `Some(true)` if `i` may be a product of `j`, `Some(false)` if `i` may
    /// be a quotient of `j`, and `None` if there is no relationship.
    product_of: IndexVec<mir::Local, IndexVec<mir::Local, Option<bool>>>,
    // /// `derive_from[i][j]` is `true` if `i` may be computed from `j`, `false` if there is no
    // /// relationship.
    // derive_from: IndexVec<mir::Local, IndexVec<mir::Local, bool>>,
}

impl fmt::Debug for BodyInfoCache<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct NullMirsa<'a, 'tcx>(Option<&'a PathForwardAnalysisResult<CombinedState<'tcx>>>);
        impl fmt::Debug for NullMirsa<'_, '_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let Some(result) = self.0 else {
                    return f.write_str("<uninitialized>");
                };
                f.debug_list()
                    .entries(result.out_states.iter().enumerate().flat_map(|(bb, state)| {
                        state
                            .entries()
                            .into_iter()
                            .map(move |(place, value)| (bb, place.local.as_usize(), value))
                    }))
                    .finish()
            }
        }
        struct InternvalMirsa<'a, 'tcx>(Option<&'a PathForwardAnalysisResult<CombinedState<'tcx>>>);
        impl fmt::Debug for InternvalMirsa<'_, '_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let Some(result) = self.0 else {
                    return f.write_str("<uninitialized>");
                };
                f.debug_list()
                    .entries(result.out_states.iter().enumerate().flat_map(|(bb, state)| {
                        state.entries().into_iter().filter_map(move |(place, value)| {
                            value
                                .strip_prefix("interval ")
                                .map(|value| (bb, place.local.as_usize(), value.to_string()))
                        })
                    }))
                    .finish()
            }
        }

        struct ProductOf<'a>(&'a IndexVec<mir::Local, IndexVec<mir::Local, Option<bool>>>);
        impl fmt::Debug for ProductOf<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list()
                    .entries(
                        self.0
                            .iter_enumerated()
                            .flat_map(|(i, j)| j.iter_enumerated().filter_map(move |(k, v)| v.map(|b| (i, k, b)))),
                    )
                    .finish()
            }
        }
        f.debug_struct("BodyInfoCache")
            .field("null_analysis", &NullMirsa(self.null_analysis.get()))
            .field("interval_analysis", &InternvalMirsa(self.interval_analysis.get()))
            .field("pointer_state_queries", &self.pointer_state_queries.borrow().len())
            .field("pointer_summaries", &self.pointer_summaries)
            .field("product_of", &ProductOf(&self.product_of))
            // .field("derive_from", &self.derive_from) // Uncomment if derive_from is implemented
            .finish()
    }
}

impl<'tcx> BodyInfoCache<'tcx> {
    #[instrument(level = "debug", skip(tcx, body, crate_cache), fields(n = body.local_decls.len()), ret)]
    pub fn new(
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        body: &mir::Body<'tcx>,
        crate_cache: &CrateAnalysisCache,
    ) -> Self {
        let _ = (tcx, typing_env);
        let n = body.local_decls.len();

        // Track the product relationship among locals, true for product, false for quotient
        let mut product_of: IndexVec<mir::Local, IndexVec<mir::Local, Option<bool>>> =
            IndexVec::from_fn_n(|_| IndexVec::from_elem_n(None, n), n);
        for i in 0..n {
            let i = mir::Local::from_usize(i);
            product_of[i][i] = Some(true);
        }
        for local in body.basic_blocks.iter() {
            for stmt in &local.statements {
                // Check if the statement is a product or quotient
                if let mir::StatementKind::Assign(box (ref lhs, ref rhs)) = stmt.kind
                    && let Some(lhs) = lhs.as_local()
                {
                    match rhs {
                        mir::Rvalue::BinaryOp(
                            mir::BinOp::Mul
                            | mir::BinOp::MulUnchecked
                            | mir::BinOp::MulWithOverflow
                            | mir::BinOp::Add
                            | mir::BinOp::AddUnchecked
                            | mir::BinOp::AddWithOverflow
                            | mir::BinOp::Sub
                            | mir::BinOp::SubUnchecked
                            | mir::BinOp::SubWithOverflow,
                            box rhs,
                        ) => {
                            if let mir::Operand::Copy(rhs1) | mir::Operand::Move(rhs1) = rhs.0
                                && let Some(rhs1) = rhs1.as_local()
                            {
                                product_of[lhs][rhs1] = Some(true);
                            }
                            if let mir::Operand::Copy(rhs2) | mir::Operand::Move(rhs2) = rhs.1
                                && let Some(rhs2) = rhs2.as_local()
                            {
                                product_of[lhs][rhs2] = Some(true);
                            }
                        },
                        mir::Rvalue::BinaryOp(mir::BinOp::Div, box rhs) => {
                            if let mir::Operand::Copy(rhs1) | mir::Operand::Move(rhs1) = rhs.0
                                && let Some(rhs1) = rhs1.as_local()
                            {
                                product_of[lhs][rhs1] = Some(true);
                            }
                            if let mir::Operand::Copy(rhs2) | mir::Operand::Move(rhs2) = rhs.1
                                && let Some(rhs2) = rhs2.as_local()
                            {
                                product_of[lhs][rhs2] = Some(false);
                            }
                        },
                        mir::Rvalue::Use(mir::Operand::Copy(rhs) | mir::Operand::Move(rhs))
                        | mir::Rvalue::Cast(_, mir::Operand::Copy(rhs) | mir::Operand::Move(rhs), _) => {
                            if let Some(rhs) = rhs.as_local() {
                                product_of[lhs][rhs] = Some(true);
                            }
                        },
                        _ => (),
                    }
                }
            }
        }
        for j in 0..n {
            let j = mir::Local::from_usize(j);
            for i in 0..n {
                for k in 0..n {
                    if i == k {
                        continue;
                    }
                    let i = mir::Local::from_usize(i);
                    let k = mir::Local::from_usize(k);
                    if let (Some(s1), Some(s2)) = (product_of[i][j], product_of[j][k]) {
                        product_of[i][k] = Some(s1 == s2);
                    }
                }
            }
        }
        Self {
            null_analysis: OnceCell::new(),
            interval_analysis: OnceCell::new(),
            pointer_state_queries: RefCell::new(FxHashMap::default()),
            pointer_summaries: Rc::clone(&crate_cache.pointer_summaries),
            product_of,
        }
    }

    fn null_analysis(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &mir::Body<'tcx>,
    ) -> &PathForwardAnalysisResult<CombinedState<'tcx>> {
        self.null_analysis
            .get_or_init(|| analyze_null(tcx, tcx.optimized_mir(body.source.def_id())))
    }

    pub(crate) fn interval_analysis(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &mir::Body<'tcx>,
    ) -> &PathForwardAnalysisResult<CombinedState<'tcx>> {
        self.interval_analysis
            .get_or_init(|| analyze_interval(tcx, tcx.optimized_mir(body.source.def_id())))
    }

    pub(super) fn is_dead_before_local(
        &self,
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        body: &mir::Body<'tcx>,
        location: mir::Location,
        local: mir::Local,
        pointer_only: bool,
    ) -> bool {
        let key = (
            (location.block.as_usize(), location.statement_index),
            local.as_usize(),
            pointer_only,
        );
        if let Some(result) = self.pointer_state_queries.borrow().get(&key).copied() {
            return result;
        }
        let result = analyze_pointer_state_query(
            tcx,
            typing_env,
            body,
            &self.pointer_summaries,
            location,
            local,
            pointer_only,
        );
        self.pointer_state_queries.borrow_mut().insert(key, result);
        result
    }

    pub(super) fn is_call_argument_used(
        &self,
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        body: &mir::Body<'tcx>,
        location: mir::Location,
        local: mir::Local,
    ) -> bool {
        call_argument_is_used(tcx, typing_env, body, &self.pointer_summaries, location, local)
    }
}

pub type LocationLocalPredsFnPtr = for<'a, 'tcx> fn(
    TyCtxt<'tcx>,
    ty::TypingEnv<'tcx>,
    &'a mir::Body<'tcx>,
    &'a BodyInfoCache<'tcx>,
    mir::Location,
    mir::Local,
) -> bool;

pub type LocationPredsFnPtr = for<'a, 'tcx> fn(
    TyCtxt<'tcx>,
    ty::TypingEnv<'tcx>,
    &'a mir::Body<'tcx>,
    &'a BodyInfoCache<'tcx>,
    mir::Location,
) -> bool;

pub type SingleLocalPredsFnPtr = for<'a, 'tcx> fn(
    TyCtxt<'tcx>,
    ty::TypingEnv<'tcx>,
    &'a mir::Body<'tcx>,
    &'a BodyInfoCache<'tcx>,
    mir::Local,
) -> bool;

/// Check if a local is stably null according to mirsa null analysis at the matched location's
/// block.
#[instrument(level = "debug", skip(tcx, body, cache), ret)]
pub(crate) fn is_null_at_location<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    super::null_analysis::is_null_at(tcx, body, cache.null_analysis(tcx, body), location, local)
}

/// Check if a statically known byte slice is empty.
#[instrument(level = "debug", skip(tcx, body, cache), ret)]
pub(crate) fn byte_slice_is_empty<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    super::internval_analysis::byte_slice_is_empty(tcx, body, cache.interval_analysis(tcx, body), location, local)
}

/// Check if a statically known non-empty byte slice does not end in NUL.
#[instrument(level = "debug", skip(tcx, body, cache), ret)]
pub(crate) fn byte_slice_last_byte_not_nul<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    super::internval_analysis::byte_slice_last_byte_not_nul(
        tcx,
        body,
        cache.interval_analysis(tcx, body),
        location,
        local,
    )
}

/// Check if a statically known byte slice contains NUL before the last byte.
#[instrument(level = "debug", skip(tcx, body, cache), ret)]
pub(crate) fn contains_interior_nul<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
) -> bool {
    super::internval_analysis::byte_slice_contains_interior_nul(
        tcx,
        body,
        cache.interval_analysis(tcx, body),
        location,
        local,
    )
}

pub type LocationLocalConstPredsFnPtr = for<'a, 'tcx> fn(
    TyCtxt<'tcx>,
    ty::TypingEnv<'tcx>,
    &'a mir::Body<'tcx>,
    &'a BodyInfoCache<'tcx>,
    mir::Location,
    mir::Local,
    Const<'tcx>,
) -> bool;

pub type LocationLocalTyPredsFnPtr = for<'a, 'tcx> fn(
    TyCtxt<'tcx>,
    ty::TypingEnv<'tcx>,
    &'a mir::Body<'tcx>,
    &'a BodyInfoCache<'tcx>,
    mir::Location,
    mir::Local,
    ty::Ty<'tcx>,
) -> bool;

pub type LocationLocalTyConstPredsFnPtr = for<'a, 'tcx> fn(
    TyCtxt<'tcx>,
    ty::TypingEnv<'tcx>,
    &'a mir::Body<'tcx>,
    &'a BodyInfoCache<'tcx>,
    mir::Location,
    mir::Local,
    ty::Ty<'tcx>,
    Const<'tcx>,
) -> bool;

#[instrument(level = "debug", skip(tcx, typing_env, body, cache), ret)]
pub(crate) fn value_invalid_for_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
    target_ty: ty::Ty<'tcx>,
) -> bool {
    super::internval_analysis::value_invalid_for_type(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        local,
        target_ty,
    )
}

#[instrument(level = "debug", skip(tcx, typing_env, cache), ret)]
pub(crate) fn eq_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::interval_eq_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        local,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, typing_env, cache), ret)]
pub(crate) fn ne_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::interval_ne_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        local,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, typing_env, cache), ret)]
pub(crate) fn lt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::interval_lt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        local,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, typing_env, cache), ret)]
pub(crate) fn le_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::interval_le_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        local,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, typing_env, cache), ret)]
pub(crate) fn gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::interval_gt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        local,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, typing_env, cache), ret)]
pub(crate) fn ge_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::interval_ge_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        local,
        konst,
    )
}

pub type LocationLocalLocalPredsFnPtr = for<'a, 'tcx> fn(
    TyCtxt<'tcx>,
    ty::TypingEnv<'tcx>,
    &'a mir::Body<'tcx>,
    &'a BodyInfoCache<'tcx>,
    mir::Location,
    mir::Local,
    mir::Local,
) -> bool;

pub type LocationLocalLocalConstPredsFnPtr = for<'a, 'tcx> fn(
    TyCtxt<'tcx>,
    ty::TypingEnv<'tcx>,
    &'a mir::Body<'tcx>,
    &'a BodyInfoCache<'tcx>,
    mir::Location,
    mir::Local,
    mir::Local,
    Const<'tcx>,
) -> bool;

/// Check if two locals are proven equal by mirsa interval analysis at the matched location.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn eq_local<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
) -> bool {
    super::internval_analysis::intervals_equal(tcx, body, cache.interval_analysis(tcx, body), location, left, right)
}

/// Check if the first local is proven less than the second local by mirsa interval analysis at the
/// matched location.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn lt_local<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
) -> bool {
    super::internval_analysis::interval_less_than(tcx, body, cache.interval_analysis(tcx, body), location, left, right)
}

/// Check if the first local is proven greater than the second local by mirsa interval analysis at
/// the matched location.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn gt_local<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
) -> bool {
    super::internval_analysis::interval_greater_than(
        tcx,
        body,
        cache.interval_analysis(tcx, body),
        location,
        left,
        right,
    )
}

/// Check if an index is proven to violate `index < len` for an array/slice.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn index_ge_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    arr_local: mir::Local,
    idx_local: mir::Local,
) -> bool {
    super::internval_analysis::index_ge_len(
        tcx,
        body,
        cache.interval_analysis(tcx, body),
        location,
        arr_local,
        idx_local,
    )
}

/// Check if an index is proven to violate `index <= len` for a slice split.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn index_gt_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    arr_local: mir::Local,
    idx_local: mir::Local,
) -> bool {
    super::internval_analysis::index_gt_len(
        tcx,
        body,
        cache.interval_analysis(tcx, body),
        location,
        arr_local,
        idx_local,
    )
}

/// Check if a range is proven to violate `start <= end`.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn range_start_gt_end<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    range: mir::Local,
) -> bool {
    super::internval_analysis::range_start_gt_end(tcx, body, cache.interval_analysis(tcx, body), location, range)
}

/// Check if a range is proven to violate `start <= len` for an array/slice.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn range_start_gt_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    arr_local: mir::Local,
    range: mir::Local,
) -> bool {
    super::internval_analysis::range_start_gt_len(
        tcx,
        body,
        cache.interval_analysis(tcx, body),
        location,
        arr_local,
        range,
    )
}

/// Check if a range is proven to violate `end <= len` for an array/slice.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn range_end_gt_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    arr_local: mir::Local,
    range: mir::Local,
) -> bool {
    super::internval_analysis::range_end_gt_len(
        tcx,
        body,
        cache.interval_analysis(tcx, body),
        location,
        arr_local,
        range,
    )
}

/// Check if an inclusive range is proven to violate `end < len` for an array/slice.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn range_end_ge_len<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    arr_local: mir::Local,
    range: mir::Local,
) -> bool {
    super::internval_analysis::range_end_ge_len(
        tcx,
        body,
        cache.interval_analysis(tcx, body),
        location,
        arr_local,
        range,
    )
}

/// Check whether a `Layout`'s size is proven equal to a constant.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn layout_size_eq_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    layout: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::layout_size_eq_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        layout,
        konst,
    )
}

/// Check whether the first `Layout`'s size is proven greater than the second's.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn layout_size_gt<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
) -> bool {
    super::internval_analysis::layout_size_gt(tcx, body, cache.interval_analysis(tcx, body), location, left, right)
}

/// Check if a local is proven not to be a power of two.
#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn not_power_of_two<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    align_local: mir::Local,
) -> bool {
    super::internval_analysis::not_power_of_two(tcx, body, cache.interval_analysis(tcx, body), location, align_local)
}

#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn add_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::add_gt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        left,
        right,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn add_lt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::add_lt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        left,
        right,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn sub_lt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::sub_lt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        left,
        right,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn sub_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::sub_gt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        left,
        right,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn mul_lt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::mul_lt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        left,
        right,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn mul_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    left: mir::Local,
    right: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::mul_gt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        left,
        right,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn size_mul_lt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    value: mir::Local,
    ty: ty::Ty<'tcx>,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::size_mul_lt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        value,
        ty,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn size_mul_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    value: mir::Local,
    ty: ty::Ty<'tcx>,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::size_mul_gt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        value,
        ty,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn slice_size_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    slice: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::slice_size_gt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        slice,
        konst,
    )
}

#[instrument(level = "debug", skip(tcx, cache), ret)]
pub(crate) fn rounded_up_gt_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    location: mir::Location,
    size_local: mir::Local,
    align_local: mir::Local,
    konst: Const<'tcx>,
) -> bool {
    super::internval_analysis::rounded_up_gt_const(
        tcx,
        typing_env,
        body,
        cache.interval_analysis(tcx, body),
        location,
        size_local,
        align_local,
        konst,
    )
}

// FIX: consider a more general way for error handling
pub type MultipleLocalsPredsFnPtr = for<'a, 'tcx> fn(
    TyCtxt<'tcx>,
    ty::TypingEnv<'tcx>,
    &'a mir::Body<'tcx>,
    &'a BodyInfoCache<'tcx>,
    Vec<mir::Local>,
) -> bool;

/// Check if former local is a product of latter local for every two consecutive locals
#[instrument(level = "debug", skip(cache), ret)]
pub(crate) fn product_of<'tcx>(
    _: TyCtxt<'tcx>,
    _: ty::TypingEnv<'tcx>,
    _: &mir::Body<'tcx>,
    cache: &BodyInfoCache<'tcx>,
    locals: Vec<mir::Local>,
) -> bool {
    locals.windows(2).all(|pair| {
        let (first, second) = (pair[0], pair[1]);
        cache.product_of[first][second].unwrap_or(false)
    })
}
