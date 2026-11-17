//! ADT (struct/enum) matching and field metavar → `FieldIdx` bindings.
//!
//! # Two paths
//!
//! - **Fn MIR path** ([`MatchAdtCtxt::match_adt_for_fn_mir`]): builds type-compatible field
//!   candidates and commits only unambiguous ones. Ambiguous field metvars (e.g. Slab `$len` when
//!   both `capacity` and `len` are `usize`) stay unbound until MIR
//!   [`PlaceElem::FieldPat`](rpl_context::pat::PlaceElem::FieldPat) commits them. Field mappings
//!   are the authoritative source for Session `adt_fields`.
//! - **ADT slot path** ([`MatchAdtCtxt::match_adt`]): eagerly resolves all field metvars by type
//!   uniqueness when no fn body is available. Only `ty_bindings` are merged into Session; field
//!   maps from ADT slots are not Session sources.

use derive_more::derive::Debug;
use rpl_context::PatCtxt;
use rpl_context::pat::{self};
use rustc_abi::FieldIdx;
use rustc_data_structures::fx::{FxHashMap, FxHashSet, FxIndexMap};
use rustc_data_structures::stack::ensure_sufficient_stack;
use rustc_hir::def_id::DefId;
use rustc_index::bit_set::MixedBitSet;
use rustc_index::{Idx, IndexSlice, IndexVec};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Symbol;

use crate::ty::MatchTy as _;
use crate::{CountedMatch, MatchTyCtxt};

