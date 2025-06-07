use either::Either;
use rpl_context::pat::{MatchedMap, Spanned};
use rustc_index::IndexVec;
use rustc_middle::mir::{Const, Local, PlaceRef};
use rustc_middle::ty::Ty;
use rustc_span::{Span, Symbol};

use crate::{Matched, StatementMatch, pat};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NormalizedMatched<'tcx> {
    pub ty_vars: IndexVec<pat::TyVarIdx, Ty<'tcx>>,
    pub const_vars: IndexVec<pat::ConstVarIdx, Const<'tcx>>,
    pub place_vars: IndexVec<pat::PlaceVarIdx, PlaceRef<'tcx>>,
    pub labels: Vec<(Symbol, Either<StatementMatch, Local>)>,
}

impl<'tcx> NormalizedMatched<'tcx> {
    #[instrument(level = "trace", ret)]
    pub fn new(matched: &Matched<'tcx>, label_map: &pat::LabelMap) -> Self {
        let ty_vars = matched.ty_vars.clone();
        let const_vars = matched.const_vars.clone();
        let place_vars = matched.place_vars.clone();
        let mut labels: Vec<_> = label_map
            .iter()
            .map(|(label, spanned)| match spanned {
                Spanned::Location(location) => (*label, Either::Left(matched[*location])),
                Spanned::Local(local) => (*label, Either::Right(matched[*local])),
            })
            .collect();
        labels.sort_by_key(|(label, _)| *label);

        NormalizedMatched {
            ty_vars,
            const_vars,
            place_vars,
            labels,
        }
    }

    /// Map [`Matched`] from one pattern to another.
    ///
    /// This is useful for normalizing patterns that have been matched against a different set of
    /// meta variables.
    #[instrument(level = "trace", ret)]
    pub fn normalize(matched_map: &MatchedMap, matched_from: &Matched<'tcx>, label_map_from: &pat::LabelMap) -> Self {
        let ty_vars = IndexVec::from_fn_n(
            |i| matched_from.ty_vars[matched_map.ty_vars[i]],
            matched_map.ty_vars.len(),
        );
        let const_vars = IndexVec::from_fn_n(
            |i| matched_from.const_vars[matched_map.const_vars[i]],
            matched_map.const_vars.len(),
        );
        let place_vars = IndexVec::from_fn_n(
            |i| matched_from.place_vars[matched_map.place_vars[i]],
            matched_map.place_vars.len(),
        );
        let mut labels: Vec<_> = label_map_from
            .iter()
            .map(|(label, spanned)| match spanned {
                Spanned::Location(location) => (
                    *matched_map.labels.get(label).unwrap_or(label),
                    Either::Left(matched_from[*location]),
                ),
                Spanned::Local(local) => (
                    *matched_map.labels.get(label).unwrap_or(label),
                    Either::Right(matched_from[*local]),
                ),
            })
            .collect();
        labels.sort_by_key(|(label, _)| *label);

        NormalizedMatched {
            ty_vars,
            const_vars,
            place_vars,
            labels,
        }
    }

    /// Map [`Matched`] from one pattern to another.
    ///
    /// This is useful for normalizing patterns that have been matched against a different set of
    /// meta variables.
    #[instrument(level = "trace", ret)]
    pub fn map(self, matched_map: &MatchedMap) -> Self {
        let ty_vars = IndexVec::from_fn_n(|i| self.ty_vars[matched_map.ty_vars[i]], matched_map.ty_vars.len());
        let const_vars = IndexVec::from_fn_n(
            |i| self.const_vars[matched_map.const_vars[i]],
            matched_map.const_vars.len(),
        );
        let place_vars = IndexVec::from_fn_n(
            |i| self.place_vars[matched_map.place_vars[i]],
            matched_map.place_vars.len(),
        );
        let mut labels: Vec<_> = self
            .labels
            .iter()
            .map(|(label, spanned)| (*matched_map.labels.get(label).unwrap_or(label), *spanned))
            .collect();
        labels.sort_by_key(|(label, _)| *label);

        NormalizedMatched {
            ty_vars,
            const_vars,
            place_vars,
            labels,
        }
    }
}

impl<'tcx> pat::Matched<'tcx> for NormalizedMatched<'tcx> {
    fn span(&self, body: &rustc_middle::mir::Body<'_>, name: &str) -> Span {
        let labels = &self.labels;
        let i = labels
            .binary_search_by_key(&Symbol::intern(name), |(label, _)| *label)
            .unwrap_or_else(|_| {
                panic!("label `{name}` not found in pattern labels: {labels:?}");
            });
        match labels[i].1 {
            Either::Left(location) => location.span_no_inline(body),
            Either::Right(local) => body.local_decls[local].source_info.span,
        }
    }
    fn type_meta_var(&self, idx: pat::TyVarIdx) -> Ty<'tcx> {
        self.ty_vars[idx]
    }
    fn const_meta_var(&self, idx: pat::ConstVarIdx) -> Const<'tcx> {
        self.const_vars[idx]
    }
    fn place_meta_var(&self, idx: pat::PlaceVarIdx) -> PlaceRef<'tcx> {
        self.place_vars[idx]
    }
}
