use std::cell::RefCell;

use rpl_constraints::predicates::{BodyInfoCache, CrateAnalysisCache};
use rpl_context::PatCtxt;
use rpl_context::pat::{self, FnPattern};
use rustc_data_structures::fx::FxHashMap;
use rustc_hir::def_id::DefId;
use rustc_middle::mir;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Symbol;

use crate::graph::{MirControlFlowGraph, MirDataDepGraph};
use crate::matches::artifact::NormalizedMatched;
use crate::mir::CheckMirCtxt;
use crate::predicate_evaluator::PredicateEvaluator;
use crate::session::bindings::BindingSnapshot;
use crate::session::slot::{AdtSlotCandidate, AdtSlotDesc, CrateAdtItem, CrateFnItem, FnSlotCandidate};

type FnCandidateCache<'tcx> = RefCell<FxHashMap<(DefId, usize, usize), Vec<FnSlotCandidate<'tcx>>>>;

pub struct MatchCollectCtxt<'a, 'pcx, 'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub pcx: PatCtxt<'pcx>,
    pub pat_name: Symbol,
    crate_cache: &'a CrateAnalysisCache,
    pub body_caches: &'a RefCell<FxHashMap<DefId, BodyInfoCache<'tcx>>>,
    fn_candidate_cache: &'a FnCandidateCache<'tcx>,
}

impl<'a, 'pcx, 'tcx> MatchCollectCtxt<'a, 'pcx, 'tcx> {
    pub fn new(
        tcx: TyCtxt<'tcx>,
        pcx: PatCtxt<'pcx>,
        pat_name: Symbol,
        crate_cache: &'a CrateAnalysisCache,
        body_caches: &'a RefCell<FxHashMap<DefId, BodyInfoCache<'tcx>>>,
        fn_candidate_cache: &'a FnCandidateCache<'tcx>,
    ) -> Self {
        Self {
            tcx,
            pcx,
            pat_name,
            crate_cache,
            body_caches,
            fn_candidate_cache,
        }
    }

    pub fn collect_fn_candidates(
        &self,
        rust_items: &'pcx pat::RustItems<'pcx>,
        fn_pat: &FnPattern<'pcx>,
        item: CrateFnItem,
    ) -> Vec<FnSlotCandidate<'tcx>> {
        let cache_key = (
            item.def_id.to_def_id(),
            fn_pat as *const FnPattern<'pcx> as usize,
            rust_items as *const pat::RustItems<'pcx> as usize,
        );
        if let Some(cached) = self.fn_candidate_cache.borrow().get(&cache_key) {
            return cached.clone();
        }
        let candidates = self.collect_fn_candidates_uncached(rust_items, fn_pat, item);
        self.fn_candidate_cache
            .borrow_mut()
            .insert(cache_key, candidates.clone());
        candidates
    }

    fn collect_fn_candidates_uncached(
        &self,
        rust_items: &'pcx pat::RustItems<'pcx>,
        fn_pat: &FnPattern<'pcx>,
        item: CrateFnItem,
    ) -> Vec<FnSlotCandidate<'tcx>> {
        if !fn_pat.filter(self.tcx, item.def_id, item.header, self.body(item.def_id)) {
            return Vec::new();
        }
        let Some(attr_map) = fn_pat.extra_span(self.tcx, item.def_id) else {
            return Vec::new();
        };

        if fn_pat.is_signature_only() {
            return self.collect_sig_candidates(rust_items, fn_pat, item, attr_map);
        }

        let body = self.body(item.def_id);
        let (mir_cfg, mir_ddg) = self.graphs(body);
        let self_ty = self.self_ty(item.def_id);

        let mir_matches = CheckMirCtxt::new(
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
        )
        .check();
        mir_matches
            .into_iter()
            .filter(|matched| self.check_constraints(fn_pat, item.def_id, body, matched, Some(&mir_ddg)))
            .map(|matched| {
                let labels = &fn_pat.expect_body().labels;
                let normalized = NormalizedMatched::new(&matched, labels, &attr_map);
                let snapshot = BindingSnapshot::from_normalized(&normalized);
                FnSlotCandidate {
                    def_id: item.def_id,
                    normalized,
                    matched,
                    snapshot,
                }
            })
            .collect()
    }

    fn collect_sig_candidates(
        &self,
        _rust_items: &'pcx pat::RustItems<'pcx>,
        fn_pat: &FnPattern<'pcx>,
        item: CrateFnItem,
        attr_map: rpl_constraints::attributes::ExtraSpan<'tcx>,
    ) -> Vec<FnSlotCandidate<'tcx>> {
        let body = self.body(item.def_id);
        let labels = &fn_pat.expect_body().labels;
        let matched = crate::matches::Matched {
            basic_blocks: Default::default(),
            locals: Default::default(),
            ty_vars: Default::default(),
            const_vars: Default::default(),
            place_vars: Default::default(),
            adt_fields: Default::default(),
        };
        if !self.check_constraints(fn_pat, item.def_id, body, &matched, None) {
            return Vec::new();
        }
        let normalized = NormalizedMatched::new(&matched, labels, &attr_map);
        vec![FnSlotCandidate {
            def_id: item.def_id,
            snapshot: BindingSnapshot::from_normalized(&normalized),
            normalized,
            matched,
        }]
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

    fn check_constraints(
        &self,
        fn_pat: &FnPattern<'pcx>,
        def_id: rustc_hir::def_id::LocalDefId,
        body: &mir::Body<'tcx>,
        matched: &crate::matches::Matched<'tcx>,
        mir_ddg: Option<&MirDataDepGraph>,
    ) -> bool {
        let typing_env = ty::TypingEnv::post_analysis(self.tcx, body.source.def_id());
        let mut caches = self.body_caches.borrow_mut();
        let cache = caches
            .entry(body.source.def_id())
            .or_insert_with(|| BodyInfoCache::new(self.tcx, typing_env, body, self.crate_cache));
        let evaluator = PredicateEvaluator::new(
            self.tcx,
            typing_env,
            def_id.into(),
            body,
            &fn_pat.expect_body().labels,
            matched,
            cache,
            fn_pat.symbol_table,
            mir_ddg,
        );
        evaluator.evaluate_constraint(&fn_pat.constraints)
    }
}
