use std::ops::Deref;

use derive_more::derive::Display;
use rpl_parser::generics::{Choice2, Choice4, Choice5, Choice10};
use rpl_parser::{SpanWrapper, pairs};
use rustc_span::Symbol;

// Attention:
// When you add a new module here,
// Try to keep all predicate signatures consistent in it.
mod false_positive_filter;
mod internval_analysis;
mod item_attr;
mod locals;
mod multiple_consts;
mod multiple_tys;
mod null_analysis;
mod places;
mod pointer_predicates;
mod ptr_state_analysis;
mod single_const;
mod single_fn;
mod single_ty;
mod translate;
mod trivial;
mod ty_const;

pub(crate) use false_positive_filter::*;
pub use locals::*;
pub use multiple_consts::*;
pub use multiple_tys::*;
pub use places::*;
pub(crate) use pointer_predicates::*;
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
    "is_borrow_guard",
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
    // local_preds
    "is_null",
    "byte_slice_is_empty",
    "byte_slice_last_byte_not_nul",
    "contains_interior_nul",
    "is_freed",
    "is_uaf_call_arg",
    "is_borrow_guard_drop_after_pointee_drop",
    "is_dangling_ptr",
    "has_dangling_ptr_at_exit",
    "is_release_helper_body",
    // local_const_preds
    "eq_const",
    "value_invalid_for_type",
    "ne_const",
    "lt_const",
    "le_const",
    "gt_const",
    "ge_const",
    // local_local_preds
    "eq_local",
    "lt_local",
    "gt_local",
    "index_ge_len",
    "index_gt_len",
    "range_start_gt_end",
    "range_start_gt_len",
    "range_end_gt_len",
    "range_end_ge_len",
    "layout_size_eq_const",
    "layout_size_gt",
    "not_power_of_two",
    // local_local_const_preds
    "add_gt_const",
    "add_lt_const",
    "sub_lt_const",
    "sub_gt_const",
    "mul_lt_const",
    "mul_gt_const",
    "size_mul_lt_const",
    "size_mul_gt_const",
    "slice_size_gt_const",
    "rounded_up_gt_const",
    // multiple_locals_preds
    "product_of",
    // multiple_places_preds
    "mentions_place",
    // dataflow preds (evaluated specially in PredicateEvaluator)
    "flows_to",
    "may_panic",
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
    Location(LocationPredsFnPtr),
    LocationLocal(LocationLocalPredsFnPtr),
    LocationLocalConst(LocationLocalConstPredsFnPtr),
    LocationLocalTy(LocationLocalTyPredsFnPtr),
    LocationLocalTyConst(LocationLocalTyConstPredsFnPtr),
    LocationLocalLocal(LocationLocalLocalPredsFnPtr),
    LocationLocalLocalConst(LocationLocalLocalConstPredsFnPtr),
    MultipleLocals(MultipleLocalsPredsFnPtr),
    MultiplePlaces(MultiplePlacesPredsFnPtr),
    ItemAttr(ItemAttrPredsFnPtr),
    /// `flows_to($local_or_place, 'src, 'sink)` — DDG reachability; evaluated in matcher.
    FlowsTo,
    /// `may_panic('sink)` — potential panic site; evaluated in matcher.
    MayPanic,
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
            "is_borrow_guard" => Self::Ty(is_borrow_guard),
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
            "is_null" => Self::LocationLocal(is_null_at_location),
            "byte_slice_is_empty" => Self::LocationLocal(byte_slice_is_empty),
            "byte_slice_last_byte_not_nul" => Self::LocationLocal(byte_slice_last_byte_not_nul),
            "contains_interior_nul" => Self::LocationLocal(contains_interior_nul),
            "is_freed" => Self::LocationLocal(is_freed_at_location),
            "is_uaf_call_arg" => Self::LocationLocal(is_uaf_call_arg_at_location),
            "is_borrow_guard_drop_after_pointee_drop" => {
                Self::LocationLocal(is_borrow_guard_drop_after_pointee_drop_at_location)
            },
            "is_dangling_ptr" => Self::LocationLocal(is_dangling_ptr_at_location),
            "has_dangling_ptr_at_exit" => Self::SingleLocal(has_dangling_ptr_at_exit),
            "is_release_helper_body" => Self::LocationLocal(is_release_helper_body),
            "eq_const" => Self::LocationLocalConst(eq_const),
            "value_invalid_for_type" => Self::LocationLocalTy(value_invalid_for_type),
            "ne_const" => Self::LocationLocalConst(ne_const),
            "lt_const" => Self::LocationLocalConst(lt_const),
            "le_const" => Self::LocationLocalConst(le_const),
            "gt_const" => Self::LocationLocalConst(gt_const),
            "ge_const" => Self::LocationLocalConst(ge_const),
            "eq_local" => Self::LocationLocalLocal(eq_local),
            "lt_local" => Self::LocationLocalLocal(lt_local),
            "gt_local" => Self::LocationLocalLocal(gt_local),
            "index_ge_len" => Self::LocationLocalLocal(index_ge_len),
            "index_gt_len" => Self::LocationLocalLocal(index_gt_len),
            "range_start_gt_end" => Self::LocationLocal(range_start_gt_end),
            "range_start_gt_len" => Self::LocationLocalLocal(range_start_gt_len),
            "range_end_gt_len" => Self::LocationLocalLocal(range_end_gt_len),
            "range_end_ge_len" => Self::LocationLocalLocal(range_end_ge_len),
            "layout_size_eq_const" => Self::LocationLocalConst(layout_size_eq_const),
            "layout_size_gt" => Self::LocationLocalLocal(layout_size_gt),
            "not_power_of_two" => Self::LocationLocal(not_power_of_two),
            "add_gt_const" => Self::LocationLocalLocalConst(add_gt_const),
            "add_lt_const" => Self::LocationLocalLocalConst(add_lt_const),
            "sub_lt_const" => Self::LocationLocalLocalConst(sub_lt_const),
            "sub_gt_const" => Self::LocationLocalLocalConst(sub_gt_const),
            "mul_lt_const" => Self::LocationLocalLocalConst(mul_lt_const),
            "mul_gt_const" => Self::LocationLocalLocalConst(mul_gt_const),
            "size_mul_lt_const" => Self::LocationLocalTyConst(size_mul_lt_const),
            "size_mul_gt_const" => Self::LocationLocalTyConst(size_mul_gt_const),
            "slice_size_gt_const" => Self::LocationLocalConst(slice_size_gt_const),
            "rounded_up_gt_const" => Self::LocationLocalLocalConst(rounded_up_gt_const),
            "mentions_place" => Self::MultiplePlaces(mentions_place),
            "has_attr" => Self::ItemAttr(has_attr),
            "flows_to" => Self::FlowsTo,
            "may_panic" => Self::MayPanic,
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
    Integer(PredicateInteger),
    Path(Vec<Symbol>),
    SelfValue,
}

