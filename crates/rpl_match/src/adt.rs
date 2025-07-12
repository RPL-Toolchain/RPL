use derive_more::derive::Debug;
use rpl_context::PatCtxt;
use rpl_context::pat::{self};
use rustc_abi::FieldIdx;
use rustc_data_structures::fx::FxIndexMap;
use rustc_index::bit_set::MixedBitSet;
use rustc_index::{Idx, IndexSlice, IndexVec};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Symbol;

use crate::ty::MatchTy as _;
use crate::{CountedMatch, MatchTyCtxt};

pub struct MatchAdtCtxt<'a, 'pcx, 'tcx> {
    ty: MatchTyCtxt<'pcx, 'tcx>,
    adt_pat: &'a pat::Adt<'pcx>,
}

impl<'a, 'pcx, 'tcx> MatchAdtCtxt<'a, 'pcx, 'tcx> {
    pub fn new(
        tcx: TyCtxt<'tcx>,
        pcx: PatCtxt<'pcx>,
        pat: &'pcx pat::RustItems<'pcx>,
        adt_pat: &'a pat::Adt<'pcx>,
    ) -> Self {
        let ty = MatchTyCtxt::new(tcx, pcx, ty::TypingEnv::fully_monomorphized(), pat, &adt_pat.meta);
        Self { ty, adt_pat }
    }

    #[instrument(level = "trace", skip(self))]
    pub fn match_adt(&self, adt: ty::AdtDef<'tcx>) -> Option<AdtMatch<'tcx>> {
        match (&self.adt_pat.kind, adt.adt_kind()) {
            (pat::AdtKind::Struct(variant_pat), ty::AdtKind::Struct) => Some(AdtMatch::new_struct(
                adt,
                self.match_fields(&variant_pat.fields, &adt.non_enum_variant().fields)?,
            )),
            (pat::AdtKind::Enum(_variants_pat), ty::AdtKind::Enum) => todo!(),
            /* {

                let mut candidates = VariantCandidates::new(variants_pat.len(), adt.variants());
                for (variant_name, variant_pat) in variants_pat.iter() {
                    for (variant_idx, variant) in candidates.variants.iter_enumerated() {
                        if let Some(matched) = self.match_variant(variant_pat, variant, variant_idx) {
                            candidates[index].candidates.push(matched);
                        }
                    }
                }
                candidates
                    .iter()
                    .all(|candidates| !candidates.candidates.is_empty())
                    .then(|| AdtMatch::new_enum(adt, VariantCandidates::new(candidates, adt.variants())))
            }, */
            (
                pat::AdtKind::Struct(_) | pat::AdtKind::Enum(_),
                ty::AdtKind::Struct | ty::AdtKind::Enum | ty::AdtKind::Union,
            ) => None,
        }
    }

    // fn match_variant(
    //     &self,
    //     variant_pat: &pat::Variant<'pcx>,
    //     variant: &'tcx ty::VariantDef,
    //     variant_idx: VariantIdx,
    // ) -> Option<VariantMatch<'tcx>> {
    //     self.match_fields(&variant_pat.fields, &variant.fields)
    //         .map(|fields| VariantMatch {
    //             variant_idx,
    //             variant,
    //             fields,
    //         })
    // }

    #[instrument(level = "trace", skip(self), ret)]
    fn match_fields(
        &self,
        fields_pat: &FxIndexMap<Symbol, pat::Field<'pcx>>,
        fields: &'tcx IndexSlice<FieldIdx, ty::FieldDef>,
    ) -> Option<FieldCandidates<'tcx>> {
        let mut candidates = FieldCandidates::new(fields_pat, fields);
        for (field_name, field_pat) in fields_pat.iter() {
            for (field_idx, field) in fields.iter_enumerated() {
                if self.match_field(field_pat, field) {
                    candidates.candidates.candidates[field_name].insert(field_idx);
                }
            }
        }
        candidates.candidates_not_empty().then_some(candidates)
    }