/// Resolved ADT field metavar bindings: `(adt_pat, field_metavar) -> FieldIdx`.
pub type AdtFieldMap = FxHashMap<(Symbol, Symbol), FieldIdx>;

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
        Self::with_typing_env(tcx, pcx, pat, adt_pat, ty::TypingEnv::fully_monomorphized())
    }

    /// Prefer the caller's [`TypingEnv`] (e.g. post-analysis) so predicates like
    /// `is_not_unpin` do not fail-open on `PhantomData<$E>` / other param-using types.
    pub fn with_typing_env(
        tcx: TyCtxt<'tcx>,
        pcx: PatCtxt<'pcx>,
        pat: &'pcx pat::RustItems<'pcx>,
        adt_pat: &'a pat::Adt<'pcx>,
        typing_env: ty::TypingEnv<'tcx>,
    ) -> Self {
        // FIXME: `self_ty` should be passed from the caller.
        let ty = MatchTyCtxt::new(tcx, pcx, typing_env, None, pat, &adt_pat.meta);
        Self { ty, adt_pat }
    }

    /// Resolved type metavar bindings after [`Self::match_adt`].
    pub fn resolved_ty_bindings(&self) -> IndexVec<pat::TyVarIdx, ty::Ty<'tcx>> {
        IndexVec::from_fn_n(
            |i| {
                let set = self.ty.ty_vars[i].borrow();
                set.iter().copied().next().unwrap_or_else(|| self.ty.tcx.types.never)
            },
            self.ty.ty_vars.len(),
        )
    }

    /// Match struct/enum shape for fn MIR matching, committing only unambiguous field bindings.
    ///
    /// Field metvars with a single type-compatible candidate are bound immediately; ambiguous
    /// metvars (e.g. Slab `$len` vs `capacity`/`len`) stay unresolved until
    /// [`PlaceElem::FieldPat`](pat::PlaceElem::FieldPat) during statement matching.
    ///
    /// `args` are the rustc generic arguments of the matched `Adt` type (`instantiate`, not
    /// identity).
    #[instrument(level = "trace", skip(self, args))]
    pub fn match_adt_for_fn_mir(
        &self,
        adt: ty::AdtDef<'tcx>,
        args: ty::GenericArgsRef<'tcx>,
    ) -> Option<AdtMatch<'tcx>> {
        let adt_match = self.match_adt_structure(adt, args)?;
        adt_match.field_candidates().candidates.commit_unique_field_candidates();
        Some(adt_match)
    }

    /// Match struct/enum shape and field-type candidates without committing field indices.
    ///
    /// Used by fn MIR matching: field metavar → `FieldIdx` bindings are established later via
    /// [`PlaceElem::FieldPat`](pat::PlaceElem::FieldPat) during statement matching.
    #[instrument(level = "trace", skip(self, args))]
    pub fn match_adt_structure(&self, adt: ty::AdtDef<'tcx>, args: ty::GenericArgsRef<'tcx>) -> Option<AdtMatch<'tcx>> {
        match (&self.adt_pat.kind, adt.adt_kind()) {
            (pat::AdtKind::Struct(variant_pat), ty::AdtKind::Struct) => {
                let fields = self.build_field_candidates(&variant_pat.fields, &adt.non_enum_variant().fields, args)?;
                Some(AdtMatch::new_struct(adt, fields))
            },
            (pat::AdtKind::Enum(variants_pat), ty::AdtKind::Enum) => {
                for (variant_name, variant_pat) in variants_pat.iter() {
                    let Some(variant_idx) = adt
                        .variants()
                        .iter_enumerated()
                        .find(|(_, v)| v.name == *variant_name)
                        .map(|(i, _)| i)
                    else {
                        continue;
                    };
                    let variant = adt.variant(variant_idx);
                    let Some(fields) = self.build_field_candidates(&variant_pat.fields, &variant.fields, args) else {
                        continue;
                    };
                    return Some(AdtMatch::new_enum(adt, variant_idx, fields));
                }
                None
            },
            (
                pat::AdtKind::Struct(_) | pat::AdtKind::Enum(_),
                ty::AdtKind::Struct | ty::AdtKind::Enum | ty::AdtKind::Union,
            ) => None,
        }
    }

    /// Match struct/enum and eagerly resolve all field metavar → `FieldIdx` bindings.
    ///
    /// Used by ADT slot collection where no fn MIR body is available to drive `FieldPat` matching.
    #[instrument(level = "trace", skip(self))]
    pub fn match_adt(&self, adt: ty::AdtDef<'tcx>) -> Option<AdtMatch<'tcx>> {
        match (&self.adt_pat.kind, adt.adt_kind()) {
            (pat::AdtKind::Struct(variant_pat), ty::AdtKind::Struct) => {
                let args = ty::GenericArgs::identity_for_item(self.ty.tcx, adt.did());
                let fields = self.build_field_candidates(&variant_pat.fields, &adt.non_enum_variant().fields, args)?;
                self.match_field_candidates(&fields, &variant_pat.fields, 0)
                    .then(|| AdtMatch::new_struct(adt, fields))
            },
            (pat::AdtKind::Enum(variants_pat), ty::AdtKind::Enum) => {
                let args = ty::GenericArgs::identity_for_item(self.ty.tcx, adt.did());
                for (variant_name, variant_pat) in variants_pat.iter() {
                    let Some(variant_idx) = adt
                        .variants()
                        .iter_enumerated()
                        .find(|(_, v)| v.name == *variant_name)
                        .map(|(i, _)| i)
                    else {
                        continue;
                    };
                    let variant = adt.variant(variant_idx);
                    let fields = self.build_field_candidates(&variant_pat.fields, &variant.fields, args)?;
                    if self.match_field_candidates(&fields, &variant_pat.fields, 0) {
                        return Some(AdtMatch::new_enum(adt, variant_idx, fields));
                    }
                    for (name, m) in fields.candidates.matches.iter() {
                        if let Some(idx) = m.get() {
                            fields.candidates.unmatch(*name, idx);
                        }
                    }
                }
                None
            },
            (
                pat::AdtKind::Struct(_) | pat::AdtKind::Enum(_),
                ty::AdtKind::Struct | ty::AdtKind::Enum | ty::AdtKind::Union,
            ) => None,
        }
    }

    fn build_field_candidates(
        &self,
        fields_pat: &FxIndexMap<Symbol, pat::Field<'pcx>>,
        fields: &'tcx IndexSlice<FieldIdx, ty::FieldDef>,
        args: ty::GenericArgsRef<'tcx>,
    ) -> Option<FieldCandidates<'tcx>> {
        let mut candidates = FieldCandidates::new(fields_pat, fields);
        // Match by field type only — field metvars (e.g. `$len`) are not Rust field names.
        for (field_name, field_pat) in fields_pat.iter() {
            for (field_idx, field) in fields.iter_enumerated() {
                if self.match_field(field_pat, field, args) {
                    candidates.candidates.candidates[field_name].insert(field_idx);
                }
            }
        }
        candidates.candidates_not_empty().then_some(candidates)
    }

    #[expect(clippy::only_used_in_recursion, reason = "for future usage")]
    fn match_field_candidates(
        &self,
        candidates: &FieldCandidates<'tcx>,
        fields_pat: &FxIndexMap<Symbol, pat::Field<'pcx>>,
        field_idx: usize,
    ) -> bool {
        let field_names: Vec<_> = fields_pat.keys().copied().collect();
        if field_idx == field_names.len() {
            return true;
        }
        let field_name = field_names[field_idx];
        for cand in candidates.candidates.candidates[&field_name].iter() {
            if candidates.candidates.r#match(field_name, cand)
                && ensure_sufficient_stack(|| self.match_field_candidates(candidates, fields_pat, field_idx + 1))
            {
                return true;
            }
            candidates.candidates.unmatch(field_name, cand);
        }
        false
    }

    #[instrument(level = "trace", skip(self, args), ret)]
    fn match_field(
        &self,
        field_pat: &pat::Field<'pcx>,
        field: &'tcx ty::FieldDef,
        args: ty::GenericArgsRef<'tcx>,
    ) -> bool {
        let pat_ty = field_pat.ty;
        let ty = self.ty.tcx.type_of(field.did).instantiate(self.ty.tcx, args);
        field_pat.vis.check(field.vis) && self.ty.match_ty(pat_ty, ty)
    }
}

