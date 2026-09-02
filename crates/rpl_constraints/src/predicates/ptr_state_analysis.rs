use std::cell::RefCell;

use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_middle::mir::visit::{PlaceContext, Visitor};
use rustc_middle::mir::{self, BasicBlock, Local, Operand, Place, ProjectionElem, Rvalue, TerminatorKind};
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::def_id::DefId;
use rustc_span::source_map::Spanned;

const MAX_VISITS: usize = 10_000;
const MAX_SUMMARY_BLOCKS: usize = 80;
const MAX_SUMMARY_LOCALS: usize = 160;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ProjectionKey {
    Deref,
    Field(usize),
    ShallowInitBox,
    SummaryDead,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PointerKind {
    RawPtr,
    Ref,
    Tuple,
    CornerAdt,
    Other,
}

#[derive(Clone, Debug)]
struct PointerNode {
    root: usize,
    kind: PointerKind,
    need_drop: bool,
    tracked: bool,
    alive: bool,
    moved_out: bool,
    referent: Option<usize>,
    aliases: FxHashSet<usize>,
    children: FxHashMap<ProjectionKey, usize>,
}

impl PointerNode {
    fn new(root: usize, local: usize, kind: PointerKind, need_drop: bool, tracked: bool) -> Self {
        let mut aliases = FxHashSet::default();
        aliases.insert(local);
        Self {
            root,
            kind,
            need_drop,
            tracked,
            alive: true,
            moved_out: false,
            referent: None,
            aliases,
            children: FxHashMap::default(),
        }
    }
}

#[derive(Clone, Debug)]
struct PointerState {
    nodes: Vec<PointerNode>,
    constants: FxHashMap<usize, u128>,
}

impl PointerState {
    fn new<'tcx>(tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, body: &mir::Body<'tcx>) -> Self {
        let mut nodes = Vec::with_capacity(body.local_decls.len());
        for (local, local_decl) in body.local_decls.iter_enumerated() {
            let local = local.as_usize();
            let (kind, need_drop, tracked) = pointer_node_props(tcx, typing_env, local_decl.ty);
            nodes.push(PointerNode::new(local, local, kind, need_drop, tracked));
        }
        Self {
            nodes,
            constants: FxHashMap::default(),
        }
    }

    fn set_constant(&mut self, node: usize, value: u128) {
        for alias in self.nodes[node].aliases.clone() {
            self.constants.insert(alias, value);
        }
    }

    fn constant_for(&self, node: usize) -> Option<u128> {
        self.nodes[node]
            .aliases
            .iter()
            .find_map(|alias| self.constants.get(alias).copied())
    }

    fn fill_alive(&mut self, node: usize) {
        let mut seen = FxHashSet::default();
        self.fill_alive_recursive(node, &mut seen);
    }

    fn reset_node_for_assignment(&mut self, node: usize) {
        let old_aliases = self.nodes[node].aliases.clone();
        for alias in old_aliases {
            if alias != node {
                self.nodes[alias].aliases.remove(&node);
            }
        }
        self.nodes[node].aliases.clear();
        self.nodes[node].aliases.insert(node);
        self.nodes[node].children.clear();
        self.nodes[node].alive = true;
        self.nodes[node].moved_out = false;
        self.nodes[node].referent = None;
        self.constants.remove(&node);
    }

    fn reset_place_for_assignment<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        place: Place<'tcx>,
    ) -> Option<usize> {
        let node = self.node_for_place(tcx, typing_env, place, false);
        let reset = node.unwrap_or_else(|| place.local.as_usize());
        self.reset_node_for_assignment(reset);
        node
    }

    fn fill_alive_recursive(&mut self, node: usize, seen: &mut FxHashSet<usize>) {
        if !seen.insert(node) {
            return;
        }
        self.nodes[node].alive = true;
        for alias in self.nodes[node].aliases.clone() {
            self.nodes[alias].alive = true;
        }
        for child in self.nodes[node].children.clone().into_values() {
            self.fill_alive_recursive(child, seen);
        }
    }

    fn mark_place_moved_out<'tcx>(&mut self, tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, place: Place<'tcx>) {
        if let Some(node) = self.node_for_place(tcx, typing_env, place, false) {
            self.nodes[node].moved_out = true;
        }
    }

    // A moved enum-variant field is excluded from the later drop of its original argument.
    fn mark_moved_out_place<'tcx>(&mut self, tcx: TyCtxt<'tcx>, typing_env: ty::TypingEnv<'tcx>, place: Place<'tcx>) {
        if !is_field_move(place) {
            return;
        }
        self.mark_place_moved_out(tcx, typing_env, place);
    }

    fn dead_node(&mut self, node: usize) {
        if self.nodes[node].moved_out {
            return;
        }
        if matches!(self.nodes[node].kind, PointerKind::CornerAdt) {
            return;
        }
        let mut seen = FxHashSet::default();
        self.dead_node_recursive(node, &mut seen);
    }

    fn add_summary_dead_reachable(&mut self, node: usize) {
        let dead = self.ensure_child(
            node,
            ProjectionKey::SummaryDead,
            self.nodes[node].root,
            PointerKind::RawPtr,
            true,
            true,
        );
        self.nodes[dead].alive = false;
    }

    fn dead_node_recursive(&mut self, node: usize, seen: &mut FxHashSet<usize>) {
        if !seen.insert(node) {
            return;
        }
        if matches!(self.nodes[node].kind, PointerKind::CornerAdt) {
            return;
        }
        for alias in self.nodes[node].aliases.clone() {
            if alias == node {
                continue;
            }
            if matches!(self.nodes[alias].kind, PointerKind::Ref) {
                for child in self.nodes[alias].children.clone().into_values() {
                    self.dead_node_recursive(child, seen);
                }
                continue;
            }
            self.dead_node_recursive(alias, seen);
        }
        for child in self.nodes[node].children.clone().into_values() {
            if self.nodes[child].moved_out {
                continue;
            }
            if self.nodes[node].kind == PointerKind::Tuple && !self.nodes[child].need_drop {
                continue;
            }
            self.dead_node_recursive(child, seen);
        }
        if self.nodes[node].tracked {
            self.nodes[node].alive = false;
        }
    }

    fn has_dead_reachable_filtered(&self, node: usize, pointer_only: bool) -> bool {
        let mut seen = FxHashSet::default();
        self.has_dead_reachable_recursive(node, &mut seen, pointer_only)
    }

    fn aliases_root(&self, left: usize, right: usize) -> bool {
        self.nodes.get(left).is_some_and(|node| node.aliases.contains(&right))
            || self.nodes.get(right).is_some_and(|node| node.aliases.contains(&left))
    }

    fn originates_from_root(&self, node: usize, root: usize) -> bool {
        let mut pending = vec![node];
        let mut seen = FxHashSet::default();
        while let Some(current) = pending.pop() {
            if !seen.insert(current) {
                continue;
            }
            if current == root || self.nodes[current].root == root {
                return true;
            }
            pending.extend(self.nodes[current].aliases.iter().copied());
            pending.extend(self.nodes[current].children.values().copied());
            if let Some(referent) = self.nodes[current].referent {
                pending.push(referent);
            }
        }
        false
    }

    fn node_before_deref(&self, place: Place<'_>) -> Option<usize> {
        let mut node = place.local.as_usize();
        for projection in place.projection.iter() {
            match projection {
                ProjectionElem::Deref => return Some(node),
                ProjectionElem::Field(field, _) => {
                    node = self.rhs_alias_representative(node, true);
                    let Some(child) = self.nodes[node].children.get(&ProjectionKey::Field(field.as_usize())) else {
                        return Some(node);
                    };
                    node = *child;
                },
                _ => {},
            }
        }
        None
    }

    fn has_dead_reachable_recursive(&self, node: usize, seen: &mut FxHashSet<usize>, pointer_only: bool) -> bool {
        if !seen.insert(node) {
            return false;
        }
        if self.nodes[node].tracked
            && !self.nodes[node].alive
            && (!pointer_only || matches!(self.nodes[node].kind, PointerKind::RawPtr | PointerKind::Ref))
        {
            return true;
        }
        if self.nodes[node].kind == PointerKind::Ref
            && let Some(referent) = self.resolve_referent(node)
        {
            return self.has_dead_reachable_recursive(referent, seen, pointer_only);
        }
        for alias in self.nodes[node].aliases.iter().copied() {
            // Reference children mirror their referent's fields and can become stale after a
            // strong field update. The referent is the authoritative storage node.
            if self.nodes[alias].kind == PointerKind::Ref && self.nodes[alias].referent.is_some() {
                continue;
            }
            if alias != node && self.has_dead_reachable_recursive(alias, seen, pointer_only) {
                return true;
            }
        }
        self.nodes[node]
            .children
            .values()
            .copied()
            .any(|child| self.has_dead_reachable_recursive(child, seen, pointer_only))
    }

    fn node_for_place<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        place: Place<'tcx>,
        is_rhs: bool,
    ) -> Option<usize> {
        let mut node = place.local.as_usize();
        for projection in place.projection.iter() {
            match projection {
                ProjectionElem::Deref => {
                    if let Some(referent) = self.resolve_referent(node) {
                        node = referent;
                        continue;
                    }
                    let representative =
                        if self.nodes[node].kind == PointerKind::Ref && self.nodes[node].aliases.len() > 1 {
                            self.alias_representative(node)
                        } else {
                            self.rhs_alias_representative(node, is_rhs)
                        };
                    if representative != node {
                        node = representative;
                        continue;
                    }
                    node = representative;
                    node = self.ensure_child(
                        node,
                        ProjectionKey::Deref,
                        self.nodes[node].root,
                        PointerKind::RawPtr,
                        true,
                        true,
                    );
                },
                ProjectionElem::Field(field, ty) => {
                    let key = ProjectionKey::Field(field.as_usize());
                    if is_rhs
                        && !self.nodes[node].children.contains_key(&key)
                        && let Some(alias) = self.nodes[node]
                            .aliases
                            .iter()
                            .copied()
                            .filter(|alias| self.nodes[*alias].children.contains_key(&key))
                            .min()
                    {
                        node = alias;
                    }
                    let (kind, need_drop, tracked) = pointer_node_props(tcx, typing_env, ty);
                    node = self.ensure_child(node, key, self.nodes[node].root, kind, need_drop, tracked);
                },
                // SafeDrop collapses unsupported projections (notably array indices)
                // into their parent node instead of discarding the alias chain.
                _ => continue,
            }
        }
        Some(node)
    }

    fn resolve_referent(&self, node: usize) -> Option<usize> {
        let mut current = node;
        let mut seen = FxHashSet::default();
        while let Some(next) = self.nodes[current].referent {
            if !seen.insert(current) {
                return None;
            }
            current = next;
        }
        (current != node).then_some(current)
    }

    fn alias_representative(&self, node: usize) -> usize {
        self.nodes[node].aliases.iter().copied().min().unwrap_or(node)
    }

    fn rhs_alias_representative(&self, node: usize, is_rhs: bool) -> usize {
        if is_rhs && self.nodes[node].aliases.len() > 1 {
            self.alias_representative(node)
        } else {
            node
        }
    }

    fn ensure_child(
        &mut self,
        parent: usize,
        key: ProjectionKey,
        root: usize,
        kind: PointerKind,
        need_drop: bool,
        tracked: bool,
    ) -> usize {
        if let Some(child) = self.nodes[parent].children.get(&key) {
            return *child;
        }
        let local = self.nodes.len();
        let mut child = PointerNode::new(root, local, kind, need_drop, tracked);
        child.alive = self.nodes[parent].alive;
        self.nodes[parent].children.insert(key, local);
        self.nodes.push(child);
        local
    }

    fn assign_place_from_place<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        left: Place<'tcx>,
        right: Place<'tcx>,
        allow_owned_raw_deref: bool,
        weak_update: bool,
    ) {
        let right_place = right;
        let Some(left) = self.prepare_place_for_assignment(tcx, typing_env, left, weak_update) else {
            return;
        };
        let Some(right) = self.node_for_place(tcx, typing_env, right_place, true) else {
            return;
        };
        let right_root = right_place.local.as_usize();
        if is_direct_deref(right_place)
            && self.nodes[right_root].kind == PointerKind::RawPtr
            && !matches!(self.nodes[left].kind, PointerKind::RawPtr | PointerKind::Ref)
            && !(allow_owned_raw_deref && self.nodes[left].need_drop)
        {
            return;
        }
        if matches!(self.nodes[left].kind, PointerKind::RawPtr | PointerKind::Ref)
            && matches!(self.nodes[right].kind, PointerKind::RawPtr | PointerKind::Ref)
        {
            self.nodes[left].referent =
                if self.nodes[left].kind == PointerKind::RawPtr && self.nodes[right].kind == PointerKind::RawPtr {
                    self.nodes[right].referent
                } else {
                    self.nodes[right].referent.or(Some(right))
                };
        }
        self.merge_alias(left, right);
    }

    fn assign_reference_from_place<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        left: Place<'tcx>,
        right: Place<'tcx>,
    ) {
        let Some(left) = self.reset_place_for_assignment(tcx, typing_env, left) else {
            return;
        };
        let Some(right) = self.node_for_place(tcx, typing_env, right, true) else {
            return;
        };
        self.nodes[left].referent = Some(right);
        self.merge_alias(left, right);
    }

    fn prepare_place_for_assignment<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        place: Place<'tcx>,
        weak_update: bool,
    ) -> Option<usize> {
        if weak_update || self.place_is_raw_pointer_derived(place) {
            self.node_for_place(tcx, typing_env, place, false)
        } else {
            self.reset_place_for_assignment(tcx, typing_env, place)
        }
    }

    // A write through `&mut T` replaces the old pointee. A write reached from a raw pointer is
    // only a weak update because its aliases cannot be resolved precisely within one body.
    fn place_is_raw_pointer_derived(&self, place: Place<'_>) -> bool {
        let mut node = place.local.as_usize();
        for projection in place.projection.iter() {
            match projection {
                ProjectionElem::Deref => return self.is_raw_pointer_derived(node),
                ProjectionElem::Field(field, _) => {
                    let Some(child) = self.nodes[node].children.get(&ProjectionKey::Field(field.as_usize())) else {
                        return false;
                    };
                    node = *child;
                },
                _ => {},
            }
        }
        false
    }

    fn is_raw_pointer_derived(&self, mut node: usize) -> bool {
        let mut seen = FxHashSet::default();
        loop {
            if !seen.insert(node) {
                return false;
            }
            if self.nodes[node].kind == PointerKind::RawPtr {
                return true;
            }
            let Some(referent) = self.nodes[node].referent else {
                return false;
            };
            node = referent;
        }
    }

    fn assign_shallow_init_box<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        left: Place<'tcx>,
        right: Place<'tcx>,
    ) {
        let Some(left) = self.prepare_place_for_assignment(tcx, typing_env, left, false) else {
            return;
        };

        let box_ptr = self.ensure_shallow_init_box_ptr(left);
        self.fill_alive(box_ptr);

        if let Some(right) = self.node_for_place(tcx, typing_env, right, true) {
            self.merge_alias(box_ptr, right);
        }
    }

    fn ensure_shallow_init_box_ptr(&mut self, root_node: usize) -> usize {
        let root = self.nodes[root_node].root;
        let mut node = root_node;
        for kind in [PointerKind::Other, PointerKind::Other, PointerKind::RawPtr] {
            node = self.ensure_child(node, ProjectionKey::ShallowInitBox, root, kind, false, true);
        }
        node
    }

    fn merge_alias(&mut self, left: usize, right: usize) {
        if self.nodes[left].root == self.nodes[right].root {
            return;
        }

        let mut merged = self.nodes[left].aliases.clone();
        merged.extend(self.nodes[right].aliases.iter().copied());
        merged.insert(left);
        merged.insert(right);
        for node in merged.iter().copied() {
            self.nodes[node].aliases = merged.clone();
        }

        for (key, right_child) in self.nodes[right].children.clone() {
            let left_child = if let Some(left_child) = self.nodes[left].children.get(&key).copied() {
                left_child
            } else {
                let child = self.clone_child_for_parent(right_child, left);
                self.nodes[left].children.insert(key, child);
                child
            };
            self.merge_alias(left_child, right_child);
        }
    }

    fn clone_child_for_parent(&mut self, source: usize, parent: usize) -> usize {
        let local = self.nodes.len();
        let mut child = self.nodes[source].clone();
        child.root = self.nodes[parent].root;
        child.aliases.clear();
        child.aliases.insert(local);
        child.referent = None;
        child.children.clear();
        child.moved_out = false;
        self.nodes.push(child);
        local
    }
}

