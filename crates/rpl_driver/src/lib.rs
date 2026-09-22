#![feature(rustc_private)]
#![warn(unused_qualifications)]
extern crate either;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_fluent_macro;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_lint_defs;
extern crate rustc_macros;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate tracing;

rustc_fluent_macro::fluent_messages! { "../messages.en.ftl" }

use std::borrow::Cow;
use std::cell::RefCell;
use std::convert::identity;

use rpl_config::RawOpInstance;
use rpl_constraints::predicates::BodyInfoCache;
use rpl_context::PatCtxt;
use rpl_context::pat::ops_resolved::{ResolvedOpBindings, resolve_ops_config};
use rpl_context::pat::{DynamicError, PatternItem};
use rpl_match::matches::artifact::{NormalizedMatched, NormalizedSpanned};
use rpl_match::session::{MatchCollectCtxt, MatchSession, SessionConfig};
use rpl_match::{CrateItemIndex, MatchSlot, MultiMatched, OwnedLintMatch};
use rpl_meta::context::MetaContext;
use rustc_data_structures::fx::FxHashMap;
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{self as hir, FnDecl};
use rustc_lint_defs::RegisteredTools;
use rustc_macros::{Diagnostic, LintDiagnostic};
use rustc_middle::hir::nested_filter;
use rustc_middle::ty::TyCtxt;
use rustc_middle::util::Providers;
use rustc_session::declare_tool_lint;
use rustc_span::symbol::Ident;
use rustc_span::{Span, Symbol};

/// Generate the cartesian product of a sequence of factors.
///
/// - Empty input (no factors): yields a single empty combination.
/// - Any factor that is empty: yields zero combinations (fold-on-empty semantics).
/// - Otherwise: yields every combination of one element from each factor, with the rightmost index
///   varying fastest (odometer order).
///
/// The "fold on empty instances" property falls out of the helper: an empty
/// factor folds the product to zero combinations, which contributes the empty
/// match-set to set-op composition (Task 11 spec).
pub fn cartesian<T: Clone, I, II>(factors: II) -> CartesianIter<T>
where
    I: Iterator<Item = T>,
    II: IntoIterator<Item = I>,
{
    let factors: Vec<Vec<T>> = factors.into_iter().map(|i| i.collect()).collect();

    if factors.is_empty() {
        // No factors → yield one empty combination.
        return CartesianIter {
            factors: Vec::new(),
            indices: Vec::new(),
            done: false,
            empty_fold: false,
        };
    }
    if factors.iter().any(|f| f.is_empty()) {
        // Any empty factor → fold to zero combos.
        return CartesianIter {
            factors: Vec::new(),
            indices: Vec::new(),
            done: true,
            empty_fold: true,
        };
    }

    let n = factors.len();
    CartesianIter {
        factors,
        indices: vec![0usize; n],
        done: false,
        empty_fold: false,
    }
}

/// Iterator returned by [`cartesian`].
pub struct CartesianIter<T> {
    factors: Vec<Vec<T>>,
    indices: Vec<usize>,
    done: bool,
    /// `true` when an empty factor forced the product to zero — done before any yield.
    empty_fold: bool,
}

impl<T: Clone> Iterator for CartesianIter<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Vec<T>> {
        if self.done || self.empty_fold {
            return None;
        }
        if self.factors.is_empty() {
            // No-factors case: one empty combo, then done.
            self.done = true;
            return Some(Vec::new());
        }

        let combo: Vec<T> = self
            .factors
            .iter()
            .zip(&self.indices)
            .map(|(f, &i)| f[i].clone())
            .collect();

        // Increment odometer (rightmost index varies fastest).
        let mut k = self.factors.len();
        loop {
            if k == 0 {
                self.done = true;
                break;
            }
            k -= 1;
            self.indices[k] += 1;
            if self.indices[k] < self.factors[k].len() {
                break;
            }
            self.indices[k] = 0;
        }
        Some(combo)
    }
}

#[cfg(feature = "timing")]
mod errors;

#[cfg(feature = "timing")]
pub use errors::{TIMING, Timing};

declare_tool_lint! {
    pub rpl::ERROR_FOUND,
    Deny,
    "detects an error"
}

#[derive(Diagnostic, LintDiagnostic)]
#[diag(rpl_driver_error_found_with_pattern)]
pub struct ErrorFound;

impl From<ErrorFound> for rustc_errors::DiagMessage {
    fn from(_: ErrorFound) -> Self {
        Self::Str(Cow::Borrowed("An error was found with input RPL pattern(s)"))
    }
}

pub fn provide(providers: &mut Providers) {
    providers.registered_tools = registered_tools;
}

fn registered_tools(tcx: TyCtxt<'_>, (): ()) -> RegisteredTools {
    let mut registered_tools = (rustc_interface::DEFAULT_QUERY_PROVIDERS.registered_tools)(tcx, ());
    registered_tools.insert(Ident::from_str("rpl"));
    registered_tools
}