#[derive(Debug, Clone)]
pub struct AdtMatch<'tcx> {
    pub adt: ty::AdtDef<'tcx>,
    kind: AdtMatchKind<'tcx>,
}

#[derive(Debug, Clone)]
enum AdtMatchKind<'tcx> {
    Struct(FieldCandidates<'tcx>),
    Enum {
        #[expect(dead_code, reason = "for future usage")]
        variant_idx: rustc_abi::VariantIdx,
        fields: FieldCandidates<'tcx>,
    },
}

impl<'tcx> AdtMatch<'tcx> {
    pub fn new_struct(adt: ty::AdtDef<'tcx>, fields: FieldCandidates<'tcx>) -> Self {
        Self {
            adt,
            kind: AdtMatchKind::Struct(fields),
        }
    }
    pub fn new_enum(adt: ty::AdtDef<'tcx>, variant_idx: rustc_abi::VariantIdx, fields: FieldCandidates<'tcx>) -> Self {
        Self {
            adt,
            kind: AdtMatchKind::Enum { variant_idx, fields },
        }
    }
    pub fn field_candidates(&self) -> &FieldCandidates<'tcx> {
        match &self.kind {
            AdtMatchKind::Struct(fields) | AdtMatchKind::Enum { fields, .. } => fields,
        }
    }
    pub fn expect_struct(&self) -> &FieldCandidates<'tcx> {
        match &self.kind {
            AdtMatchKind::Struct(variant_match) => variant_match,
            AdtMatchKind::Enum { .. } => panic!("expected struct, got enum"),
        }
    }

    /// Extract committed field metavar bindings for this ADT pattern.
    pub fn field_bindings(&self, adt_pat: Symbol) -> AdtFieldMap {
        let mut map = AdtFieldMap::default();
        for (field_name, matched_idx) in self.field_candidates().candidates.matches.iter() {
            if let Some(idx) = matched_idx.get() {
                map.insert((adt_pat, *field_name), idx);
            }
        }
        map
    }

    /// Whether every field metavar declared in the pattern struct has a committed binding.
    pub fn all_fields_resolved(&self, fields_pat: &FxIndexMap<Symbol, pat::Field<'_>>) -> bool {
        fields_pat.keys().all(|field_name| {
            self.field_candidates()
                .candidates
                .matches
                .get(field_name)
                .is_some_and(|m| m.get().is_some())
        })
    }
}

#[derive(Debug, Clone)]
#[debug("{candidates:?}")]
pub struct Candidates<I: Idx> {
    pub candidates: FxIndexMap<Symbol, MixedBitSet<I>>,
    pub matches: FxIndexMap<Symbol, CountedMatch<I>>,
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
        if self.matches[&name].get().is_some_and(|matched| matched == idx) {
            self.matches[&name].unmatch();
        }
        if self.lookup[idx].get().is_some_and(|matched| matched == name) {
            self.lookup[idx].unmatch();
        }
    }

    /// Commit bindings for field metvars with exactly one type-compatible candidate.
    pub fn commit_unique_field_candidates(&self) {
        for (field_name, bitset) in &self.candidates {
            if self.matches[field_name].get().is_some() {
                continue;
            }
            let mut iter = bitset.iter();
            let Some(first) = iter.next() else { continue };
            if iter.next().is_none() {
                self.r#match(*field_name, first);
            }
        }
    }

    /// Clear all committed field bindings (ignores refcounts) and re-commit unique candidates.
    ///
    /// Needed after [`CheckMirCtxt`] candidate probing, which may temporarily bind FieldPat
    /// matches as a side effect without unmatching them.
    pub fn reset_matches_after_probe(&self) {
        for m in self.matches.values() {
            m.clear();
        }
        for m in &self.lookup {
            m.clear();
        }
        self.commit_unique_field_candidates();
    }
}

