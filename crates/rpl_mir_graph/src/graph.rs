use core::default::Default;
use std::ops::Index;

use rustc_data_structures::fx::{FxHashMap, FxIndexMap};
use rustc_data_structures::packed::Pu128;
use rustc_index::bit_set::MixedBitSet;
use rustc_index::{Idx, IndexSlice, IndexVec};
use rustc_middle::mir::visit::{MutatingUseContext, NonMutatingUseContext, PlaceContext};

use crate::rwstate::RWCStates;

rustc_index::newtype_index! {
    #[debug_format = "Node{}"]
    #[orderable]
    pub struct NodeIdx {}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind<BasicBlock, Local> {
    StmtOrTerm(BasicBlock, usize),
    BlockEnter(BasicBlock),
    BlockExit(BasicBlock),
    Local(Local),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind<Local> {
    /// There is a goto edge from the `from` block to the `to` block.
    Goto,
    /// There is a goto edge from the `from` block to the `to` block when it unwinds.
    UnwindGoto,
    /// There is a reversed data dependency from the `from` statement to the `to` statement.
    ///
    /// This means that the `from` statement must be executed before the `to` statement.
    DataRdep(Local),
    /// There is an access to a local variable, by given access pattern and order.
    Access(usize, PlaceContext),
    /// There is a switch jump with the given value.
    SwitchInt(Pu128),
    /// There is a switch jump of the otherwise branch.
    Otherwise,
}

pub struct ProgramDepGraph<BasicBlock: Idx, Local: Idx> {
    nodes: IndexVec<NodeIdx, NodeKind<BasicBlock, Local>>,
    edges: Vec<Edge<Local>>,
    locals_offset: NodeIdx,
    blocks_offset: NodeIdx,
    stmts_offsets: IndexVec<BasicBlock, NodeIdx>,
}

impl<BasicBlock: Idx, Local: Idx> Index<NodeIdx> for ProgramDepGraph<BasicBlock, Local> {
    type Output = NodeKind<BasicBlock, Local>;

    fn index(&self, node: NodeIdx) -> &Self::Output {
        &self.nodes[node]
    }
}

pub struct Edge<Local> {
    pub from: NodeIdx,
    pub to: NodeIdx,
    pub kind: EdgeKind<Local>,
}

impl<Local> Edge<Local> {
    fn new(from: NodeIdx, to: NodeIdx, kind: EdgeKind<Local>) -> Self {
        Self { from, to, kind }
    }
    pub fn nodes(&self) -> (NodeIdx, NodeIdx) {
        (self.from, self.to)
    }
}

impl<BasicBlock: Idx, Local: Idx> ProgramDepGraph<BasicBlock, Local> {
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }
    pub fn nodes(&self) -> impl Iterator<Item = (NodeIdx, NodeKind<BasicBlock, Local>)> + '_ {
        self.nodes.iter_enumerated().map(|(idx, &node)| (idx, node))
    }
    pub fn edges(&self) -> impl Iterator<Item = &Edge<Local>> + '_ {
        self.edges.iter()
    }
    pub fn find_edge(&self, from: NodeIdx, to: NodeIdx) -> Option<&Edge<Local>> {
        Some(&self.edges[self.search_edge(from, to).ok()?])
    }
    pub fn edges_from(&self, from: NodeIdx) -> &[Edge<Local>] {
        &self.edges[self.search_edges_for_node(from)]
    }
    pub fn block_nodes(&self, bb: BasicBlock) -> [NodeIdx; 2] {
        let start = NodeIdx::plus(self.blocks_offset, bb.index() * 2);
        let end = start.plus(1);
        [start, end]
    }
    pub fn local_node(&self, local: Local) -> NodeIdx {
        NodeIdx::plus(self.locals_offset, local.index())
    }
    pub fn stmt_node(&self, bb: BasicBlock, statement_index: usize) -> NodeIdx {
        NodeIdx::plus(self.stmts_offsets[bb], statement_index)
    }

    pub fn build_from(cfg: &ControlFlowGraph<BasicBlock>, ddg: &DataDepGraph<BasicBlock, Local>) -> Self {
        ProgramDepGraphBuilder::new(cfg, ddg).build()
    }
    fn search_edge(&self, from: NodeIdx, to: NodeIdx) -> Result<usize, usize> {
        self.edges.binary_search_by_key(&(from, to), Edge::nodes)
    }
    fn search_edges_for_node(&self, from: NodeIdx) -> std::ops::Range<usize> {
        let start = self
            .search_edge(from, NodeIdx::ZERO)
            .unwrap_or_else(std::convert::identity);
        let end = self
            .search_edge(from.plus(1), NodeIdx::ZERO)
            .unwrap_or_else(std::convert::identity);
        start..end
    }
    fn add_locals(&mut self, num_locals: usize) -> impl Copy + Fn(Local) -> NodeIdx + use<BasicBlock, Local> {
        self.locals_offset = self.nodes.next_index();
        let offset = self.locals_offset;
        self.nodes.extend((0..num_locals).map(Local::new).map(NodeKind::Local));
        move |local| NodeIdx::plus(offset, local.index())
    }
    fn add_block_starts_and_ends(
        &mut self,
        num_blocks: usize,
    ) -> (
        impl Copy + Fn(BasicBlock) -> NodeIdx + use<BasicBlock, Local>,
        impl Copy + Fn(BasicBlock) -> NodeIdx + use<BasicBlock, Local>,
    ) {
        self.blocks_offset = self.nodes.next_index();
        let offset = self.blocks_offset;
        self.nodes.extend(
            (0..num_blocks)
                .map(BasicBlock::new)
                .flat_map(|bb| [NodeKind::BlockEnter(bb), NodeKind::BlockExit(bb)]),
        );
        let starts = move |bb: BasicBlock| NodeIdx::plus(offset, bb.index() * 2);
        let ends = move |bb: BasicBlock| NodeIdx::plus(starts(bb), 1);
        (starts, ends)
    }
    fn add_statements_and_terminator(
        &mut self,
        bb: BasicBlock,
        block: &BlockDataDepGraph<Local>,
    ) -> impl Copy + Fn(usize) -> NodeIdx + use<BasicBlock, Local> {
        self.stmts_offsets[bb] = self.nodes.next_index();
        let offset = self.stmts_offsets[bb];
        self.nodes
            .extend((0..block.num_statements()).map(|stmt| NodeKind::StmtOrTerm(bb, stmt)));
        move |stmt| NodeIdx::plus(offset, stmt)
    }
    fn add_accesses(
        &mut self,
        block: &BlockDataDepGraph<Local>,
        stmts: impl Copy + Fn(usize) -> NodeIdx,
        locals: impl Copy + Fn(Local) -> NodeIdx,
    ) {
        self.edges
            .extend(block.accesses.iter().enumerate().flat_map(|(stmt, accesses)| {
                accesses.iter().enumerate().map(move |(idx, &(local, access))| {
                    Edge::new(stmts(stmt), locals(local), EdgeKind::Access(idx, access))
                })
            }));
    }

    fn add_deps(
        &mut self,
        block: &BlockDataDepGraph<Local>,
        stmts: impl Copy + Fn(usize) -> NodeIdx,
        start: NodeIdx,
        end: NodeIdx,
    ) {
        self.edges.extend((0..block.num_statements()).flat_map(|stmt| {
            block
                .deps(stmt)
                .map(move |(dep, local)| Edge::new(stmts(dep), stmts(stmt), EdgeKind::DataRdep(local)))
        }));
        self.edges.extend(
            block
                .rdep_start()
                .map(|(rdep, local)| Edge::new(start, stmts(rdep), EdgeKind::DataRdep(local))),
        );
        self.edges.extend(
            block
                .dep_end()
                .map(|(dep, local)| Edge::new(stmts(dep), end, EdgeKind::DataRdep(local))),
        );
        self.edges.extend(
            block
                .rdep_start_end()
                .map(|local| Edge::new(start, end, EdgeKind::DataRdep(local))),
        );
    }

    fn add_cfg_edges(
        &mut self,
        bb: BasicBlock,
        terminator_edges: &TerminatorEdges<BasicBlock>,
        starts: impl Copy + Fn(BasicBlock) -> NodeIdx,
        ends: impl Copy + Fn(BasicBlock) -> NodeIdx,
    ) {
        match terminator_edges {
            TerminatorEdges::None => {},
            &TerminatorEdges::Single(target) => {
                self.edges.push(Edge::new(ends(bb), starts(target), EdgeKind::Goto));
            },
            &TerminatorEdges::Double(target, unwind) => {
                self.edges.push(Edge::new(ends(bb), starts(target), EdgeKind::Goto));
                self.edges
                    .push(Edge::new(ends(bb), starts(unwind), EdgeKind::UnwindGoto));
            },
            TerminatorEdges::AssignOnReturn { return_, cleanup } => {
                self.edges.extend(
                    return_
                        .iter()
                        .map(|&ret| Edge::new(ends(bb), starts(ret), EdgeKind::Goto)),
                );
                if let &Some(cleanup) = cleanup {
                    self.edges
                        .push(Edge::new(ends(bb), starts(cleanup), EdgeKind::UnwindGoto));
                }
            },
            TerminatorEdges::SwitchInt(switch_targets) => {
                self.edges.extend(
                    switch_targets
                        .targets
                        .iter()
                        .map(|(&value, &next)| Edge::new(ends(bb), starts(next), EdgeKind::SwitchInt(value))),
                );
                if let Some(otherwise) = switch_targets.otherwise {
                    self.edges
                        .push(Edge::new(ends(bb), starts(otherwise), EdgeKind::Otherwise));
                }
            },
        }
    }
}

