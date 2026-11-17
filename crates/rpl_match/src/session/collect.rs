use std::cell::RefCell;

use rpl_constraints::Constraints;
use rpl_constraints::predicates::{BodyInfoCache, PredicateArg};
use rpl_context::PatCtxt;
use rpl_context::pat::{self, FnPattern};
use rpl_meta::symbol_table::MetaVariable;
use rustc_data_structures::fx::FxHashMap;
use rustc_hir::def_id::DefId;
use rustc_middle::mir;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Symbol;

use crate::graph::{MirControlFlowGraph, MirDataDepGraph};
use crate::matches::artifact::NormalizedMatched;
use crate::mir::CheckMirCtxt;
use crate::predicate_evaluator::PredicateEvaluator;
use crate::session::bindings::{BindingSnapshot, MetaBindings};
use crate::session::slot::{AdtSlotCandidate, AdtSlotDesc, CrateAdtItem, CrateFnItem, FnSlotCandidate};

pub struct MatchCollectCtxt<'a, 'pcx, 'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub pcx: PatCtxt<'pcx>,
    pub pat_name: Symbol,
    pub body_caches: &'a RefCell<FxHashMap<DefId, BodyInfoCache>>,
}

impl<'a, 'pcx, 'tcx> MatchCollectCtxt<'a, 'pcx, 'tcx> {
    pub fn new(
        tcx: TyCtxt<'tcx>,
        pcx: PatCtxt<'pcx>,
        pat_name: Symbol,
        body_caches: &'a RefCell<FxHashMap<DefId, BodyInfoCache>>,
    ) -> Self {
        Self {
            tcx,
            pcx,
            pat_name,
            body_caches,
        }
    }

