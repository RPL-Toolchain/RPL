use std::ops::Deref;

use derive_more::derive::Display;
use rpl_parser::generics::{Choice2, Choice4};
use rpl_parser::{SpanWrapper, pairs};
use rustc_span::Symbol;

// Attention:
// When you add a new module here,
// Try to keep all predicate signatures consistent in it.
mod item_attr;
mod locals;
mod multiple_consts;
mod multiple_tys;
mod places;
mod single_const;
mod single_fn;
mod single_ty;
mod translate;
mod trivial;
mod ty_const;

pub use locals::*;
pub use multiple_consts::*;
pub use multiple_tys::*;
pub use places::*;
pub use single_const::*;
pub use single_fn::*;
pub use single_ty::*;
use thiserror::Error;
pub use translate::*;
pub use trivial::*;
pub use ty_const::*;

use crate::predicates::item_attr::{ItemAttrPredsFnPtr, has_attr};

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
    "is_char",
    "is_copy",
    "is_float",
    "is_fn_ptr",
    "is_not_unpin",
    "is_send",
    "is_sync",
    "is_primitive",
    "is_ptr",
    "is_ref",
    "is_zst",
    "needs_drop",
    // translate_preds
    "translate_from_function",
    // trivial_preds
    "false",
    "true",
    // multiple_tys_preds
    "compatible_layout",
    "niche_ordered",
    "same_abi_and_pref_align",
    "same_size",
    // single_fn_preds
    "requires_monomorphization",
    "runs_outside_main",
    // ty_const_preds
    "maybe_misaligned",
    // single_const_preds
    "is_null_ptr",
    "is_nonzero",
    // multiple_consts_preds
    "usize_lt",
    // single_local_preds
    "is_null",
    // multiple_locals_preds
    "product_of",
    // multiple_places_preds
    "mentions_place",
    // dataflow preds (evaluated specially in PredicateEvaluator)
    "flows_to",
    "may_panic",
    // Rudra Unsafe Dataflow preds (evaluated specially in PredicateEvaluator)
    "lifetime_bypass",
    "strong_bypass",
    "weak_bypass",
    "unresolvable_generic",
    "generic_drop",
    "cfg_reaches",
    "bypass_on_copy",
    "set_len_to_zero",
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
    SingleLocal(SingleLocalPredsFnPtr),
    MultipleLocals(MultipleLocalsPredsFnPtr),
    MultiplePlaces(MultiplePlacesPredsFnPtr),
    ItemAttr(ItemAttrPredsFnPtr),
    /// `flows_to($local_or_place, 'src, 'sink)` — DDG reachability; evaluated in matcher.
    FlowsTo,
    /// `may_panic('sink)` — potential panic site; evaluated in matcher.
    MayPanic,
    /// `lifetime_bypass('loc)` — Rudra strong∪weak lifetime-bypass Call.
    LifetimeBypass,
    /// `strong_bypass('loc)` — Rudra strong lifetime-bypass Call.
    StrongBypass,
    /// `weak_bypass('loc)` — Rudra weak lifetime-bypass Call.
    WeakBypass,
    /// `unresolvable_generic('loc)` — `Instance::try_resolve` → None/Err (strict Rudra sink).
    UnresolvableGeneric,
    /// `generic_drop('loc)` — `ptr::drop_in_place` Call (Rudra GENERIC_FN_LIST sink).
    GenericDrop,
    /// `cfg_reaches('src, 'sink)` — MIR CFG reachability between two labels.
    CfgReaches,
    /// `bypass_on_copy('loc)` — read/write bypass on a Copy pointed-to type (Rudra skip).
    BypassOnCopy,
    /// `set_len('loc, len)` — `Vec::set_len(len)` (Rudra skip; leak is safe).
    SetLen,
}

impl<'i> TryFrom<SpanWrapper<'i>> for PredicateKind {
    type Error = PredicateError<'i>;
    fn try_from(span: SpanWrapper<'i>) -> Result<Self, Self::Error> {
        Ok(match span.inner().as_str() {
            "can_be_uninit" => Self::Ty(can_be_uninit),
            "is_all_safe_trait" => Self::Ty(is_all_safe_trait),
            "is_integral" => Self::Ty(is_integral),
            "is_char" => Self::Ty(is_char),
            "is_copy" => Self::Ty(is_copy),
            "is_float" => Self::Ty(is_float),
            "is_fn_ptr" => Self::Ty(is_fn_ptr),
            "is_not_unpin" => Self::Ty(is_not_unpin),
            "is_ref" => Self::Ty(is_ref),
            "is_send" => Self::Ty(is_send),
            "is_sync" => Self::Ty(is_sync),
            "is_primitive" => Self::Ty(is_primitive),
            "is_ptr" => Self::Ty(is_ptr),
            "is_zst" => Self::Ty(is_zst),
            "needs_drop" => Self::Ty(needs_drop),
            "compatible_layout" => Self::MultipleTys(compatible_layout),
            "niche_ordered" => Self::MultipleTys(niche_ordered),
            "translate_from_function" => Self::Translate(translate_from_function),
            "false" => Self::Trivial(r#false),
            "true" => Self::Trivial(r#true),
            "same_abi_and_pref_align" => Self::MultipleTys(same_abi_and_pref_align),
            "same_size" => Self::MultipleTys(same_size),
            "requires_monomorphization" => Self::Fn(requires_monomorphization),
            "runs_outside_main" => Self::Fn(runs_outside_main),
            "maybe_misaligned" => Self::TyConst(maybe_misaligned),
            "is_null_ptr" => Self::SingleConst(is_null_ptr),
            "is_nonzero" => Self::SingleConst(is_nonzero),
            "usize_lt" => Self::MultipleConsts(usize_lt),
            "product_of" => Self::MultipleLocals(product_of),
            "is_null" => Self::SingleLocal(is_null),
            "mentions_place" => Self::MultiplePlaces(mentions_place),
            "has_attr" => Self::ItemAttr(has_attr),
            "flows_to" => Self::FlowsTo,
            "may_panic" => Self::MayPanic,
            "lifetime_bypass" => Self::LifetimeBypass,
            "strong_bypass" => Self::StrongBypass,
            "weak_bypass" => Self::WeakBypass,
            "unresolvable_generic" => Self::UnresolvableGeneric,
            "generic_drop" => Self::GenericDrop,
            "cfg_reaches" => Self::CfgReaches,
            "bypass_on_copy" => Self::BypassOnCopy,
            "set_len" => Self::SetLen,
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
