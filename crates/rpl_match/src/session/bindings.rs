use rpl_constraints::Const;
use rpl_context::pat::{ConstVarIdx, MatchedMap, NonLocalMetaVars, PlaceVarIdx, TyVarIdx};
use rustc_data_structures::fx::FxHashMap;
use rustc_hir::def_id::DefId;
use rustc_index::IndexVec;
use rustc_middle::mir::PlaceRef;
use rustc_middle::ty::Ty;
use rustc_span::Symbol;

use crate::AdtFieldMap;
use crate::matches::artifact::NormalizedMatched;

/// Snapshot of metavar bindings projected onto a shared [`NonLocalMetaVars`] index space.
///
/// `place_vars` are recorded for per-slot diagnostics but are **not** merged into
/// [`MetaBindings`] (SharedEnv): places carry locals and must stay slot-local.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingSnapshot<'tcx> {
    pub ty_vars: IndexVec<TyVarIdx, Ty<'tcx>>,
    pub const_vars: IndexVec<ConstVarIdx, Const<'tcx>>,
    pub place_vars: IndexVec<PlaceVarIdx, PlaceRef<'tcx>>,
    pub adt_fields: AdtFieldMap,
    /// AdtPat name → concrete `AdtDef` observed in this slot's MIR match.
    pub adt_defs: FxHashMap<Symbol, DefId>,
}

impl<'tcx> BindingSnapshot<'tcx> {
    pub fn from_normalized(matched: &NormalizedMatched<'tcx>) -> Self {
        Self {
            ty_vars: matched.ty_vars.clone(),
            const_vars: matched.const_vars.clone(),
            place_vars: matched.place_vars.clone(),
            adt_fields: matched.adt_fields.clone(),
            adt_defs: FxHashMap::default(),
        }
    }

    pub fn from_normalized_with_adt_defs(
        matched: &NormalizedMatched<'tcx>,
        adt_defs: FxHashMap<Symbol, DefId>,
    ) -> Self {
        Self {
            ty_vars: matched.ty_vars.clone(),
            const_vars: matched.const_vars.clone(),
            place_vars: matched.place_vars.clone(),
            adt_fields: matched.adt_fields.clone(),
            adt_defs,
        }
    }

    /// Build a partial snapshot containing only type metavar bindings (e.g. from ADT matching).
    pub fn from_ty_vars(meta: &NonLocalMetaVars<'_>, ty_vars: IndexVec<TyVarIdx, Ty<'tcx>>) -> Self {
        debug_assert_eq!(ty_vars.len(), meta.ty_vars.len());
        Self {
            ty_vars,
            const_vars: IndexVec::from_fn_n(
                |i| {
                    // Placeholder: ADT-only matching does not bind const vars yet.
                    let _ = i;
                    Const::Param(rustc_middle::ty::ParamConst {
                        index: 0,
                        name: Symbol::intern("_"),
                    })
                },
                meta.const_vars.len(),
            ),
            place_vars: IndexVec::from_fn_n(
                |i| {
                    let _ = i;
                    PlaceRef {
                        local: rustc_middle::mir::Local::from_u32(0),
                        projection: &[],
                    }
                },
                meta.place_vars.len(),
            ),
            adt_fields: AdtFieldMap::default(),
            adt_defs: FxHashMap::default(),
        }
    }
}

impl<'tcx> BindingSnapshot<'tcx> {
    /// Merge only type metavar rows into global bindings (ignores placeholder const/place rows).
    pub fn merge_ty_vars_into(&self, bindings: &mut MetaBindings<'tcx>) -> bool {
        bindings.merge_ty_vars(&self.ty_vars)
    }
}

/// SharedEnv: global metavar environment shared across all slots in a match session.
///
/// Cross-slot: `ty_vars`, non-param `const_vars`, `adt_defs`, `adt_fields`.
/// Not shared: `place_vars` (kept empty; places stay per-slot).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaBindings<'tcx> {
    pub ty_vars: IndexVec<TyVarIdx, Option<Ty<'tcx>>>,
    pub const_vars: IndexVec<ConstVarIdx, Option<Const<'tcx>>>,
    pub place_vars: IndexVec<PlaceVarIdx, Option<PlaceRef<'tcx>>>,
    pub adt_fields: AdtFieldMap,
    pub adt_defs: FxHashMap<Symbol, DefId>,
}

