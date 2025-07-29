#![feature(rustc_private)]
#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(map_try_insert)]
#![feature(box_patterns)]

extern crate rustc_abi;
extern crate rustc_const_eval;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_fluent_macro;
extern crate rustc_hir;
extern crate rustc_index;
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

use attributes::FnAttr;
pub use konst::Const;
use predicates::PredicateConjunction;
use rpl_parser::generics::Choice2;
use rpl_parser::pairs;

use crate::predicates::PredicateError;

pub mod attributes;
mod konst;
pub mod predicates;
pub mod tribool;

#[derive(Debug, Clone, Default)]
pub struct Constraints {
    pub preds: Vec<predicates::PredicateConjunction>,
    pub attrs: attributes::FnAttr,
}

impl Constraints {
    pub fn from_where_block_opt<'i>(
        pre_attrs: impl Iterator<Item = &'i pairs::Attr<'i>>,
        where_block: &Option<pairs::WhereBlock<'i>>,
        path: &'i std::path::Path,
    ) -> Result<Self, PredicateError<'i>> {
        if let Some(where_block) = where_block
            && let Some(constraints) = where_block.ConstraintsSeparatedByComma()
        {
            let (first, following, _) = constraints.get_matched();
            let following = following
                .iter_matched()
                .map(|comma_with_elem| comma_with_elem.get_matched().1);
            // FIXME: this is will return early errors if any of the constraints are invalid, consider
            // collecting all errors instead
            let mut all = std::iter::once(first).chain(following);
            let (preds, attrs): (Vec<PredicateConjunction>, Vec<&pairs::Attribute<'_>>) =
                all.try_fold((Vec::new(), Vec::new()), |(mut preds, mut attrs), constraint| {
                    match constraint.deref() {
                        Choice2::_0(attr) => attrs.push(attr),
                        Choice2::_1(preds_data) => preds.push(PredicateConjunction::from_pairs(preds_data, path)?),
                    }
                    Ok((preds, attrs))
                })?;
            let attrs = FnAttr::parse(pre_attrs, &attrs);
            Ok(Self { preds, attrs })
        } else {
            let preds = Vec::new();
            let attrs = FnAttr::parse(pre_attrs, &[]);
            Ok(Self { preds, attrs })
        }
    }
}
