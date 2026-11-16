//! Unified multi-slot matching with function-owned locals/locations.
//!
//! All [`RustItems`](rpl_context::pat::RustItems) matching goes through [`SessionMatching`]:
//! shared metavars live in one [`MetaBindings`] (SharedEnv); MIR locals/locations are keyed by
//! [`MatchSlot`] and carry [`LocalDefId`]. Search fills all slots (optional slots may be skipped),
//! then pushes one [`SessionResult`] per complete assignment.

use std::cell::Cell;

use rpl_constraints::Const;
use rpl_context::pat::{self, ConstVarIdx, PlaceVarIdx, TyVarIdx};
use rustc_data_structures::fx::{FxHashMap, FxIndexSet};
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_index::IndexVec;
use rustc_middle::mir::{self, PlaceRef};
use rustc_middle::ty::Ty;
use rustc_span::Symbol;

use crate::CountedMatch;
use crate::matches::StatementMatch;
use crate::session::bindings::MetaBindings;
use crate::session::collect::MatchCollectCtxt;
use crate::session::config::SessionConfig;
use crate::session::slot::{
    AdtSlotCandidate, AdtSlotDesc, CrateFnItem, CrateItemIndex, FnMatchContext, FnSlotCandidate, FnSlotDesc, MatchSlot,
    SessionOutcome, SessionResult, SlotAssignment, SlotCandidate,
};

/// Pattern local owned by a fn slot.
pub type OwnedLocalPat = (MatchSlot, pat::Local);
/// Pattern statement location owned by a fn slot.
pub type OwnedLocationPat = (MatchSlot, pat::Location);
/// MIR local tagged with the function it belongs to.
pub type OwnedMirLocal = (LocalDefId, mir::Local);
/// MIR location tagged with the function it belongs to.
pub type OwnedMirLocation = (LocalDefId, mir::Location);

#[derive(Debug, Default)]
struct DefMatches {
    candidates: Vec<LocalDefId>,
    matched: CountedMatch<LocalDefId>,
}

impl DefMatches {
    fn add_candidate(&mut self, def_id: LocalDefId) {
        if !self.candidates.contains(&def_id) {
            self.candidates.push(def_id);
        }
    }
}

#[derive(Debug, Default)]
struct OwnedLocalMatches {
    candidates: Vec<OwnedMirLocal>,
    matched: CountedMatch<OwnedMirLocal>,
}

impl OwnedLocalMatches {
    fn add_candidate(&mut self, cand: OwnedMirLocal) {
        if !self.candidates.contains(&cand) {
            self.candidates.push(cand);
        }
    }
}

#[derive(Debug, Default)]
struct OwnedStmtMatches {
    candidates: Vec<OwnedMirLocation>,
    matched: CountedMatch<OwnedMirLocation>,
}

impl OwnedStmtMatches {
    fn add_candidate(&mut self, cand: OwnedMirLocation) {
        if !self.candidates.contains(&cand) {
            self.candidates.push(cand);
        }
    }
}

#[derive(Debug, Default)]
struct TyVarMatches<'tcx> {
    #[expect(dead_code, reason = "ty vars commit at slot assign; no outer cartesian product")]
    candidates: FxIndexSet<Ty<'tcx>>,
    #[expect(dead_code, reason = "ty vars commit at slot assign; no outer cartesian product")]
    matched: CountedMatch<Ty<'tcx>>,
}

#[derive(Debug, Default)]
struct ConstVarMatches<'tcx> {
    #[expect(dead_code, reason = "const var does not share between functions")]
    candidates: FxIndexSet<Const<'tcx>>,
    #[expect(dead_code, reason = "const var does not share between functions")]
    matched: CountedMatch<Const<'tcx>>,
}

#[derive(Debug, Default)]
struct PlaceVarMatches<'tcx> {
    #[expect(dead_code, reason = "place var does not share between functions")]
    candidates: FxIndexSet<PlaceRef<'tcx>>,
    #[expect(dead_code, reason = "place var does not share between functions")]
    matched: CountedMatch<PlaceRef<'tcx>>,
}

#[derive(Clone)]
struct AdtProbe<'tcx> {
    candidate: AdtSlotCandidate<'tcx>,
}