struct DerefUseCollector<'a> {
    state: &'a PointerState,
    summary: &'a mut PointerSummary,
}

impl<'tcx> Visitor<'tcx> for DerefUseCollector<'_> {
    fn visit_place(&mut self, place: &Place<'tcx>, _: PlaceContext, _: mir::Location) {
        if let Some(node) = self.state.node_before_deref(*place) {
            self.summary.record_use(self.state, node);
        }
    }
}

#[derive(Clone, Debug, Default)]
struct BlockInfo {
    next: Vec<BasicBlock>,
    sub_blocks: Vec<BasicBlock>,
}

#[derive(Clone, Debug)]
struct ControlFlowInfo {
    blocks: Vec<BlockInfo>,
    father: Vec<BasicBlock>,
}

impl ControlFlowInfo {
    fn new(body: &mir::Body<'_>) -> Self {
        let mut blocks = vec![BlockInfo::default(); body.basic_blocks.len()];
        let mut father = Vec::with_capacity(body.basic_blocks.len());
        for (bb, data) in body.basic_blocks.iter_enumerated() {
            father.push(bb);
            if let Some(terminator) = &data.terminator {
                blocks[bb.as_usize()].next = terminator_successors(terminator);
            }
        }
        let mut this = Self { blocks, father };
        this.solve_scc();
        this
    }

