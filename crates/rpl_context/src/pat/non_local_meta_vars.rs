use std::ops::Deref;

use rpl_meta::collect_elems_separated_by_comma;
use rpl_meta::symbol_table::{GetType, WithPath};
use rpl_parser::generics::Choice3;
use rpl_parser::pairs;
use rpl_predicates::PredicateConjunction;
use rustc_index::IndexVec;
use rustc_span::Symbol;

use crate::PatCtxt;
use crate::pat::{Ty, with_path};

rustc_index::newtype_index! {
    #[debug_format = "?T{}"]
    #[orderable]
    pub struct TyVarIdx {}
}

rustc_index::newtype_index! {
    #[debug_format = "?C{}"]
    #[orderable]
    pub struct ConstVarIdx {}
}

rustc_index::newtype_index! {
    #[debug_format = "?P{}"]
    #[orderable]
    pub struct PlaceVarIdx {}
}

#[derive(Clone)]
pub struct TyVar {
    pub idx: TyVarIdx,
    pub name: Symbol,
    pub pred: PredicateConjunction,
}

#[derive(Clone, Copy)]
pub struct ConstVar<'pcx> {
    pub idx: ConstVarIdx,
    pub name: Symbol,
    pub ty: Ty<'pcx>,
}

impl<'pcx> ConstVar<'pcx> {
    #[expect(unused_variables, reason = "predicates on const variables are not handle yet")] //FIXME
    pub(crate) fn from(
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx impl GetType<'pcx>,
        idx: usize,
        ty: WithPath<'pcx, &'pcx pairs::Type<'pcx>>,
        pred: rpl_predicates::PredicateConjunction,
    ) -> Self {
        let name = Symbol::intern(ty.span.as_str());
        let ty = Ty::from(ty, pcx, fn_sym_tab);
        Self {
            idx: idx.into(),
            name,
            ty,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlaceVar<'pcx> {
    pub idx: PlaceVarIdx,
    pub name: Symbol,
    pub ty: Ty<'pcx>,
}

impl<'pcx> PlaceVar<'pcx> {
    pub fn new(idx: PlaceVarIdx, name: Symbol, ty: Ty<'pcx>) -> Self {
        Self { idx, name, ty }
    }
}

#[derive(Default, Debug)]
pub struct NonLocalMetaVars<'pcx> {
    pub ty_vars: IndexVec<TyVarIdx, TyVar>,
    pub const_vars: IndexVec<ConstVarIdx, ConstVar<'pcx>>,
    pub place_vars: IndexVec<PlaceVarIdx, PlaceVar<'pcx>>,
}

impl<'pcx> NonLocalMetaVars<'pcx> {
    pub fn add_ty_var(&mut self, name: Symbol, preds: Option<&pairs::PredicateConjunction<'_>>) {
        let idx = self.ty_vars.next_index();
        let pred = PredicateConjunction::from_pairs_opt(preds);
        let ty_var = TyVar { idx, name, pred };
        self.ty_vars.push(ty_var);
    }
    pub fn add_const_var(&mut self, name: Symbol, ty: Ty<'pcx>) {
        let idx = self.const_vars.next_index();
        let const_var = ConstVar { idx, name, ty };
        self.const_vars.push(const_var);
    }
    pub fn add_place_var(&mut self, name: Symbol, ty: Ty<'pcx>) {
        let idx = self.place_vars.next_index();
        let place_var = PlaceVar { idx, name, ty };
        self.place_vars.push(place_var);
    }

    pub fn from_meta_decls(
        meta_decls: Option<WithPath<'pcx, &'pcx pairs::MetaVariableDeclList<'pcx>>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx impl GetType<'pcx>,
    ) -> Self {
        let mut meta = Self::default();
        if let Some(decls) = meta_decls
            && let p = decls.path
            && let Some(decls) = decls.get_matched().1
        {
            let decls = collect_elems_separated_by_comma!(decls).collect::<Vec<_>>();
            // handle the type meta variable first
            let mut type_vars = Vec::new();
            let mut konst_vars = Vec::new();
            let mut place_vars = Vec::new();
            for decl in decls {
                let (ident, _, ty, preds) = decl.get_matched();
                let ident = Symbol::intern(ident.span.as_str());
                let preds = preds.as_ref().map(|preds| preds.get_matched().1);
                match ty.deref() {
                    Choice3::_0(_ty) => type_vars.push((ident, preds)),
                    Choice3::_1(konst) => konst_vars.push((ident, konst)),
                    Choice3::_2(place) => place_vars.push((ident, place)),
                }
            }
            for (ident, pred_opt) in type_vars {
                meta.add_ty_var(ident, pred_opt);
            }
            for (ident, konst) in konst_vars {
                let ty = Ty::from(with_path(p, konst.get_matched().2), pcx, fn_sym_tab);
                // FIXME: konst vars' predicates
                meta.add_const_var(ident, ty);
            }
            for (ident, place) in place_vars {
                let ty = Ty::from(with_path(p, place.get_matched().2), pcx, fn_sym_tab);
                // FIXME: place vars' predicates
                meta.add_place_var(ident, ty);
            }
        }
        meta
    }
}