#[derive(Debug, Clone)]
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

/// Collect all committed ADT field bindings from a fn MIR match context.
pub fn collect_adt_field_bindings(ty: &MatchTyCtxt<'_, '_>) -> AdtFieldMap {
    let mut map = AdtFieldMap::default();
    for (adt_pat, matches) in ty.adt_matches.borrow().iter() {
        for adt_match in matches.values() {
            map.extend(adt_match.field_bindings(*adt_pat));
        }
    }
    map
}

/// Project `AdtPat → AdtDef` pins from a fn MIR match context for SharedEnv merge.
///
/// Returns `None` when any AdtPat is bound to more than one `DefId` in this context
/// (ambiguous for session unify). Empty map means the body did not pin any AdtPat.
pub fn collect_adt_def_bindings(ty: &MatchTyCtxt<'_, '_>) -> Option<FxHashMap<Symbol, DefId>> {
    let mut map = FxHashMap::default();
    for (adt_pat, per_def) in ty.adt_matches.borrow().iter() {
        match per_def.len() {
            0 => continue,
            1 => {
                let def_id = *per_def.keys().next().expect("len == 1");
                map.insert(*adt_pat, def_id);
            },
            _ => return None,
        }
    }
    Some(map)
}

/// Reset FieldPat bindings left behind by statement candidate probing, then re-commit
/// unambiguous fields. Call after [`CheckMirCtxt`] `build_candidates`.
pub fn reset_adt_field_bindings_after_probe(ty: &MatchTyCtxt<'_, '_>) {
    for matches in ty.adt_matches.borrow().values() {
        for adt_match in matches.values() {
            adt_match.field_candidates().candidates.reset_matches_after_probe();
        }
    }
}

/// Ensure ty-var candidates include every type from ADT field candidate bitsets.
///
/// Statement probing may lock the first FieldPat (e.g. `codec`) and skip later sites
/// (`io`), omitting their types from [`MatchTyCtxt::ty_vars`]. Re-seed from the bitset
/// built by [`MatchAdtCtxt::match_adt_structure`] so ambiguous fields remain tryable.
///
/// Only types that still satisfy the field metavar's predicates (under this context's
/// [`TypingEnv`](ty::TypingEnv)) are inserted — the ADT structure bitset may be looser
/// when predicates fail-open on param-dependent types like `PhantomData<$E>`.
pub fn seed_ty_vars_from_adt_field_candidates(ty: &MatchTyCtxt<'_, '_>) {
    for (adt_pat_sym, per_def) in ty.adt_matches.borrow().iter() {
        let Some(adt_pat) = ty.pat.get_adt(*adt_pat_sym) else {
            continue;
        };
        let fields_pat = match &adt_pat.kind {
            pat::AdtKind::Struct(variant) => &variant.fields,
            pat::AdtKind::Enum(_) => continue,
        };
        for adt_match in per_def.values() {
            let field_defs = adt_match.field_candidates().fields;
            for (field_name, bitset) in &adt_match.field_candidates().candidates.candidates {
                let Some(field_pat) = fields_pat.get(field_name) else {
                    continue;
                };
                let pat::TyKind::TyVar(ty_var) = field_pat.ty.kind() else {
                    continue;
                };
                if ty.is_ty_var_pinned(ty_var.idx) {
                    continue;
                }
                // Put FieldIdx-ordered field types at the front of the candidate set.
                let mut typed: Vec<_> = bitset
                    .iter()
                    .filter_map(|field_idx| {
                        let rust_ty = ty.tcx.type_of(field_defs[field_idx].did).instantiate_identity();
                        ty.match_ty(field_pat.ty, rust_ty).then_some((field_idx, rust_ty))
                    })
                    .collect();
                typed.sort_by_key(|(idx, _)| *idx);
                let mut cell = ty.ty_vars[ty_var.idx].borrow_mut();
                let rest: Vec<_> = cell
                    .iter()
                    .copied()
                    .filter(|t| !typed.iter().any(|(_, rt)| rt == t))
                    .collect();
                cell.clear();
                for (_, rust_ty) in typed {
                    cell.insert(rust_ty);
                }
                for t in rest {
                    cell.insert(t);
                }
            }
        }
    }
}

