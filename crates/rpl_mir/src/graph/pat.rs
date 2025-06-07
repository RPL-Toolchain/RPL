use rpl_mir_graph::{ControlFlowGraph, DataDepGraph, ProgramDepGraph, SwitchTargets, TerminatorEdges};

use super::BlockDataDepGraphVisitor;
use crate::pat;
use crate::pat::visitor::{PatternVisitor, PlaceContext};

pub type PatProgramDepGraph = ProgramDepGraph<pat::BasicBlock, pat::Local>;
pub type PatDataDepGraph = DataDepGraph<pat::BasicBlock, pat::Local>;
pub type PatControlFlowGraph = ControlFlowGraph<pat::BasicBlock>;
pub type PatSwitchTargets = SwitchTargets<pat::BasicBlock>;
type PatTerminatorEdges = TerminatorEdges<pat::BasicBlock>;

pub fn pat_program_dep_graph(patterns: &pat::FnPatternBody<'_>, pointer_bytes: u64) -> PatProgramDepGraph {
    let cfg = pat_control_flow_graph(patterns, pointer_bytes);
    ProgramDepGraph::build_from(&cfg, &pat_data_dep_graph(patterns, &cfg))
}

pub fn pat_data_dep_graph(patterns: &pat::FnPatternBody<'_>, cfg: &PatControlFlowGraph) -> PatDataDepGraph {
    let mut graph = DataDepGraph::new(
        patterns.basic_blocks.len(),
        |bb| patterns[bb].num_statements_and_terminator(),
        patterns.locals.len(),
    );
    for (bb, block) in patterns.basic_blocks.iter_enumerated() {
        BlockDataDepGraphVisitor::new(&mut graph.blocks[bb]).visit_basic_block_data(bb, block);
    }
    graph.build_interblock_edges(cfg);
    graph
}

pub fn pat_control_flow_graph(patterns: &pat::FnPatternBody<'_>, pointer_bytes: u64) -> PatControlFlowGraph {
    ControlFlowGraph::new(patterns.basic_blocks.len(), |block| {
        normalized_terminator_edges(patterns[block].terminator.as_ref(), pointer_bytes)
    })
}

impl<'tcx> PatternVisitor<'tcx> for BlockDataDepGraphVisitor<'_, pat::Local> {
    fn visit_place(&mut self, place: pat::Place<'_>, pcx: PlaceContext, location: pat::Location) {
        match place.base {
            pat::PlaceBase::Local(local) => self.graph.access_local(local, pcx, location.statement_index),
            pat::PlaceBase::Var(_) => {}, //FIXME: handle var
        }
        self.super_place(place, pcx, location);
    }
    fn visit_local(&mut self, local: pat::Local, pcx: PlaceContext, location: pat::Location) {
        self.graph.access_local(local, pcx, location.statement_index);
    }
    fn visit_statement(&mut self, statement: &pat::StatementKind<'tcx>, location: pat::Location) {
        self.super_statement(statement, location);
        self.graph.update_deps(location.statement_index);
    }
    fn visit_terminator(&mut self, terminator: &pat::TerminatorKind<'tcx>, location: pat::Location) {
        self.super_terminator(terminator, location);
        self.graph.update_deps(location.statement_index);
        self.graph.update_dep_end();
    }
}

pub fn normalized_terminator_edges(
    terminator: Option<&pat::TerminatorKind<'_>>,
    pointer_bytes: u64,
) -> PatTerminatorEdges {
    use pat::TerminatorKind::{Call, Drop, Goto, PatEnd, Return, SwitchInt};
    match terminator {
        None | Some(Return | PatEnd) => TerminatorEdges::None,
        Some(&Goto(target) | &Drop { target, .. }) => TerminatorEdges::Single(target),
        Some(&Call { target, .. }) => TerminatorEdges::AssignOnReturn {
            return_: Box::new([target]),
            cleanup: None,
        },
        Some(SwitchInt { targets, .. }) => {
            TerminatorEdges::SwitchInt(pat_normalized_switch_targets(targets, pointer_bytes))
        },
    }
}

pub fn pat_normalized_switch_targets(targets: &pat::SwitchTargets, pointer_bytes: u64) -> PatSwitchTargets {
    PatSwitchTargets {
        targets: targets
            .targets
            .iter()
            .map(|(&value, &bb)| (value.normalize(pointer_bytes), bb))
            .collect(),
        otherwise: targets.otherwise,
    }
}
