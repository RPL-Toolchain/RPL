use rpl_constraints::attributes::ExtraSpan;
use rpl_context::pat::{MatchedMap, Spanned};
use rustc_hir::FnDecl;
use rustc_index::IndexVec;
use rustc_middle::mir::{Body, Local, PlaceRef};
use rustc_middle::ty::Ty;
use rustc_span::{Span, Symbol};

use super::{Const, Matched, StatementMatch, pat};

/// A normalized version of [`Spanned`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NormalizedSpanned {
    Location(StatementMatch),
    Local(Local),
    Body,
    Output,
    /// A span that is not associated with a specific location, local, or body.
    /// For example, [`rustc_hir::Attribute`]
    Span(Span),
}

impl NormalizedSpanned {
    pub fn span(self, body: &Body<'_>, decl: &FnDecl<'_>) -> Span {
        match self {
            Self::Location(location) => location.span_no_inline(body),
            Self::Local(local) => body.local_decls[local].source_info.span,
            // Special case for the function name, which is not a label.
            Self::Body => body.span,
            Self::Output => decl.output.span(),
            Self::Span(span) => span,
        }
    }
}

/// A normalized version of [`Matched`].
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NormalizedMatched<'tcx> {
    pub ty_vars: IndexVec<pat::TyVarIdx, Ty<'tcx>>,
    pub const_vars: IndexVec<pat::ConstVarIdx, Const<'tcx>>,
    pub place_vars: IndexVec<pat::PlaceVarIdx, PlaceRef<'tcx>>,
    /// Labels and attributes. Sorted by label.
    extra: Vec<(Symbol, NormalizedSpanned)>,
}

impl<'tcx> NormalizedMatched<'tcx> {
    /// Create a new [`NormalizedMatched`] from a [`Matched`] and a [`pat::LabelMap`].
    #[instrument(level = "trace", ret)]
    pub fn new(matched: &Matched<'tcx>, label_map: &pat::LabelMap, extra_spans: &ExtraSpan<'tcx>) -> Self {
        let ty_vars = matched.ty_vars.clone();
        let const_vars = matched.const_vars.clone();
        let place_vars = matched.place_vars.clone();
        let mut labels: Vec<_> = label_map
            .iter()
            .map(|(label, spanned)| match spanned {
                Spanned::Location(location) => (*label, NormalizedSpanned::Location(matched[*location])),
                Spanned::Local(local) => (*label, NormalizedSpanned::Local(matched[*local])),
                Spanned::Body => (*label, NormalizedSpanned::Body),
                Spanned::Output => (*label, NormalizedSpanned::Output),
            })
            .collect();
        labels.extend(
            extra_spans
                .iter()
                .map(|(label, span)| (*label, NormalizedSpanned::Span(span.span()))),
        );
        labels.sort_by_key(|(label, _)| *label);

        NormalizedMatched {
            ty_vars,
            const_vars,
            place_vars,
            extra: labels,
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
            .map(|(label, spanned)| {
                let mapped_label = *matched_map.labels.get(label).unwrap_or(label);
                match spanned {
                    Spanned::Location(location) => (mapped_label, NormalizedSpanned::Location(matched_from[*location])),
                    Spanned::Local(local) => (mapped_label, NormalizedSpanned::Local(matched_from[*local])),
                    Spanned::Body => (mapped_label, NormalizedSpanned::Body),
                    Spanned::Output => (mapped_label, NormalizedSpanned::Output),
                }
            })
            .collect();
        labels.sort_by_key(|(label, _)| *label);

        NormalizedMatched {
            ty_vars,
            const_vars,
            place_vars,
            extra: labels,
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
            .extra
            .iter()
            .map(|(label, spanned)| (*matched_map.labels.get(label).unwrap_or(label), *spanned))
            .collect();
        labels.sort_by_key(|(label, _)| *label);

        NormalizedMatched {
            ty_vars,
            const_vars,
            place_vars,
            extra: labels,
        }
    }

    #[instrument(level = "trace", ret)]
    pub fn has_same_head(&self, other: &Self) -> bool {
        self.ty_vars.len() == other.ty_vars.len()
            && self.const_vars.len() == other.const_vars.len()
            && self.place_vars.len() == other.place_vars.len()
            && self.extra.len() == other.extra.len()
            && self
                .extra
                .iter()
                .zip(&other.extra)
                .all(|((label1, _), (label2, _))| label1 == label2)
    }
}

impl<'tcx> pat::Matched<'tcx> for NormalizedMatched<'tcx> {
    fn span(&self, body: &Body<'_>, decl: &FnDecl<'tcx>, name: &str) -> Span {
        let labels = &self.extra;
        let i = labels
            .binary_search_by_key(&Symbol::intern(name), |(label, _)| *label)
            .unwrap_or_else(|_| {
                panic!("label `{name}` not found in pattern labels: {labels:?}");
            });
        labels[i].1.span(body, decl)
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