    fn solve_scc(&mut self) {
        let mut stack = Vec::new();
        let mut in_stack = FxHashSet::default();
        let mut index = vec![None; self.blocks.len()];
        let mut low_link = vec![0; self.blocks.len()];
        let mut next_index = 0;
        if !self.blocks.is_empty() {
            self.tarjan(
                mir::START_BLOCK,
                &mut stack,
                &mut in_stack,
                &mut index,
                &mut low_link,
                &mut next_index,
            );
        }
    }

    fn tarjan(
        &mut self,
        bb: BasicBlock,
        stack: &mut Vec<BasicBlock>,
        in_stack: &mut FxHashSet<BasicBlock>,
        index: &mut [Option<usize>],
        low_link: &mut [usize],
        next_index: &mut usize,
    ) {
        let bb_index = bb.as_usize();
        index[bb_index] = Some(*next_index);
        low_link[bb_index] = *next_index;
        *next_index += 1;
        stack.push(bb);
        in_stack.insert(bb);

        for successor in self.blocks[bb_index].next.clone() {
            let successor_index = successor.as_usize();
            if index[successor_index].is_none() {
                self.tarjan(successor, stack, in_stack, index, low_link, next_index);
                low_link[bb_index] = low_link[bb_index].min(low_link[successor_index]);
            } else if in_stack.contains(&successor) {
                low_link[bb_index] = low_link[bb_index].min(index[successor_index].unwrap());
            }
        }

        if low_link[bb_index] != index[bb_index].unwrap() {
            return;
        }

        let mut component = Vec::new();
        loop {
            let top = stack.pop().unwrap();
            in_stack.remove(&top);
            self.father[top.as_usize()] = bb;
            component.push(top);
            if top == bb {
                break;
            }
        }
        component.reverse();
        self.blocks[bb_index].sub_blocks = component.into_iter().filter(|&block| block != bb).collect();

        let mut next = self.blocks[bb_index].next.clone();
        for block in self.blocks[bb_index].sub_blocks.clone() {
            next.extend(self.blocks[block.as_usize()].next.iter().copied());
        }
        next.retain(|target| self.father[target.as_usize()] != bb);
        next.sort_by_key(|target| target.as_usize());
        next.dedup();
        self.blocks[bb_index].next = next;
    }
}

