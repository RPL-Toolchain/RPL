#![feature(rustc_private)]
#![feature(let_chains)]
#![feature(if_let_guard)]

extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_fluent_macro;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_lint_defs;
extern crate rustc_macros;
extern crate rustc_middle;
extern crate rustc_passes;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_trait_selection;
#[macro_use]
extern crate tracing;
extern crate either;

use std::ops::Deref;

use rpl_parser::generics::Choice2;
use rpl_parser::pairs;

use rustc_middle::mir;
use rustc_middle::ty::{self, Ty, TyCtxt};

// Attention:
// When you add a new module here,
// Try to keep all predicate signatures consistent in it.
mod single_ty_preds;
mod translate_preds;
mod tribool;
mod trivial_preds;

pub use single_ty_preds::*;
pub use translate_preds::*;
pub use trivial_preds::*;

// FIXME: performance
// Attention:
// When you add a new predicate,
// Add it to the list below.
pub const ALL_PREDICATES: &[&str] = &[
    // ty_preds
    "is_all_safe_trait",
    "is_not_unpin",
    "is_sync",
    "is_integral",
    "is_ptr",
    "is_primitive",
    // translate_preds
    "translate_from_hir_function",
    // trivial_preds
    "false",
    "true",
];

#[derive(Clone, Copy, Debug)]
pub enum PredicateKind {
    Ty(SingleTyPredsFnPtr),
    Translate(TranslatePredsFnPtr),
    Trivial(TrivialPredsFnPtr),
}

impl From<&str> for PredicateKind {
    fn from(s: &str) -> Self {
        match s {
            "is_all_safe_trait" => Self::Ty(is_all_safe_trait),
            "is_not_unpin" => Self::Ty(is_not_unpin),
            "is_sync" => Self::Ty(is_sync),
            "is_integral" => Self::Ty(is_integral),
            "is_ptr" => Self::Ty(is_ptr),
            "is_primitive" => Self::Ty(is_primitive),
            "translate_from_hir_function" => Self::Translate(translate_from_hir_function),
            "is_false" => Self::Trivial(is_false),
            "is_true" => Self::Trivial(is_true),
            _ => unreachable!("Unknown predicate: {}", s),
        }
    }
}

impl PredicateKind {
    fn evaluate<'tcx>(
        &self,
        tcx: Option<TyCtxt<'tcx>>,
        typing_env: Option<ty::TypingEnv<'tcx>>,
        ty: Option<Ty<'tcx>>,
        mir_location: Option<mir::Location>,
        hir_fn_path: Option<&str>,
        body: Option<&mir::Body<'tcx>>,
    ) -> bool {
        match self {
            Self::Ty(p) => {
                debug_assert!(tcx.is_some(), "tcx is required when evaluating trait_bound predicates");
                debug_assert!(
                    typing_env.is_some(),
                    "typing_env is required when evaluating trait_bound predicates"
                );
                debug_assert!(ty.is_some(), "ty is required when evaluating trait_bound predicates");
                p(tcx.unwrap(), typing_env.unwrap(), ty.unwrap())
            },
            Self::Translate(p) => {
                debug_assert!(
                    mir_location.is_some(),
                    "mir_location is required when evaluating translate predicates"
                );
                debug_assert!(
                    hir_fn_path.is_some(),
                    "hir_fn_path is required when evaluating translate predicates"
                );
                debug_assert!(tcx.is_some(), "tcx is required when evaluating translate predicates");
                debug_assert!(body.is_some(), "body is required when evaluating translate predicates");
                p(mir_location.unwrap(), hir_fn_path.unwrap(), tcx.unwrap(), body.unwrap())
            },
            Self::Trivial(p) => p(),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct PredicateConjunction {
    clauses: Vec<PredicateClause>,
}

impl PredicateConjunction {
    pub fn from_pairs_opt(preds: Option<&pairs::PredicateConjunction<'_>>) -> Self {
        if let Some(preds) = preds {
            let (first, following) = preds.get_matched();
            let clauses = std::iter::once(first)
                .chain(following.iter_matched().map(|and_pred| and_pred.get_matched().1))
                .map(|pred| PredicateClause::from_pairs(pred))
                .collect();
            Self { clauses }
        } else {
            Self::default()
        }
    }

    pub fn evaluate<'tcx>(
        &self,
        tcx: Option<TyCtxt<'tcx>>,
        typing_env: Option<ty::TypingEnv<'tcx>>,
        ty: Option<Ty<'tcx>>,
        mir_location: Option<mir::Location>,
        hir_fn_path: Option<&str>,
        body: Option<&mir::Body<'tcx>>,
    ) -> bool {
        self.clauses
            .iter()
            .all(|clause| clause.evaluate(tcx, typing_env, ty, mir_location, hir_fn_path, body))
    }
}

// PredicateClause is a `||` of PredicateTerms
#[derive(Clone, Default, Debug)]
struct PredicateClause {
    terms: Vec<PredicateTerm>,
}

impl PredicateClause {
    fn from_pairs(pred: &pairs::PredicateClause<'_>) -> Self {
        let terms = match pred.deref() {
            Choice2::_0(pred) => vec![PredicateTerm::from_pairs(pred)],
            Choice2::_1(preds) => {
                let (_, first, following, _) = preds.get_matched();
                std::iter::once(first)
                    .chain(following.iter_matched().map(|or_pred| or_pred.get_matched().1))
                    .map(|pred| PredicateTerm::from_pairs(pred))
                    .collect()
            },
        };
        Self { terms }
    }

    fn evaluate<'tcx>(
        &self,
        tcx: Option<TyCtxt<'tcx>>,
        typing_env: Option<ty::TypingEnv<'tcx>>,
        ty: Option<Ty<'tcx>>,
        mir_location: Option<mir::Location>,
        hir_fn_path: Option<&str>,
        body: Option<&mir::Body<'tcx>>,
    ) -> bool {
        self.terms
            .iter()
            .any(|term| term.evaluate(tcx, typing_env, ty, mir_location, hir_fn_path, body))
    }
}

#[derive(Clone, Debug)]
struct PredicateTerm {
    kind: PredicateKind,
    is_neg: bool,
}

impl PredicateTerm {
    fn from_pairs(pred: &pairs::PredicateTerm<'_>) -> Self {
        let (pred, is_neg) = match pred.deref() {
            Choice2::_0(pred) => (pred, false),
            Choice2::_1(pred) => (pred.get_matched().1, true),
        };
        let pred_name = pred.get_matched().0.span.as_str();
        let kind = PredicateKind::from(pred_name);
        Self { kind, is_neg }
    }

    fn evaluate<'tcx>(
        &self,
        tcx: Option<TyCtxt<'tcx>>,
        typing_env: Option<ty::TypingEnv<'tcx>>,
        ty: Option<Ty<'tcx>>,
        mir_location: Option<mir::Location>,
        hir_fn_path: Option<&str>,
        body: Option<&mir::Body<'tcx>>,
    ) -> bool {
        let result = self.kind.evaluate(tcx, typing_env, ty, mir_location, hir_fn_path, body);
        if self.is_neg { !result } else { result }
    }
}