struct ProgramDepGraphBuilder<'a, BasicBlock: Idx, Local: Idx> {
    cfg: &'a ControlFlowGraph<BasicBlock>,
    ddg: &'a DataDepGraph<BasicBlock, Local>,
    graph: ProgramDepGraph<BasicBlock, Local>,
}

impl<'a, BasicBlock: Idx, Local: Idx> ProgramDepGraphBuilder<'a, BasicBlock, Local> {
    fn new(cfg: &'a ControlFlowGraph<BasicBlock>, ddg: &'a DataDepGraph<BasicBlock, Local>) -> Self {
        assert_eq!(cfg.num_blocks(), ddg.num_blocks(), "Mismatched number of blocks");
        Self {
            cfg,
            ddg,
            graph: ProgramDepGraph {
                nodes: IndexVec::new(),
                edges: Vec::new(),
                locals_offset: NodeIdx::ZERO,
                blocks_offset: NodeIdx::ZERO,
                stmts_offsets: IndexVec::from_elem_n(NodeIdx::ZERO, cfg.num_blocks()),
            },
        }
    }
    fn build(mut self) -> ProgramDepGraph<BasicBlock, Local> {
        let locals = self.graph.add_locals(self.ddg.num_locals());
        let (starts, ends) = self.graph.add_block_starts_and_ends(self.ddg.num_blocks());
        let graph = &mut self.graph;
        for (bb, block) in self.ddg.blocks() {
            let stmts = graph.add_statements_and_terminator(bb, block);
            graph.add_accesses(block, stmts, locals);
            graph.add_deps(block, stmts, starts(bb), ends(bb));
            graph.add_cfg_edges(bb, &self.cfg[bb], starts, ends);
        }
        graph.edges.sort_unstable_by_key(Edge::nodes);
        // FIXME: add interblock dependency edges
        self.graph
    }
}