/// Unified session matching context for all [`RustItems`](rpl_context::pat::RustItems).
pub struct SessionMatching<'a, 'pcx, 'tcx> {
    collect: &'a MatchCollectCtxt<'a, 'pcx, 'tcx>,
    config: SessionConfig,
    rust_items: &'pcx pat::RustItems<'pcx>,
    fn_slots: &'a [FnSlotDesc<'pcx>],
    adt_slots: &'a [AdtSlotDesc<'pcx>],

    #[allow(dead_code)]
    ty_vars: IndexVec<TyVarIdx, TyVarMatches<'tcx>>,
    #[allow(dead_code)]
    const_vars: IndexVec<ConstVarIdx, ConstVarMatches<'tcx>>,
    #[allow(dead_code)]
    place_vars: IndexVec<PlaceVarIdx, PlaceVarMatches<'tcx>>,

    fn_defs: FxHashMap<MatchSlot, DefMatches>,
    adt_defs: FxHashMap<MatchSlot, DefMatches>,
    adt_probes: FxHashMap<(MatchSlot, LocalDefId), AdtProbe<'tcx>>,
    fn_items: FxHashMap<(MatchSlot, LocalDefId), CrateFnItem>,

    locals: FxHashMap<OwnedLocalPat, OwnedLocalMatches>,
    stmts: FxHashMap<OwnedLocationPat, OwnedStmtMatches>,

    fn_skipped: FxHashMap<MatchSlot, Cell<bool>>,

    results: Vec<SessionResult<'tcx>>,
    truncated: bool,
}