fn terminator_successors(terminator: &mir::Terminator<'_>) -> Vec<BasicBlock> {
    match terminator.kind.edges() {
        mir::TerminatorEdges::None => Vec::new(),
        mir::TerminatorEdges::Single(target) => vec![target],
        mir::TerminatorEdges::Double(target, cleanup) => vec![target, cleanup],
        mir::TerminatorEdges::AssignOnReturn { return_, cleanup, .. } => {
            let mut successors = return_.to_vec();
            successors.extend(cleanup);
            successors
        },
        mir::TerminatorEdges::SwitchInt { targets, .. } => targets
            .iter()
            .map(|(_, target)| target)
            .chain([targets.otherwise()])
            .collect(),
    }
}

fn call_cleanup_successor(terminator: &mir::Terminator<'_>) -> Option<BasicBlock> {
    match terminator.kind.edges() {
        mir::TerminatorEdges::AssignOnReturn { cleanup, .. } => cleanup,
        _ => None,
    }
}

#[derive(Clone, Debug)]
struct PointerSummary {
    return_has_dead: bool,
    arg_has_dead_at_return: Vec<bool>,
    arg_has_dead_in_cleanup: Vec<bool>,
    arg_is_used: Vec<bool>,
    return_aliases_args: Vec<usize>,
}

impl PointerSummary {
    fn new(arg_count: usize) -> Self {
        Self {
            return_has_dead: false,
            arg_has_dead_at_return: vec![false; arg_count + 1],
            arg_has_dead_in_cleanup: vec![false; arg_count + 1],
            arg_is_used: vec![false; arg_count + 1],
            return_aliases_args: Vec::new(),
        }
    }

    fn record_return(&mut self, state: &PointerState) {
        self.return_has_dead |= state.has_dead_reachable_filtered(mir::RETURN_PLACE.as_usize(), false);
        for arg in 1..self.arg_has_dead_at_return.len() {
            self.arg_has_dead_at_return[arg] |=
                state.nodes[arg].kind == PointerKind::RawPtr && state.has_dead_reachable_filtered(arg, true);
            if state.aliases_root(mir::RETURN_PLACE.as_usize(), arg)
                && let Err(index) = self.return_aliases_args.binary_search(&arg)
            {
                self.return_aliases_args.insert(index, arg);
            }
        }
    }

    fn record_cleanup(&mut self, state: &PointerState) {
        for arg in 1..self.arg_has_dead_in_cleanup.len() {
            self.arg_has_dead_in_cleanup[arg] |=
                state.nodes[arg].kind == PointerKind::RawPtr && state.has_dead_reachable_filtered(arg, true);
        }
    }

    fn record_use(&mut self, state: &PointerState, node: usize) {
        for arg in 1..self.arg_is_used.len() {
            self.arg_is_used[arg] |= state.originates_from_root(node, arg);
        }
    }
}

#[derive(Clone, Debug)]
enum CachedPointerSummary {
    Available(PointerSummary),
    NoMir,
    Unavailable,
}

#[derive(Debug, Default)]
pub(crate) struct PointerSummaryCache {
    summaries: RefCell<FxHashMap<DefId, CachedPointerSummary>>,
    in_progress: RefCell<FxHashSet<DefId>>,
}

impl PointerSummaryCache {
    fn store(&self, def_id: DefId, summary: CachedPointerSummary) -> CachedPointerSummary {
        self.summaries.borrow_mut().insert(def_id, summary.clone());
        summary
    }

    fn summary_for<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        def_id: DefId,
    ) -> CachedPointerSummary {
        if let Some(summary) = self.summaries.borrow().get(&def_id).cloned() {
            return summary;
        }

        let Some(local_def_id) = def_id.as_local() else {
            return self.store(def_id, CachedPointerSummary::NoMir);
        };
        if !tcx.is_mir_available(local_def_id) || tcx.generics_of(local_def_id).requires_monomorphization(tcx) {
            return self.store(def_id, CachedPointerSummary::NoMir);
        }

        {
            let mut in_progress = self.in_progress.borrow_mut();
            if !in_progress.insert(def_id) {
                return CachedPointerSummary::Unavailable;
            }
        }

        let body = tcx.optimized_mir(local_def_id);
        if body.basic_blocks.len() > MAX_SUMMARY_BLOCKS || body.local_decls.len() > MAX_SUMMARY_LOCALS {
            self.in_progress.borrow_mut().remove(&def_id);
            return self.store(def_id, CachedPointerSummary::NoMir);
        }
        let summary = Analyzer::with_summary(tcx, typing_env, body, self)
            .run_summary()
            .map(CachedPointerSummary::Available)
            .unwrap_or(CachedPointerSummary::Unavailable);

        self.in_progress.borrow_mut().remove(&def_id);
        self.store(def_id, summary)
    }
}

#[derive(Clone, Copy)]
struct PointerStateQuery {
    location: mir::Location,
    local: Local,
    pointer_only: bool,
    explore_all_successors: bool,
}

struct Analyzer<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &'a mir::Body<'tcx>,
    summary_cache: &'a PointerSummaryCache,
    cfg: ControlFlowInfo,
    summary: Option<PointerSummary>,
    visit_times: usize,
    visit_limit_reached: bool,
    query_result: bool,
    query: Option<PointerStateQuery>,
}

impl<'a, 'tcx> Analyzer<'a, 'tcx> {
    fn with_query(
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        body: &'a mir::Body<'tcx>,
        summary_cache: &'a PointerSummaryCache,
        query: Option<PointerStateQuery>,
    ) -> Self {
        Self {
            tcx,
            typing_env,
            body,
            summary_cache,
            cfg: ControlFlowInfo::new(body),
            summary: None,
            visit_times: 0,
            visit_limit_reached: false,
            query_result: false,
            query,
        }
    }

    fn with_summary(
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        body: &'a mir::Body<'tcx>,
        summary_cache: &'a PointerSummaryCache,
    ) -> Self {
        let mut this = Self::with_query(tcx, typing_env, body, summary_cache, None);
        this.summary = Some(PointerSummary::new(body.arg_count));
        this
    }

    fn run_query(mut self) -> bool {
        let state = PointerState::new(self.tcx, self.typing_env, self.body);
        self.visit_block(mir::START_BLOCK, state);
        !self.visit_limit_reached && self.query_result
    }

    fn run_summary(mut self) -> Option<PointerSummary> {
        if !super::should_check_dangling_fn(self.tcx, self.body) {
            return None;
        }
        let state = PointerState::new(self.tcx, self.typing_env, self.body);
        self.visit_block(mir::START_BLOCK, state);
        if self.visit_limit_reached {
            return None;
        }
        self.summary.take()
    }