    /// Run inner matching for one `(fn_pat, def_id)` under the current SharedEnv prefix.
    pub fn match_fn_slot(
        &self,
        rust_items: &'pcx pat::RustItems<'pcx>,
        env: &MetaBindings<'tcx>,
        fn_pat: &FnPattern<'pcx>,
        item: CrateFnItem,
        mut on_cand: impl FnMut(FnSlotCandidate<'tcx>),
    ) {
        let Some(attr_map) = fn_pat.extra_span(self.tcx, item.def_id) else {
            return;
        };

        if fn_pat.is_signature_only() {
            if let Some(cand) = self.match_sig_candidate(rust_items, env, fn_pat, item, attr_map) {
                on_cand(cand);
            }
            return;
        }

        if !self.tcx.is_mir_available(item.def_id) {
            return;
        }

        let body = self.body(item.def_id);
        let (mir_cfg, mir_ddg) = self.graphs(body);
        let self_ty = self.self_ty(item.def_id);
        let cx = CheckMirCtxt::new(
            self.tcx,
            self.pcx,
            body,
            item.has_self,
            self_ty,
            rust_items,
            self.pat_name,
            fn_pat,
            &mir_cfg,
            &mir_ddg,
        );
        seed_from_env(&cx.ty, env);
        cx.check_with(|matched| {
            if !self.check_constraints(
                fn_pat,
                item.def_id,
                body,
                matched,
                env,
                &fn_pat.constraints,
                Some(&mir_ddg),
                Some(&mir_cfg),
            ) {
                return;
            }
            let Some(adt_defs) = crate::collect_adt_def_bindings(&cx.ty) else {
                return;
            };
            let labels = &fn_pat.expect_body().labels;
            let normalized = NormalizedMatched::new(matched, labels, &attr_map);
            let snapshot = BindingSnapshot::from_normalized_with_adt_defs(&normalized, adt_defs);
            on_cand(FnSlotCandidate {
                def_id: item.def_id,
                snapshot,
                normalized,
                matched: matched.clone(),
            });
        });
    }

    fn match_sig_candidate(
        &self,
        rust_items: &'pcx pat::RustItems<'pcx>,
        env: &MetaBindings<'tcx>,
        fn_pat: &FnPattern<'pcx>,
        item: CrateFnItem,
        attr_map: rpl_constraints::attributes::ExtraSpan<'tcx>,
    ) -> Option<FnSlotCandidate<'tcx>> {
        let def_id = item.def_id;
        let mir_available = self.tcx.is_mir_available(def_id);
        // Foreign/`extern` fns have no MIR; signature matching must not require a body.
        if !mir_available && !fn_pat.constraints.preds.is_empty() {
            return None;
        }
        let typing_env = ty::TypingEnv::post_analysis(self.tcx, def_id.to_def_id());
        let self_ty = self.self_ty(def_id);
        let cx = crate::MatchFnCtxt::with_typing_env(self.tcx, self.pcx, rust_items, fn_pat, typing_env, self_ty);
        seed_from_env(cx.ty(), env);
        if !cx.match_fn(def_id.to_def_id()) {
            return None;
        }
        let adt_defs = crate::collect_adt_def_bindings(cx.ty())?;
        let ty_vars = project_unique_ty_vars(cx.ty())?;
        let const_vars = project_unique_const_vars(cx.ty())?;
        let meta = rust_items.meta.as_ref();
        let mir_pat = fn_pat.expect_body();
        let labels = &mir_pat.labels;
        let mut locals = rustc_index::IndexVec::from_elem_n(mir::Local::from_u32(0), mir_pat.locals.len());
        if mir_available {
            let body = self.body(def_id);
            if let Some(ret) = mir_pat.return_idx {
                locals[ret] = mir::RETURN_PLACE;
            }
            let mut args = body.args_iter();
            if let Some(self_local) = mir_pat.self_idx {
                if let Some(arg) = args.next() {
                    locals[self_local] = arg;
                }
            }
            for (local, _) in mir_pat.locals.iter_enumerated() {
                if mir_pat.params_idx.contains(&local)
                    && let Some(arg) = args.next()
                {
                    locals[local] = arg;
                }
            }
        }
        let matched = crate::matches::Matched {
            basic_blocks: Default::default(),
            locals,
            ty_vars,
            const_vars,
            place_vars: rustc_index::IndexVec::from_fn_n(
                |_| mir::PlaceRef {
                    local: mir::Local::from_u32(0),
                    projection: &[],
                },
                meta.place_vars.len(),
            ),
            adt_fields: Default::default(),
        };
        if mir_available {
            let body = self.body(def_id);
            if !self.check_constraints(fn_pat, def_id, body, &matched, env, &fn_pat.constraints, None, None) {
                return None;
            }
        } else if mentioned_meta_unbound(
            &fn_pat.constraints,
            fn_pat.symbol_table,
            &overlay_matched(&matched, env),
        ) {
            return None;
        }
        let normalized = NormalizedMatched::new(&matched, labels, &attr_map);
        Some(FnSlotCandidate {
            def_id,
            snapshot: BindingSnapshot::from_normalized_with_adt_defs(&normalized, adt_defs),
            normalized,
            matched,
        })
    }

