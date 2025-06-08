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

use std::convert::identity;

use either::Either;
use rpl_context::PatCtxt;
use rpl_meta::context::MetaContext;
use rpl_mir::graph::{MirControlFlowGraph, MirDataDepGraph};
use rpl_mir::pat::{LabelMap, PatternItem};
use rpl_mir::{CheckMirCtxt, Matched, MatchedWithLabelMap, NormalizedMatched, pat};
use rustc_data_structures::fx::FxHashSet;
use rustc_hir as hir;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_lint_defs::RegisteredTools;
use rustc_macros::{Diagnostic, LintDiagnostic};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::mir;
use rustc_middle::ty::TyCtxt;
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
    _ = tcx.hir_crate_items(()).par_items(|item_id| {
        check_item(tcx, pcx, item_id);
        Ok(())
    });
    rpl_utils::visit_crate(tcx);
}

pub fn check_item(tcx: TyCtxt<'_>, pcx: PatCtxt<'_>, item_id: hir::ItemId) {
    let item = tcx.hir().item(item_id);
    // let def_id = item_id.owner_id.def_id;
    let mut check_ctxt = CheckFnCtxt { tcx, pcx };
    check_ctxt.visit_item(item);
}

/// Used for finding pattern matches in given Rust crate.
struct CheckFnCtxt<'pcx, 'tcx> {
    tcx: TyCtxt<'tcx>,
    pcx: PatCtxt<'pcx>,
}

impl<'tcx> Visitor<'tcx> for CheckFnCtxt<'_, 'tcx> {
    type NestedFilter = All;
    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) -> Self::Result {
        match item.kind {
            hir::ItemKind::Trait(hir::IsAuto::No, hir::Safety::Safe, ..) | hir::ItemKind::Fn { .. } => {},
            hir::ItemKind::Impl(impl_) => self.check_impl(impl_),
            // hir::ItemKind::Struct(struct_, generics) => self.check_struct(item.owner_id.def_id, struct_, generics),
            // hir::ItemKind::Enum(enum_, generics) => self.check_enum(item.owner_id.def_id, enum_, generics),
            _ => return,
        }
        intravisit::walk_item(self, item);
    }

    fn visit_fn(
        &mut self,
        kind: intravisit::FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body_id: hir::BodyId,
        _span: Span,
        def_id: LocalDefId,
    ) -> Self::Result {
        self.check_fn(def_id);
        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}

impl<'tcx, 'pcx> CheckFnCtxt<'pcx, 'tcx> {
    fn impl_matched<'a>(
        &self,
        name: Symbol,
        rpl_rust_items: &'pcx pat::RustItems<'pcx>,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = (Matched<'tcx>, &'pcx LabelMap)> {
        rpl_rust_items.impls.values().flat_map(move |impl_pat| {
            //FIXME: check impl_pat.ty and impl_pat.trait_id
            impl_pat.fns.values().flat_map(move |fn_pat| {
                //FIXME: sometimes we need to check function name
                // if *fn_name != impl_item.ident.name {
                //     continue;
                // }
                CheckMirCtxt::new(self.tcx, self.pcx, body, rpl_rust_items, name, fn_pat, mir_cfg, mir_ddg)
                    .check()
                    .into_iter()
                    .map(move |matched| {
                        let labels = &fn_pat.expect_mir_body().labels;
                        (matched, labels)
                    })
            })
        })
    }

    fn impl_matched_pat_op<'a>(
        &self,
        name: Symbol,
        pat_op: &pat::PatternOperation<'pcx>,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = NormalizedMatched<'tcx>> {
        let positive = self
            .impl_matched_pat_item(name, pat_op.positive.0, body, mir_cfg, mir_ddg)
            .map(|matched| matched.map(&pat_op.positive.1));
        let negative: FxHashSet<_> = pat_op
            .negative
            .iter()
            .flat_map(|negative| {
                self.impl_matched_pat_item(name, negative.0, body, mir_cfg, mir_ddg)
                    .map(|matched| matched.map(&negative.1))
            })
            .collect();
        positive
            .filter(move |matched| !negative.contains(matched))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn impl_matched_pat_item<'a>(
        &self,
        name: Symbol,
        pat_op: &'pcx PatternItem<'pcx>,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = NormalizedMatched<'tcx>> {
        match pat_op {
            PatternItem::RustItems(rust_items) => Either::Left(
                self.impl_matched(name, rust_items, body, mir_cfg, mir_ddg)
                    .map(|(matched, label_map)| NormalizedMatched::new(&matched, label_map)),
            ),
            PatternItem::RPLPatternOperation(pat_op) => {
                Either::Right(self.impl_matched_pat_op(name, pat_op, body, mir_cfg, mir_ddg))
            },
        }
    }

    fn fn_matched<'a>(
        &self,
        name: Symbol,
        rpl_rust_items: &'pcx pat::RustItems<'pcx>,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = (Matched<'tcx>, &'pcx LabelMap)> {
        rpl_rust_items.fns.iter().flat_map(move |fn_pat| {
            // // FIXME: a more general way to handle this
            // if matches!(fn_pat.visibility, Visibility::Public)
            //     && !self.tcx.visibility(def_id).is_public()
            // {
            //     continue;
            // }
            CheckMirCtxt::new(self.tcx, self.pcx, body, rpl_rust_items, name, fn_pat, mir_cfg, mir_ddg)
                .check()
                .into_iter()
                .map(move |matched| {
                    let labels = &fn_pat.expect_mir_body().labels;
                    (matched, labels)
                })
        })
    }

