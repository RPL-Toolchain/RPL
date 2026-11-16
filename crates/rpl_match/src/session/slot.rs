use rpl_context::pat::{self, FnPattern};
use rustc_hir::FnHeader;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::LocalDefId;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Symbol;

use crate::AdtMatch;
use crate::matches::artifact::NormalizedMatched;
use crate::session::bindings::BindingSnapshot;

/// Identifies a pattern slot within a [`MatchSession`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MatchSlot {
    /// Function pattern at index in `RustItems.fns.all_fns`.
    Fn(usize),
    /// Struct/enum pattern keyed by metavar name in `RustItems.adts`.
    Adt(Symbol),
    /// Associated function inside an impl pattern.
    ImplFn { impl_name: Symbol, fn_name: Symbol },
}

impl MatchSlot {
    pub fn fn_index(self) -> Option<usize> {
        match self {
            MatchSlot::Fn(idx) => Some(idx),
            _ => None,
        }
    }
}

/// A single function-slot match candidate.
#[derive(Debug, Clone)]
pub struct FnSlotCandidate<'tcx> {
    pub def_id: LocalDefId,
    pub normalized: NormalizedMatched<'tcx>,
    pub matched: crate::matches::Matched<'tcx>,
    pub snapshot: BindingSnapshot<'tcx>,
}

/// A single ADT-slot match candidate.
#[derive(Debug, Clone)]
pub struct AdtSlotCandidate<'tcx> {
    pub def_id: LocalDefId,
    pub adt_match: AdtMatch<'tcx>,
    pub ty_bindings: rustc_index::IndexVec<pat::TyVarIdx, ty::Ty<'tcx>>,
}

/// Union of slot-specific candidates.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)] // FIXME: box the `Fn` variant if this shows up in profiles.
pub enum SlotCandidate<'tcx> {
    Fn(FnSlotCandidate<'tcx>),
    Adt(AdtSlotCandidate<'tcx>),
}

/// Context needed to evaluate constraints and diagnostics for a function assignment.
#[derive(Debug, Clone, Copy)]
pub struct FnMatchContext<'tcx> {
    pub def_id: LocalDefId,
    pub fn_name: Option<Symbol>,
    pub header: Option<FnHeader>,
    pub has_self: bool,
    pub self_ty: Option<ty::Ty<'tcx>>,
}

/// One complete assignment of pattern slots to Rust items.
#[derive(Debug, Clone)]
pub struct SlotAssignment<'tcx> {
    pub slot: MatchSlot,
    pub candidate: SlotCandidate<'tcx>,
}

/// Result of a successful match session.
#[derive(Debug, Clone)]
pub struct SessionResult<'tcx> {
    pub assignments: Vec<SlotAssignment<'tcx>>,
    pub bindings: super::bindings::MetaBindings<'tcx>,
    /// Primary function context for single-slot diagnostics (first fn assignment).
    pub primary_fn: Option<FnMatchContext<'tcx>>,
}

/// Session match output, including whether [`super::config::SessionConfig::max_results`] stopped
/// the search.
#[derive(Debug, Clone)]
pub struct SessionOutcome<'tcx> {
    pub results: Vec<SessionResult<'tcx>>,
    pub truncated: bool,
}

impl SessionOutcome<'_> {
    pub fn empty() -> Self {
        Self {
            results: Vec::new(),
            truncated: false,
        }
    }
}

