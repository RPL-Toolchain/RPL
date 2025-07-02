#![feature(rustc_private)]
#![warn(unused_qualifications)]
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
#[macro_use]
extern crate tracing;
extern crate either;

rustc_fluent_macro::fluent_messages! { "../messages.en.ftl" }

use std::borrow::Cow;
use std::convert::identity;

use either::Either;
use rpl_context::PatCtxt;
use rpl_context::pat::DynamicError;
use rpl_match::graph::{MirControlFlowGraph, MirDataDepGraph};
use rpl_match::matches::Matched;
use rpl_match::matches::artifact::NormalizedMatched;
use rpl_match::mir::pat::PatternItem;
use rpl_match::mir::{CheckMirCtxt, pat};
use rpl_match::predicate_evaluator::PredicateEvaluator;
use rpl_meta::context::MetaContext;
use rustc_data_structures::fx::FxHashSet;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{self as hir, FnHeader};
use rustc_lint_defs::RegisteredTools;
use rustc_macros::{Diagnostic, LintDiagnostic};
use rustc_middle::hir::nested_filter;
use rustc_middle::mir;
use rustc_middle::ty::{self, TyCtxt};
use rustc_middle::util::Providers;
use rustc_session::declare_tool_lint;
use rustc_span::symbol::Ident;
use rustc_span::{Span, Symbol};

declare_tool_lint! {
    /// The `rpl::error_found` lint detects an error.
    ///
    /// ### Example
    ///
    /// ```rust
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// This lint detects an error.
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
    pcx.add_parsed_patterns(mctx);
    // _ = tcx.hir_crate_items(()).par_items(|item_id| {
    //     check_item(tcx, pcx, item_id);
    //     Ok(())
    // });
    let mut check_ctxt = CheckFnCtxt { tcx, pcx };
    tcx.hir().walk_toplevel_module(&mut check_ctxt);
    rpl_utils::visit_crate(tcx);
}

// pub fn check_item(tcx: TyCtxt<'_>, pcx: PatCtxt<'_>, item_id: hir::ItemId) {
//     let item = tcx.hir().item(item_id);
//     // let def_id = item_id.owner_id.def_id;
//     let mut check_ctxt = CheckFnCtxt { tcx, pcx };
//     check_ctxt.visit_item(item);
// }

/// Used for finding pattern matches in given Rust crate.
struct CheckFnCtxt<'pcx, 'tcx> {
    tcx: TyCtxt<'tcx>,
    pcx: PatCtxt<'pcx>,
}

impl<'tcx> Visitor<'tcx> for CheckFnCtxt<'_, 'tcx> {
    type NestedFilter = nested_filter::All;
    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    #[instrument(level = "debug", skip_all, fields(item_id = ?item.owner_id.def_id))]
    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) -> Self::Result {
        match item.kind {
            // hir::ItemKind::Trait(hir::IsAuto::No, hir::Safety::Safe, ..) | hir::ItemKind::Fn { .. } => {},
            hir::ItemKind::Impl(impl_) => self.check_impl(impl_),
            // hir::ItemKind::Fn { sig, .. } => self.check_fn(
            //     Some(item.ident),
            //     &sig.decl,
            //     Some(sig.header),
            //     sig.decl.implicit_self.has_implicit_self(),
            //     item.owner_id.def_id,
            // ),
            // hir::ItemKind::Struct(struct_, generics) => self.check_struct(item.owner_id.def_id, struct_, generics),
            // hir::ItemKind::Enum(enum_, generics) => self.check_enum(item.owner_id.def_id, enum_, generics),
            _ => {},
        }
        intravisit::walk_item(self, item);
    }

    #[instrument(level = "debug", skip(self, kind, decl, body_id, _span))]
    fn visit_fn(
        &mut self,
        kind: intravisit::FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body_id: hir::BodyId,
        _span: Span,
        def_id: LocalDefId,
    ) -> Self::Result {
        let (name, header) = match kind {
            intravisit::FnKind::ItemFn(name, _, fn_header) => (Some(name), Some(fn_header)),
            intravisit::FnKind::Method(name, fn_sig) => (Some(name), Some(fn_sig.header)),
            intravisit::FnKind::Closure => (None, None),
        };
        self.check_fn(name, decl, header, decl.implicit_self.has_implicit_self(), def_id);

        let attrs: Vec<_> = self
            .tcx
            .get_attrs_by_path(def_id.to_def_id(), &[Symbol::intern("rpl"), Symbol::intern("dynamic")])
            .collect();
        for attr in &attrs {
            let error = DynamicError::from_attr(attr, self.tcx.def_span(def_id.to_def_id()));
            self.tcx.emit_node_span_lint(
                error.lint(),
                self.tcx.local_def_id_to_hir_id(def_id),
                error.primary_span(),
                error,
            );
        }

        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}