    pub fn collect_adt_candidates(
        &self,
        rust_items: &'pcx pat::RustItems<'pcx>,
        desc: AdtSlotDesc<'pcx>,
        item: CrateAdtItem,
    ) -> Vec<AdtSlotCandidate<'tcx>> {
        let adt_def = self.tcx.adt_def(item.def_id);
        let match_ctxt = crate::MatchAdtCtxt::new(self.tcx, self.pcx, rust_items, desc.adt_pat);
        let Some(adt_match) = match_ctxt.match_adt(adt_def) else {
            return Vec::new();
        };
        let ty_bindings = match_ctxt.resolved_ty_bindings();
        vec![AdtSlotCandidate {
            def_id: item.def_id,
            adt_match,
            ty_bindings,
        }]
    }

    fn body(&self, def_id: rustc_hir::def_id::LocalDefId) -> &mir::Body<'tcx> {
        self.tcx.optimized_mir(def_id)
    }

    fn graphs(&self, body: &mir::Body<'tcx>) -> (MirControlFlowGraph, MirDataDepGraph) {
        let mir_cfg = crate::graph::mir_control_flow_graph(body);
        let mir_ddg = crate::graph::mir_data_dep_graph(body, &mir_cfg);
        (mir_cfg, mir_ddg)
    }

    fn self_ty(&self, def_id: rustc_hir::def_id::LocalDefId) -> Option<ty::Ty<'tcx>> {
        self.tcx
            .impl_of_method(def_id.into())
            .map(|impl_| self.tcx.type_of(impl_).instantiate_identity())
    }

    pub(crate) fn check_impl_constraints(
        &self,
        rust_items: &'pcx pat::RustItems<'pcx>,
        impl_name: Symbol,
        def_id: rustc_hir::def_id::LocalDefId,
        fn_pat: &FnPattern<'pcx>,
        matched: &crate::matches::Matched<'tcx>,
        env: &MetaBindings<'tcx>,
    ) -> bool {
        let Some(impl_pat) = rust_items.impls.get(&impl_name) else {
            return true;
        };
        if impl_pat.constraints.preds.is_empty() {
            return true;
        }
        let body = self.body(def_id);
        let (mir_cfg, mir_ddg) = self.graphs(body);
        self.check_constraints(
            fn_pat,
            def_id,
            body,
            matched,
            env,
            &impl_pat.constraints,
            Some(&mir_ddg),
            Some(&mir_cfg),
        )
    }

    #[allow(clippy::too_many_arguments)] // FIXME: introduce a constraints context struct.
    fn check_constraints(
        &self,
        fn_pat: &FnPattern<'pcx>,
        def_id: rustc_hir::def_id::LocalDefId,
        body: &mir::Body<'tcx>,
        matched: &crate::matches::Matched<'tcx>,
        env: &MetaBindings<'tcx>,
        constraints: &Constraints,
        mir_ddg: Option<&MirDataDepGraph>,
        mir_cfg: Option<&MirControlFlowGraph>,
    ) -> bool {
        let eval_matched = overlay_matched(matched, env);
        if mentioned_meta_unbound(constraints, fn_pat.symbol_table, &eval_matched) {
            return false;
        }
        let typing_env = ty::TypingEnv::post_analysis(self.tcx, body.source.def_id());
        let mut caches = self.body_caches.borrow_mut();
        let cache = caches
            .entry(body.source.def_id())
            .or_insert_with(|| BodyInfoCache::new(self.tcx, typing_env, body));
        let evaluator = PredicateEvaluator::new(
            self.tcx,
            typing_env,
            def_id.into(),
            body,
            &fn_pat.expect_body().labels,
            &eval_matched,
            cache,
            fn_pat.symbol_table,
            mir_ddg,
            mir_cfg,
        );
        evaluator.evaluate_constraint(constraints)
    }
}

fn overlay_matched<'tcx>(
    matched: &crate::matches::Matched<'tcx>,
    env: &MetaBindings<'tcx>,
) -> crate::matches::Matched<'tcx> {
    let mut out = matched.clone();
    for (idx, ty) in out.ty_vars.iter_enumerated_mut() {
        if MetaBindings::should_skip_ty_binding(*ty)
            && let Some(bound) = env.ty_vars[idx]
        {
            *ty = bound;
        }
    }
    for (idx, konst) in out.const_vars.iter_enumerated_mut() {
        if MetaBindings::should_skip_const_binding(*konst)
            && let Some(bound) = env.const_vars[idx]
        {
            *konst = bound;
        }
    }
    out
}