impl<'tcx> MetaBindings<'tcx> {
    pub fn new(meta: &NonLocalMetaVars<'_>) -> Self {
        Self {
            ty_vars: IndexVec::from_elem_n(None, meta.ty_vars.len()),
            const_vars: IndexVec::from_elem_n(None, meta.const_vars.len()),
            place_vars: IndexVec::from_elem_n(None, meta.place_vars.len()),
            adt_fields: AdtFieldMap::default(),
            adt_defs: FxHashMap::default(),
        }
    }

    /// Merge a function-slot snapshot into SharedEnv.
    ///
    /// Does **not** merge `place_vars` (slot-local). Skips `Const::Param` (ADT generic params).
    pub fn merge_snapshot(&mut self, snapshot: &BindingSnapshot<'tcx>) -> bool {
        self.merge_ty_vars(&snapshot.ty_vars)
            && self.merge_const_vars(&snapshot.const_vars)
            && self.merge_adt_fields(&snapshot.adt_fields)
            && self.merge_adt_defs(&snapshot.adt_defs)
    }

    pub fn merge_normalized(&mut self, matched: &NormalizedMatched<'tcx>) -> bool {
        self.merge_snapshot(&BindingSnapshot::from_normalized(matched))
    }

    /// Merge type metavar bindings from ADT slot matching (shape/ty only).
    pub fn merge_adt_ty_bindings(&mut self, ty_bindings: &IndexVec<TyVarIdx, Ty<'tcx>>) -> bool {
        self.merge_ty_vars(ty_bindings)
    }

    pub fn bind_adt_def(&mut self, adt_pat: Symbol, def_id: DefId) -> bool {
        match self.adt_defs.get(&adt_pat) {
            None => {
                self.adt_defs.insert(adt_pat, def_id);
                true
            },
            Some(existing) if *existing == def_id => true,
            Some(_) => false,
        }
    }

    pub fn merge_adt_defs(&mut self, defs: &FxHashMap<Symbol, DefId>) -> bool {
        for (&adt_pat, &def_id) in defs {
            if !self.bind_adt_def(adt_pat, def_id) {
                return false;
            }
        }
        true
    }

    pub fn merge_adt_fields(&mut self, fields: &AdtFieldMap) -> bool {
        for (key, idx) in fields {
            match self.adt_fields.get(key) {
                None => {
                    self.adt_fields.insert(*key, *idx);
                },
                Some(existing) if *existing == *idx => {},
                Some(_) => return false,
            }
        }
        true
    }

    pub(crate) fn merge_ty_vars(&mut self, vars: &IndexVec<TyVarIdx, Ty<'tcx>>) -> bool {
        for (idx, value) in vars.iter_enumerated() {
            if Self::should_skip_ty_binding(*value) {
                continue;
            }
            match &self.ty_vars[idx] {
                None => self.ty_vars[idx] = Some(*value),
                Some(existing) if existing == value => {},
                Some(_) => return false,
            }
        }
        true
    }

    /// ADT definition matching may bind type metas to generic parameters; concrete
    /// bindings come from monomorphized function slots instead.
    pub(crate) fn should_skip_ty_binding(ty: Ty<'tcx>) -> bool {
        matches!(
            ty.kind(),
            rustc_middle::ty::TyKind::Param(_) | rustc_middle::ty::TyKind::Never
        )
    }

    /// ADT generic const params are placeholders; wait for monomorphized fn slots.
    pub(crate) fn should_skip_const_binding(konst: Const<'tcx>) -> bool {
        matches!(konst, Const::Param(_))
    }

    fn merge_const_vars(&mut self, vars: &IndexVec<ConstVarIdx, Const<'tcx>>) -> bool {
        for (idx, value) in vars.iter_enumerated() {
            if Self::should_skip_const_binding(*value) {
                continue;
            }
            match &self.const_vars[idx] {
                None => self.const_vars[idx] = Some(*value),
                Some(existing) if existing == value => {},
                Some(_) => return false,
            }
        }
        true
    }

    pub fn equivalent_to(&self, other: &Self) -> bool {
        self.ty_vars == other.ty_vars
            && self.const_vars == other.const_vars
            && self.adt_fields == other.adt_fields
            && self.adt_defs == other.adt_defs
    }

    /// Project SharedEnv into another pattern's metavar index space (`MatchedMap`).
    pub fn map(&self, map: &MatchedMap) -> Self {
        Self {
            ty_vars: IndexVec::from_fn_n(|i| self.ty_vars[map.ty_vars[i]], map.ty_vars.len()),
            const_vars: IndexVec::from_fn_n(|i| self.const_vars[map.const_vars[i]], map.const_vars.len()),
            place_vars: IndexVec::from_fn_n(|i| self.place_vars[map.place_vars[i]], map.place_vars.len()),
            adt_fields: self.adt_fields.clone(),
            adt_defs: self.adt_defs.clone(),
        }
    }
}