impl<'tcx, 'pcx> CheckFnCtxt<'pcx, 'tcx> {
    #[expect(clippy::too_many_arguments)]
    #[instrument(level = "trace", skip(self, rpl_rust_items, header, body, mir_cfg, mir_ddg), fields(pat_name = ?name))]
    fn impl_matched<'a>(
        &self,
        name: Symbol,
        rpl_rust_items: &'pcx pat::RustItems<'pcx>,
        def_id: LocalDefId,
        header: Option<FnHeader>,
        has_self: bool,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = NormalizedMatched<'tcx>> {
        let iter = rpl_rust_items.impls.values().flat_map(move |impl_pat| {
            // FIXME: check impl_pat.ty and impl_pat.trait_id
            impl_pat
                .fns
                .values()
                .filter(move |fn_pat| fn_pat.filter(self.tcx, def_id, header))
                .filter_map(move |fn_pat| Some((fn_pat, fn_pat.extra_span(self.tcx, def_id)?)))
                .flat_map(move |(fn_pat, attr_map)| {
                    // FIXME: sometimes we need to check function name
                    // if *fn_name != impl_item.ident.name {
                    //     continue;
                    // }
                    CheckMirCtxt::new(
                        self.tcx,
                        self.pcx,
                        body,
                        has_self,
                        rpl_rust_items,
                        name,
                        fn_pat,
                        mir_cfg,
                        mir_ddg,
                    )
                    .check()
                    .into_iter()
                    .filter(move |matched| self.check_constraints(name, fn_pat, body, matched))
                    .map(move |matched| {
                        let labels = &fn_pat.expect_body().labels;
                        (matched, labels, attr_map.clone())
                    })
                })
                .map(|(matched, label_map, attr_map)| NormalizedMatched::new(&matched, label_map, &attr_map))
        });
        rpl_rust_items.post_process(iter)
    }

    #[instrument(level = "trace", skip(self, pat_op, header, body, mir_cfg, mir_ddg), fields(pat_name = ?name))]
    #[expect(clippy::too_many_arguments)]
    fn impl_matched_pat_op<'a>(
        &self,
        name: Symbol,
        pat_op: &pat::PatternOperation<'pcx>,
        def_id: LocalDefId,
        header: Option<FnHeader>,
        has_self: bool,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = NormalizedMatched<'tcx>> {
        let positive = &pat_op.positive;
        let positive = self
            .impl_matched_pat_item(positive.0, positive.1, def_id, header, has_self, body, mir_cfg, mir_ddg)
            .map(|matched| matched.map(&positive.2));
        let negative: FxHashSet<_> = pat_op
            .negative
            .iter()
            .flat_map(|negative| {
                self.impl_matched_pat_item(negative.0, negative.1, def_id, header, has_self, body, mir_cfg, mir_ddg)
                    .map(|matched| matched.map(&negative.2))
            })
            .collect();
        debug!(negative = ?negative, "impl_matched_pat_op");

        let iter = positive
            .inspect(|positive| debug!(positive = ?positive, "impl_matched_pat_op positive"))
            .filter(move |matched| {
                debug_assert!(negative.iter().all(|neg| neg.has_same_head(matched)));
                !negative.contains(matched)
            })
            .collect::<Vec<_>>()
            .into_iter();

        pat_op.post_process(iter)
    }

    #[expect(clippy::too_many_arguments)]
    fn impl_matched_pat_item<'a>(
        &self,
        name: Symbol,
        pat_item: &'pcx PatternItem<'pcx>,
        def_id: LocalDefId,
        header: Option<FnHeader>,
        has_self: bool,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = NormalizedMatched<'tcx>> {
        match pat_item {
            PatternItem::RustItems(rust_items) => {
                Either::Left(self.impl_matched(name, rust_items, def_id, header, has_self, body, mir_cfg, mir_ddg))
            },
            PatternItem::RPLPatternOperation(pat_op) => {
                Either::Right(self.impl_matched_pat_op(name, pat_op, def_id, header, has_self, body, mir_cfg, mir_ddg))
            },
        }
    }

    #[expect(clippy::too_many_arguments)]
    #[instrument(level = "trace", skip(self, rpl_rust_items, header, body, mir_cfg, mir_ddg), fields(pat_name = ?name))]
    fn fn_matched<'a>(
        &self,
        name: Symbol,
        rpl_rust_items: &'pcx pat::RustItems<'pcx>,
        def_id: LocalDefId,
        header: Option<FnHeader>,
        has_self: bool,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = NormalizedMatched<'tcx>> {
        let iter = rpl_rust_items
            .fns
            .iter()
            .filter(move |fn_pat| fn_pat.filter(self.tcx, def_id, header))
            .filter_map(move |fn_pat| Some((fn_pat, fn_pat.extra_span(self.tcx, def_id)?)))
            .flat_map(move |(fn_pat, attr_map)| {
                // FIXME: check constraints::attributes, i.e. pre-match filtering
                CheckMirCtxt::new(
                    self.tcx,
                    self.pcx,
                    body,
                    has_self,
                    rpl_rust_items,
                    name,
                    fn_pat,
                    mir_cfg,
                    mir_ddg,
                )
                .check()
                .into_iter()
                .filter(move |matched| self.check_constraints(name, fn_pat, body, matched))
                .map(move |matched| {
                    let labels = &fn_pat.expect_body().labels;
                    (matched, labels, attr_map.clone())
                })
            })
            .map(|(matched, label_map, attr_map)| NormalizedMatched::new(&matched, label_map, &attr_map));

        rpl_rust_items.post_process(iter)
    }

    #[instrument(level = "trace", skip(self, pat_op, header, body, mir_cfg, mir_ddg), fields(pat_name = ?name))]
    #[expect(clippy::too_many_arguments)]
    fn fn_matched_pat_op<'a>(
        &self,
        name: Symbol,
        pat_op: &pat::PatternOperation<'pcx>,
        def_id: LocalDefId,
        header: Option<FnHeader>,
        has_self: bool,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = NormalizedMatched<'tcx>> {
        let positive = &pat_op.positive;
        let positive = self
            .fn_matched_pat_item(positive.0, positive.1, def_id, header, has_self, body, mir_cfg, mir_ddg)
            .map(|matched| matched.map(&positive.2));
        let negative: FxHashSet<_> = pat_op
            .negative
            .iter()
            .flat_map(|negative| {
                self.fn_matched_pat_item(negative.0, negative.1, def_id, header, has_self, body, mir_cfg, mir_ddg)
                    .map(|matched| matched.map(&negative.2))
            })
            .collect();
        debug!(negative = ?negative, "impl_matched_pat_op");

        let iter = positive
            .inspect(|positive| debug!(positive = ?positive, "impl_matched_pat_op positive"))
            .filter(move |matched| {
                debug_assert!(negative.iter().all(|neg| neg.has_same_head(matched)));
                !negative.contains(matched)
            })
            .collect::<Vec<_>>()
            .into_iter();

        pat_op.post_process(iter)
    }

    #[expect(clippy::too_many_arguments)]
    fn fn_matched_pat_item<'a>(
        &self,
        name: Symbol,
        pat_item: &'pcx PatternItem<'pcx>,
        def_id: LocalDefId,
        header: Option<FnHeader>,
        has_self: bool,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = NormalizedMatched<'tcx>> {
        match pat_item {
            PatternItem::RustItems(rust_items) => {
                Either::Left(self.fn_matched(name, rust_items, def_id, header, has_self, body, mir_cfg, mir_ddg))
            },
            PatternItem::RPLPatternOperation(pat_op) => {
                Either::Right(self.fn_matched_pat_op(name, pat_op, def_id, header, has_self, body, mir_cfg, mir_ddg))
            },
        }
    }

    #[instrument(level = "debug", skip(self, fn_pat, body), fields(pat_name = ?name, fn_name = ?fn_pat.name, constraints = ?fn_pat.constraints), ret)]
    fn check_constraints(
        &self,
        name: Symbol,
        fn_pat: &pat::FnPattern<'pcx>,
        body: &mir::Body<'tcx>,
        matched: &Matched<'tcx>,
    ) -> bool {
        let evaluator = PredicateEvaluator::new(
            self.tcx,
            ty::TypingEnv::post_analysis(self.tcx, body.source.def_id()),
            body,
            &fn_pat.expect_body().labels,
            matched,
            &fn_pat.symbol_table.meta_vars,
        );
        evaluator.evaluate_constraint(&fn_pat.constraints)
    }
}