impl PredicateArg {
    pub fn from_pairs(arg: &pairs::PredicateArg<'_>) -> Self {
        match arg.deref() {
            Choice5::_0(label) => Self::Label(Symbol::intern(label.LabelName().span.as_str())),
            Choice5::_1(meta_var) => Self::MetaVar(Symbol::intern(meta_var.span.as_str())),
            Choice5::_2(integer) => Self::Integer(PredicateInteger::from_pairs(integer)),
            Choice5::_3(path) => Self::Path(path.span.as_str().split("::").map(Symbol::intern).collect()),
            Choice5::_4(_self_value) => Self::SelfValue,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PredicateInteger {
    pub value: u128,
    pub ty: Option<PredicateIntegerTy>,
}

impl PredicateInteger {
    fn from_pairs(integer: &pairs::Integer<'_>) -> Self {
        let (literal, suffix) = integer.get_matched();
        let (digits, radix) = match literal {
            Choice4::_0(binary) => (&binary.span.as_str()[2..], 2),
            Choice4::_1(octal) => (&octal.span.as_str()[2..], 8),
            Choice4::_2(hexadecimal) => (&hexadecimal.span.as_str()[2..], 16),
            Choice4::_3(decimal) => (decimal.span.as_str(), 10),
        };
        let digits = digits.replace('_', "");
        let value = u128::from_str_radix(&digits, radix)
            .unwrap_or_else(|error| panic!("invalid integer literal `{}`: {}", integer.span.as_str(), error));
        let ty = suffix.as_ref().map(PredicateIntegerTy::from_pairs);
        Self { value, ty }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PredicateIntegerTy {
    U8,
    U16,
    U32,
    U64,
    Usize,
    I8,
    I16,
    I32,
    I64,
    Isize,
}

impl PredicateIntegerTy {
    fn from_pairs(suffix: &pairs::IntegerSuffix<'_>) -> Self {
        match suffix.deref() {
            Choice10::_0(_) => Self::U8,
            Choice10::_1(_) => Self::U16,
            Choice10::_2(_) => Self::U32,
            Choice10::_3(_) => Self::U64,
            Choice10::_4(_) => Self::Usize,
            Choice10::_5(_) => Self::I8,
            Choice10::_6(_) => Self::I16,
            Choice10::_7(_) => Self::I32,
            Choice10::_8(_) => Self::I64,
            Choice10::_9(_) => Self::Isize,
        }
    }
}