/// Whether every **used** ADT field metavar has a committed binding.
///
/// `used_field_pats` are `FieldPat` symbols appearing in the current fn MIR pattern.
/// Declared-but-unused field metvars (e.g. `$second` when the body only reads `$first`)
/// may stay unbound so they do not block a match or invent permutations.
///
/// Ambiguous fields that *are* used via `FieldPat` (e.g. Slab `$len`) must still be resolved.
pub fn all_adt_fields_resolved(ty: &MatchTyCtxt<'_, '_>, used_field_pats: &FxHashSet<Symbol>) -> bool {
    ty.adt_matches.borrow().iter().all(|(adt_pat_sym, per_def)| {
        let Some(adt_pat) = ty.pat.get_adt(*adt_pat_sym) else {
            return false;
        };
        per_def.values().all(|adt_match| match &adt_pat.kind {
            pat::AdtKind::Struct(variant) => used_field_pats.iter().all(|field_name| {
                if !variant.fields.contains_key(field_name) {
                    return true;
                }
                adt_match
                    .field_candidates()
                    .candidates
                    .matches
                    .get(field_name)
                    .is_some_and(|m| m.get().is_some())
            }),
            // Enum: FieldCandidates was built for one matched variant; require used metvars.
            pat::AdtKind::Enum(_) => used_field_pats.iter().all(|field_name| {
                match adt_match.field_candidates().candidates.matches.get(field_name) {
                    None => true,
                    Some(m) => m.get().is_some(),
                }
            }),
        })
    })
}

