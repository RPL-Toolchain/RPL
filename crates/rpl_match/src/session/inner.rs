use rpl_context::pat::{self, PatternItem};

use crate::session::collect::MatchCollectCtxt;
use crate::session::config::SessionConfig;
use crate::session::matching::SessionMatching;
use crate::session::slot::{
    CrateItemIndex, MatchSlot, SessionOutcome, SessionResult, SlotCandidate, collect_slot_descs,
};

/// Orchestrates candidate collection and multi-slot matching for one pattern item.
pub struct MatchSession<'a, 'pcx, 'tcx> {
    collect: MatchCollectCtxt<'a, 'pcx, 'tcx>,
    config: SessionConfig,
}

impl<'a, 'pcx, 'tcx> MatchSession<'a, 'pcx, 'tcx> {
    pub fn new(collect: MatchCollectCtxt<'a, 'pcx, 'tcx>, config: SessionConfig) -> Self {
        Self { collect, config }
    }

    pub fn with_defaults(collect: MatchCollectCtxt<'a, 'pcx, 'tcx>) -> Self {
        Self::new(collect, SessionConfig::default())
    }

    pub fn match_rust_items(
        &self,
        index: &CrateItemIndex,
        rust_items: &'pcx pat::RustItems<'pcx>,
    ) -> SessionOutcome<'tcx> {
        let (fn_slots, adt_slots) = collect_slot_descs(rust_items);

        if fn_slots.is_empty() && adt_slots.is_empty() {
            return SessionOutcome::empty();
        }

        let mut outcome = SessionMatching::run(&self.collect, self.config, index, rust_items, &fn_slots, &adt_slots);
        self.enrich_results(index, &mut outcome.results);
        outcome.results = Self::deduplicate_results(rust_items.attr.should_deduplicate(), outcome.results);
        outcome
    }

    fn deduplicate_results(deduplicate: bool, results: Vec<SessionResult<'tcx>>) -> Vec<SessionResult<'tcx>> {
        if !deduplicate {
            return results;
        }
        let mut kept = Vec::new();
        for result in results {
            if kept
                .iter()
                .all(|existing: &SessionResult<'tcx>| !existing.equivalent(&result))
            {
                kept.push(result);
            }
        }
        kept
    }

    fn enrich_results(&self, index: &CrateItemIndex, results: &mut [SessionResult<'tcx>]) {
        for result in results.iter_mut() {
            if let Some(ctx) = &mut result.primary_fn {
                if let Some(item) = index.fns.iter().find(|i| i.def_id == ctx.def_id) {
                    ctx.fn_name = item.fn_name;
                    ctx.header = item.header;
                    ctx.has_self = item.has_self;
                }
                ctx.self_ty = index.self_ty(self.collect.tcx, ctx.def_id);
            }
        }
    }

    pub fn match_pattern_item(
        &self,
        index: &CrateItemIndex,
        pat_item: &'pcx PatternItem<'pcx>,
    ) -> SessionOutcome<'tcx> {
        match pat_item {
            PatternItem::RustItems(items) => self.match_rust_items(index, items),
            PatternItem::RPLPatternOperation(op) => {
                let mut outcome = self.match_pattern_operation(index, op);
                outcome.results = Self::deduplicate_results(op.attr.should_deduplicate(), outcome.results);
                outcome
            },
        }
    }

    fn match_pattern_operation(
        &self,
        index: &CrateItemIndex,
        op: &pat::PatternOperation<'pcx>,
    ) -> SessionOutcome<'tcx> {
        let mut truncated = false;
        let positive: Vec<_> = op
            .positive
            .iter()
            .flat_map(|(_, item, map)| {
                let out = self.match_pattern_item(index, item);
                truncated |= out.truncated;
                out.results.into_iter().map(|result| result.map_bindings(map))
            })
            .collect();

        let negative: Vec<_> = op
            .negative
            .iter()
            .flat_map(|(_, item, map)| {
                let out = self.match_pattern_item(index, item);
                truncated |= out.truncated;
                out.results.into_iter().map(|result| result.map_bindings(map))
            })
            .collect();

        let results = positive
            .into_iter()
            .filter(|pos| {
                if !pos.has_operation_key() {
                    return true;
                }
                !negative.iter().any(|neg| pos.subtracted_by(neg))
            })
            .collect();

        SessionOutcome { results, truncated }
    }
}

impl SessionResult<'_> {
    fn map_bindings(self, map: &pat::MatchedMap) -> Self {
        let assignments = self
            .assignments
            .into_iter()
            .map(|mut a| {
                if let SlotCandidate::Fn(ref mut c) = a.candidate {
                    let adt_defs = c.snapshot.adt_defs.clone();
                    c.normalized = c.normalized.clone().map(map);
                    c.snapshot =
                        super::bindings::BindingSnapshot::from_normalized_with_adt_defs(&c.normalized, adt_defs);
                }
                a
            })
            .collect();
        Self {
            assignments,
            bindings: self.bindings.map(map),
            primary_fn: self.primary_fn,
        }
    }

    /// Slot → DefId for MIR-body Fn slots and ADT slots.
    ///
    /// Signature-only Fn slots (empty `basic_blocks`, e.g. `fn $isnan(_: $T) -> bool;`) are
    /// omitted so `#[deduplicate]` can collapse f32/f64 `$isnan` pairings of the same MIR hit.
    /// `$f1`/`$f2` swaps still differ because both are MIR-body slots.
    fn mir_slot_def_signature(&self) -> Vec<(MatchSlot, rustc_hir::def_id::LocalDefId)> {
        let mut sig: Vec<_> = self
            .assignments
            .iter()
            .filter_map(|a| match &a.candidate {
                SlotCandidate::Fn(c) if !c.matched.basic_blocks.is_empty() => Some((a.slot, c.def_id)),
                SlotCandidate::Fn(_) => None,
                SlotCandidate::Adt(c) => Some((a.slot, c.def_id)),
            })
            .collect();
        sig.sort_by_key(|(slot, def_id)| (*slot, def_id.local_def_index));
        sig
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        // MIR-body (+ ADT) DefIds only: signature-only slots (e.g. `$isnan`) may differ
        // without meaning a distinct lint. Still require SharedEnv so diag metavars like
        // `{$T}` keep distinct instantiations.
        if self.mir_slot_def_signature() != other.mir_slot_def_signature()
            || !self.bindings.equivalent_to(&other.bindings)
        {
            return false;
        }
        for a in &self.assignments {
            let SlotCandidate::Fn(pos) = &a.candidate else {
                continue;
            };
            if pos.matched.basic_blocks.is_empty() {
                continue;
            }
            let Some(SlotCandidate::Fn(neg)) = other
                .assignments
                .iter()
                .find(|b| b.slot == a.slot)
                .map(|b| &b.candidate)
            else {
                return false;
            };
            // Label sites (ignore ty_vars noise on NormalizedMatched).
            if !pos.normalized.same_lint_sites(&neg.normalized) {
                return false;
            }
        }
        true
    }
}