impl<'tcx> SessionResult<'tcx> {
    pub fn fn_assignment(&self, slot: MatchSlot) -> Option<&FnSlotCandidate<'tcx>> {
        self.assignments.iter().find_map(|a| {
            if a.slot == slot {
                match &a.candidate {
                    SlotCandidate::Fn(c) => Some(c),
                    SlotCandidate::Adt(_) => None,
                }
            } else {
                None
            }
        })
    }

    pub fn normalized_for_lint(&self) -> Option<&NormalizedMatched<'tcx>> {
        self.primary_fn_candidate().map(|c| &c.normalized)
    }

    pub fn primary_fn_candidate(&self) -> Option<&FnSlotCandidate<'tcx>> {
        // Prefer a MIR-body match (non-empty basic_blocks) so signature-only slots like
        // `fn $callback(...);` are not treated as the lint primary.
        self.assignments
            .iter()
            .find_map(|a| match &a.candidate {
                SlotCandidate::Fn(c) if !c.matched.basic_blocks.is_empty() => Some(c),
                SlotCandidate::Fn(_) | SlotCandidate::Adt(_) => None,
            })
            .or_else(|| {
                self.assignments.iter().find_map(|a| match &a.candidate {
                    SlotCandidate::Fn(c) => Some(c),
                    SlotCandidate::Adt(_) => None,
                })
            })
    }

    pub fn primary_fn_slot(&self) -> Option<MatchSlot> {
        self.assignments
            .iter()
            .find_map(|a| match &a.candidate {
                SlotCandidate::Fn(c) if !c.matched.basic_blocks.is_empty() => Some(a.slot),
                SlotCandidate::Fn(_) | SlotCandidate::Adt(_) => None,
            })
            .or_else(|| {
                self.assignments.iter().find_map(|a| match &a.candidate {
                    SlotCandidate::Fn(_) => Some(a.slot),
                    SlotCandidate::Adt(_) => None,
                })
            })
    }

    /// Whether this result can participate in
    /// [`PatternOperation`](rpl_context::pat::PatternOperation) subtraction (`p - q`). Results
    /// with no function slot are never filtered.
    pub fn has_operation_key(&self) -> bool {
        self.primary_fn_candidate().is_some()
    }

    /// Negative filter for `p - q`: mapped SharedEnv, alignable slot DefIds, and
    /// [`NormalizedMatched`] on Fn slots present on both sides.
    ///
    /// Slots present on only one side (e.g. a single-fn negative vs a multi-fn positive)
    /// are ignored for alignment, matching the multi-instance contract.
    pub fn subtracted_by(&self, neg: &Self) -> bool {
        let Some(pos_primary) = self.primary_fn_candidate() else {
            return false;
        };
        let Some(neg_primary) = neg.primary_fn_candidate() else {
            return false;
        };
        if pos_primary.def_id != neg_primary.def_id {
            return false;
        }
        if !self.bindings.equivalent_to(&neg.bindings) {
            return false;
        }
        for a in &self.assignments {
            let Some(neg_a) = neg.assignments.iter().find(|b| b.slot == a.slot) else {
                continue;
            };
            if assignment_def_id(a) != assignment_def_id(neg_a) {
                return false;
            }
            // Different MIR subgraphs must not subtract each other. Compare only the columns
            // shared between positive and negative, so a fragment-local label (e.g. `'reset` in
            // `without!('reset: ...)`) does not block the subtraction.
            if let (SlotCandidate::Fn(pos_fn), SlotCandidate::Fn(neg_fn)) = (&a.candidate, &neg_a.candidate)
                && !pos_fn.normalized.same_common_columns(&neg_fn.normalized)
            {
                return false;
            }
        }
        true
    }
}

fn assignment_def_id(a: &SlotAssignment<'_>) -> LocalDefId {
    match &a.candidate {
        SlotCandidate::Fn(c) => c.def_id,
        SlotCandidate::Adt(c) => c.def_id,
    }
}

/// Describes a function pattern slot for CSP solving.
#[derive(Debug, Clone, Copy)]
pub struct FnSlotDesc<'pcx> {
    pub slot: MatchSlot,
    pub fn_pat: &'pcx FnPattern<'pcx>,
    /// When true (`fn _`), each session result matches at most one def.
    pub optional: bool,
}

/// Describes an ADT pattern slot for CSP solving.
#[derive(Debug, Clone, Copy)]
pub struct AdtSlotDesc<'pcx> {
    pub slot: MatchSlot,
    pub adt_pat: &'pcx pat::Adt<'pcx>,
    pub adt_pat_name: Symbol,
}

/// Collect slot descriptors from a [`RustItems`] block.
pub fn collect_slot_descs<'pcx>(
    rust_items: &'pcx pat::RustItems<'pcx>,
) -> (Vec<FnSlotDesc<'pcx>>, Vec<AdtSlotDesc<'pcx>>) {
    let mut fn_slots: Vec<FnSlotDesc<'pcx>> = rust_items
        .fns
        .all_fns
        .iter()
        .enumerate()
        .map(|(idx, fn_pat)| FnSlotDesc {
            slot: MatchSlot::Fn(idx),
            fn_pat,
            optional: fn_pat.name.as_str() == "_",
        })
        .collect();

    for (&impl_name, impl_pat) in &rust_items.impls {
        for (&fn_name, fn_pat) in &impl_pat.fns {
            fn_slots.push(FnSlotDesc {
                slot: MatchSlot::ImplFn { impl_name, fn_name },
                fn_pat,
                optional: fn_pat.name.as_str() == "_",
            });
        }
    }

    let adt_slots = rust_items
        .adts
        .iter()
        .map(|(&name, adt_pat)| AdtSlotDesc {
            slot: MatchSlot::Adt(name),
            adt_pat,
            adt_pat_name: name,
        })
        .collect();

    (fn_slots, adt_slots)
}