    fn fn_matched_pat_op<'a>(
        &self,
        name: Symbol,
        pat_op: &pat::PatternOperation<'pcx>,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = NormalizedMatched<'tcx>> {
        let positive = self
            .fn_matched_pat_item(name, pat_op.positive.0, body, mir_cfg, mir_ddg)
            .map(|matched| matched.map(&pat_op.positive.1));
        let negative: FxHashSet<_> = pat_op
            .negative
            .iter()
            .flat_map(|negative| {
                self.fn_matched_pat_item(name, negative.0, body, mir_cfg, mir_ddg)
                    .map(|matched| matched.map(&negative.1))
            })
            .collect();
        positive
            .filter(move |matched| !negative.contains(matched))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn fn_matched_pat_item<'a>(
        &self,
        name: Symbol,
        pat_op: &'pcx PatternItem<'pcx>,
        body: &'a mir::Body<'tcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> impl Iterator<Item = NormalizedMatched<'tcx>> {
        match pat_op {
            PatternItem::RustItems(rust_items) => Either::Left(
                self.fn_matched(name, rust_items, body, mir_cfg, mir_ddg)
                    .map(|(matched, label_map)| NormalizedMatched::new(&matched, label_map)),
            ),
            PatternItem::RPLPatternOperation(pat_op) => {
                Either::Right(self.fn_matched_pat_op(name, pat_op, body, mir_cfg, mir_ddg))
            },
        }
    }
}

impl<'tcx> CheckFnCtxt<'_, 'tcx> {
    #[instrument(level = "debug", skip_all)]
    fn check_impl(&mut self, impl_: &hir::Impl<'tcx>) {
        for impl_item in impl_.items {
            if let hir::AssocItemKind::Fn { .. } = impl_item.kind {
                let id = impl_item.id;
                let impl_item = self.tcx.hir().impl_item(id);
                let def_id = impl_item.owner_id.def_id;
                match impl_item.kind {
                    hir::ImplItemKind::Fn(..) => {
                        if self.tcx.is_mir_available(def_id) {
                            let body = self.tcx.optimized_mir(def_id);
                            let mir_cfg = rpl_mir::graph::mir_control_flow_graph(body);
                            let mir_ddg = rpl_mir::graph::mir_data_dep_graph(body, &mir_cfg);
                            self.pcx.for_each_rpl_pattern(|_id, pattern| {
                                for (&name, pat_item) in &pattern.patt_block {
                                    match pat_item {
                                        PatternItem::RustItems(rpl_rust_items) => {
                                            for (matched, labels) in
                                                self.impl_matched(name, rpl_rust_items, body, &mir_cfg, &mir_ddg)
                                            {
                                                let error = pattern
                                                    .get_diag(name, body, &MatchedWithLabelMap(labels, &matched))
                                                    .unwrap_or_else(identity);
                                                self.tcx.emit_node_span_lint(
                                                    error.lint(),
                                                    self.tcx.local_def_id_to_hir_id(def_id),
                                                    error.primary_span(),
                                                    *error,
                                                );
                                            }
                                        },
                                        PatternItem::RPLPatternOperation(pat_op) => {
                                            for matched in
                                                self.impl_matched_pat_op(name, pat_op, body, &mir_cfg, &mir_ddg)
                                            {
                                                let error =
                                                    pattern.get_diag(name, body, &matched).unwrap_or_else(identity);
                                                self.tcx.emit_node_span_lint(
                                                    error.lint(),
                                                    self.tcx.local_def_id_to_hir_id(def_id),
                                                    error.primary_span(),
                                                    *error,
                                                );
                                            }
                                        },
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
    #[instrument(level = "debug", skip(self))]
    fn check_fn(&mut self, def_id: LocalDefId) {
        if self.tcx.is_mir_available(def_id) {
            let body = self.tcx.optimized_mir(def_id);
            let mir_cfg = rpl_mir::graph::mir_control_flow_graph(body);
            let mir_ddg = rpl_mir::graph::mir_data_dep_graph(body, &mir_cfg);
            self.pcx.for_each_rpl_pattern(|_id, pattern| {
                for (&name, pat_item) in &pattern.patt_block {
                    match pat_item {
                        PatternItem::RustItems(rpl_rust_items) => {
                            for (matched, labels) in self.fn_matched(name, rpl_rust_items, body, &mir_cfg, &mir_ddg) {
                                let error = pattern
                                    .get_diag(name, body, &MatchedWithLabelMap(labels, &matched))
                                    .unwrap_or_else(identity);
                                self.tcx.emit_node_span_lint(
                                    error.lint(),
                                    self.tcx.local_def_id_to_hir_id(def_id),
                                    error.primary_span(),
                                    *error,
                                );
                            }
                        },
                        PatternItem::RPLPatternOperation(pat_op) => {
                            for matched in self.fn_matched_pat_op(name, pat_op, body, &mir_cfg, &mir_ddg) {
                                let error = pattern.get_diag(name, body, &matched).unwrap_or_else(identity);
                                self.tcx.emit_node_span_lint(
                                    error.lint(),
                                    self.tcx.local_def_id_to_hir_id(def_id),
                                    error.primary_span(),
                                    *error,
                                );
                            }
                        },
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