    fn visit_block(&mut self, bb: BasicBlock, mut state: PointerState) {
        self.visit_times += 1;
        if self.visit_times > MAX_VISITS {
            self.visit_limit_reached = true;
            return;
        }

        let root = self.cfg.father[bb.as_usize()];
        let in_collapsed_scc = !self.cfg.blocks[root.as_usize()].sub_blocks.is_empty();
        if in_collapsed_scc && bb != root {
            return;
        }

        let mut cleanup_branches = Vec::new();
        if self.process_component(root, &mut state, &mut cleanup_branches) {
            return;
        }

        for (successor, cleanup_state) in cleanup_branches {
            let successor = self.cfg.father[successor.as_usize()];
            self.visit_block(successor, cleanup_state);
        }

        if in_collapsed_scc
            && self.use_fixed_switch_successor()
            && let Some(successor) = self.fixed_switch_successor(root, &mut state)
            && self.cfg.father[successor.as_usize()] != root
        {
            self.visit_block(successor, state);
            return;
        }

        let next = self.cfg.blocks[root.as_usize()].next.clone();
        match next.as_slice() {
            [] => {},
            [successor] => self.visit_block(*successor, state),
            _ => {
                if self.use_fixed_switch_successor()
                    && let Some(successor) = self.fixed_switch_successor(root, &mut state)
                {
                    self.visit_block(successor, state);
                    return;
                }
                for successor in next {
                    if self.visit_times > MAX_VISITS {
                        self.visit_limit_reached = true;
                        return;
                    }
                    self.visit_block(successor, state.clone());
                }
            },
        }
    }

    fn process_component(
        &mut self,
        root: BasicBlock,
        state: &mut PointerState,
        cleanup_branches: &mut Vec<(BasicBlock, PointerState)>,
    ) -> bool {
        if self.process_block(root, root, state, cleanup_branches) {
            return true;
        }
        for sub_block in self.cfg.blocks[root.as_usize()].sub_blocks.clone() {
            if self.process_block(root, sub_block, state, cleanup_branches) {
                return true;
            }
        }
        false
    }

    fn use_fixed_switch_successor(&self) -> bool {
        !self.query.is_some_and(|query| query.explore_all_successors)
    }

    fn process_block(
        &mut self,
        component: BasicBlock,
        bb: BasicBlock,
        state: &mut PointerState,
        cleanup_branches: &mut Vec<(BasicBlock, PointerState)>,
    ) -> bool {
        let data = &self.body.basic_blocks[bb];

        // SafeDrop classifies a block first, then evaluates aliases, calls, and drops in that
        // order. Keep the same phase order so queries observe the same state boundary.
        for (statement_index, statement) in data.statements.iter().enumerate() {
            let location = mir::Location {
                block: bb,
                statement_index,
            };
            if self.record_dead_before(location, state) {
                return true;
            }
            self.record_statement_uses(statement, location, state);
            self.transfer_statement(statement, state);
        }

        let location = mir::Location {
            block: bb,
            statement_index: data.statements.len(),
        };

        let Some(terminator) = &data.terminator else {
            return self.record_dead_before(location, state);
        };
        self.record_summary_before_terminator(data.is_cleanup, terminator, state);

        match &terminator.kind {
            TerminatorKind::Call {
                func,
                args,
                destination,
                ..
            } => {
                if self.record_dead_before(location, state) {
                    return true;
                }
                if let Some(cleanup) = call_cleanup_successor(terminator)
                    && self.cfg.father[cleanup.as_usize()] != component
                {
                    let mut cleanup_state = state.clone();
                    if self.transfer_call_cleanup(func, args, &mut cleanup_state) {
                        cleanup_branches.push((cleanup, cleanup_state));
                    }
                } else if self.summary.is_some()
                    && matches!(
                        terminator.kind,
                        TerminatorKind::Call {
                            unwind: mir::UnwindAction::Continue,
                            ..
                        }
                    )
                {
                    let mut cleanup_state = state.clone();
                    self.transfer_call_cleanup(func, args, &mut cleanup_state);
                    if let Some(summary) = &mut self.summary {
                        summary.record_cleanup(&cleanup_state);
                    }
                }
                self.transfer_call(func, args, *destination, state);
            },
            TerminatorKind::Drop { place, .. } => {
                if self.record_dead_before(location, state) {
                    return true;
                }
                if let Some(node) = self.node_for_place(state, *place, false) {
                    if let Some(summary) = &mut self.summary {
                        summary.record_use(state, node);
                    }
                    state.dead_node(node);
                }
            },
            _ => {
                if self.record_dead_before(location, state) {
                    return true;
                }
            },
        }
        false
    }

    fn record_summary_before_terminator(
        &mut self,
        is_cleanup: bool,
        terminator: &mir::Terminator<'tcx>,
        state: &PointerState,
    ) {
        let Some(summary) = &mut self.summary else {
            return;
        };
        if !is_cleanup && matches!(terminator.kind, TerminatorKind::Return) {
            summary.record_return(state);
        }
        if is_cleanup {
            summary.record_cleanup(state);
        }
    }

    fn record_dead_before(&mut self, location: mir::Location, state: &PointerState) -> bool {
        let Some(query) = self.query else {
            return false;
        };
        if query.location == location {
            let local = query.local.as_usize();
            let is_dead = state.has_dead_reachable_filtered(local, query.pointer_only);
            self.query_result |= is_dead;
            return true;
        }
        false
    }

    fn assign_place_from_place(
        &self,
        state: &mut PointerState,
        left: Place<'tcx>,
        right: Place<'tcx>,
        allow_owned_raw_deref: bool,
        weak_update: bool,
    ) {
        let root = state.rhs_alias_representative(right.local.as_usize(), true);
        if state.nodes[root].children.contains_key(&ProjectionKey::Deref)
            && right
                .projection
                .iter()
                .all(|projection| matches!(projection, ProjectionElem::Field(..)))
            && let Some(pointee) = state.nodes[root].children.get(&ProjectionKey::Deref).copied()
            && let Some(left) = state.prepare_place_for_assignment(self.tcx, self.typing_env, left, weak_update)
        {
            state.merge_alias(left, pointee);
            return;
        }
        state.assign_place_from_place(
            self.tcx,
            self.typing_env,
            left,
            right,
            allow_owned_raw_deref,
            weak_update,
        );
    }

    fn reset_place_for_assignment(&self, state: &mut PointerState, place: Place<'tcx>) -> Option<usize> {
        state.reset_place_for_assignment(self.tcx, self.typing_env, place)
    }

    fn node_for_place(&self, state: &mut PointerState, place: Place<'tcx>, is_rhs: bool) -> Option<usize> {
        state.node_for_place(self.tcx, self.typing_env, place, is_rhs)
    }

    fn record_statement_uses(
        &mut self,
        statement: &mir::Statement<'tcx>,
        location: mir::Location,
        state: &PointerState,
    ) {
        let Some(summary) = &mut self.summary else {
            return;
        };
        let mut collector = DerefUseCollector { state, summary };
        collector.visit_statement(statement, location);
    }

