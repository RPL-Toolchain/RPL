use std::ops::Deref;

use derive_more::derive::Display;
use rpl_parser::generics::{Choice2, Choice4};
use rpl_parser::{SpanWrapper, pairs};
use rustc_span::Symbol;

// Attention:
// When you add a new module here,
// Try to keep all predicate signatures consistent in it.
mod multiple_consts;
mod multiple_tys;
mod single_const;
mod single_fn;
mod single_ty;
mod translate;
mod trivial;
mod ty_const;

pub use multiple_consts::*;
pub use multiple_tys::*;
pub use single_const::*;
pub use single_fn::*;
pub use single_ty::*;
use thiserror::Error;
pub use translate::*;
pub use trivial::*;
pub use ty_const::*;

#[derive(Clone, Debug, Display, Error)]
pub enum PredicateError<'i> {
    #[display("Invalid predicate: {pred}\n{span}")]
    InvalidPredicate { pred: &'i str, span: SpanWrapper<'i> },
    #[display("Invalid predicate argument: {_0}")]
    InvalidArgs(String),
}

// FIXME: performance
// Attention:
// When you add a new predicate,
// Add it to the list below.
pub const ALL_PREDICATES: &[&str] = &[
    // single_ty_preds
    "can_be_uninit",
    "is_all_safe_trait",
    "is_integral",
    "is_copy",
    "is_not_unpin",
    "is_sync",
    "is_primitive",
    "is_ptr",
    "needs_drop",
    // translate_preds
    "translate_from_function",
    // trivial_preds
    "false",
    "true",
    // multiple_tys_preds
    "same_abi_and_pref_align",
    "same_size",
    // single_fn_preds
    "requires_monomorphization",
    // ty_const_preds
    "maybe_misaligned",
    // single_const_preds
    "is_null_ptr",
    // multiple_consts_preds
    "usize_lt",
];

#[derive(Clone, Copy, Debug)]
pub enum PredicateKind {
    Ty(SingleTyPredsFnPtr),
    Translate(TranslatePredsFnPtr),
    Trivial(TrivialPredsFnPtr),
    MultipleTys(MultipleTysPredsFnPtr),
    Fn(SingleFnPredsFnPtr),
    TyConst(TyConstPredsFnPtr),
    SingleConst(SingleConstPredsFnPtr),
    MultipleConsts(MultipleConstsPredsFnPtr),
}

impl<'i> TryFrom<SpanWrapper<'i>> for PredicateKind {
    type Error = PredicateError<'i>;
    fn try_from(span: SpanWrapper<'i>) -> Result<Self, Self::Error> {
        Ok(match span.inner().as_str() {
            "can_be_uninit" => Self::Ty(can_be_uninit),
            "is_all_safe_trait" => Self::Ty(is_all_safe_trait),
            "is_integral" => Self::Ty(is_integral),
            "is_copy" => Self::Ty(is_copy),
            "is_not_unpin" => Self::Ty(is_not_unpin),
            "is_null_ptr" => Self::SingleConst(is_null_ptr),
            "is_sync" => Self::Ty(is_sync),
            "is_primitive" => Self::Ty(is_primitive),
            "is_ptr" => Self::Ty(is_ptr),
            "needs_drop" => Self::Ty(needs_drop),
            "translate_from_function" => Self::Translate(translate_from_function),
            "false" => Self::Trivial(r#false),
            "true" => Self::Trivial(r#true),
            "same_abi_and_pref_align" => Self::MultipleTys(same_abi_and_pref_align),
            "same_size" => Self::MultipleTys(same_size),
            "requires_monomorphization" => Self::Fn(requires_monomorphization),
            "maybe_misaligned" => Self::TyConst(maybe_misaligned),
            "usize_lt" => Self::MultipleConsts(usize_lt),
            _ => {
                return Err(PredicateError::InvalidPredicate {
                    pred: span.inner().as_str(),
                    span,
                });
            },
        })
    }
}

#[derive(Clone, Default, Debug)]
pub struct PredicateConjunction {
    pub clauses: Vec<PredicateClause>,
}

pub type Predicate<'pcx> = &'pcx PredicateConjunction;

impl PredicateConjunction {
    pub fn from_pairs<'i>(
        preds: &pairs::PredicateConjunction<'i>,
        path: &'i std::path::Path,
    ) -> Result<Self, PredicateError<'i>> {
        let (first, following) = preds.get_matched();
        let clauses = std::iter::once(first)
            .chain(following.iter_matched().map(|and_pred| and_pred.get_matched().1))
            .map(|pred| PredicateClause::from_pairs(pred, path))
            .collect::<Result<_, _>>()?;
        Ok(Self { clauses })
    }
}

// PredicateClause is a `||` of PredicateTerms
#[derive(Clone, Default, Debug)]
pub struct PredicateClause {
    pub terms: Vec<PredicateTerm>,
}

impl PredicateClause {
    fn from_pairs<'i>(
        pred: &pairs::PredicateClause<'i>,
        path: &'i std::path::Path,
    ) -> Result<Self, PredicateError<'i>> {
        let terms = match pred.deref() {
            Choice2::_0(pred) => vec![PredicateTerm::from_pairs(pred, path)?],
            Choice2::_1(preds) => {
                let (_, first, following, _) = preds.get_matched();
                // FIXME: this is will return early errors if any of the terms are invalid, consider
                // collecting all errors instead
                std::iter::once(first)
                    .chain(following.iter_matched().map(|or_pred| or_pred.get_matched().1))
                    .map(|pred| PredicateTerm::from_pairs(pred, path))
                    .collect::<Result<_, _>>()?
            },
        };
        Ok(Self { terms })
    }
}

#[derive(Clone, Debug)]
pub struct PredicateTerm {
    pub kind: PredicateKind,
    pub args: Vec<PredicateArg>,
    pub is_neg: bool,
}

impl PredicateTerm {
    fn from_pairs<'i>(pred: &pairs::PredicateTerm<'i>, path: &'i std::path::Path) -> Result<Self, PredicateError<'i>> {
        let (pred, is_neg) = match pred.deref() {
            Choice2::_0(pred) => (pred, false),
            Choice2::_1(pred) => (pred.get_matched().1, true),
        };
        let (pred_name, _, args, _) = pred.get_matched();
        let kind = PredicateKind::try_from(SpanWrapper::new(pred_name.span, path))?;
        let args = if let Some(args) = args {
            let (first, following, _) = args.get_matched();
            let following = following
                .iter_matched()
                .map(|comma_with_elem| comma_with_elem.get_matched().1);
            std::iter::once(first)
                .chain(following)
                .map(PredicateArg::from_pairs)
                .collect()
        } else {
            vec![]
        };
        Ok(Self { kind, is_neg, args })
    }
}

#[derive(Clone, Debug)]
pub enum PredicateArg {
    Label(Symbol),
    MetaVar(Symbol),
    Path(Vec<Symbol>),
    SelfValue,
}

impl PredicateArg {
    pub fn from_pairs(arg: &pairs::PredicateArg<'_>) -> Self {
        match arg.deref() {
            Choice4::_0(label) => Self::Label(Symbol::intern(label.LabelName().span.as_str())),
            Choice4::_1(meta_var) => Self::MetaVar(Symbol::intern(meta_var.span.as_str())),
            Choice4::_2(path) => Self::Path(path.span.as_str().split("::").map(Symbol::intern).collect()),
            Choice4::_3(_self_value) => Self::SelfValue,
        }
    }
}
