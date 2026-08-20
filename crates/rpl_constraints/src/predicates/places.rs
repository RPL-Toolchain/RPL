use rustc_data_structures::fx::FxHashSet;
use rustc_middle::mir::{self, Operand, PlaceRef, ProjectionElem, Rvalue, StatementKind, TerminatorKind};
use rustc_middle::ty::{self, TyCtxt};

use crate::predicates::BodyInfoCache;

pub type MultiplePlacesPredsFnPtr = for<'tcx> fn(
    TyCtxt<'tcx>,
    ty::TypingEnv<'tcx>,
    &mir::Body<'tcx>,
    &BodyInfoCache<'tcx>,
    Vec<PlaceRef<'tcx>>,
) -> bool;

/// Structural key for comparing places after peeling Use/Copy/Move temps.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PlaceKey {
    Local(mir::Local),
    /// Canonical place built from a root local and projection keys.
    Proj(mir::Local, Vec<ProjKey>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ProjKey {
    Deref,
    Field(u32),
    Index(u64),
    ConstantIndex {
        offset: u64,
        min_length: u64,
        from_end: bool,
    },
    Subslice {
        from: u64,
        to: u64,
        from_end: bool,
    },
    Downcast(u32),
    Opaque,
}

/// Returns true if the computation of `$cond` mentions `$src` (or an alias of it).
#[instrument(level = "debug", skip(tcx, typing_env, body, _cache), ret)]
pub fn mentions_place<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    _cache: &BodyInfoCache<'tcx>,
    places: Vec<PlaceRef<'tcx>>,
) -> bool {
    let [cond, src] = places.as_slice() else {
        debug!(len = places.len(), "mentions_place expects exactly two places");
        return false;
    };
    let src_key = place_key(tcx, typing_env, body, *src);
    let mut visited = FxHashSet::default();
    place_mentions(tcx, typing_env, body, *cond, &src_key, &mut visited)
}

fn place_key<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    place: PlaceRef<'tcx>,
) -> PlaceKey {
    let (root, projs) = peel_copies(tcx, typing_env, body, place);
    if projs.is_empty() {
        PlaceKey::Local(root)
    } else {
        PlaceKey::Proj(root, projs)
    }
}

fn peel_copies<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    place: PlaceRef<'tcx>,
) -> (mir::Local, Vec<ProjKey>) {
    let mut local = place.local;
    let mut projs: Vec<ProjKey> = place
        .projection
        .iter()
        .map(|elem| proj_key(tcx, typing_env, body, elem))
        .collect();
    let mut visited = FxHashSet::default();
    while visited.insert(local) {
        let Some(origin) = single_copy_origin(body, local) else {
            break;
        };
        local = origin.local;
        let mut prefix: Vec<ProjKey> = origin
            .projection
            .iter()
            .map(|e| proj_key(tcx, typing_env, body, e))
            .collect();
        prefix.append(&mut projs);
        projs = prefix;
    }
    (local, projs)
}

fn proj_key<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    elem: &ProjectionElem<mir::Local, ty::Ty<'tcx>>,
) -> ProjKey {
    match *elem {
        ProjectionElem::Deref => ProjKey::Deref,
        ProjectionElem::Field(f, _) => ProjKey::Field(f.as_u32()),
        ProjectionElem::Index(local) => {
            if let Some(n) = const_local_u64(tcx, typing_env, body, local) {
                ProjKey::Index(n)
            } else {
                ProjKey::Opaque
            }
        },
        ProjectionElem::ConstantIndex {
            offset,
            min_length,
            from_end,
        } => ProjKey::ConstantIndex {
            offset,
            min_length,
            from_end,
        },
        ProjectionElem::Subslice { from, to, from_end } => ProjKey::Subslice { from, to, from_end },
        ProjectionElem::Downcast(_, v) => ProjKey::Downcast(v.as_u32()),
        _ => ProjKey::Opaque,
    }
}

fn const_local_u64<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    local: mir::Local,
) -> Option<u64> {
    for block in body.basic_blocks.iter() {
        for stmt in &block.statements {
            if let StatementKind::Assign(box (lhs, Rvalue::Use(Operand::Constant(c)))) = &stmt.kind
                && lhs.as_local() == Some(local)
            {
                return c.const_.try_eval_bits(tcx, typing_env).map(|b| b as u64);
            }
        }
    }
    None
}

fn single_copy_origin<'tcx>(body: &mir::Body<'tcx>, local: mir::Local) -> Option<PlaceRef<'tcx>> {
    let mut found = None;
    for block in body.basic_blocks.iter() {
        for stmt in &block.statements {
            if let StatementKind::Assign(box (lhs, rhs)) = &stmt.kind
                && lhs.as_local() == Some(local)
            {
                let origin = match rhs {
                    Rvalue::Use(Operand::Copy(p) | Operand::Move(p)) => Some(p.as_ref()),
                    Rvalue::CopyForDeref(p) => Some(p.as_ref()),
                    _ => return None,
                };
                if found.is_some() {
                    return None;
                }
                found = origin;
            }
        }
    }
    found
}