    fn record_call_arg_uses(
        &mut self,
        callee_summary: Option<&CachedPointerSummary>,
        arg_nodes: &[Option<usize>],
        state: &PointerState,
    ) {
        let Some(summary) = &mut self.summary else {
            return;
        };
        for (index, node) in arg_nodes.iter().copied().enumerate() {
            let is_used = match callee_summary {
                Some(CachedPointerSummary::Available(summary)) => {
                    summary.arg_is_used.get(index + 1).copied().unwrap_or(false)
                },
                _ => true,
            };
            if is_used && let Some(node) = node {
                summary.record_use(state, node);
            }
        }
    }

    fn transfer_statement(&self, statement: &mir::Statement<'tcx>, state: &mut PointerState) {
        let mir::StatementKind::Assign(box (left, right)) = &statement.kind else {
            return;
        };

        match right {
            Rvalue::Use(Operand::Copy(right)) | Rvalue::Cast(_, Operand::Copy(right), _) => {
                self.assign_place_from_place(
                    state,
                    *left,
                    *right,
                    self.is_inlined_ptr_operation(statement.source_info.scope, "::ptr::read"),
                    self.is_inlined_ptr_operation(statement.source_info.scope, "::ptr::write"),
                );
            },
            Rvalue::Use(Operand::Move(right)) | Rvalue::Cast(_, Operand::Move(right), _) => {
                self.assign_place_from_place(
                    state,
                    *left,
                    *right,
                    self.is_inlined_ptr_operation(statement.source_info.scope, "::ptr::read"),
                    self.is_inlined_ptr_operation(statement.source_info.scope, "::ptr::write"),
                );
                if is_direct_return_place(*left) {
                    state.mark_place_moved_out(self.tcx, self.typing_env, *right);
                }
                if self.is_moved_out_argument_variant(*right) {
                    state.mark_moved_out_place(self.tcx, self.typing_env, *right);
                }
            },
            Rvalue::Use(Operand::Constant(konst)) => {
                let Some(left) = self.reset_place_for_assignment(state, *left) else {
                    return;
                };
                if let Some(value) = const_to_u128(self.tcx, self.typing_env, konst) {
                    state.set_constant(left, value);
                }
            },
            Rvalue::Ref(_, _, right) | Rvalue::RawPtr(_, right) => {
                state.assign_reference_from_place(self.tcx, self.typing_env, *left, *right);
            },
            Rvalue::CopyForDeref(right) => {
                self.assign_place_from_place(state, *left, *right, false, false);
            },
            Rvalue::BinaryOp(mir::BinOp::Offset, box (Operand::Copy(right), _)) => {
                self.assign_place_from_place(state, *left, *right, false, false);
            },
            Rvalue::BinaryOp(mir::BinOp::Offset, box (Operand::Move(right), _)) => {
                self.assign_place_from_place(state, *left, *right, false, false);
                if self.is_moved_out_argument_variant(*right) {
                    state.mark_moved_out_place(self.tcx, self.typing_env, *right);
                }
            },
            Rvalue::ShallowInitBox(Operand::Copy(right), _) => {
                state.assign_shallow_init_box(self.tcx, self.typing_env, *left, *right);
            },
            Rvalue::ShallowInitBox(Operand::Move(right), _) => {
                state.assign_shallow_init_box(self.tcx, self.typing_env, *left, *right);
                if self.is_moved_out_argument_variant(*right) {
                    state.mark_moved_out_place(self.tcx, self.typing_env, *right);
                }
            },
            Rvalue::Aggregate(box kind, operands) => {
                if matches!(kind, mir::AggregateKind::RawPtr(_, _)) {
                    if let Some(Operand::Copy(right) | Operand::Move(right)) = operands.iter().next() {
                        self.assign_place_from_place(state, *left, *right, false, false);
                    }
                    return;
                }
                let Some(_) = self.reset_place_for_assignment(state, *left) else {
                    return;
                };
                if matches!(kind, mir::AggregateKind::Array(_)) {
                    return;
                }
                for (field, operand) in operands.iter_enumerated() {
                    let right_place = match operand {
                        Operand::Copy(right) | Operand::Move(right) => *right,
                        Operand::Constant(_) => continue,
                    };
                    let field = match kind {
                        mir::AggregateKind::Adt(_, _, _, _, Some(field)) => *field,
                        _ => field,
                    };
                    let field_ty = operand.ty(&self.body.local_decls, self.tcx);
                    let left_field = left.project_deeper(&[ProjectionElem::Field(field, field_ty)], self.tcx);
                    let Some(left) = self.node_for_place(state, left_field, false) else {
                        continue;
                    };
                    let Some(right_node) = self.node_for_place(state, right_place, true) else {
                        continue;
                    };
                    state.merge_alias(left, right_node);
                }
            },
            _ => {
                self.reset_place_for_assignment(state, *left);
            },
        }
    }

    fn is_inlined_ptr_operation(&self, mut scope: mir::SourceScope, suffix: &str) -> bool {
        loop {
            let scope_data = &self.body.source_scopes[scope];
            if let Some((instance, _)) = scope_data.inlined {
                let path = self.tcx.def_path_str(instance.def.def_id());
                if path.ends_with(suffix) {
                    return true;
                }
            }
            let Some(parent_scope) = scope_data.inlined_parent_scope else {
                return false;
            };
            scope = parent_scope;
        }
    }

    fn is_moved_out_argument_variant(&self, place: Place<'tcx>) -> bool {
        let local = place.local.as_usize();
        (1..=self.body.arg_count).contains(&local) && is_field_move(place)
    }