impl<'tcx> CheckFnCtxt<'_, 'tcx> {
    #[instrument(level = "debug", skip_all)]
    fn check_impl(&mut self, impl_: &hir::Impl<'tcx>) {
        for impl_item in impl_.items {
            if let hir::AssocItemKind::Fn { has_self } = impl_item.kind {
                let id = impl_item.id;
                let impl_item = self.tcx.hir().impl_item(id);
                let def_id = impl_item.owner_id.def_id;
                match impl_item.kind {
                    hir::ImplItemKind::Fn(sig, _) => {
                        if self.tcx.is_mir_available(def_id) {
                            let decl = sig.decl;
                            let body = self.tcx.optimized_mir(def_id);
                            let mir_cfg = rpl_match::graph::mir_control_flow_graph(body);
                            let mir_ddg = rpl_match::graph::mir_data_dep_graph(body, &mir_cfg);
                            let header = Some(sig.header);
                            let source_map = self.tcx.sess.source_map();
                            self.pcx.for_each_rpl_pattern(|_id, pattern| {
                                for (&name, pat_item) in &pattern.patt_block {
                                    for matched in self.impl_matched_pat_item(
                                        name, pat_item, def_id, header, has_self, body, &mir_cfg, &mir_ddg,
                                    ) {
                                        let error = pattern
                                            .get_diag(name, source_map, None, body, decl, &matched)
                                            .unwrap_or_else(identity);
                                        self.tcx.emit_node_span_lint(
                                            error.lint(),
                                            self.tcx.local_def_id_to_hir_id(def_id),
                                            error.primary_span(),
                                            error,
                                        );
                                    }
                                }
                            });
                        }
                    },
                    _ => (), // Actually impossible, but we handle it gracefully.
                }
            }
        }
    }
    #[instrument(level = "debug", skip(self, decl, header))]
    fn check_fn(
        &mut self,
        fn_name: Option<Ident>,
        decl: &hir::FnDecl<'tcx>,
        header: Option<FnHeader>,
        has_self: bool,
        def_id: LocalDefId,
    ) {
        if self.tcx.is_mir_available(def_id) {
            let body = self.tcx.optimized_mir(def_id);
            let mir_cfg = rpl_match::graph::mir_control_flow_graph(body);
            let mir_ddg = rpl_match::graph::mir_data_dep_graph(body, &mir_cfg);
            let fn_name = fn_name.map(|ident| ident.name);
            let source_map = self.tcx.sess.source_map();
            self.pcx.for_each_rpl_pattern(|_id, pattern| {
                for (&name, pat_item) in &pattern.patt_block {
                    for matched in
                        self.fn_matched_pat_item(name, pat_item, def_id, header, has_self, body, &mir_cfg, &mir_ddg)
                    {
                        let error = pattern
                            .get_diag(name, source_map, fn_name, body, decl, &matched)
                            .unwrap_or_else(identity);
                        self.tcx.emit_node_span_lint(
                            error.lint(),
                            self.tcx.local_def_id_to_hir_id(def_id),
                            error.primary_span(),
                            error,
                        );
                    }
                }
            });
        }
    }
    // #[instrument(level = "debug", skip(self))]
    // fn check_struct(
    //     &mut self,
    //     def_id: LocalDefId,
    //     variant: hir::VariantData<'tcx>,
    //     generics: &'tcx hir::Generics<'tcx>,
    // ) {
    //     let adt_def = self.tcx.adt_def(def_id);
    //     self.pcx.for_each_rpl_pattern(|_id, pattern| {
    //         for (&name, pat_item) in &pattern.patt_block {
    //             match pat_item {
    //                 rpl_context::pat::PatternItem::RustItems(rpl_rust_items) => {
    //                     for (_, adt_pat) in &rpl_rust_items.adts {
    //                         for matched in
    //                             MatchAdtCtxt::new(self.tcx, self.pcx, rpl_rust_items,
    // adt_pat).match_adt(adt_def)                         {
    //                             let error = pattern
    //                                 .get_diag(name, &fn_pat.expect_mir_body().labels, body, &matched)
    //                                 .unwrap_or_else(identity);
    //                             self.tcx.emit_node_span_lint(
    //                                 error.lint(),
    //                                 self.tcx.local_def_id_to_hir_id(def_id),
    //                                 error.primary_span(),
    //                                 error,
    //                             );
    //                         }
    //                     }
    //                 },
    //                 _ => unreachable!(),
    //             }
    //         }
    //     });
    // }
    // #[instrument(level = "debug", skip(self))]
    // fn check_enum(&mut self, def_id: LocalDefId, variants: hir::EnumDef<'tcx>, generics: &'tcx
    // hir::Generics<'tcx>) {}
}