fn mentioned_meta_unbound(
    constraints: &Constraints,
    symbol_table: &pat::FnSymbolTable<'_>,
    matched: &crate::matches::Matched<'_>,
) -> bool {
    for name in mentioned_meta_vars(constraints) {
        let lookup = name.as_str();
        let meta = symbol_table.meta_vars.get_meta_var_from_name(lookup).or_else(|| {
            lookup
                .strip_prefix('$')
                .and_then(|n| symbol_table.meta_vars.get_meta_var_from_name(n))
        });
        match meta {
            Some(MetaVariable::Type(idx, _)) => {
                let ty_idx = pat::TyVarIdx::from_usize(idx);
                // `Never` is the empty-candidate sentinel from `project_unique_ty_vars`.
                // `Param` is a real binding (generic MIR) and must be allowed in predicates
                // such as `maybe_misaligned($T, $alignment)` — do not reuse
                // `should_skip_ty_binding`, which also skips `Param` for SharedEnv merge.
                if matches!(matched.ty_vars[ty_idx].kind(), ty::TyKind::Never) {
                    return true;
                }
            },
            Some(MetaVariable::Const(idx, _, _)) => {
                let const_idx = pat::ConstVarIdx::from_usize(idx);
                // Only `Const::Param` placeholders are "unbound" for SharedEnv; MIR
                // `Const::MIR` values are concrete even when unevaluable.
                if MetaBindings::should_skip_const_binding(matched.const_vars[const_idx]) {
                    return true;
                }
            },
            Some(MetaVariable::Place(idx, _, _)) => {
                let _ = idx;
            },
            Some(MetaVariable::AdtPat(_, _)) | None => {},
        }
    }
    false
}

fn mentioned_meta_vars(constraints: &Constraints) -> impl Iterator<Item = Symbol> + '_ {
    constraints.preds.iter().flat_map(|conj| {
        conj.clauses.iter().flat_map(|clause| {
            clause.terms.iter().flat_map(|term| {
                term.args.iter().filter_map(|arg| match arg {
                    PredicateArg::MetaVar(sym) => Some(*sym),
                    _ => None,
                })
            })
        })
    })
}

fn seed_from_env<'tcx>(ty: &crate::MatchTyCtxt<'_, 'tcx>, env: &MetaBindings<'tcx>) {
    for (idx, bound) in env.ty_vars.iter_enumerated() {
        if let Some(bound_ty) = *bound {
            ty.pin_ty_var(idx, bound_ty);
        }
    }
    for (idx, bound) in env.const_vars.iter_enumerated() {
        if let Some(konst) = *bound
            && !MetaBindings::should_skip_const_binding(konst)
        {
            ty.pin_const_var(idx, konst);
        }
    }
    for (&name, &def_id) in &env.adt_defs {
        ty.pin_adt_def(name, def_id);
    }
}

fn project_unique_ty_vars<'tcx>(
    ty: &crate::MatchTyCtxt<'_, 'tcx>,
) -> Option<rustc_index::IndexVec<pat::TyVarIdx, ty::Ty<'tcx>>> {
    let mut failed = false;
    let out = rustc_index::IndexVec::from_fn_n(
        |i| {
            let set = ty.ty_vars[i].borrow();
            match set.len() {
                0 => ty.tcx.types.never,
                1 => *set.iter().next().expect("len == 1"),
                _ => {
                    failed = true;
                    ty.tcx.types.never
                },
            }
        },
        ty.ty_vars.len(),
    );
    (!failed).then_some(out)
}

fn project_unique_const_vars<'tcx>(
    ty: &crate::MatchTyCtxt<'_, 'tcx>,
) -> Option<rustc_index::IndexVec<pat::ConstVarIdx, rpl_constraints::Const<'tcx>>> {
    use rpl_constraints::Const;
    let dummy = Const::Param(ty::ParamConst {
        index: 0,
        name: Symbol::intern("_"),
    });
    let mut failed = false;
    let out = rustc_index::IndexVec::from_fn_n(
        |i| {
            let set = ty.const_vars[i].borrow();
            match set.len() {
                0 => dummy,
                1 => *set.iter().next().expect("len == 1"),
                _ => {
                    failed = true;
                    dummy
                },
            }
        },
        ty.const_vars.len(),
    );
    (!failed).then_some(out)
}