    fn transfer_call(
        &mut self,
        func: &Operand<'tcx>,
        args: &[Spanned<Operand<'tcx>>],
        destination: Place<'tcx>,
        state: &mut PointerState,
    ) {
        let def_id = callee_def_id(func);
        let path = def_id.map(|def_id| self.tcx.def_path_str(def_id));
        let path = path.as_deref();
        let arg_nodes = self.arg_nodes(args, state);
        let callee_summary = self.cached_callee_summary(def_id);
        self.record_call_arg_uses(callee_summary.as_ref(), &arg_nodes, state);

        if path.is_some_and(is_forget_path) {
            if let Some(arg) = args.first() {
                match arg.node {
                    Operand::Copy(place) | Operand::Move(place) => {
                        _ = state.reset_place_for_assignment(self.tcx, self.typing_env, place);
                    },
                    Operand::Constant(_) => {},
                }
            }
            _ = self.reset_place_for_assignment(state, destination);
            return;
        }

        if path.is_some_and(is_free_like_path)
            && let Some(node) = arg_node(&arg_nodes, 0)
        {
            state.dead_node(node);
        }

        if let Some(box_call) = path.and_then(box_call_kind) {
            let Some(destination_node) = self.reset_place_for_assignment(state, destination) else {
                return;
            };
            if let Some(arg_node) = arg_node(&arg_nodes, 0) {
                let (parent, kind) = match box_call {
                    BoxCallKind::New => (destination_node, PointerKind::RawPtr),
                    BoxCallKind::IntoRaw | BoxCallKind::FromRaw => (arg_node, PointerKind::Other),
                };
                let pointee =
                    state.ensure_child(parent, ProjectionKey::Deref, state.nodes[parent].root, kind, true, true);
                match box_call {
                    BoxCallKind::New => state.merge_alias(pointee, arg_node),
                    BoxCallKind::IntoRaw | BoxCallKind::FromRaw => {
                        state.merge_alias(destination_node, pointee);
                    },
                }
            }
            return;
        }

        let Some(destination_node) = self.reset_place_for_assignment(state, destination) else {
            return;
        };
        let use_callee_result = should_apply_safedrop_call_result(path, destination_node, &arg_nodes, state);

        if path.is_some_and(is_ptr_read_path)
            && let Some(ptr_node) = arg_node(&arg_nodes, 0)
        {
            let ty = destination.ty(&self.body.local_decls, self.tcx).ty;
            let (kind, need_drop, tracked) = pointer_node_props(self.tcx, self.typing_env, ty);
            let pointee = state.ensure_child(
                ptr_node,
                ProjectionKey::Deref,
                state.nodes[ptr_node].root,
                kind,
                need_drop,
                tracked,
            );
            state.merge_alias(destination_node, pointee);
        } else if path.is_some_and(is_replace_path)
            && let Some(place_node) = arg_node(&arg_nodes, 0)
        {
            let ty = destination.ty(&self.body.local_decls, self.tcx).ty;
            let (kind, need_drop, tracked) = pointer_node_props(self.tcx, self.typing_env, ty);
            let old_value_node = state.ensure_child(
                place_node,
                ProjectionKey::Deref,
                state.nodes[place_node].root,
                kind,
                need_drop,
                tracked,
            );
            if let Some(new_value_node) = arg_node(&arg_nodes, 1) {
                state.merge_alias(destination_node, old_value_node);
                state.reset_node_for_assignment(old_value_node);
                state.merge_alias(old_value_node, new_value_node);
            } else {
                state.merge_alias(destination_node, old_value_node);
            }
        } else if !(use_callee_result
            && path.is_some()
            && self.apply_callee_result(callee_summary.as_ref(), &arg_nodes, destination_node, state))
            && let Some(path) = path
            && let Some(arg_node) = transparent_return_arg_node(path, destination_node, &arg_nodes, state)
        {
            state.merge_alias(destination_node, arg_node);
        }

        if let Some(CachedPointerSummary::Available(summary)) = callee_summary.as_ref() {
            self.apply_callee_arg_effects(&summary.arg_has_dead_at_return, &arg_nodes, state);
        }
    }

    fn apply_callee_result(
        &self,
        summary: Option<&CachedPointerSummary>,
        arg_nodes: &[Option<usize>],
        destination_node: usize,
        state: &mut PointerState,
    ) -> bool {
        match summary {
            Some(CachedPointerSummary::Available(summary)) => {
                for arg in summary.return_aliases_args.iter().copied() {
                    if let Some(arg_node) = arg.checked_sub(1).and_then(|index| arg_node(arg_nodes, index)) {
                        state.merge_alias(destination_node, arg_node);
                    }
                }
                if summary.return_has_dead {
                    state.add_summary_dead_reachable(destination_node);
                }
                true
            },
            Some(CachedPointerSummary::Unavailable) => true,
            Some(CachedPointerSummary::NoMir) | None => false,
        }
    }

    fn cached_callee_summary(&self, def_id: Option<DefId>) -> Option<CachedPointerSummary> {
        def_id.map(|def_id| self.summary_cache.summary_for(self.tcx, self.typing_env, def_id))
    }

    fn apply_callee_arg_effects(
        &self,
        effects: &[bool],
        arg_nodes: &[Option<usize>],
        state: &mut PointerState,
    ) -> bool {
        let mut applied = false;
        for (index, dead) in effects.iter().copied().enumerate().skip(1) {
            if !dead {
                continue;
            }
            let Some(node) = arg_node(arg_nodes, index - 1) else {
                continue;
            };
            state.dead_node(node);
            applied = true;
        }
        applied
    }

    fn transfer_call_cleanup(
        &self,
        func: &Operand<'tcx>,
        args: &[Spanned<Operand<'tcx>>],
        state: &mut PointerState,
    ) -> bool {
        let arg_nodes = self.arg_nodes(args, state);
        let Some(CachedPointerSummary::Available(summary)) = self.cached_callee_summary(callee_def_id(func)) else {
            return false;
        };
        self.apply_callee_arg_effects(&summary.arg_has_dead_in_cleanup, &arg_nodes, state)
    }

    fn arg_nodes(&self, args: &[Spanned<Operand<'tcx>>], state: &mut PointerState) -> Vec<Option<usize>> {
        args.iter()
            .map(|arg| match arg.node {
                Operand::Copy(place) | Operand::Move(place) => self.node_for_place(state, place, true),
                Operand::Constant(_) => None,
            })
            .collect()
    }

    fn fixed_switch_successor(&self, bb: BasicBlock, state: &mut PointerState) -> Option<BasicBlock> {
        let terminator = self.body.basic_blocks[bb].terminator.as_ref()?;
        let TerminatorKind::SwitchInt { discr, targets } = &terminator.kind else {
            return None;
        };
        let value = match discr {
            Operand::Constant(konst) => const_to_u128(self.tcx, self.typing_env, konst)?,
            Operand::Copy(place) | Operand::Move(place) => {
                let node = state.node_for_place(self.tcx, self.typing_env, *place, true)?;
                state.constant_for(node)?
            },
        };
        for (target_value, target) in targets.iter() {
            if target_value == value {
                return Some(target);
            }
        }
        Some(targets.otherwise())
    }
}

pub(crate) fn analyze_pointer_state_query<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    summary_cache: &PointerSummaryCache,
    location: mir::Location,
    local: Local,
    pointer_only: bool,
) -> bool {
    let query = PointerStateQuery {
        location,
        local,
        pointer_only,
        explore_all_successors: body.basic_blocks[location.block].is_cleanup,
    };
    Analyzer::with_query(tcx, typing_env, body, summary_cache, Some(query)).run_query()
}

pub(crate) fn call_argument_is_used<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    summary_cache: &PointerSummaryCache,
    location: mir::Location,
    local: Local,
) -> bool {
    let data = &body.basic_blocks[location.block];
    if location.statement_index != data.statements.len() {
        return false;
    }
    let Some(mir::Terminator {
        kind: TerminatorKind::Call { func, args, .. },
        ..
    }) = &data.terminator
    else {
        return false;
    };
    let Some(def_id) = callee_def_id(func) else {
        return false;
    };
    if def_id.as_local().is_none() {
        return false;
    }
    let CachedPointerSummary::Available(summary) = summary_cache.summary_for(tcx, typing_env, def_id) else {
        return false;
    };
    args.iter().enumerate().any(|(index, arg)| {
        matches!(
            arg.node,
            Operand::Copy(place) | Operand::Move(place) if place.local == local
        ) && summary.arg_is_used.get(index + 1).copied().unwrap_or(false)
    })
}

fn const_to_u128<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    konst: &mir::ConstOperand<'tcx>,
) -> Option<u128> {
    konst
        .const_
        .try_eval_scalar_int(tcx, typing_env)
        .map(|value| value.to_bits_unchecked())
}

