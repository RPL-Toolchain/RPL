use std::cell::RefCell;

use rpl_context::PatCtxt;
pub use rpl_context::pat;
pub use rpl_context::pat::MatchedMap;
use rustc_data_structures::fx::FxIndexSet;
use rustc_index::IndexVec;
use rustc_index::bit_set::MixedBitSet;
use rustc_middle::mir::interpret::PointerArithmetic;
use rustc_middle::ty::TyCtxt;
use rustc_middle::{mir, ty};
use rustc_span::Symbol;

use crate::graph::{MirControlFlowGraph, MirDataDepGraph, PatControlFlowGraph, PatDataDepGraph};
use crate::matches::{Matched, matches};
use crate::statement::MatchStatement;
use crate::ty::MatchTy as _;
use crate::{MatchPlaceCtxt, MatchTyCtxt};

pub struct CheckMirCtxt<'a, 'pcx, 'tcx> {
    pub(crate) ty: MatchTyCtxt<'pcx, 'tcx>,
    pub(crate) place: MatchPlaceCtxt<'pcx, 'tcx>,
    pub(crate) body: &'a mir::Body<'tcx>,
    pub(crate) has_self: bool,
    pub(crate) pat_name: Symbol,
    pub(crate) fn_pat: &'a pat::FnPattern<'pcx>,
    pub(crate) mir_pat: &'a pat::FnPatternBody<'pcx>,
    pub(crate) pat_cfg: PatControlFlowGraph,
    pub(crate) pat_ddg: PatDataDepGraph,
    pub(crate) mir_cfg: &'a MirControlFlowGraph,
    pub(crate) mir_ddg: &'a MirDataDepGraph,
    // pat_pdg: PatProgramDepGraph,
    // mir_pdg: MirProgramDepGraph,
    pub(crate) locals: IndexVec<pat::Local, RefCell<MixedBitSet<mir::Local>>>,
    pub(crate) places: IndexVec<pat::PlaceVarIdx, RefCell<FxIndexSet<mir::PlaceRef<'tcx>>>>,
}

