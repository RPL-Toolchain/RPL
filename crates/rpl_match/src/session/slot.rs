use rpl_context::pat::visitor::PatternVisitor;
use rpl_context::pat::{self, FnPattern};
use rustc_data_structures::fx::FxHashSet;
use rustc_hir::FnHeader;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::LocalDefId;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Symbol;

use crate::AdtMatch;
use crate::matches::artifact::NormalizedMatched;
use crate::session::bindings::BindingSnapshot;

/// Identifies a pattern slot within a [`MatchSession`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// Key for [`PatternOperation`](rpl_context::pat::PatternOperation) negative filtering:
    /// compare matches within the same function using full [`NormalizedMatched`] equality.
    pub fn operation_match_key(&self) -> Option<(LocalDefId, &NormalizedMatched<'tcx>)> {
        self.primary_fn_candidate().map(|c| (c.def_id, &c.normalized))
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
    let referenced_fn_patterns = referenced_fn_patterns(rust_items);
    let mut fn_slots: Vec<FnSlotDesc<'pcx>> = rust_items
        .fns
        .all_fns
        .iter()
        .enumerate()
        .filter(|(_, fn_pat)| !(fn_pat.is_signature_only() && referenced_fn_patterns.contains(&fn_pat.name)))
        .map(|(idx, fn_pat)| FnSlotDesc {
            slot: MatchSlot::Fn(idx),
            fn_pat,
            optional: fn_pat.name.as_str() == "_",
        })
        .collect();

    let mut next_idx = rust_items.fns.all_fns.len();
    for impl_pat in rust_items.impls.values() {
        for fn_pat in impl_pat.fns.values() {
            if fn_pat.is_signature_only() && referenced_fn_patterns.contains(&fn_pat.name) {
                continue;
            }
            fn_slots.push(FnSlotDesc {
                slot: MatchSlot::Fn(next_idx),
                fn_pat,
                optional: fn_pat.name.as_str() == "_",
            });
            next_idx += 1;
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

fn referenced_fn_patterns<'pcx>(rust_items: &'pcx pat::RustItems<'pcx>) -> FxHashSet<Symbol> {
    #[derive(Default)]
    struct Collector {
        names: FxHashSet<Symbol>,
    }

    impl PatternVisitor<'_> for Collector {
        fn visit_fn_pat(&mut self, fn_pat: Symbol) {
            self.names.insert(fn_pat);
        }
    }

    fn visit_fn<'pcx>(collector: &mut Collector, fn_pat: &'pcx FnPattern<'pcx>) {
        let Some(body) = fn_pat.body else {
            return;
        };
        for (block, data) in body.basic_blocks.iter_enumerated() {
            collector.visit_basic_block_data(block, data);
        }
    }

    let mut collector = Collector::default();
    for fn_pat in &rust_items.fns.all_fns {
        visit_fn(&mut collector, fn_pat);
    }
    for impl_pat in rust_items.impls.values() {
        for fn_pat in impl_pat.fns.values() {
            visit_fn(&mut collector, fn_pat);
        }
    }
    collector.names
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
                if !self.tcx.is_mir_available(def_id) {
                    return rustc_hir::intravisit::walk_fn(self, kind, decl, _body_id, def_id);
                }
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
        }

        hir.walk_toplevel_module(&mut FnCollector { tcx, index: &mut index });

        let mut seen: FxHashSet<LocalDefId> = index.fns.iter().map(|item| item.def_id).collect();
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