pub fn check_crate<'tcx, 'pcx, 'mcx: 'pcx>(tcx: TyCtxt<'tcx>, pcx: PatCtxt<'pcx>, mctx: &'mcx MetaContext<'mcx>) {
    #[cfg(feature = "timing")]
    let start = std::time::Instant::now();

    pcx.add_parsed_patterns(mctx);

    #[cfg(feature = "timing")]
    timing_lint(tcx, start, "add_parsed_patterns");

    #[cfg(feature = "timing")]
    let start = std::time::Instant::now();

    // Missing rpl.toml or [ops] is an empty configuration; malformed config
    // remains non-fatal, as with the original abstract-ops driver.
    let raw_ops = match rpl_config::load_raw_ops(None) {
        Ok(map) => map.into_iter().collect(),
        Err(err) => {
            tcx.dcx().warn(format!("rpl: failed to load rpl.toml ops: {err}"));
            Vec::new()
        },
    };

    let index = CrateItemIndex::build(tcx);
    let mut check_ctxt = CheckFnCtxt {
        tcx,
        pcx,
        body_caches: RefCell::default(),
        fn_candidate_cache: RefCell::default(),
        index,
        raw_ops,
    };

    check_ctxt.match_all_patterns();

    tcx.hir().walk_toplevel_module(&mut check_ctxt);
    rpl_utils::visit_crate(tcx);

    #[cfg(feature = "timing")]
    timing_lint(tcx, start, "do_match");
}

#[cfg(feature = "timing")]
fn timing_lint(tcx: TyCtxt<'_>, start: std::time::Instant, stage: &'static str) {
    use rustc_hir::def_id::CrateNum;

    use crate::errors::TIMING;

    let time = start.elapsed().as_nanos().try_into().unwrap();
    let hir_id = rustc_hir::hir_id::CRATE_HIR_ID;
    let crate_name = tcx.crate_name(CrateNum::ZERO);
    tcx.emit_node_span_lint(
        TIMING,
        hir_id,
        tcx.hir().span(hir_id),
        Timing {
            time,
            stage,
            crate_name,
        },
    );
}

/// Used for finding pattern matches in given Rust crate.
struct CheckFnCtxt<'pcx, 'tcx> {
    tcx: TyCtxt<'tcx>,
    pcx: PatCtxt<'pcx>,
    body_caches: RefCell<FxHashMap<DefId, BodyInfoCache>>,
    fn_candidate_cache: RefCell<FxHashMap<(DefId, usize, usize), Vec<rpl_match::FnSlotCandidate<'tcx>>>>,
    index: CrateItemIndex,
    raw_ops: Vec<(String, Vec<RawOpInstance>)>,
}

struct PendingLint<'pcx, 'tcx> {
    pattern: &'pcx rpl_context::pat::Pattern<'pcx>,
    pat_name: Symbol,
    pat_idx: usize,
    owned: OwnedLintMatch<'tcx>,
}

impl<'tcx, 'pcx> CheckFnCtxt<'pcx, 'tcx> {
    fn match_all_patterns(&self) {
        let mut pending: Vec<PendingLint<'pcx, 'tcx>> = Vec::new();

        self.pcx.for_each_rpl_pattern(|_id, pattern| {
            let (ops, _ops_diags) = resolve_ops_config(pattern, &self.raw_ops);
            // Resolution diagnostics remain deferred, as in the abstract-ops driver.
            for (pat_idx, (&pat_name, pat_item)) in pattern.patt_block.iter().enumerate() {
                let groups_used: Vec<Symbol> = match pat_item {
                    PatternItem::RustItems(items) => items.referenced_op_groups().iter().copied().collect(),
                    PatternItem::RPLPatternOperation(op) => op.referenced_op_groups().into_iter().collect(),
                };
                let factors = groups_used.iter().map(|group| ops.instances_of(group.as_str()).iter());

                // Choose one assignment for the entire pattern item. The same
                // collector then serves every function slot and nested set-op
                // operand, so they cannot independently choose other instances.
                // No referenced groups yields one empty assignment; a group
                // without instances folds the whole pattern item to no matches.
                for combo in cartesian(factors) {
                    let bindings = ResolvedOpBindings::from_combo(&groups_used, combo);
                    let collect = MatchCollectCtxt::new(
                        self.tcx,
                        self.pcx,
                        pat_name,
                        &self.body_caches,
                        &self.fn_candidate_cache,
                    )
                    .with_op_bindings(bindings);
                    let session = MatchSession::new(collect, SessionConfig::default());
                    for result in session.match_pattern_item(&self.index, pat_item) {
                        for target in result.lint_targets() {
                            pending.push(PendingLint {
                                pattern,
                                pat_name,
                                pat_idx,
                                owned: target.owned,
                            });
                        }
                    }
                }
            }
        });

        pending.sort_by_key(|lint| (lint.owned.def_id.local_def_index, lint.pat_idx));
        pending = Self::dedupe_pending(pending);
        for lint in pending {
            let matched = lint.owned.as_matched();
            self.emit_session_lint(lint.pattern, lint.pat_name, lint.owned.def_id, &matched);
        }
    }

