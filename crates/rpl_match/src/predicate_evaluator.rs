use rpl_constraints::Constraints;
use rpl_constraints::predicates::{PredicateArg, PredicateClause, PredicateConjunction, PredicateKind, PredicateTerm};
use rpl_context::pat::{ConstVarIdx, LabelMap, PlaceVarIdx, Spanned, TyVarIdx};
use rpl_meta::symbol_table::{MetaVariable, NonLocalMetaSymTab};
use rustc_middle::mir::{self, Const, PlaceRef};
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::Symbol;

use crate::matches::{Matched, StatementMatch};

/// PredicateArgInstance is the matched instance of a [PredicateArg]
#[allow(unused)]
#[derive(Clone, Debug)]
enum PredicateArgInstance<'tcx> {
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
    body: &'e mir::Body<'tcx>,
    label_map: &'e LabelMap,
    matched: &'e Matched<'tcx>,
    symbol_table: &'e NonLocalMetaSymTab<'m>,
}

impl<'e, 'm, 'tcx> PredicateEvaluator<'e, 'm, 'tcx> {
    pub fn new(
        tcx: TyCtxt<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
        body: &'e mir::Body<'tcx>,
        label_map: &'e LabelMap,
        matched: &'e Matched<'tcx>,
        symbol_table: &'e NonLocalMetaSymTab<'m>,
    ) -> Self {
        Self {
            tcx,
            typing_env,
            body,
            label_map,
            matched,
            symbol_table,
        }
    }

    #[instrument(level = "debug", skip(self), ret)]
    pub fn evaluate_constraint(&self, constraint: &Constraints) -> bool {
        constraint.preds.iter().all(|pred| self.evaluate_conjunction(pred))
        // FIX: we should possibly check attributes here
    }

    fn evaluate_conjunction(&self, conjunction: &PredicateConjunction) -> bool {
        conjunction.clauses.iter().all(|clause| self.evaluate_clause(clause))
    }

    fn evaluate_clause(&self, clause: &PredicateClause) -> bool {
        clause.terms.iter().any(|term| self.evaluate_term(term))
    }

    fn evaluate_term(&self, term: &PredicateTerm) -> bool {
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
            PredicateKind::Fn(_) => {
                assert!(
                    arg_instance.len() == 1,
                    "PredicateKind::Fn should have exactly one argument"
                );
                todo!("Implement PredicateKind::Fn evaluation");
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
        };
        if term.is_neg { !result } else { result }
    }

    fn instantiate_arg(&self, arg: &PredicateArg) -> Result<PredicateArgInstance<'tcx>, String> {
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
                let meta_var = self.symbol_table.get_from_symbol(*name);
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
                } else {
                    Err(format!("meta_var `{}` not found in {:?}", name, self.symbol_table))
                }
            },
            PredicateArg::Path(path) => Ok(PredicateArgInstance::Path(path.clone())),
            PredicateArg::SelfValue => panic!("SelfValue should not be used in predicate evaluation."),
        }
    }
}