fn callee_def_id(func: &Operand<'_>) -> Option<DefId> {
    let Operand::Constant(box mir::ConstOperand {
        const_: mir::Const::Val(mir::ConstValue::ZeroSized, ty),
        ..
    }) = func
    else {
        return None;
    };
    let ty::FnDef(def_id, _) = *ty.kind() else {
        return None;
    };
    Some(def_id)
}

fn is_free_like_path(path: &str) -> bool {
    path.ends_with("::ptr::drop_in_place")
        || path.ends_with("::alloc::dealloc")
        || path.ends_with("::__rust_dealloc")
        || matches!(path.rsplit("::").next(), Some("release" | "destroy"))
}

fn is_safedrop_unchecked_callee_path(path: &str) -> bool {
    ["drop", "dealloc", "release", "destroy"]
        .into_iter()
        .any(|needle| path.contains(needle))
}

fn is_ptr_read_path(path: &str) -> bool {
    path.ends_with("::ptr::read") || path.ends_with("::ptr::read_unaligned") || path.ends_with("::ptr::read_volatile")
}

fn is_forget_path(path: &str) -> bool {
    path.ends_with("::mem::forget")
}

fn is_replace_path(path: &str) -> bool {
    path.ends_with("::mem::replace") || path.ends_with("::ptr::replace")
}

fn is_transparent_first_arg_return_path(path: &str) -> bool {
    matches!(
        path.rsplit("::").next(),
        Some(
            "ok" | "take"
                | "expect"
                | "into_inner"
                | "with_header"
                | "cast"
                | "as_ptr"
                | "as_mut_ptr"
                | "add"
                | "wrapping_add"
                | "byte_add"
                | "from_raw_parts"
                | "from_raw_parts_in"
        )
    ) || matches!(box_call_kind(path), Some(BoxCallKind::FromRaw))
}

fn is_raw_constructor_return_path(path: &str) -> bool {
    matches!(path.rsplit("::").next(), Some("from_raw_parts" | "from_raw_parts_in"))
        || matches!(box_call_kind(path), Some(BoxCallKind::FromRaw))
}

#[derive(Clone, Copy)]
enum BoxCallKind {
    New,
    IntoRaw,
    FromRaw,
}

fn box_call_kind(path: &str) -> Option<BoxCallKind> {
    if !path.contains("::boxed::") {
        return None;
    }
    match path.rsplit("::").next()? {
        "new" => Some(BoxCallKind::New),
        "into_raw" | "into_raw_with_allocator" => Some(BoxCallKind::IntoRaw),
        "from_raw" => Some(BoxCallKind::FromRaw),
        _ => None,
    }
}

fn should_apply_safedrop_call_result(
    path: Option<&str>,
    destination_node: usize,
    arg_nodes: &[Option<usize>],
    state: &PointerState,
) -> bool {
    state.nodes[destination_node].tracked
        && (arg_nodes.iter().flatten().any(|node| state.nodes[*node].tracked)
            || path.is_some_and(is_safedrop_unchecked_callee_path))
}

fn transparent_return_arg_node(
    path: &str,
    destination_node: usize,
    arg_nodes: &[Option<usize>],
    state: &PointerState,
) -> Option<usize> {
    if !is_transparent_first_arg_return_path(path) {
        return None;
    }
    if is_raw_constructor_return_path(path) {
        return arg_node(arg_nodes, 0);
    }
    if !matches!(
        state.nodes[destination_node].kind,
        PointerKind::RawPtr | PointerKind::Ref
    ) {
        return None;
    }
    single_tracked_arg_node(arg_nodes, state)
}

fn single_tracked_arg_node(arg_nodes: &[Option<usize>], state: &PointerState) -> Option<usize> {
    let mut tracked_args = arg_nodes
        .iter()
        .copied()
        .flatten()
        .filter(|node| state.nodes[*node].tracked);
    let node = tracked_args.next()?;
    tracked_args.next().is_none().then_some(node)
}

fn arg_node(arg_nodes: &[Option<usize>], index: usize) -> Option<usize> {
    arg_nodes.get(index).copied().flatten()
}

fn is_direct_deref(place: Place<'_>) -> bool {
    place.projection.len() == 1 && matches!(place.projection[0], ProjectionElem::Deref)
}

fn is_field_move(place: Place<'_>) -> bool {
    let mut has_field = false;
    let mut has_downcast = false;
    for projection in place.projection.iter() {
        match projection {
            ProjectionElem::Deref => return false,
            ProjectionElem::Field(..) => has_field = true,
            ProjectionElem::Downcast(..) => has_downcast = true,
            _ => {},
        }
    }
    has_field && has_downcast
}

fn is_direct_return_place(place: Place<'_>) -> bool {
    place.local == mir::RETURN_PLACE && place.projection.is_empty()
}

fn pointer_kind(ty: Ty<'_>) -> PointerKind {
    match ty.kind() {
        ty::RawPtr(_, _) => PointerKind::RawPtr,
        ty::Ref(_, _, _) => PointerKind::Ref,
        ty::Tuple(_) => PointerKind::Tuple,
        ty::Adt(adt_def, _) if is_corner_adt(format!("{adt_def:?}")) => PointerKind::CornerAdt,
        _ => PointerKind::Other,
    }
}

fn pointer_node_props<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    ty: Ty<'tcx>,
) -> (PointerKind, bool, bool) {
    let need_drop = ty.needs_drop(tcx, typing_env);
    let tracked = need_drop || !is_plain_value(ty);
    (pointer_kind(ty), need_drop, tracked)
}

fn is_plain_value(ty: Ty<'_>) -> bool {
    match ty.kind() {
        ty::Bool | ty::Char | ty::Int(_) | ty::Uint(_) | ty::Float(_) => true,
        ty::Array(ty, _) | ty::Slice(ty) => is_plain_value(*ty),
        ty::Adt(_, args) => args.types().all(is_plain_value),
        ty::Tuple(tys) => tys.iter().all(is_plain_value),
        _ => false,
    }
}

fn is_corner_adt(name: String) -> bool {
    name.contains("cell::RefMut")
        || name.contains("cell::Ref")
        || name.contains("rc::Rc")
        // Lock guards borrow the protected value; dropping a guard only releases the lock.
        || name.contains("MutexGuard")
        || name.contains("RwLockReadGuard")
        || name.contains("RwLockWriteGuard")
}