fn places_match<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    a: PlaceRef<'tcx>,
    src_key: &PlaceKey,
) -> bool {
    let a_key = place_key(tcx, typing_env, body, a);
    if &a_key == src_key {
        return true;
    }
    // `&src` — peel one Deref from a
    if let PlaceKey::Proj(root, projs) = &a_key
        && let Some((last, rest)) = projs.split_last()
        && *last == ProjKey::Deref
    {
        let peeled = if rest.is_empty() {
            PlaceKey::Local(*root)
        } else {
            PlaceKey::Proj(*root, rest.to_vec())
        };
        return &peeled == src_key;
    }
    false
}

fn place_mentions<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    place: PlaceRef<'tcx>,
    src_key: &PlaceKey,
    visited: &mut FxHashSet<mir::Local>,
) -> bool {
    if places_match(tcx, typing_env, body, place, src_key) {
        return true;
    }
    // Follow reborrows: `(*_ref)` → look through `_ref = &place`
    if let [proj @ .., ProjectionElem::Deref] = place.projection {
        let inner = PlaceRef {
            local: place.local,
            projection: proj,
        };
        return place_mentions(tcx, typing_env, body, inner, src_key, visited);
    }
    let Some(local) = place.as_local() else {
        return false;
    };
    if !visited.insert(local) {
        return false;
    }
    for (bb_id, block) in body.basic_blocks.iter_enumerated() {
        for stmt in &block.statements {
            if let StatementKind::Assign(box (lhs, rhs)) = &stmt.kind
                && lhs.as_local() == Some(local)
                && rvalue_mentions(tcx, typing_env, body, rhs, src_key, visited)
            {
                return true;
            }
        }
        if let Some(term) = &block.terminator
            && let TerminatorKind::Call { args, destination, .. } = &term.kind
            && destination.as_local() == Some(local)
        {
            for arg in args {
                if operand_mentions(tcx, typing_env, body, &arg.node, src_key, visited) {
                    return true;
                }
            }
        }
        // Short-circuit `&&` / `||`: earlier operands appear as SwitchInt discriminants
        // on CFG edges into blocks that assign this local.
        if block_assigns_local(block, local) {
            for pred in body.basic_blocks.predecessors()[bb_id].iter() {
                if let Some(term) = &body.basic_blocks[*pred].terminator
                    && let TerminatorKind::SwitchInt { discr, .. } = &term.kind
                    && operand_mentions(tcx, typing_env, body, discr, src_key, visited)
                {
                    return true;
                }
            }
        }
    }
    false
}

fn block_assigns_local(block: &mir::BasicBlockData<'_>, local: mir::Local) -> bool {
    for stmt in &block.statements {
        if let StatementKind::Assign(box (lhs, _)) = &stmt.kind
            && lhs.as_local() == Some(local)
        {
            return true;
        }
    }
    if let Some(term) = &block.terminator
        && let TerminatorKind::Call { destination, .. } = &term.kind
        && destination.as_local() == Some(local)
    {
        return true;
    }
    false
}

fn rvalue_mentions<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    rvalue: &Rvalue<'tcx>,
    src_key: &PlaceKey,
    visited: &mut FxHashSet<mir::Local>,
) -> bool {
    match rvalue {
        Rvalue::Use(op) | Rvalue::Repeat(op, _) | Rvalue::Cast(_, op, _) | Rvalue::UnaryOp(_, op) => {
            operand_mentions(tcx, typing_env, body, op, src_key, visited)
        },
        Rvalue::Ref(_, _, place) | Rvalue::RawPtr(_, place) => {
            place_mentions(tcx, typing_env, body, place.as_ref(), src_key, visited)
        },
        Rvalue::BinaryOp(_, box (lhs, rhs)) => {
            operand_mentions(tcx, typing_env, body, lhs, src_key, visited)
                || operand_mentions(tcx, typing_env, body, rhs, src_key, visited)
        },
        Rvalue::Aggregate(_, ops) => ops
            .iter()
            .any(|op| operand_mentions(tcx, typing_env, body, op, src_key, visited)),
        Rvalue::Discriminant(place) | Rvalue::Len(place) => {
            place_mentions(tcx, typing_env, body, place.as_ref(), src_key, visited)
        },
        Rvalue::ShallowInitBox(op, _) => operand_mentions(tcx, typing_env, body, op, src_key, visited),
        _ => false,
    }
}

fn operand_mentions<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    op: &Operand<'tcx>,
    src_key: &PlaceKey,
    visited: &mut FxHashSet<mir::Local>,
) -> bool {
    match op {
        Operand::Copy(place) | Operand::Move(place) => {
            let place = place.as_ref();
            if places_match(tcx, typing_env, body, place, src_key) {
                return true;
            }
            place_mentions(tcx, typing_env, body, place, src_key, visited)
        },
        Operand::Constant(_) => false,
    }
}