impl<'a, 'pcx, 'tcx> CheckMirCtxt<'a, 'pcx, 'tcx> {
    #[expect(clippy::too_many_arguments)]
    #[instrument(level = "debug", skip_all, fields(
        def_id = ?body.source.def_id(),
        pat_name = ?pat_name,
    ))]
    pub fn new(
        tcx: TyCtxt<'tcx>,
        pcx: PatCtxt<'pcx>,
        body: &'a mir::Body<'tcx>,
        has_self: bool,
        pat: &'pcx pat::RustItems<'pcx>,
        pat_name: Symbol,
        fn_pat: &'a pat::FnPattern<'pcx>,
        mir_cfg: &'a MirControlFlowGraph,
        mir_ddg: &'a MirDataDepGraph,
    ) -> Self {
        let typing_env = ty::TypingEnv::post_analysis(tcx, body.source.def_id());
        let ty = MatchTyCtxt::new(tcx, pcx, typing_env, pat, &fn_pat.meta);
        let place = MatchPlaceCtxt::new(tcx, pcx, &fn_pat.meta);
        // let places = pat.locals.iter().map(|&local| ty.mk_ty(pat.locals[local].ty)).collect();
        let mir_pat = fn_pat.expect_body();
        // let pat_pdg = crate::graph::pat_program_dep_graph(&patterns, tcx.pointer_size().bytes_usize());
        // let mir_pdg = crate::graph::mir_program_dep_graph(body);
        let pat_cfg = crate::graph::pat_control_flow_graph(mir_pat, tcx.pointer_size().bytes());
        let pat_ddg = crate::graph::pat_data_dep_graph(mir_pat, &pat_cfg);
        Self {
            ty,
            place,
            body,
            has_self,
            pat_name,
            fn_pat,
            mir_pat,
            pat_cfg,
            pat_ddg,
            mir_cfg,
            mir_ddg,
            // pat_pdg,
            // mir_pdg,
            locals: IndexVec::from_elem_n(
                RefCell::new(MixedBitSet::new_empty(body.local_decls.len())),
                mir_pat.locals.len(),
            ),
            places: IndexVec::from_elem_n(RefCell::new(FxIndexSet::default()), fn_pat.meta.place_vars.len()),
        }
    }
    #[instrument(level = "info", skip_all, fields(
        def_id = ?self.body.source.def_id(),
        pat_name = ?self.pat_name,
    ))]
    pub fn check(&self) -> Vec<Matched<'tcx>> {
        matches(self)
    }
    /*
    pub fn check(&self) {
        use NodeKind::{BlockEnter, BlockExit, Local, StmtOrTerm};
        for (bb_pat, block_pat) in self.patterns.basic_blocks.iter_enumerated() {
            for (bb, block) in self.body.basic_blocks.iter_enumerated() {}
        }
        for (pat_node_idx, pat_node) in self.pat_pdg.nodes() {
            for (mir_node_idx, mir_node) in self.mir_pdg.nodes() {
                let matched = match (pat_node, mir_node) {
                    (StmtOrTerm(bb_pat, stmt_pat), StmtOrTerm(block, statement_index)) => self
                        .match_statement_or_terminator(
                            (bb_pat, stmt_pat).into(),
                            mir::Location { block, statement_index },
                        ),
                    (BlockEnter(_), BlockEnter(_)) | (BlockExit(_), BlockExit(_)) => true,
                    (Local(local_pat), Local(local)) => self.match_local(local_pat, local),
                    _ => continue,
                };
                if matched {
                    self.candidates[pat_node_idx].push(NodeMatch {
                        mir_node_idx,
                        edges_matched: 0,
                    });
                }
            }
        }
        // Pattern:               MIR:
        //             alignment
        // pat_node(u1) ------> mir_node(u2)
        //     |                   |
        //     | pat_edge          | mir_edge
        //     |                   |
        //     v       alignment   v
        // pat_node(v1) ------> mir_node(v2)
        //
        for (pat_node_idx, _) in self.pat_pdg.nodes() {
            let mut iter = self.candidates[pat_node_idx].iter().enumerate().skip(0);
            while let Some((candidate_idx, &NodeMatch { mir_node_idx, .. })) = iter.next() {
                let edges_matched = self
                    .pat_pdg
                    .edges_from(pat_node_idx)
                    .iter()
                    .filter(|pat_edge| {
                        self.candidates[pat_edge.to].iter().any(
                            |&NodeMatch {
                                 mir_node_idx: mir_node_to,
                                 ..
                             }| {
                                self.mir_pdg.find_edge(mir_node_idx, mir_node_to).is_some()
                            },
                        )
                    })
                    .count();
                self.candidates[pat_node_idx][candidate_idx].edges_matched = edges_matched;
                iter = self.candidates[pat_node_idx].iter().enumerate().skip(candidate_idx + 1);
            }
        }
        for candidate in &mut self.candidates {
            candidate.sort_unstable_by_key(|candidate| std::cmp::Reverse(candidate.edges_matched));
        }
    }
    */

    /*
    #[instrument(level = "info", skip(self), fields(def_id = ?self.body.source.def_id()))]
    pub fn check(&mut self) {
        self.check_args();
        let mut visited = BitSet::new_empty(self.body.basic_blocks.len());
        let mut block = Some(mir::START_BLOCK);
        let next_block = |b: mir::BasicBlock| {
            if b.as_usize() + 1 == self.body.basic_blocks.len() {
                mir::START_BLOCK
            } else {
                b.plus(1)
            }
        };
        let mut num_visited = 0;
        while let Some(b) = block {
            if !visited.insert(b) {
                debug!("skip visited block {b:?}");
                block = Some(next_block(b));
                continue;
            }
            let matched = self.check_block(b).is_some();
            let &mut b = block.insert(match self.body[b].terminator().edges() {
                mir::TerminatorEdges::None => next_block(b),
                mir::TerminatorEdges::Single(next) => next,
                mir::TerminatorEdges::Double(next, _) => next,
                mir::TerminatorEdges::AssignOnReturn { return_: &[next], .. } => next,
                _ => next_block(b),
                // mir::TerminatorEdges::AssignOnReturn { .. } => todo!(),
                // mir::TerminatorEdges::SwitchInt { targets, discr } => todo!(),
            });
            debug!("jump to block {b:?}");
            if matched {
                visited.remove(b);
            }
            num_visited += 1;
            if num_visited >= self.body.basic_blocks.len() {
                debug!("all blocks has been visited");
                break;
            }
        }
    }

    fn check_args(&mut self) {
        for (pat, pattern) in self.patterns.ready_patterns() {
            let pat::PatternKind::Init(local) = pattern.kind else {
                continue;
            };
            for arg in self.body.args_iter() {
                if self.match_local(local, arg) {
                    self.patterns.add_match(pat, pat::MatchKind::Argument(arg));
                }
            }
        }
    }

    #[instrument(level = "info", skip(self))]
    fn check_block(&mut self, block: mir::BasicBlock) -> Option<pat::MatchIdx> {
        info!("BasicBlock: {}", {
            let mut buffer = Vec::new();
            mir::pretty::write_basic_block(self.tcx, block, self.body, &mut |_, _| Ok(()), &mut buffer).unwrap();
            String::from_utf8_lossy(&buffer).into_owned()
        });
        for (statement_index, statement) in self.body[block].statements.iter().enumerate() {
            let location = mir::Location { block, statement_index };
            self.check_statement(location, statement);
        }
        self.check_terminator(block, self.body[block].terminator())
    }

    fn check_statement(&mut self, location: mir::Location, statement: &mir::Statement<'tcx>) {
        self.match_statement(location, statement);
    }
    fn check_terminator(
        &mut self,
        block: mir::BasicBlock,
        terminator: &'tcx mir::Terminator<'tcx>,
    ) -> Option<pat::MatchIdx> {
        self.match_terminator(block, terminator)
    }
    */
}

