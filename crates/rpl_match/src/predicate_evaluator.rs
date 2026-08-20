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

use crate::graph::MirDataDepGraph;
use crate::matches::{Matched, StatementMatch};

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
    body_cache: &'e BodyInfoCache<'tcx>,
    symbol_table: &'e pat::FnSymbolTable<'m>,
    mir_ddg: Option<&'e MirDataDepGraph>,
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
        body_cache: &'e BodyInfoCache<'tcx>,
        symbol_table: &'e pat::FnSymbolTable<'m>,
        mir_ddg: Option<&'e MirDataDepGraph>,
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
            PredicateKind::Location(p) => {
                assert_eq!(arg_instance.len(), 1, "Location predicate expects one argument");
                match &arg_instance[0] {
                    PredicateArgInstance::Location(location) => {
                        p(self.tcx, self.typing_env, self.body, self.body_cache, *location)
                    },
                    other => panic!("Location predicate expected a location, got {other:?}"),
                }
            },
            PredicateKind::LocationLocal(p) => {
                assert_eq!(arg_instance.len(), 2, "LocationLocal predicate expects two arguments");
                match (&arg_instance[0], &arg_instance[1]) {
                    (PredicateArgInstance::Location(location), PredicateArgInstance::Local(local)) => {
                        p(self.tcx, self.typing_env, self.body, self.body_cache, *location, *local)
                    },
                    args => panic!("LocationLocal predicate expected a location and local, got {args:?}"),
                }
            },
            PredicateKind::LocationLocalConst(p) => {
                assert_eq!(
                    arg_instance.len(),
                    3,
                    "LocationLocalConst predicate expects three arguments"
                );
                match (&arg_instance[0], &arg_instance[1], &arg_instance[2]) {
                    (
                        PredicateArgInstance::Location(location),
                        PredicateArgInstance::Local(local),
                        PredicateArgInstance::Const(konst),
                    ) => p(
                        self.tcx,
                        self.typing_env,
                        self.body,
                        self.body_cache,
                        *location,
                        *local,
                        *konst,
                    ),
                    args => panic!("LocationLocalConst predicate received invalid arguments: {args:?}"),
                }
            },
            PredicateKind::LocationLocalTy(p) => {
                assert_eq!(
                    arg_instance.len(),
                    3,
                    "LocationLocalTy predicate expects three arguments"
                );
                match (&arg_instance[0], &arg_instance[1], &arg_instance[2]) {
                    (
                        PredicateArgInstance::Location(location),
                        PredicateArgInstance::Local(local),
                        PredicateArgInstance::Ty(ty),
                    ) => p(
                        self.tcx,
                        self.typing_env,
                        self.body,
                        self.body_cache,
                        *location,
                        *local,
                        *ty,
                    ),
                    args => panic!("LocationLocalTy predicate received invalid arguments: {args:?}"),
                }
            },
            PredicateKind::LocationLocalLocal(p) => {
                assert_eq!(
                    arg_instance.len(),
                    3,
                    "LocationLocalLocal predicate expects three arguments"
                );
                match (&arg_instance[0], &arg_instance[1], &arg_instance[2]) {
                    (
                        PredicateArgInstance::Location(location),
                        PredicateArgInstance::Local(left),
                        PredicateArgInstance::Local(right),
                    ) => p(
                        self.tcx,
                        self.typing_env,
                        self.body,
                        self.body_cache,
                        *location,
                        *left,
                        *right,
                    ),
                    args => panic!("LocationLocalLocal predicate received invalid arguments: {args:?}"),
                }
            },
            PredicateKind::LocationLocalLocalConst(p) => {
                assert_eq!(
                    arg_instance.len(),
                    4,
                    "LocationLocalLocalConst predicate expects four arguments"
                );
                match (&arg_instance[0], &arg_instance[1], &arg_instance[2], &arg_instance[3]) {
                    (
                        PredicateArgInstance::Location(location),
                        PredicateArgInstance::Local(left),
                        PredicateArgInstance::Local(right),
                        PredicateArgInstance::Const(konst),
                    ) => p(
                        self.tcx,
                        self.typing_env,
                        self.body,
                        self.body_cache,
                        *location,
                        *left,
                        *right,
                        *konst,
                    ),
                    args => panic!("LocationLocalLocalConst predicate received invalid arguments: {args:?}"),
                }
            },
            PredicateKind::LocationLocalTyConst(p) => {
                assert_eq!(
                    arg_instance.len(),
                    4,
                    "LocationLocalTyConst predicate expects four arguments"
                );
                match (&arg_instance[0], &arg_instance[1], &arg_instance[2], &arg_instance[3]) {
                    (
                        PredicateArgInstance::Location(location),
                        PredicateArgInstance::Local(local),
                        PredicateArgInstance::Ty(ty),
                        PredicateArgInstance::Const(konst),
                    ) => p(
                        self.tcx,
                        self.typing_env,
                        self.body,
                        self.body_cache,
                        *location,
                        *local,
                        *ty,
                        *konst,
                    ),
                    args => panic!("LocationLocalTyConst predicate received invalid arguments: {args:?}"),
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
            PredicateArg::Integer(integer) => {
                use rpl_constraints::predicates::PredicateIntegerTy;

                let ty = match integer.ty {
                    None | Some(PredicateIntegerTy::Usize) => self.tcx.types.usize,
                    Some(PredicateIntegerTy::U8) => self.tcx.types.u8,
                    Some(PredicateIntegerTy::U16) => self.tcx.types.u16,
                    Some(PredicateIntegerTy::U32) => self.tcx.types.u32,
                    Some(PredicateIntegerTy::U64) => self.tcx.types.u64,
                    Some(PredicateIntegerTy::I8) => self.tcx.types.i8,
                    Some(PredicateIntegerTy::I16) => self.tcx.types.i16,
                    Some(PredicateIntegerTy::I32) => self.tcx.types.i32,
                    Some(PredicateIntegerTy::I64) => self.tcx.types.i64,
                    Some(PredicateIntegerTy::Isize) => self.tcx.types.isize,
                };
                let konst = mir::Const::from_bits(self.tcx, integer.value, self.typing_env, ty);
                Ok(PredicateArgInstance::Const(Const::MIR(konst)))
            },
            PredicateArg::Path(path) => Ok(PredicateArgInstance::Path(path.clone())),
            PredicateArg::SelfValue => panic!("SelfValue should not be used in predicate evaluation."),
        }
    }
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