    #[instrument(level = "trace", skip(self), ret)]
    fn match_field(&self, field_pat: &pat::Field<'pcx>, field: &'tcx ty::FieldDef) -> bool {
        let pat_ty = field_pat.ty;
        let ty = self.ty.tcx.type_of(field.did).instantiate_identity();
        self.ty.match_ty(pat_ty, ty)
    }
}

#[derive(Debug)]
#[debug("{adt:?}")]
pub struct AdtMatch<'tcx> {
    pub adt: ty::AdtDef<'tcx>,
    kind: AdtMatchKind<'tcx>,
}

enum AdtMatchKind<'tcx> {
    Struct(FieldCandidates<'tcx>),
    // Enum(VariantCandidates<'tcx>),
}

impl<'tcx> AdtMatch<'tcx> {
    pub fn new_struct(adt: ty::AdtDef<'tcx>, fields: FieldCandidates<'tcx>) -> Self {
        Self {
            adt,
            kind: AdtMatchKind::Struct(fields),
        }
    }
    // pub fn new_enum(adt: ty::AdtDef<'tcx>, variants: VariantCandidates<'tcx>) -> Self {
    //     Self {
    //         adt,
    //         kind: AdtMatchKind::Enum(variants),
    //     }
    // }
    pub fn expect_struct(&self) -> &FieldCandidates<'tcx> {
        match &self.kind {
            AdtMatchKind::Struct(variant_match) => variant_match,
            // AdtMatchKind::Enum(_) => panic!("expected struct, got enum"),
        }
    }
    // pub fn expect_enum(&self) -> &VariantCandidates<'tcx> {
    //     match &self.kind {
    //         AdtMatchKind::Enum(variants) => variants,
    //         AdtMatchKind::Struct(_) => panic!("expected enum, got struct"),
    //     }
    // }
}

#[derive(Debug)]
#[debug("{candidates:?}")]
pub struct Candidates<I: Idx> {
    pub candidates: FxIndexMap<Symbol, MixedBitSet<I>>,
    matches: FxIndexMap<Symbol, CountedMatch<I>>,
    lookup: IndexVec<I, CountedMatch<Symbol>>,
}

impl<I: Idx> Candidates<I> {
    fn new<P, T>(pats: &FxIndexMap<Symbol, P>, elems: &IndexSlice<I, T>) -> Self {
        Self {
            candidates: pats
                .keys()
                .map(|&name| (name, MixedBitSet::new_empty(elems.len())))
                .collect(),
            matches: pats.keys().map(|&name| (name, CountedMatch::new())).collect(),
            lookup: IndexVec::from_elem(CountedMatch::new(), elems),
        }
    }
    pub fn r#match(&self, name: Symbol, idx: I) -> bool {
        match (self.matches[&name].r#match(idx), self.lookup[idx].r#match(name)) {
            (true, true) => return true,
            (true, false) => self.matches[&name].unmatch(),
            (false, true) => self.lookup[idx].unmatch(),
            (false, false) => {},
        }
        false
    }
    pub fn unmatch(&self, name: Symbol, idx: I) {
        debug_assert!(self.matches.contains_key(&name));
        debug_assert!(self.matches[&name].get().is_some());
        debug_assert!(self.matches[&name].get().is_some_and(|matched| matched == idx));
        if self.matches[&name].get().is_some_and(|matched| matched == idx) {
            self.matches[&name].unmatch();
        }
        if self.lookup[idx].get().is_some_and(|matched| matched == name) {
            self.lookup[idx].unmatch();
        }
    }
}

#[derive(Debug)]
#[debug("{candidates:?}")]
pub struct FieldCandidates<'tcx> {
    pub fields: &'tcx IndexSlice<FieldIdx, ty::FieldDef>,
    pub candidates: Candidates<FieldIdx>,
}

impl<'tcx> FieldCandidates<'tcx> {
    fn new(field_pats: &FxIndexMap<Symbol, pat::Field<'_>>, fields: &'tcx IndexSlice<FieldIdx, ty::FieldDef>) -> Self {
        let candidates = Candidates::new(field_pats, fields);
        Self { fields, candidates }
    }
    fn candidates_not_empty(&self) -> bool {
        self.candidates
            .candidates
            .values()
            .all(|candidates| !candidates.is_empty())
    }
}