/// Collect `FieldPat` metavar names referenced in a fn MIR pattern body.
pub fn collect_used_field_pats(mir_pat: &pat::FnPatternBody<'_>) -> FxHashSet<Symbol> {
    use pat::visitor::{PatternVisitor, PlaceContext};

    struct Collect {
        fields: FxHashSet<Symbol>,
    }

    impl<'pcx> PatternVisitor<'pcx> for Collect {
        fn visit_projection_elem(
            &mut self,
            place_ref: pat::Place<'pcx>,
            elem: pat::PlaceElem<'pcx>,
            context: PlaceContext,
            location: pat::Location,
        ) {
            if let pat::PlaceElem::FieldPat(field) = elem {
                self.fields.insert(field);
            }
            self.super_projection_elem(place_ref, elem, context, location);
        }
    }

    let mut collect = Collect {
        fields: FxHashSet::default(),
    };
    for (bb, block) in mir_pat.basic_blocks.iter_enumerated() {
        collect.visit_basic_block_data(bb, block);
    }
    collect.fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_unique_field_candidates_leaves_ambiguous_unbound() {
        rustc_span::create_session_if_not_set_then(rustc_span::edition::LATEST_STABLE_EDITION, |_| {
            let fields_pat: FxIndexMap<Symbol, ()> = [(Symbol::intern("$len"), ()), (Symbol::intern("$mem"), ())]
                .into_iter()
                .collect();
            let field_defs: IndexVec<FieldIdx, ()> = IndexVec::from_raw(vec![(), (), ()]);
            let mut candidates = Candidates::new(&fields_pat, &field_defs);
            let len = FieldIdx::from_u32(1);
            let capacity = FieldIdx::from_u32(0);
            let mem = FieldIdx::from_u32(2);

            candidates.candidates[&Symbol::intern("$len")].insert(len);
            candidates.candidates[&Symbol::intern("$len")].insert(capacity);
            candidates.candidates[&Symbol::intern("$mem")].insert(mem);

            candidates.commit_unique_field_candidates();
            assert!(candidates.matches[&Symbol::intern("$len")].get().is_none());
            assert_eq!(candidates.matches[&Symbol::intern("$mem")].get(), Some(mem));
        });
    }

    #[test]
    fn candidates_match_and_unmatch_field_metavar() {
        rustc_span::create_session_if_not_set_then(rustc_span::edition::LATEST_STABLE_EDITION, |_| {
            let fields_pat: FxIndexMap<Symbol, ()> = [(Symbol::intern("$len"), ()), (Symbol::intern("$mem"), ())]
                .into_iter()
                .collect();
            let field_defs: IndexVec<FieldIdx, ()> = IndexVec::from_raw(vec![(), (), ()]);
            let candidates = Candidates::new(&fields_pat, &field_defs);
            let len = FieldIdx::from_u32(1);
            let capacity = FieldIdx::from_u32(0);

            assert!(candidates.r#match(Symbol::intern("$len"), len));
            assert_eq!(candidates.matches[&Symbol::intern("$len")].get(), Some(len));
            // conflicting binding for the same field metavar fails
            assert!(!candidates.r#match(Symbol::intern("$len"), capacity));
            // same binding is idempotent (refcounted)
            assert!(candidates.r#match(Symbol::intern("$len"), len));

            candidates.unmatch(Symbol::intern("$len"), len);
            // one unmatch decrements refcount; binding remains until count hits zero
            assert_eq!(candidates.matches[&Symbol::intern("$len")].get(), Some(len));
            candidates.unmatch(Symbol::intern("$len"), len);
            assert!(candidates.matches[&Symbol::intern("$len")].get().is_none());
            // after unmatch, a different field index can bind
            assert!(candidates.r#match(Symbol::intern("$len"), capacity));
        });
    }

    #[test]
    fn all_fields_resolved_requires_every_metavar() {
        rustc_span::create_session_if_not_set_then(rustc_span::edition::LATEST_STABLE_EDITION, |_| {
            // Use empty Field pats via a stub map keyed like Candidates::matches.
            let fields_pat: FxIndexMap<Symbol, ()> = [(Symbol::intern("$len"), ()), (Symbol::intern("$mem"), ())]
                .into_iter()
                .collect();
            let field_defs: IndexVec<FieldIdx, ()> = IndexVec::from_raw(vec![(), (), ()]);
            let candidates = Candidates::new(&fields_pat, &field_defs);
            let len = FieldIdx::from_u32(1);
            let mem = FieldIdx::from_u32(2);

            // Helper: resolved iff every key in matches has a binding (mirrors AdtMatch::all_fields_resolved).
            let resolved = |c: &Candidates<FieldIdx>| c.matches.values().all(|m| m.get().is_some());

            assert!(!resolved(&candidates));
            assert!(candidates.r#match(Symbol::intern("$len"), len));
            assert!(!resolved(&candidates));
            assert!(candidates.r#match(Symbol::intern("$mem"), mem));
            assert!(resolved(&candidates));
        });
    }

    #[test]
    fn slab_ambiguous_len_binds_via_field_pat_not_unique_commit() {
        rustc_span::create_session_if_not_set_then(rustc_span::edition::LATEST_STABLE_EDITION, |_| {
            // capacity(0) and len(1) are both type-compatible with `$len`; mem(2) is unique.
            let fields_pat: FxIndexMap<Symbol, ()> = [(Symbol::intern("$len"), ()), (Symbol::intern("$mem"), ())]
                .into_iter()
                .collect();
            let field_defs: IndexVec<FieldIdx, ()> = IndexVec::from_raw(vec![(), (), ()]);
            let mut candidates = Candidates::new(&fields_pat, &field_defs);
            let capacity = FieldIdx::from_u32(0);
            let len = FieldIdx::from_u32(1);
            let mem = FieldIdx::from_u32(2);

            candidates.candidates[&Symbol::intern("$len")].insert(capacity);
            candidates.candidates[&Symbol::intern("$len")].insert(len);
            candidates.candidates[&Symbol::intern("$mem")].insert(mem);

            candidates.commit_unique_field_candidates();
            assert!(candidates.matches[&Symbol::intern("$len")].get().is_none());
            assert_eq!(candidates.matches[&Symbol::intern("$mem")].get(), Some(mem));

            // FieldPat visiting `.len` commits the ambiguous metavar.
            assert!(candidates.r#match(Symbol::intern("$len"), len));
            assert_eq!(candidates.matches[&Symbol::intern("$len")].get(), Some(len));
            // Conflicting FieldPat (e.g. capacity) must fail.
            assert!(!candidates.r#match(Symbol::intern("$len"), capacity));
        });
    }
}