#[derive(Clone)]
pub struct ControlFlowGraph<BasicBlock: Idx> {
    pub(crate) terminator_edges: IndexVec<BasicBlock, TerminatorEdges<BasicBlock>>,
}

impl<BasicBlock: Idx> ControlFlowGraph<BasicBlock> {
    pub fn new(num_blocks: usize, terminator_edges: impl FnMut(BasicBlock) -> TerminatorEdges<BasicBlock>) -> Self {
        Self {
            terminator_edges: IndexVec::from_fn_n(terminator_edges, num_blocks),
        }
    }
    pub fn num_blocks(&self) -> usize {
        self.terminator_edges.len()
    }
    pub fn blocks(&self) -> &IndexSlice<BasicBlock, TerminatorEdges<BasicBlock>> {
        &self.terminator_edges
    }
}

impl<BasicBlock: Idx> Index<BasicBlock> for ControlFlowGraph<BasicBlock> {
    type Output = TerminatorEdges<BasicBlock>;

    fn index(&self, bb: BasicBlock) -> &Self::Output {
        &self.terminator_edges[bb]
    }
}

#[derive(Debug, Clone)]
pub enum TerminatorEdges<BasicBlock: Idx> {
    /// For terminators that have no successor, like `return`.
    None,
    /// For terminators that a single successor, like `goto`, and `assert` without cleanup block.
    Single(BasicBlock),
    /// For terminators that two successors, `assert` with cleanup block and `falseEdge`.
    Double(BasicBlock, BasicBlock),
    /// Special action for `Yield`, `Call` and `InlineAsm` terminators.
    AssignOnReturn {
        return_: Box<[BasicBlock]>,
        cleanup: Option<BasicBlock>,
    },
    /// Special edge for `SwitchInt`.
    SwitchInt(SwitchTargets<BasicBlock>),
}

