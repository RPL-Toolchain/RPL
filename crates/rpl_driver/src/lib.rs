#![feature(rustc_private)]

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

rustc_fluent_macro::fluent_messages! { "../messages.en.ftl" }

use rpl_context::PatCtxt;
use rpl_meta::context::MetaContext;
use rpl_mir::CheckMirCtxt;
use rustc_hir as hir;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_lint_defs::RegisteredTools;
use rustc_macros::{Diagnostic, LintDiagnostic};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::ty::TyCtxt;
use rustc_middle::util::Providers;
use rustc_session::declare_tool_lint;
use rustc_span::Span;
use rustc_span::symbol::Ident;

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
            hir::ItemKind::Trait(hir::IsAuto::No, hir::Safety::Safe, ..)
            | hir::ItemKind::Impl(_)
            | hir::ItemKind::Fn { .. } => {},
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
        if self.tcx.is_mir_available(def_id) {
            let body = self.tcx.optimized_mir(def_id);
            let mir_cfg = rpl_mir::graph::mir_control_flow_graph(body);
            let mir_ddg = rpl_mir::graph::mir_data_dep_graph(body, &mir_cfg);
            self.pcx.for_each_rpl_pattern(|_id, pattern| {
                for (&name, pat_item) in &pattern.patt_block {
                    match pat_item {
                        rpl_context::pat::PatternItem::RustItems(rpl_rust_items) => {
                            for fn_pat in &rpl_rust_items.fns.unnamed_fns {
                                for matched in CheckMirCtxt::new(
                                    self.tcx,
                                    self.pcx,
                                    body,
                                    rpl_rust_items,
                                    name,
                                    fn_pat,
                                    mir_cfg.clone(),
                                    mir_ddg.clone(),
                                )
                                .check()
                                {
                                    let error =
                                        pattern.get_diag(name, &fn_pat.expect_mir_body().labels, body, &matched);
                                    self.tcx.emit_node_span_lint(
                                        error.lint(),
                                        self.tcx.local_def_id_to_hir_id(def_id),
                                        error.primary_span(),
                                        error,
                                    );
                                }
                            }
                        },
                        _ => unreachable!(),
                    }
                }
            });
        }
        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}