impl<'pcx, 'tcx> MatchStatement<'pcx, 'tcx> for CheckMirCtxt<'_, 'pcx, 'tcx> {
    fn body(&self) -> &mir::Body<'tcx> {
        self.body
    }
    fn mir_pat(&self) -> &pat::FnPatternBody<'pcx> {
        self.mir_pat
    }

    fn pat_cfg(&self) -> &PatControlFlowGraph {
        &self.pat_cfg
    }
    fn pat_ddg(&self) -> &PatDataDepGraph {
        &self.pat_ddg
    }
    fn mir_cfg(&self) -> &MirControlFlowGraph {
        self.mir_cfg
    }
    fn mir_ddg(&self) -> &MirDataDepGraph {
        self.mir_ddg
    }

    fn pat(&self) -> &'pcx pat::RustItems<'pcx> {
        self.ty.pat
    }
    fn pcx(&self) -> PatCtxt<'pcx> {
        self.ty.pcx
    }
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.ty.tcx
    }
    fn typing_env(&self) -> ty::TypingEnv<'tcx> {
        self.ty.typing_env
    }

    type MatchTy = MatchTyCtxt<'pcx, 'tcx>;
    fn ty(&self) -> &Self::MatchTy {
        &self.ty
    }

    #[instrument(level = "debug", skip(self), ret)]
    fn match_local(&self, pat: pat::Local, local: mir::Local) -> bool {
        let mut locals = self.locals[pat].borrow_mut();
        debug!(?locals, ?pat, ?local, "match_local");
        if locals.contains(local) {
            return true;
        }

        if self.mir_pat.params_idx.contains(&pat) {
            // If the local variable is a parameter, we only need to match the
            // corresponding local variable in the MIR graph.
            trace!(?pat, "expected parameter");
            if !(local.as_usize() > 0 && local.as_usize() <= self.body.arg_count) {
                debug!(?local, "found non-parameter local");
                return false;
            }
        }

        let matched = self
            .ty()
            .match_ty(self.mir_pat().locals[pat], self.body().local_decls[local].ty);
        debug!(?pat, ?local, matched, "match_local");
        if matched {
            locals.insert(local);
        }
        matched
    }
    #[instrument(level = "trace", skip(self), ret)]
    fn match_place_var(&self, pat: pat::PlaceVarIdx, place: mir::PlaceRef<'tcx>) -> bool {
        let mut places = self.places[pat].borrow_mut();
        trace!(?places, ?pat, ?place, "match_place_var");
        if places.contains(&place) {
            return true;
        }
        let place_ty = place.ty(&self.body.local_decls, self.ty().tcx);
        let matched = self.ty().match_ty(self.place.places[pat], place_ty.ty);
        debug!(?pat, ?place, matched, "match_place_var");
        if matched {
            places.insert(place);
        }
        matched
    }

    fn get_place_ty_from_place_var(&self, var: pat::PlaceVarIdx) -> pat::PlaceTy<'pcx> {
        pat::PlaceTy::from_ty(self.place.places[var])
        // pat::PlaceTy::from_ty(var.ty)
    }
}