impl<BasicBlock: Idx> TerminatorEdges<BasicBlock> {
    pub gen fn successors(&self) -> BasicBlock {
        match self {
            Self::None => {},
            &Self::Single(bb) => yield bb,
            &Self::Double(ret, cleanup) => {
                yield ret;
                yield cleanup;
            },
            Self::AssignOnReturn { return_, cleanup } => {
                for &bb in std::iter::chain(return_, cleanup) {
                    yield bb;
                }
            },
            Self::SwitchInt(targets) => {
                for &target in std::iter::chain(targets.targets.values(), &targets.otherwise) {
                    yield target;
                }
            },
        };
    }
}

#[derive(Debug, Clone)]
pub struct SwitchTargets<BasicBlock: Idx> {
    pub targets: FxIndexMap<Pu128, BasicBlock>,
    pub otherwise: Option<BasicBlock>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Location<BasicBlock> {
    block: BasicBlock,
    statement_index: usize,
}

impl<BasicBlock: Idx> std::fmt::Debug for Location<BasicBlock> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{}]", self.block, self.statement_index)
    }
}

impl<BasicBlock> Location<BasicBlock> {
    pub fn new(block: BasicBlock, statement_index: usize) -> Self {
        Self { block, statement_index }
    }
}

#[derive(Clone)]
pub struct DataDepGraph<BasicBlock: Idx, Local: Idx> {
    num_locals: usize,
    pub blocks: IndexVec<BasicBlock, BlockDataDepGraph<Local>>,
    /// Record the interblock data dependencies.
    /// i.e. the bb[stmt] depends on the dep_bb[dep_stmt] writing to the local.
    /// and bb[Out] depends on the dep_bb[dep_stmt] writing to the local.
    interblock_edges: IndexVec<BasicBlock, InterblockEdges<BasicBlock, Local>>,
}

impl<BasicBlock: Idx, Local: Idx> Index<BasicBlock> for DataDepGraph<BasicBlock, Local> {
    type Output = BlockDataDepGraph<Local>;

    fn index(&self, bb: BasicBlock) -> &Self::Output {
        &self.blocks[bb]
    }
}

