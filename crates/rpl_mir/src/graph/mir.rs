use rpl_mir_graph::{ControlFlowGraph, DataDepGraph, ProgramDepGraph, SwitchTargets, TerminatorEdges};
use rustc_data_structures::packed::Pu128;
use rustc_middle::mir::visit::{PlaceContext, Visitor};
use rustc_middle::mir::{self};

use super::BlockDataDepGraphVisitor;

pub type MirProgramDepGraph = ProgramDepGraph<mir::BasicBlock, mir::Local>;
pub type MirDataDepGraph = DataDepGraph<mir::BasicBlock, mir::Local>;
pub type MirControlFlowGraph = ControlFlowGraph<mir::BasicBlock>;
pub type MirSwitchTargets = SwitchTargets<mir::BasicBlock>;
type MirTerminatorEdges = TerminatorEdges<mir::BasicBlock>;

pub fn mir_program_dep_graph(body: &mir::Body<'_>) -> MirProgramDepGraph {
    let cfg = mir_control_flow_graph(body);
    ProgramDepGraph::build_from(&cfg, &mir_data_dep_graph(body, &cfg))
}

pub fn mir_data_dep_graph(body: &mir::Body<'_>, cfg: &MirControlFlowGraph) -> MirDataDepGraph {
    let mut graph = DataDepGraph::new(
        body.basic_blocks.len(),
        |bb| body.basic_blocks[bb].statements.len() + 1,
        body.local_decls.len(),
    );
    for (bb, block) in body.basic_blocks.iter_enumerated() {
        BlockDataDepGraphVisitor::new(&mut graph.blocks[bb]).visit_basic_block_data(bb, block);
    }
    graph.build_interblock_edges(cfg);
    graph
}

pub fn mir_control_flow_graph(body: &mir::Body<'_>) -> MirControlFlowGraph {
    ControlFlowGraph::new(body.basic_blocks.len(), |block| {
        terminator_edges(&body[block].terminator().kind)
    })
}

impl<'tcx> Visitor<'tcx> for BlockDataDepGraphVisitor<'_, mir::Local> {
    fn visit_place(&mut self, place: &mir::Place<'tcx>, pcx: PlaceContext, location: mir::Location) {
        self.graph.access_local(place.local, pcx, location.statement_index);
        self.super_place(place, pcx, location);
    }
    fn visit_local(&mut self, local: mir::Local, pcx: PlaceContext, location: mir::Location) {
        self.graph.access_local(local, pcx, location.statement_index);
    }
    fn visit_statement(&mut self, statement: &mir::Statement<'tcx>, location: mir::Location) {
        self.super_statement(statement, location);
        self.graph.update_deps(location.statement_index);
    }
    fn visit_terminator(&mut self, terminator: &mir::Terminator<'tcx>, location: mir::Location) {
        self.super_terminator(terminator, location);
        self.graph.update_deps(location.statement_index);
        self.graph.update_dep_end();
    }
}

fn terminator_edges(terminator: &mir::TerminatorKind<'_>) -> MirTerminatorEdges {
    match terminator.edges() {
        mir::TerminatorEdges::None => TerminatorEdges::None,
        mir::TerminatorEdges::Single(bb) => TerminatorEdges::Single(bb),
        mir::TerminatorEdges::Double(bb0, bb1) => TerminatorEdges::Double(bb0, bb1),
        mir::TerminatorEdges::AssignOnReturn { return_, cleanup, .. } => TerminatorEdges::AssignOnReturn {
            return_: return_.into(),
            cleanup,
        },
        mir::TerminatorEdges::SwitchInt { targets, .. } => TerminatorEdges::SwitchInt(mir_switch_targets(targets)),
    }
}

pub fn mir_switch_targets(targets: &mir::SwitchTargets) -> MirSwitchTargets {
    MirSwitchTargets {
        targets: targets.iter().map(|(value, bb)| (Pu128(value), bb)).collect(),
        otherwise: Some(targets.otherwise()),
    }
}
