use rpl_constraints::predicates::{
    BodyInfoCache, PredicateArg, PredicateClause, PredicateConjunction, PredicateKind, PredicateTerm,
};
use rpl_constraints::{Const, Constraints};
use rpl_context::pat::{self, ConstVarIdx, LabelMap, PlaceVarIdx, Spanned, TyVarIdx};
use rpl_meta::symbol_table::MetaVariable;
use rustc_hir::def_id::DefId;
use rustc_middle::mir::{self, Operand, PlaceRef, TerminatorKind};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt};
use rustc_span::Symbol;

use crate::graph::{MirControlFlowGraph, MirDataDepGraph};
use crate::matches::{Matched, StatementMatch};
use crate::rudra_paths;

/// PredicateArgInstance is the matched instance of a [PredicateArg]
#[allow(unused)]
#[derive(Clone, Debug)]
enum PredicateArgInstance<'tcx> {
    Item(DefId),             // mapped from [PredicateArg::Item]
    Location(mir::Location), // mapped from [PredicateArg::Label]
    Local(mir::Local),       // mapped from [PredicateArg::Local]
    Ty(Ty<'tcx>),            // mapped from [PredicateArg::MetaVar]
    Const(Const<'tcx>),      // mapped from [PredicateArg::MetaVar]
    Place(PlaceRef<'tcx>),   // mapped from [PredicateArg::MetaVar]
    Path(Vec<Symbol>),       // mapped from [PredicateArg::Path]
}

pub struct PredicateEvaluator<'e, 'm, 'tcx> {
    // 'e means eval, 'm means meta
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    item: DefId,
    body: &'e mir::Body<'tcx>,
    label_map: &'e LabelMap,
    matched: &'e Matched<'tcx>,
    body_cache: &'e BodyInfoCache,
    symbol_table: &'e pat::FnSymbolTable<'m>,
    mir_ddg: Option<&'e MirDataDepGraph>,
    mir_cfg: Option<&'e MirControlFlowGraph>,
}

impl<'e, 'm, 'tcx> PredicateEvaluator<'e, 'm, 'tcx> {
    #[expect(
        clippy::too_many_arguments,
        reason = "This is a constructor, and must take all arguments"
    )]
    pub fn new(
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        item: DefId,
        body: &'e mir::Body<'tcx>,
        label_map: &'e LabelMap,
        matched: &'e Matched<'tcx>,
        body_cache: &'e BodyInfoCache,
        symbol_table: &'e pat::FnSymbolTable<'m>,
        mir_ddg: Option<&'e MirDataDepGraph>,
        mir_cfg: Option<&'e MirControlFlowGraph>,
    ) -> Self {
        Self {
            tcx,
            typing_env,
            item,
            body,
            label_map,
            matched,
            body_cache,
            symbol_table,
            mir_ddg,
            mir_cfg,
        }
    }

    #[instrument(level = "debug", skip(self), ret)]
    pub fn evaluate_constraint(&self, constraint: &'m Constraints) -> bool {
        constraint.preds.iter().all(|pred| self.evaluate_conjunction(pred))
        // FIX: we should possibly check attributes here
    }

    fn evaluate_conjunction(&self, conjunction: &'m PredicateConjunction) -> bool {
        conjunction.clauses.iter().all(|clause| self.evaluate_clause(clause))
    }

    fn evaluate_clause(&self, clause: &'m PredicateClause) -> bool {
        clause.terms.iter().any(|term| self.evaluate_term(term))
    }

    fn evaluate_term(&self, term: &'m PredicateTerm) -> bool {
        let mut arg_instance = Vec::new();
        for arg in term.args.iter() {
            let instance = self.instantiate_arg(arg).unwrap();
            arg_instance.push(instance);
        }
        let result = match term.kind {
            PredicateKind::Ty(p) => {
                assert!(
                    arg_instance.len() == 1,
                    "PredicateKind::Ty should have exactly one argument"
                );
                match &arg_instance[0] {
                    PredicateArgInstance::Ty(ty) => p(self.tcx, self.typing_env, *ty),
                    _ => panic!("PredicateArgInstance::Ty expected, got {:?}", arg_instance[0]),
                }
            },
            PredicateKind::MultipleTys(p) => {
                let mut args = Vec::new();
                for arg in arg_instance.iter() {
                    match arg {
                        PredicateArgInstance::Ty(ty) => args.push(*ty),
                        _ => panic!("PredicateArgInstance::Ty expected, got {:?}", arg),
                    }
                }
                p(self.tcx, self.typing_env, args)
            },
            PredicateKind::Fn(p) => {
                assert!(
                    arg_instance.len() == 1,
                    "PredicateKind::Fn should have exactly one argument"
                );
                match &arg_instance[0] {
                    PredicateArgInstance::Item(item) => item.as_local().is_some_and(|local| p(self.tcx, local)),
                    _ => panic!("PredicateArgInstance::Item expected, got {:?}", arg_instance[0]),
                }
            },
            PredicateKind::Translate(p) => {
                assert!(
                    arg_instance.len() == 2,
                    "PredicateKind::Translate should have exactly two arguments"
                );
                match (&arg_instance[0], &arg_instance[1]) {
                    (PredicateArgInstance::Location(loc), PredicateArgInstance::Path(path)) => {
                        p(*loc, path.clone(), self.tcx, self.body)
                    },
                    _ => panic!(
                        "PredicateArgInstance::Location and PredicateArgInstance::Path expected, got {:?} and {:?}",
                        &arg_instance[0], &arg_instance[1]
                    ),
                }
            },
            PredicateKind::Trivial(p) => p(),
            PredicateKind::TyConst(p) => {
                assert!(
                    arg_instance.len() == 2,
                    "PredicateKind::TyConst should have exactly two arguments"
                );
                match (&arg_instance[0], &arg_instance[1]) {
                    (PredicateArgInstance::Ty(ty), PredicateArgInstance::Const(konst)) => {
                        p(self.tcx, self.body, self.typing_env, *ty, *konst)
                    },
                    _ => panic!(
                        "PredicateArgInstance::Ty and PredicateArgInstance::Const expected, got {:?} and {:?}",
                        &arg_instance[0], &arg_instance[1]
                    ),
                }
            },
            PredicateKind::SingleConst(p) => {
                assert!(
                    arg_instance.len() == 1,
                    "PredicateKind::SingleConst should have exactly one argument"
                );
                match &arg_instance[0] {
                    PredicateArgInstance::Const(konst) => p(self.tcx, self.typing_env, *konst),
                    _ => panic!("PredicateArgInstance::Const expected, got {:?}", arg_instance[0]),
                }
            },
            PredicateKind::MultipleConsts(p) => {
                let mut args = Vec::new();
                for arg in arg_instance.iter() {
                    match arg {
                        PredicateArgInstance::Const(konst) => args.push(*konst),
                        _ => panic!("PredicateArgInstance::Ty expected, got {:?}", arg),
                    }
                }
                p(self.tcx, self.typing_env, args)
            },
            PredicateKind::MultipleLocals(p) => {
                let mut args = Vec::new();
                for arg in arg_instance.iter() {
                    match arg {
                        PredicateArgInstance::Local(local) => args.push(*local),
                        _ => panic!("PredicateArgInstance::Local expected, got {:?}", arg),
                    }
                }
                p(self.tcx, self.typing_env, self.body, self.body_cache, args)
            },
            PredicateKind::MultiplePlaces(p) => {
                let mut args = Vec::new();
                for arg in arg_instance.iter() {
                    match arg {
                        PredicateArgInstance::Place(place) => args.push(*place),
                        _ => panic!("PredicateArgInstance::Place expected, got {:?}", arg),
                    }
                }
                p(self.tcx, self.typing_env, self.body, self.body_cache, args)
            },
            PredicateKind::SingleLocal(p) => {
                assert!(
                    arg_instance.len() == 1,
                    "PredicateKind::SingleLocal should have exactly one argument"
                );
                match &arg_instance[0] {
                    PredicateArgInstance::Local(local) => {
                        p(self.tcx, self.typing_env, self.body, self.body_cache, *local)
                    },
                    _ => panic!("PredicateArgInstance::Local expected, got {:?}", arg_instance[0]),
                }
            },
            PredicateKind::ItemAttr(p) => {
                assert!(
                    arg_instance.len() == 2,
                    "PredicateKind::ItemAttr should have exactly two argument"
                );
                match (&arg_instance[0], &arg_instance[1]) {
                    (PredicateArgInstance::Item(item), PredicateArgInstance::Path(symbol)) => {
                        p(self.tcx, *item, symbol)
                    },
                    _ => panic!(
                        "PredicateArgInstance::Item and PredicateArgInstance::Symbol expected, got {:?} and {:?}",
                        &arg_instance[0], &arg_instance[1]
                    ),
                }
            },
            PredicateKind::FlowsTo => self.eval_flows_to(&arg_instance),
            PredicateKind::MayPanic => self.eval_may_panic(&arg_instance),
            PredicateKind::LifetimeBypass => self.eval_call_pred(&arg_instance, "lifetime_bypass", |tcx, did| {
                rudra_paths::is_lifetime_bypass(tcx, did)
            }),
            PredicateKind::StrongBypass => self.eval_call_pred(&arg_instance, "strong_bypass", |tcx, did| {
                rudra_paths::is_strong_bypass(tcx, did)
            }),
            PredicateKind::WeakBypass => self.eval_call_pred(&arg_instance, "weak_bypass", |tcx, did| {
                rudra_paths::is_weak_bypass(tcx, did)
            }),
            PredicateKind::UnresolvableGeneric => self.eval_unresolvable_generic(&arg_instance),
            PredicateKind::GenericDrop => self.eval_call_pred(&arg_instance, "generic_drop", |tcx, did| {
                rudra_paths::is_generic_drop(tcx, did)
            }),
            PredicateKind::CfgReaches => self.eval_cfg_reaches(&arg_instance),
            PredicateKind::BypassOnCopy => self.eval_bypass_on_copy(&arg_instance),
            PredicateKind::SetLen => self.eval_set_len(&arg_instance),
        };
        if term.is_neg { !result } else { result }
    }

    /// `flows_to($x, 'src, 'sink)` — `$x` is a pattern local or place; labels are statement
    /// anchors.
    #[instrument(level = "debug", skip(self, args), ret)]
    fn eval_flows_to(&self, args: &[PredicateArgInstance<'tcx>]) -> bool {
        assert!(
            args.len() == 3,
            "flows_to expects ($local_or_place, 'src, 'sink), got {} args",
            args.len()
        );
        let Some(ddg) = self.mir_ddg else {
            debug!("flows_to: no MIR DDG available");
            return false;
        };
        let local = match &args[0] {
            PredicateArgInstance::Local(local) => *local,
            PredicateArgInstance::Place(place) => place.local,
            other => panic!("flows_to first arg must be Local or Place, got {other:?}"),
        };
        let (PredicateArgInstance::Location(src), PredicateArgInstance::Location(sink)) = (&args[1], &args[2]) else {
            panic!(
                "flows_to expects Location labels for src/sink, got {:?} and {:?}",
                args[1], args[2]
            );
        };
        ddg.flows_to(src.block, src.statement_index, sink.block, sink.statement_index, local)
    }

    /// `may_panic('sink)` — Assert, or Call whose callee is Rudra-unresolvable / Fn* / local
    /// generic.
    #[instrument(level = "debug", skip(self, args), ret)]
    fn eval_may_panic(&self, args: &[PredicateArgInstance<'tcx>]) -> bool {
        assert!(args.len() == 1, "may_panic expects ('sink), got {} args", args.len());
        let PredicateArgInstance::Location(loc) = &args[0] else {
            panic!("may_panic expects a Location label, got {:?}", args[0]);
        };
        location_may_panic(self.tcx, self.typing_env, self.body, *loc)
    }

    fn eval_call_pred(
        &self,
        args: &[PredicateArgInstance<'tcx>],
        name: &str,
        f: impl FnOnce(TyCtxt<'tcx>, DefId) -> bool,
    ) -> bool {
        assert!(args.len() == 1, "{name} expects ('loc), got {} args", args.len());
        let PredicateArgInstance::Location(loc) = &args[0] else {
            panic!("{name} expects a Location label, got {:?}", args[0]);
        };
        call_fn_def(self.tcx, self.body, *loc).is_some_and(|(did, _, _)| f(self.tcx, did))
    }

    #[instrument(level = "debug", skip(self, args), ret)]
    fn eval_unresolvable_generic(&self, args: &[PredicateArgInstance<'tcx>]) -> bool {
        assert!(
            args.len() == 1,
            "unresolvable_generic expects ('loc), got {} args",
            args.len()
        );
        let PredicateArgInstance::Location(loc) = &args[0] else {
            panic!("unresolvable_generic expects a Location label, got {:?}", args[0]);
        };
        location_unresolvable_generic(self.tcx, self.typing_env, self.body, *loc)
    }

    #[instrument(level = "debug", skip(self, args), ret)]
    fn eval_cfg_reaches(&self, args: &[PredicateArgInstance<'tcx>]) -> bool {
        assert!(
            args.len() == 2,
            "cfg_reaches expects ('src, 'sink), got {} args",
            args.len()
        );
        let Some(cfg) = self.mir_cfg else {
            debug!("cfg_reaches: no MIR CFG available");
            return false;
        };
        let (PredicateArgInstance::Location(src), PredicateArgInstance::Location(sink)) = (&args[0], &args[1]) else {
            panic!(
                "cfg_reaches expects Location labels, got {:?} and {:?}",
                args[0], args[1]
            );
        };
        cfg_reaches(cfg, *src, *sink)
    }

    #[instrument(level = "debug", skip(self, args), ret)]
    fn eval_bypass_on_copy(&self, args: &[PredicateArgInstance<'tcx>]) -> bool {
        assert!(
            args.len() == 1,
            "bypass_on_copy expects ('loc), got {} args",
            args.len()
        );
        let PredicateArgInstance::Location(loc) = &args[0] else {
            panic!("bypass_on_copy expects a Location label, got {:?}", args[0]);
        };
        let Some((def_id, _, call_args)) = call_fn_def(self.tcx, self.body, *loc) else {
            return false;
        };
        if !rudra_paths::is_ptr_read(self.tcx, def_id) && !rudra_paths::is_ptr_write(self.tcx, def_id) {
            return false;
        }
        let Some(first) = call_args.first() else {
            return false;
        };
        let ty = first.node.ty(self.body, self.tcx);
        let ty::RawPtr(pointee, _) = *ty.kind() else {
            return false;
        };
        self.tcx.type_is_copy_modulo_regions(self.typing_env, pointee)
    }

    #[instrument(level = "debug", skip(self, args), ret)]
    fn eval_set_len(&self, args: &[PredicateArgInstance<'tcx>]) -> bool {
        let (loc, len) = match args {
            [PredicateArgInstance::Location(loc)] => (*loc, None),
            [PredicateArgInstance::Location(loc), PredicateArgInstance::Const(len)] => (*loc, Some(len)),
            _ => panic!("set_len expects ('loc, len) or ('loc), got {:?}", args),
        };

        // Direct (non-inlined) `set_len` call.
        if let Some((def_id, _, call_args)) = call_fn_def(self.tcx, self.body, loc) {
            if rudra_paths::is_set_len(self.tcx, def_id) {
                // `set_len(self, new_len)` — new_len is the second arg.
                let Some(len_arg) = call_args.get(1) else {
                    info!("set_len: no length argument found in call to set_len at {:?}", loc);
                    return false;
                };
                return match len {
                    None => true, // no length specified, so any length is fine
                    Some(Const::MIR(len)) => matches!(&len_arg.node, Operand::Constant(c) if &c.const_ == len),
                    Some(Const::Param(len)) => match &len_arg.node {
                        Operand::Constant(box mir::ConstOperand {
                            const_: mir::Const::Ty(_, const_),
                            ..
                        }) if let ty::ConstKind::Param(const_) = const_.kind() => const_ == *len,
                        _ => false,
                    },
                };
            }
        }

        // Inlined `set_len`: the call itself was inlined away, but the surviving statement keeps
        // rustc's `SourceScopeData::inlined` pointing at the `set_len` callee. Only the one-arg
        // form is supported here (the length lives in the inlined write, not in the call).
        len.is_none() && is_inlined_from_set_len(self.tcx, self.body, loc)
    }

    fn instantiate_arg(&self, arg: &'m PredicateArg) -> Result<PredicateArgInstance<'tcx>, String> {
        match arg {
            PredicateArg::Label(label) => {
                let pat_loc = self
                    .label_map
                    .get(label)
                    .ok_or_else(|| format!("label `{}` not found in {:?}", label, self.label_map))?;
                match pat_loc {
                    Spanned::Local(local) => Ok(PredicateArgInstance::Local(self.matched[*local])),
                    Spanned::Location(location) => {
                        let stmt_match = self.matched[*location];
                        match stmt_match {
                            StatementMatch::Location(loc) => Ok(PredicateArgInstance::Location(loc)),
                            StatementMatch::Arg(local) => Ok(PredicateArgInstance::Local(local)),
                        }
                    },
                    _ => Err(format!("label `{}` is not a valid location or local", label)),
                }
            },
            PredicateArg::MetaVar(name) => {
                let meta_var = self.symbol_table.meta_vars.get_meta_var_from_name(name.as_str());
                if let Some(meta_var) = meta_var {
                    match meta_var {
                        MetaVariable::Type(idx, _) => {
                            let ty_var_idx: TyVarIdx = idx.into();
                            let ty = self.matched[ty_var_idx];
                            Ok(PredicateArgInstance::Ty(ty))
                        },
                        MetaVariable::Const(idx, _, _) => {
                            let const_var_idx: ConstVarIdx = idx.into();
                            let const_var = self.matched[const_var_idx];
                            Ok(PredicateArgInstance::Const(const_var))
                        },
                        MetaVariable::Place(idx, _, _) => {
                            let place_var_idx: PlaceVarIdx = idx.into();
                            let place_var = self.matched[place_var_idx];
                            Ok(PredicateArgInstance::Place(place_var))
                        },
                        MetaVariable::AdtPat(_, _) => Err(format!("meta_var `{}` is an ADT pattern", name)),
                    }
                } else if let Some(idx) = self.symbol_table.inner.try_get_local_idx(name.as_str()) {
                    let local = pat::Local::from_usize(idx);
                    let local = self.matched[local];
                    Ok(PredicateArgInstance::Local(local))
                } else if let Some(idx) = self.symbol_table.inner.get_fn_name()
                    && idx == name.as_str()
                {
                    Ok(PredicateArgInstance::Item(self.item))
                } else {
                    Err(format!(
                        "meta_var `{}` not found in {:?}",
                        name, self.symbol_table.meta_vars,
                    ))
                }
            },
            PredicateArg::Path(path) => Ok(PredicateArgInstance::Path(path.clone())),
            PredicateArg::SelfValue => panic!("SelfValue should not be used in predicate evaluation."),
        }
    }
}

fn terminator_at<'a, 'tcx>(body: &'a mir::Body<'tcx>, loc: mir::Location) -> Option<&'a mir::Terminator<'tcx>> {
    let bb_data = &body[loc.block];
    if loc.statement_index < bb_data.statements.len() {
        return None;
    }
    bb_data.terminator.as_ref()
}

/// Extract FnDef from a call terminator using `tcx`.
fn call_fn_def<'a, 'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &'a mir::Body<'tcx>,
    loc: mir::Location,
) -> Option<(
    DefId,
    ty::GenericArgsRef<'tcx>,
    &'a [rustc_span::source_map::Spanned<Operand<'tcx>>],
)> {
    let term = terminator_at(body, loc)?;
    let (func, args) = match &term.kind {
        TerminatorKind::Call { func, args, .. } | TerminatorKind::TailCall { func, args, .. } => (func, args.as_ref()),
        _ => return None,
    };
    let ty = func.ty(body, tcx);
    if let ty::FnDef(def_id, gargs) = *ty.kind() {
        Some((def_id, gargs, args))
    } else {
        None
    }
}

/// The functions this MIR location's code was inlined from, innermost first.
///
/// rustc records the inlined callee's `Instance` in `SourceScopeData::inlined` for every statement
/// that originated from an inlined call. Walking the scope chain lets us recover those callees even
/// though the call itself no longer appears in MIR.
pub(crate) fn inlined_instances<'tcx>(body: &mir::Body<'tcx>, loc: mir::Location) -> Vec<ty::Instance<'tcx>> {
    let mut out = Vec::new();
    let mut scope = Some(body.source_info(loc).scope);
    while let Some(s) = scope {
        let data = &body.source_scopes[s];
        scope = data.inlined_parent_scope.or(data.parent_scope);
        if let Some((inst, _)) = data.inlined {
            out.push(inst);
        }
    }
    out
}

/// Whether the statement/terminator at `loc` was inlined from a `set_len` function.
fn is_inlined_from_set_len(tcx: TyCtxt<'_>, body: &mir::Body<'_>, loc: mir::Location) -> bool {
    inlined_instances(body, loc)
        .into_iter()
        .any(|inst| rudra_paths::is_set_len(tcx, inst.def_id()))
}

fn location_unresolvable_generic<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    loc: mir::Location,
) -> bool {
    let term = match terminator_at(body, loc) {
        Some(term) => term,
        None => return false,
    };
    let func = match &term.kind {
        TerminatorKind::Call { func, .. } | TerminatorKind::TailCall { func, .. } => func,
        _ => return false,
    };
    let ty = func.ty(body, tcx);
    match *ty.kind() {
        ty::FnDef(def_id, args) => rudra_instance_unresolvable(tcx, typing_env, def_id, args),
        // Higher-order / unknown callee shapes treated as unresolvable sinks (Rudra intent).
        ty::FnPtr(..)
        | ty::Closure(..)
        | ty::CoroutineClosure(..)
        | ty::Coroutine(..)
        | ty::Dynamic(..)
        | ty::Param(_)
        | ty::Alias(..) => true,
        _ => false,
    }
}

/// CFG reachability from `src` to `sink` (Rudra block taint + intrablock statement order).
pub fn cfg_reaches(cfg: &MirControlFlowGraph, src: mir::Location, sink: mir::Location) -> bool {
    if src == sink {
        return false;
    }
    if src.block == sink.block {
        return src.statement_index < sink.statement_index;
    }
    let mut visited = rustc_data_structures::fx::FxHashSet::default();
    let mut stack: Vec<mir::BasicBlock> = cfg[src.block].successors().collect();
    while let Some(bb) = stack.pop() {
        if !visited.insert(bb) {
            continue;
        }
        if bb == sink.block {
            return true;
        }
        stack.extend(cfg[bb].successors());
    }
    false
}

/// Potential panic / higher-order sink sites (Assert, or Call via Rudra resolve + fallbacks).
fn location_may_panic<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    loc: mir::Location,
) -> bool {
    let bb_data = &body[loc.block];
    // Assert and Call are terminators (statement_index == statements.len()).
    if loc.statement_index < bb_data.statements.len() {
        return false;
    }
    let Some(term) = bb_data.terminator.as_ref() else {
        return false;
    };
    match &term.kind {
        TerminatorKind::Assert { .. } => true,
        TerminatorKind::Call { func, .. } => operand_may_panic(tcx, typing_env, body, func),
        TerminatorKind::TailCall { func, .. } => operand_may_panic(tcx, typing_env, body, func),
        _ => false,
    }
}

fn operand_may_panic<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    func: &Operand<'tcx>,
) -> bool {
    let ty = func.ty(body, tcx);
    match *ty.kind() {
        ty::FnDef(def_id, args) => {
            // Known non-unwinding / non-user std helpers (e.g. `mem::forget`).
            if is_known_non_panicking_callee(tcx, def_id) {
                return false;
            }
            // Rudra: `resolve(def_id, args)` with analysis typing env; `Ok(None)` ⇒ sink.
            if rudra_instance_unresolvable(tcx, typing_env, def_id, args) {
                return true;
            }
            // Fallback: local generic items (inherent wrappers) often `Ok(Some)` even with
            // params still present; keep may_panic for retain / insert_from-style patterns.
            if def_id.is_local() && tcx.generics_of(def_id).requires_monomorphization(tcx) {
                return true;
            }
            args.iter().any(arg_has_param) && def_id.is_local()
        },
        ty::FnPtr(..) => true,
        ty::Closure(..) | ty::CoroutineClosure(..) | ty::Coroutine(..) => true,
        ty::Dynamic(..) => true,
        ty::Param(_) | ty::Alias(..) => true,
        _ => ty.is_fn(),
    }
}

/// Rudra-style unresolvable generic: `Instance::try_resolve` cannot pick a concrete instance
/// when call-site args may still contain `ty::Param` (empty concrete substs).
fn rudra_instance_unresolvable<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    def_id: DefId,
    args: ty::GenericArgsRef<'tcx>,
) -> bool {
    match ty::Instance::try_resolve(tcx, typing_env, def_id, args) {
        Ok(None) => true,
        Ok(Some(_)) => false,
        Err(_) => true,
    }
}

fn is_known_non_panicking_callee(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    let path = tcx.def_path_str(def_id);
    matches!(
        path.as_str(),
        "core::mem::forget"
            | "std::mem::forget"
            | "core::mem::ManuallyDrop::new"
            | "std::mem::ManuallyDrop::new"
            | "core::ptr::read"
            | "std::ptr::read"
            | "core::ptr::write"
            | "std::ptr::write"
            | "core::ptr::copy"
            | "std::ptr::copy"
            | "core::ptr::copy_nonoverlapping"
            | "std::ptr::copy_nonoverlapping"
    )
}

fn arg_has_param(arg: ty::GenericArg<'_>) -> bool {
    match arg.unpack() {
        ty::GenericArgKind::Type(ty) => ty.has_param(),
        ty::GenericArgKind::Const(ct) => ct.has_param(),
        ty::GenericArgKind::Lifetime(_) => false,
    }
}