impl<'a, 'pcx, 'tcx> SessionMatching<'a, 'pcx, 'tcx> {
    pub fn run(
        collect: &'a MatchCollectCtxt<'a, 'pcx, 'tcx>,
        config: SessionConfig,
        index: &CrateItemIndex,
        rust_items: &'pcx pat::RustItems<'pcx>,
        fn_slots: &'a [FnSlotDesc<'pcx>],
        adt_slots: &'a [AdtSlotDesc<'pcx>],
    ) -> SessionOutcome<'tcx> {
        let meta = rust_items.meta.as_ref();
        let mut matching = Self {
            collect,
            config,
            rust_items,
            fn_slots,
            adt_slots,
            ty_vars: IndexVec::from_fn_n(|_| TyVarMatches::default(), meta.ty_vars.len()),
            const_vars: IndexVec::from_fn_n(|_| ConstVarMatches::default(), meta.const_vars.len()),
            place_vars: IndexVec::from_fn_n(|_| PlaceVarMatches::default(), meta.place_vars.len()),
            fn_defs: FxHashMap::default(),
            adt_defs: FxHashMap::default(),
            adt_probes: FxHashMap::default(),
            fn_items: FxHashMap::default(),
            locals: FxHashMap::default(),
            stmts: FxHashMap::default(),
            fn_skipped: FxHashMap::default(),
            results: Vec::new(),
            truncated: false,
        };

        for desc in fn_slots {
            matching.fn_defs.insert(desc.slot, DefMatches::default());
            matching.fn_skipped.insert(desc.slot, Cell::new(false));
        }
        for desc in adt_slots {
            matching.adt_defs.insert(desc.slot, DefMatches::default());
        }

        matching.probe(index);
        matching.match_candidates();
        if matching.truncated {
            warn!(
                pat = ?matching.collect.pat_name,
                max_results = matching.config.max_results,
                "session matching truncated at max_results; results may be incomplete"
            );
        }
        SessionOutcome {
            results: matching.results,
            truncated: matching.truncated,
        }
    }

    fn probe(&mut self, index: &CrateItemIndex) {
        for desc in self.adt_slots {
            for item in &index.adts {
                for adt_cand in self.collect.collect_adt_candidates(self.rust_items, *desc, *item) {
                    self.adt_defs
                        .get_mut(&desc.slot)
                        .expect("adt slot")
                        .add_candidate(adt_cand.def_id);
                    self.adt_probes
                        .insert((desc.slot, adt_cand.def_id), AdtProbe { candidate: adt_cand });
                }
            }
        }

        for desc in self.fn_slots {
            for &item in &index.fns {
                if !self.fn_item_in_domain(*desc, item) {
                    continue;
                }
                let mir_available = self.collect.tcx.is_mir_available(item.def_id);
                if !mir_available && !desc.fn_pat.is_signature_only() {
                    continue;
                }
                if mir_available {
                    let body = self.collect.tcx.optimized_mir(item.def_id);
                    if !desc.fn_pat.filter(self.collect.tcx, item.def_id, item.header, body) {
                        continue;
                    }
                } else if !desc
                    .fn_pat
                    .filter_signature_only(self.collect.tcx, item.def_id, item.header)
                {
                    continue;
                }
                if desc.fn_pat.extra_span(self.collect.tcx, item.def_id).is_none() {
                    continue;
                }
                self.fn_defs
                    .get_mut(&desc.slot)
                    .expect("fn slot")
                    .add_candidate(item.def_id);
                self.fn_items.insert((desc.slot, item.def_id), item);
            }
        }
    }

    fn fn_item_in_domain(&self, desc: FnSlotDesc<'pcx>, item: CrateFnItem) -> bool {
        let MatchSlot::ImplFn { fn_name, .. } = desc.slot else {
            return true;
        };
        if self.collect.tcx.impl_of_method(item.def_id.to_def_id()).is_none() {
            return false;
        }
        let name = fn_name.as_str();
        if !name.starts_with('$') && name != "_" {
            return item.fn_name == Some(fn_name);
        }
        true
    }

    fn match_candidates(&mut self) {
        let mut bindings = MetaBindings::new(self.rust_items.meta.as_ref());
        let mut used_defs = Vec::new();
        let mut assignments = Vec::new();
        let mut impl_pins = FxHashMap::default();
        // Types commit when slots assign; do not cartesian-product ty_vars first.
        self.match_adt_slots(0, &mut bindings, &mut used_defs, &mut assignments, &mut impl_pins);
    }

    fn at_max_results(&mut self) -> bool {
        if self.config.is_at_cap(self.results.len()) {
            self.truncated = true;
            true
        } else {
            false
        }
    }

    fn match_adt_slots(
        &mut self,
        slot_i: usize,
        bindings: &mut MetaBindings<'tcx>,
        used_defs: &mut Vec<LocalDefId>,
        assignments: &mut Vec<SlotAssignment<'tcx>>,
        impl_pins: &mut FxHashMap<Symbol, DefId>,
    ) {
        if self.at_max_results() {
            return;
        }
        if slot_i >= self.adt_slots.len() {
            self.match_fn_slots(0, bindings, used_defs, assignments, impl_pins);
            return;
        }

        let desc = self.adt_slots[slot_i];
        let def_cands = self.adt_defs[&desc.slot].candidates.clone();
        // Required ADT with empty domain fails the whole prefix (no skip).
        if def_cands.is_empty() {
            return;
        }

        for def_id in def_cands {
            if used_defs.contains(&def_id) {
                continue;
            }
            let probe = self.adt_probes[&(desc.slot, def_id)].clone();
            let mut trial = bindings.clone();
            if !trial.merge_adt_ty_bindings(&probe.candidate.ty_bindings) {
                continue;
            }
            if !trial.bind_adt_def(desc.adt_pat_name, def_id.to_def_id()) {
                continue;
            }
            if !self.adt_defs.get_mut(&desc.slot).unwrap().matched.r#match(def_id) {
                continue;
            }
            used_defs.push(def_id);
            assignments.push(SlotAssignment {
                slot: desc.slot,
                candidate: SlotCandidate::Adt(probe.candidate.clone()),
            });
            self.match_adt_slots(slot_i + 1, &mut trial, used_defs, assignments, impl_pins);
            assignments.pop();
            used_defs.pop();
            self.adt_defs.get_mut(&desc.slot).unwrap().matched.unmatch();
        }
    }

    fn match_fn_slots(
        &mut self,
        slot_i: usize,
        bindings: &mut MetaBindings<'tcx>,
        used_defs: &mut Vec<LocalDefId>,
        assignments: &mut Vec<SlotAssignment<'tcx>>,
        impl_pins: &mut FxHashMap<Symbol, DefId>,
    ) {
        if self.at_max_results() {
            return;
        }
        if slot_i >= self.fn_slots.len() {
            self.push_result(bindings, assignments);
            return;
        }

        let desc = self.fn_slots[slot_i];
        let def_cands = self.fn_defs[&desc.slot].candidates.clone();

        if desc.optional {
            // Optional slot may be skipped entirely (e.g. Concurrent `fn _` among required slots).
            self.fn_skipped[&desc.slot].set(true);
            self.match_fn_slots(slot_i + 1, bindings, used_defs, assignments, impl_pins);
            self.fn_skipped[&desc.slot].set(false);

            for def_id in def_cands {
                if used_defs.contains(&def_id) {
                    continue;
                }
                self.try_fn_candidate(desc, def_id, slot_i, bindings, used_defs, assignments, impl_pins);
            }
        } else {
            // Required fn with empty domain fails this prefix.
            if def_cands.is_empty() {
                return;
            }
            for def_id in def_cands {
                if used_defs.contains(&def_id) {
                    continue;
                }
                self.try_fn_candidate(desc, def_id, slot_i, bindings, used_defs, assignments, impl_pins);
            }
        }
    }

    #[allow(clippy::too_many_arguments)] // FIXME: bundle the search state into a struct.
    fn try_fn_candidate(
        &mut self,
        desc: FnSlotDesc<'pcx>,
        def_id: LocalDefId,
        slot_i: usize,
        bindings: &mut MetaBindings<'tcx>,
        used_defs: &mut Vec<LocalDefId>,
        assignments: &mut Vec<SlotAssignment<'tcx>>,
        impl_pins: &mut FxHashMap<Symbol, DefId>,
    ) {
        if let MatchSlot::ImplFn { impl_name, .. } = desc.slot {
            let Some(impl_did) = self.collect.tcx.impl_of_method(def_id.to_def_id()) else {
                return;
            };
            if let Some(&pinned) = impl_pins.get(&impl_name)
                && pinned != impl_did
            {
                return;
            }
        }
        let item = self.fn_items.get(&(desc.slot, def_id)).copied().unwrap_or(CrateFnItem {
            def_id,
            header: None,
            has_self: false,
            fn_name: None,
        });
        let env = bindings.clone();
        let collect = self.collect;
        collect.match_fn_slot(self.rust_items, &env, desc.fn_pat, item, |cand| {
            self.commit_fn_candidate(desc, def_id, slot_i, bindings, used_defs, assignments, impl_pins, cand);
        });
    }

    #[allow(clippy::too_many_arguments)] // FIXME: bundle the search state into a struct.
    fn commit_fn_candidate(
        &mut self,
        desc: FnSlotDesc<'pcx>,
        def_id: LocalDefId,
        slot_i: usize,
        bindings: &mut MetaBindings<'tcx>,
        used_defs: &mut Vec<LocalDefId>,
        assignments: &mut Vec<SlotAssignment<'tcx>>,
        impl_pins: &mut FxHashMap<Symbol, DefId>,
        cand: FnSlotCandidate<'tcx>,
    ) {
        let mut trial = bindings.clone();
        if !trial.merge_snapshot(&cand.snapshot) {
            return;
        }
        self.ingest_fn_matched(desc.slot, &cand);

        if !self.fn_defs.get_mut(&desc.slot).unwrap().matched.r#match(def_id) {
            self.unenest_fn_matched(desc.slot, &cand);
            return;
        }

        if !self.locals_consistent_with_def(desc.slot, def_id) {
            self.fn_defs.get_mut(&desc.slot).unwrap().matched.unmatch();
            self.unenest_fn_matched(desc.slot, &cand);
            return;
        }

        let mut new_impl_pin = None;
        if let MatchSlot::ImplFn { impl_name, .. } = desc.slot {
            let Some(impl_did) = self.collect.tcx.impl_of_method(def_id.to_def_id()) else {
                self.fn_defs.get_mut(&desc.slot).unwrap().matched.unmatch();
                self.unenest_fn_matched(desc.slot, &cand);
                return;
            };
            if let std::collections::hash_map::Entry::Vacant(entry) = impl_pins.entry(impl_name) {
                entry.insert(impl_did);
                new_impl_pin = Some(impl_name);
            }
            if !self.collect.check_impl_constraints(
                self.rust_items,
                impl_name,
                def_id,
                desc.fn_pat,
                &cand.matched,
                &trial,
            ) {
                if let Some(name) = new_impl_pin {
                    impl_pins.remove(&name);
                }
                self.fn_defs.get_mut(&desc.slot).unwrap().matched.unmatch();
                self.unenest_fn_matched(desc.slot, &cand);
                return;
            }
        }

        used_defs.push(def_id);
        assignments.push(SlotAssignment {
            slot: desc.slot,
            candidate: SlotCandidate::Fn(cand.clone()),
        });

        self.match_fn_slots(slot_i + 1, &mut trial, used_defs, assignments, impl_pins);

        assignments.pop();
        used_defs.pop();
        if let Some(name) = new_impl_pin {
            impl_pins.remove(&name);
        }
        self.fn_defs.get_mut(&desc.slot).unwrap().matched.unmatch();
        self.unenest_fn_matched(desc.slot, &cand);
    }

    fn locals_consistent_with_def(&self, slot: MatchSlot, def_id: LocalDefId) -> bool {
        self.locals.iter().all(|(&(s, _), m)| {
            if s != slot {
                return true;
            }
            match m.matched.get() {
                Some((d, _)) => d == def_id,
                None => true,
            }
        })
    }

    fn ingest_fn_matched(&mut self, slot: MatchSlot, cand: &FnSlotCandidate<'tcx>) {
        for (local_pat, &local) in cand.matched.locals.iter_enumerated() {
            let entry = self.locals.entry((slot, local_pat)).or_default();
            entry.add_candidate((cand.def_id, local));
            let _ = entry.matched.r#match((cand.def_id, local));
        }
        for (bb_pat, block) in cand.matched.basic_blocks.iter_enumerated() {
            for (stmt_idx, stmt_match) in block.statements.iter().enumerate() {
                let loc_pat = pat::Location {
                    block: bb_pat,
                    statement_index: stmt_idx,
                };
                let Some(mir_loc) = stmt_match.location() else {
                    continue;
                };
                let entry = self.stmts.entry((slot, loc_pat)).or_default();
                entry.add_candidate((cand.def_id, mir_loc));
                let _ = entry.matched.r#match((cand.def_id, mir_loc));
            }
        }
    }

    fn unenest_fn_matched(&mut self, slot: MatchSlot, cand: &FnSlotCandidate<'tcx>) {
        for (local_pat, _) in cand.matched.locals.iter_enumerated() {
            if let Some(m) = self.locals.get(&(slot, local_pat))
                && m.matched.get().is_some()
            {
                m.matched.unmatch();
            }
        }
        for (bb_pat, block) in cand.matched.basic_blocks.iter_enumerated() {
            for (stmt_idx, _) in block.statements.iter().enumerate() {
                let loc_pat = pat::Location {
                    block: bb_pat,
                    statement_index: stmt_idx,
                };
                if let Some(m) = self.stmts.get(&(slot, loc_pat))
                    && m.matched.get().is_some()
                {
                    m.matched.unmatch();
                }
            }
        }
    }

    fn push_result(&mut self, bindings: &MetaBindings<'tcx>, assignments: &[SlotAssignment<'tcx>]) {
        let has_fn = assignments.iter().any(|a| matches!(a.candidate, SlotCandidate::Fn(_)));
        let has_adt = assignments.iter().any(|a| matches!(a.candidate, SlotCandidate::Adt(_)));
        if !has_fn && !has_adt {
            return;
        }
        let requires_fn = self.fn_slots.iter().any(|s| !s.optional);
        if requires_fn && !has_fn {
            return;
        }

        let primary_fn = assignments
            .iter()
            .find_map(|a| match &a.candidate {
                SlotCandidate::Fn(c) if !c.matched.basic_blocks.is_empty() => Some(FnMatchContext {
                    def_id: c.def_id,
                    fn_name: None,
                    header: None,
                    has_self: false,
                    self_ty: None,
                }),
                SlotCandidate::Fn(_) | SlotCandidate::Adt(_) => None,
            })
            .or_else(|| {
                assignments.iter().find_map(|a| match &a.candidate {
                    SlotCandidate::Fn(c) => Some(FnMatchContext {
                        def_id: c.def_id,
                        fn_name: None,
                        header: None,
                        has_self: false,
                        self_ty: None,
                    }),
                    SlotCandidate::Adt(_) => None,
                })
            });

        self.results.push(SessionResult {
            assignments: assignments.to_vec(),
            bindings: bindings.clone(),
            primary_fn,
        });
    }
}

impl StatementMatch {
    fn location(self) -> Option<mir::Location> {
        match self {
            StatementMatch::Location(loc) => Some(loc),
            StatementMatch::Arg(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_keys_distinguish_slots() {
        let a = (MatchSlot::Fn(0), pat::Local::from_u32(1));
        let b = (MatchSlot::Fn(1), pat::Local::from_u32(1));
        assert_ne!(a, b);
    }

    #[test]
    fn counted_match_conflict() {
        let m = CountedMatch::<u32>::new();
        assert!(m.r#match(1));
        assert!(!m.r#match(2));
        assert!(m.r#match(1));
        m.unmatch();
        m.unmatch();
        assert!(m.get().is_none());
    }
}