impl<BasicBlock: Idx, Local: Idx> DataDepGraph<BasicBlock, Local> {
    pub fn new(num_blocks: usize, mut num_statements: impl FnMut(BasicBlock) -> usize, num_locals: usize) -> Self {
        Self {
            num_locals,
            blocks: IndexVec::from_fn_n(|bb| BlockDataDepGraph::new(num_statements(bb), num_locals), num_blocks),
            interblock_edges: IndexVec::from_fn_n(|bb| InterblockEdges::new(num_statements(bb)), num_blocks),
        }
    }
    pub fn blocks(&self) -> impl Iterator<Item = (BasicBlock, &BlockDataDepGraph<Local>)> + '_ {
        self.blocks.iter_enumerated()
    }
    pub fn deps(&self, bb_from: BasicBlock, stmt_from: usize) -> impl Iterator<Item = ((BasicBlock, usize), Local)> {
        self.blocks[bb_from]
            .deps(stmt_from)
            .map(move |(stmt, local)| ((bb_from, stmt), local))
            .chain(
                self.interblock_edges[bb_from].deps[stmt_from]
                    .iter()
                    .map(|(loc, local)| ((loc.block, loc.statement_index), local)),
            )
    }
    #[instrument(level = "debug", skip_all, fields(?bb, dep_loc = %format!("{dep_bb:?}[{dep_stmt}]")), ret)]
    pub fn get_dep_end(&self, bb: BasicBlock, dep_bb: BasicBlock, dep_stmt: usize) -> Option<Local> {
        let mut ret = None;
        if bb == dep_bb {
            ret = ret.or(self.blocks[bb].get_dep_end(dep_stmt));
        }
        ret.or_else(|| {
            self.interblock_edges[bb]
                .dep_end
                .entry
                .get(&Location {
                    block: dep_bb,
                    statement_index: dep_stmt,
                })
                .copied()
        })
    }
    pub fn dep_end(&self, bb: BasicBlock) -> impl Iterator<Item = ((BasicBlock, usize), Local)> + '_ {
        self.blocks[bb]
            .dep_end()
            .map(move |(stmt, local)| ((bb, stmt), local))
            .chain(
                self.interblock_edges[bb]
                    .dep_end
                    .iter()
                    .map(|(loc, local)| ((loc.block, loc.statement_index), local)),
            )
    }
    #[instrument(level = "debug", skip(self), ret)]
    pub fn get_dep(&self, bb: BasicBlock, stmt: usize, dep_bb: BasicBlock, dep_stmt: usize) -> Option<Local> {
        let mut ret = None;
        if bb == dep_bb {
            ret = ret.or(self.blocks[bb].get_dep(stmt, dep_stmt));
        }
        ret.or_else(|| {
            self.interblock_edges[bb].deps[stmt]
                .entry
                .get(&(Location::new(dep_bb, dep_stmt)))
                .copied()
        })
    }
    pub fn interblock_edges(&self) -> impl Iterator<Item = (BasicBlock, usize, BasicBlock, usize, Local)> {
        self.interblock_edges.iter_enumerated().flat_map(|(bb, edges)| {
            edges.deps.iter().enumerate().flat_map(move |(stmt, edges)| {
                edges
                    .iter()
                    .map(move |(loc, local)| (bb, stmt, loc.block, loc.statement_index, local))
            })
        })
    }
    pub fn build_interblock_edges(&mut self, cfg: &ControlFlowGraph<BasicBlock>) {
        loop {
            let mut changed = false;
            for (bb, block) in self.blocks.iter_enumerated() {
                for bb_succ in cfg.terminator_edges[bb].successors().filter(|&bb_succ| bb_succ != bb) {
                    let (curr, succ) = self.interblock_edges.pick2_mut(bb, bb_succ);
                    for (loc, local) in block
                        .dep_end
                        .iter()
                        .map(|(&stmt, &local)| (Location::new(bb, stmt), local))
                        .chain(curr.dep_end.iter())
                    {
                        //            local
                        // Given: loc <---- (bb, OUT)
                        // and bb <---- bb_succ in CFG
                        //                   local
                        // and (bb_succ, IN) <---- (bb_succ, OUT) in DDG(bb_succ)
                        //              local
                        // We have: loc <---- (bb_succ, OUT)
                        changed |=
                            succ.dep_end
                                .union_with_intersection(loc, local, &self.blocks[bb_succ].rdep_start_end);
                        //            local
                        // Given: loc <---- (bb, OUT)
                        // and bb <---- bb_succ in CFG
                        //                   local
                        // and (bb_succ, IN) <---- (bb_succ, stmt) in DDG(bb_succ)
                        //              local
                        // We have: loc <---- (bb_succ, next_stmt)
                        for (&stmt_succ, locals_succ) in &self.blocks[bb_succ].rdep_start {
                            changed |= succ.deps[stmt_succ].union_with_intersection(loc, local, locals_succ);
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }
    pub(crate) fn num_blocks(&self) -> usize {
        self.blocks.len()
    }
    pub(crate) fn num_locals(&self) -> usize {
        self.num_locals
    }
}

#[derive(Clone)]
struct InterblockEdges<BasicBlock, Local> {
    /// deps[stmt] records the interblock dependencies of the `stmt` in basic block.
    deps: Vec<InterblockEdgesEntry<BasicBlock, Local>>,
    /// records the interblock dependencies of bb[Out]
    dep_end: InterblockEdgesEntry<BasicBlock, Local>,
}

impl<BasicBlock: Idx, Local: Idx> InterblockEdges<BasicBlock, Local> {
    fn new(statements: usize) -> Self {
        Self {
            deps: vec![Default::default(); statements],
            dep_end: Default::default(),
        }
    }
}

#[derive(Clone)]
struct InterblockEdgesEntry<BasicBlock, Local> {
    entry: FxHashMap<Location<BasicBlock>, Local>,
}

impl<BasicBlock, Local> Default for InterblockEdgesEntry<BasicBlock, Local> {
    fn default() -> Self {
        Self {
            entry: FxHashMap::default(),
        }
    }
}

impl<BasicBlock: Idx, Local: Idx> InterblockEdgesEntry<BasicBlock, Local> {
    fn union_with_intersection(
        &mut self,
        loc: Location<BasicBlock>,
        local: Local,
        locals_succ: &MixedBitSet<Local>,
    ) -> bool {
        if locals_succ.contains(local) {
            let old = self.entry.insert(loc, local);
            assert!(old.is_none_or(|l| l == local));
            return old.is_none();
        }
        false
    }
    fn iter(&self) -> impl Iterator<Item = (Location<BasicBlock>, Local)> + '_ {
        self.entry.iter().map(|(&loc, &local)| (loc, local))
    }
}

/// Record the data dependencies of a BasicBlock.
#[derive(Clone)]
pub struct BlockDataDepGraph<Local: Idx> {
    /// Records the dependencies of each statement.
    /// If deps[stmt].get(&prev_stmt) == Some(local),
    /// Then the statement `stmt` depends on the statement `prev_stmt` writing to the local `local`.
    deps: Vec<FxIndexMap<usize, Local>>,
    /// Records the reverse dependencies of each statement.
    /// If rdeps[stmt].get(&next_stmt) == Some(local),
    /// Then the statement `next_stmt` depends on the statement `stmt` writing to the local `local`.
    rdeps: Vec<FxIndexMap<usize, Local>>,
    /// Records the start points of reverse dependencies for each statement.
    /// If a statement `stmt` reads a local variable `local`,
    /// but no statement writes to `local` before `stmt` (in current block)
    /// then `local` is recorded in rdep_start[stmt],
    /// indicating the dependency is introduced outside of the block (from parameters or other
    /// blocks). (is used to analyze [In_bb] variables.)
    rdep_start: FxIndexMap<usize, MixedBitSet<Local>>,
    /// Records the end points of dependencies for each statement.
    /// If a statement `stmt` writes to a local variable `local`,
    /// and the write is still valid at the end of the block (i.e., not consumed later),
    /// then `local` is recorded in dep_end\[stmt\],
    /// (is used to analyze \[Out_bb\] variables.)
    dep_end: FxIndexMap<usize, Local>,
    /// If a local is introduced outside of the block
    /// and (its last write in the block) is not consumed at the end of the block,
    /// then it is recorded in rdep_start_end,
    rdep_start_end: MixedBitSet<Local>,
    /// Store the read/write/consume states of local variables across all statements in a block.
    /// (a compact representation)
    rw_states: RWCStates<Local>,
    /// Record the specific access information of each statement.
    /// accesses[stmt] indicating which Locals the `stmt` accessed and their access context.
    accesses: Vec<Vec<(Local, PlaceContext)>>,
}

impl<Local: Idx> BlockDataDepGraph<Local> {
    pub fn num_statements(&self) -> usize {
        self.rw_states.num_statements()
    }
    #[instrument(level = "debug", skip(self))]
    pub fn get_rdep_start(&self, statement: usize) -> impl Iterator<Item = Local> + '_ {
        self.rdep_start.get(&statement).into_iter().flat_map(MixedBitSet::iter)
    }
    #[instrument(level = "debug", skip(self))]
    pub fn is_rdep_start(&self, statement: usize, local: Local) -> bool {
        self.rdep_start
            .get(&statement)
            .is_some_and(|locals| locals.contains(local))
    }
    #[instrument(level = "debug", skip(self), ret)]
    pub fn get_dep_end(&self, statement: usize) -> Option<Local> {
        self.dep_end.get(&statement).copied()
    }
    #[instrument(level = "debug", skip(self))]
    pub fn rdep_start(&self) -> impl Iterator<Item = (usize, Local)> + '_ {
        self.rdep_start
            .iter()
            .flat_map(|(&stmt, locals)| locals.iter().map(move |local| (stmt, local)))
    }
    pub fn dep_end(&self) -> impl Iterator<Item = (usize, Local)> + '_ {
        self.dep_end.iter().map(|(&stmt, &local)| (stmt, local))
    }
    pub fn rdep_start_end(&self) -> impl Iterator<Item = Local> + '_ {
        self.rdep_start_end.iter()
    }
    pub fn is_rdep_start_end(&self, local: Local) -> bool {
        self.rdep_start_end.contains(local)
    }
    pub fn full_rdep_start_end(&self) -> bool {
        (0..self.rw_states.num_locals())
            .map(Local::new)
            .all(|local| self.rdep_start_end.contains(local))
    }
    pub fn deps(&self, statement: usize) -> impl Iterator<Item = (usize, Local)> + '_ {
        self.deps[statement].iter().map(|(&stmt, &local)| (stmt, local))
    }
    #[instrument(level = "debug", skip(self))]
    pub fn get_dep(&self, statement: usize, dep_statement: usize) -> Option<Local> {
        self.deps[statement].get(&dep_statement).copied()
    }
    pub fn rdeps(&self, statement: usize) -> impl Iterator<Item = (usize, Local)> + '_ {
        self.rdeps[statement].iter().map(|(&stmt, &local)| (stmt, local))
    }
    pub fn accesses(&self, statement: usize) -> &[(Local, PlaceContext)] {
        &self.accesses[statement]
    }

    pub(crate) fn new(statements: usize, locals: usize) -> Self {
        Self {
            deps: vec![FxIndexMap::default(); statements],
            rdeps: vec![FxIndexMap::default(); statements],
            rdep_start: FxIndexMap::default(),
            dep_end: FxIndexMap::default(),
            rdep_start_end: MixedBitSet::new_empty(locals),
            rw_states: RWCStates::new(statements, locals),
            accesses: vec![Vec::new(); statements],
        }
    }
    pub fn update_deps(&mut self, stmt: usize) {
        for local in self.rw_states.get_reads(stmt) {
            match (0..stmt)
                .rev()
                .find(|&prev_stmt| self.rw_states.get_write(prev_stmt, local))
            {
                Some(prev_stmt) => {
                    self.deps[stmt].insert(prev_stmt, local);
                    self.rdeps[prev_stmt].insert(stmt, local);
                },
                None => {
                    self.rdep_start
                        .entry(stmt)
                        .or_insert_with(|| MixedBitSet::new_empty(self.rw_states.num_locals()))
                        .insert(local);
                },
            }
        }
    }
    pub fn access_local(&mut self, local: Local, pcx: PlaceContext, statement: usize) {
        self.accesses[statement].push((local, pcx));
        self.rw_states
            .set_access(statement, local, Access::from_place_context(pcx));
    }
    pub fn update_dep_end(&mut self) {
        'locals: for local in (0..self.rw_states.num_locals()).map(Local::new) {
            for stmt in (0..self.rw_states.num_statements()).rev() {
                if self.rw_states.get_write(stmt, local) && !self.local_consumed_since(local, stmt + 1) {
                    self.dep_end.insert(stmt, local);
                    continue 'locals;
                }
            }
            if !self.local_consumed_since(local, 0) {
                self.rdep_start_end.insert(local);
            }
        }
    }
    fn local_consumed_since(&self, local: Local, stmt: usize) -> bool {
        (stmt..self.rw_states.num_statements()).any(|prev_stmt| self.rw_states.get_consume(prev_stmt, local))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Access {
    Read,
    Write,
    ReadWrite,
    ReadConsume,
    NoAccess,
}

impl Access {
    pub fn from_place_context(pcx: PlaceContext) -> Self {
        use PlaceContext::{MutatingUse, NonMutatingUse, NonUse};
        match pcx {
            NonMutatingUse(pcx) => {
                use NonMutatingUseContext::{
                    Copy, FakeBorrow, Inspect, Move, PlaceMention, Projection, RawBorrow, SharedBorrow,
                };
                match pcx {
                    Move => Self::ReadConsume,
                    Inspect | Copy | SharedBorrow | FakeBorrow | RawBorrow | Projection => Self::Read,
                    PlaceMention => Self::NoAccess,
                }
            },
            MutatingUse(pcx) => {
                use MutatingUseContext::{
                    AsmOutput, Borrow, Call, Deinit, Drop, Projection, RawBorrow, Retag, SetDiscriminant, Store, Yield,
                };
                match pcx {
                    Store | AsmOutput | Call | Yield => Self::Write,
                    Borrow | RawBorrow | Projection => Self::Read,
                    SetDiscriminant | Drop => Self::ReadWrite,
                    Retag | Deinit => Self::NoAccess,
                }
            },
            NonUse(_) => Self::NoAccess,
        }
    }
}