#[cfg(test)]
pub(crate) fn merge_index_vec<I: rustc_index::Idx, T: Clone + PartialEq>(
    target: &mut IndexVec<I, Option<T>>,
    source: &IndexVec<I, T>,
    eq: impl Fn(&T, &T) -> bool,
) -> bool {
    for (idx, value) in source.iter_enumerated() {
        match &target[idx] {
            None => target[idx] = Some(value.clone()),
            Some(existing) if eq(existing, value) => {},
            Some(_) => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use rustc_data_structures::fx::FxHashMap;
    use rustc_hir::def_id::{DefId, DefIndex, LocalDefId};
    use rustc_index::IndexVec;
    use rustc_span::Symbol;

    use super::{MetaBindings, merge_index_vec};
    use crate::AdtFieldMap;

    fn dummy_def_id(index: u32) -> DefId {
        LocalDefId {
            local_def_index: DefIndex::from_u32(index),
        }
        .to_def_id()
    }

    #[test]
    fn merge_index_vec_consistent() {
        let mut target: IndexVec<u32, Option<u32>> = IndexVec::from_elem_n(None, 2);
        let source: IndexVec<u32, u32> = IndexVec::from_raw(vec![1, 2]);
        assert!(merge_index_vec(&mut target, &source, |a, b| a == b));
        assert_eq!(target[0], Some(1));
        assert_eq!(target[1], Some(2));
        assert!(merge_index_vec(&mut target, &source, |a, b| a == b));
    }

    #[test]
    fn merge_index_vec_conflict() {
        let mut target: IndexVec<u32, Option<u32>> = IndexVec::from_elem_n(None, 1);
        let a: IndexVec<u32, u32> = IndexVec::from_raw(vec![1]);
        let b: IndexVec<u32, u32> = IndexVec::from_raw(vec![2]);
        assert!(merge_index_vec(&mut target, &a, |x, y| x == y));
        assert!(!merge_index_vec(&mut target, &b, |x, y| x == y));
    }

    #[test]
    fn merge_adt_fields_consistent_and_conflict() {
        rustc_span::create_session_if_not_set_then(rustc_span::edition::LATEST_STABLE_EDITION, |_| {
            use rustc_abi::FieldIdx;

            let mut bindings = MetaBindings {
                ty_vars: IndexVec::new(),
                const_vars: IndexVec::new(),
                place_vars: IndexVec::new(),
                adt_fields: AdtFieldMap::default(),
                adt_defs: FxHashMap::default(),
            };
            let adt = Symbol::intern("$SlabT");
            let len = Symbol::intern("$len");
            let mem = Symbol::intern("$mem");
            let mut fn_fields = AdtFieldMap::default();
            fn_fields.insert((adt, len), FieldIdx::from_u32(1));
            fn_fields.insert((adt, mem), FieldIdx::from_u32(2));
            assert!(bindings.merge_adt_fields(&fn_fields));

            let mut conflicting = AdtFieldMap::default();
            conflicting.insert((adt, len), FieldIdx::from_u32(0));
            assert!(!bindings.merge_adt_fields(&conflicting));
        });
    }

    #[test]
    fn merge_adt_defs_consistent_and_conflict() {
        rustc_span::create_session_if_not_set_then(rustc_span::edition::LATEST_STABLE_EDITION, |_| {
            let mut bindings = MetaBindings {
                ty_vars: IndexVec::new(),
                const_vars: IndexVec::new(),
                place_vars: IndexVec::new(),
                adt_fields: AdtFieldMap::default(),
                adt_defs: FxHashMap::default(),
            };
            let pair = Symbol::intern("$Pair");
            let a = dummy_def_id(1);
            let b = dummy_def_id(2);
            assert!(bindings.bind_adt_def(pair, a));
            assert!(bindings.bind_adt_def(pair, a));
            assert!(!bindings.bind_adt_def(pair, b));

            let mut other = FxHashMap::default();
            other.insert(pair, b);
            let mut fresh = MetaBindings {
                ty_vars: IndexVec::new(),
                const_vars: IndexVec::new(),
                place_vars: IndexVec::new(),
                adt_fields: AdtFieldMap::default(),
                adt_defs: FxHashMap::default(),
            };
            assert!(fresh.bind_adt_def(pair, a));
            assert!(!fresh.merge_adt_defs(&other));
        });
    }
}