    fn dedupe_pending(pending: Vec<PendingLint<'pcx, 'tcx>>) -> Vec<PendingLint<'pcx, 'tcx>> {
        let mut seen: Vec<(Symbol, LocalDefId, MatchSlot, NormalizedMatched<'tcx>)> = Vec::new();
        pending
            .into_iter()
            .filter(|lint| {
                !seen.iter().any(|(pat, def, slot, normalized)| {
                    *pat == lint.pat_name
                        && *def == lint.owned.def_id
                        && *slot == lint.owned.primary_slot
                        && normalized == &lint.owned.normalized
                }) && {
                    seen.push((
                        lint.pat_name,
                        lint.owned.def_id,
                        lint.owned.primary_slot,
                        lint.owned.normalized.clone(),
                    ));
                    true
                }
            })
            .collect()
    }

    fn emit_session_lint(
        &self,
        pattern: &rpl_context::pat::Pattern<'pcx>,
        pat_name: Symbol,
        def_id: LocalDefId,
        matched: &MultiMatched<'_, 'tcx>,
    ) {
        let body = self.tcx.optimized_mir(def_id);
        let Some(decl) = fn_decl(self.tcx, def_id) else {
            return;
        };
        let fn_name = self
            .index
            .fns
            .iter()
            .find(|f| f.def_id == def_id)
            .and_then(|f| f.fn_name);
        let error = pattern
            .get_diag(pat_name, self.tcx.sess.source_map(), fn_name, body, decl, matched)
            .unwrap_or_else(identity);
        let fn_hir_id = self.tcx.local_def_id_to_hir_id(def_id);
        let primary_labels = pattern.primary_labels(pat_name).unwrap_or(&[]);
        // Prefer primary labels from last to first for nested `#[allow]` (e.g. `primary(reserve,
        // set_len)`). Skip Body/Output/Span here so a trailing location primary can still win.
        let hir_id = primary_labels
            .iter()
            .rev()
            .find_map(|name| match matched.normalized.spanned(name)? {
                NormalizedSpanned::Location(stmt_match) => {
                    let scope = stmt_match.source_info(body).scope;
                    scope.lint_root(&body.source_scopes)
                },
                NormalizedSpanned::Local(local) => {
                    let scope = body.local_decls[local].source_info.scope;
                    scope.lint_root(&body.source_scopes)
                },
                NormalizedSpanned::Body | NormalizedSpanned::Output | NormalizedSpanned::Span(_) => None,
            })
            .unwrap_or(fn_hir_id);
        self.tcx
            .emit_node_span_lint(error.lint(), hir_id, error.primary_span().clone(), error);
    }
}

fn fn_decl(tcx: TyCtxt<'_>, def_id: LocalDefId) -> Option<&FnDecl<'_>> {
    tcx.hir().fn_decl_by_hir_id(tcx.local_def_id_to_hir_id(def_id))
}

impl<'tcx> Visitor<'tcx> for CheckFnCtxt<'_, 'tcx> {
    type NestedFilter = nested_filter::All;
    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) -> Self::Result {
        match item.kind {
            hir::ItemKind::Trait(_, _, _, _, impl_) => {
                for trait_item in impl_ {
                    self.visit_trait_item_ref(trait_item);
                }
            },
            hir::ItemKind::Impl(impl_) => {
                for impl_item in impl_.items {
                    self.visit_impl_item_ref(impl_item);
                }
            },
            _ => {},
        }
        intravisit::walk_item(self, item);
    }

    fn visit_fn(
        &mut self,
        kind: intravisit::FnKind<'tcx>,
        _decl: &'tcx FnDecl<'tcx>,
        body_id: hir::BodyId,
        _span: Span,
        def_id: LocalDefId,
    ) -> Self::Result {
        let attrs: Vec<_> = self
            .tcx
            .get_attrs_by_path(def_id.to_def_id(), &[Symbol::intern("rpl"), Symbol::intern("dynamic")])
            .collect();
        for attr in &attrs {
            let error = DynamicError::from_attr(attr, self.tcx.def_span(def_id.to_def_id()));
            self.tcx.emit_node_span_lint(
                error.lint(),
                self.tcx.local_def_id_to_hir_id(def_id),
                error.primary_span().clone(),
                error,
            );
        }

        intravisit::walk_fn(self, kind, _decl, body_id, def_id);
    }
}

impl<'tcx> CheckFnCtxt<'_, 'tcx> {
    fn visit_trait_item_ref(&mut self, trait_item: &'tcx hir::TraitItemRef) {
        let _ = trait_item;
    }

    fn visit_impl_item_ref(&mut self, impl_item: &'tcx hir::ImplItemRef) {
        let _ = impl_item;
    }
}