/// Indexed Rust items in the crate used for full M:N matching.
#[derive(Default)]
pub struct CrateItemIndex {
    pub fns: Vec<CrateFnItem>,
    pub adts: Vec<CrateAdtItem>,
}

#[derive(Debug, Clone, Copy)]
pub struct CrateFnItem {
    pub def_id: LocalDefId,
    pub header: Option<FnHeader>,
    pub has_self: bool,
    pub fn_name: Option<Symbol>,
}

#[derive(Debug, Clone, Copy)]
pub struct CrateAdtItem {
    pub def_id: LocalDefId,
}

impl CrateItemIndex {
    pub fn build(tcx: TyCtxt<'_>) -> Self {
        let mut index = Self::default();
        let hir = tcx.hir();

        for item_id in hir.items() {
            let item = hir.item(item_id);
            let def_id = item.owner_id.def_id;
            if matches!(
                item.kind,
                rustc_hir::ItemKind::Struct { .. } | rustc_hir::ItemKind::Enum { .. }
            ) {
                index.adts.push(CrateAdtItem { def_id });
            }
        }

        struct FnCollector<'a, 'tcx> {
            tcx: TyCtxt<'tcx>,
            index: &'a mut CrateItemIndex,
        }

        impl<'tcx> rustc_hir::intravisit::Visitor<'tcx> for FnCollector<'_, 'tcx> {
            type NestedFilter = rustc_middle::hir::nested_filter::All;
            fn nested_visit_map(&mut self) -> Self::Map {
                self.tcx.hir()
            }

            fn visit_fn(
                &mut self,
                kind: rustc_hir::intravisit::FnKind<'tcx>,
                decl: &'tcx rustc_hir::FnDecl<'tcx>,
                _body_id: rustc_hir::BodyId,
                _span: rustc_span::Span,
                def_id: LocalDefId,
            ) -> Self::Result {
                // Index even when MIR is unavailable (e.g. some foreign/ABI cases) so
                // signature-only slots can still match. MIR matching skips these later.
                let (fn_name, header) = match kind {
                    rustc_hir::intravisit::FnKind::ItemFn(name, _, fn_header) => (Some(name.name), Some(fn_header)),
                    rustc_hir::intravisit::FnKind::Method(name, fn_sig) => (Some(name.name), Some(fn_sig.header)),
                    rustc_hir::intravisit::FnKind::Closure => (None, None),
                };
                self.index.fns.push(CrateFnItem {
                    def_id,
                    header,
                    has_self: decl.implicit_self.has_implicit_self(),
                    fn_name,
                });
                rustc_hir::intravisit::walk_fn(self, kind, decl, _body_id, def_id)
            }

            fn visit_foreign_item(&mut self, item: &'tcx rustc_hir::ForeignItem<'tcx>) -> Self::Result {
                if let rustc_hir::ForeignItemKind::Fn(sig, ..) = item.kind {
                    let def_id = item.owner_id.def_id;
                    self.index.fns.push(CrateFnItem {
                        def_id,
                        header: None,
                        has_self: sig.decl.implicit_self.has_implicit_self(),
                        fn_name: Some(item.ident.name),
                    });
                }
                rustc_hir::intravisit::walk_foreign_item(self, item)
            }
        }

        hir.walk_toplevel_module(&mut FnCollector { tcx, index: &mut index });

        let mut seen: rustc_data_structures::fx::FxHashSet<LocalDefId> =
            index.fns.iter().map(|item| item.def_id).collect();
        for def_id in tcx.hir_crate_items(()).nested_bodies() {
            if !seen.insert(def_id) {
                continue;
            }
            if !matches!(tcx.def_kind(def_id), DefKind::Closure) || !tcx.is_mir_available(def_id) {
                continue;
            }
            index.fns.push(CrateFnItem {
                def_id,
                fn_name: None,
                header: None,
                has_self: false,
            });
        }

        index
    }

    pub fn self_ty<'tcx>(&self, tcx: TyCtxt<'tcx>, def_id: LocalDefId) -> Option<ty::Ty<'tcx>> {
        tcx.impl_of_method(def_id.into())
            .map(|impl_| tcx.type_of(impl_).instantiate_identity())
    }
}
