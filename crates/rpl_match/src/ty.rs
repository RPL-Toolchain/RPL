use std::cell::RefCell;
use std::iter::zip;

use derive_more::derive::{Debug, Display};
use either::Either;
use rpl_constraints::predicates::{PredicateArg, PredicateKind};
use rpl_context::{PatCtxt, pat};
use rpl_resolve::{PatItemKind, def_path_res};
use rustc_abi::FieldIdx;
use rustc_data_structures::fx::{FxHashMap, FxIndexSet};
use rustc_hir::def::Res;
use rustc_hir::def_id::{DefId, LOCAL_CRATE};
use rustc_hir::definitions::{DefPathData, DefPathDataName};
use rustc_index::IndexVec;
use rustc_middle::mir;
use rustc_middle::ty::{self, TyCtxt, TypingEnv, ValTreeKind};
use rustc_span::Symbol;
use rustc_span::symbol::kw;

use crate::resolve::{lang_item_res, ty_res};
use crate::{AdtMatch, Candidates, MatchAdtCtxt};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Const<'tcx> {
    #[debug("{_0:?}")]
    #[display("{_0}")]
    MIR(mir::Const<'tcx>),
    #[debug("{_0:?}")]
    #[display("{_0}")]
    Param(ty::ParamConst),
}

impl<'tcx> Const<'tcx> {
    pub fn try_eval_target_usize(self, tcx: TyCtxt<'tcx>, typing_env: TypingEnv<'tcx>) -> Option<u64> {
        match self {
            Self::MIR(konst) => Some(konst.eval_target_usize(tcx, typing_env)),
            Self::Param(_) => None,
        }
    }
    /// Returns if `self` may be greater than or equal to `other`.
    #[instrument(level = "info", skip(tcx, typing_env), ret)]
    pub fn maybe_ge(self, other: Self, tcx: TyCtxt<'tcx>, typing_env: TypingEnv<'tcx>) -> bool {
        match (self, other) {
            (Self::MIR(konst1), Self::MIR(konst2)) => {
                let val1 = konst1.eval_target_usize(tcx, typing_env);
                let val2 = konst2.eval_target_usize(tcx, typing_env);
                val1 > val2
            },
            (_, _) => true,
        }
    }
}

pub struct MatchTyCtxt<'pcx, 'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub pcx: PatCtxt<'pcx>,
    pub pat: &'pcx pat::RustItems<'pcx>,
    pub typing_env: ty::TypingEnv<'tcx>,
    pub self_ty: Option<ty::Ty<'tcx>>,
    pub const_vars: IndexVec<pat::ConstVarIdx, RefCell<FxIndexSet<mir::Const<'tcx>>>>,
    pub ty_vars: IndexVec<pat::TyVarIdx, RefCell<FxIndexSet<ty::Ty<'tcx>>>>,
    pub adt_matches: RefCell<FxHashMap<Symbol, FxHashMap<DefId, AdtMatch<'tcx>>>>,
}

impl<'pcx, 'tcx> MatchTyCtxt<'pcx, 'tcx> {
    #[instrument(level = "trace", skip(tcx, pcx, typing_env, pat))]
    pub fn new(
        tcx: TyCtxt<'tcx>,
        pcx: PatCtxt<'pcx>,
        typing_env: ty::TypingEnv<'tcx>,
        self_ty: Option<ty::Ty<'tcx>>,
        pat: &'pcx pat::RustItems<'pcx>,
        meta: &pat::NonLocalMetaVars<'pcx>,
    ) -> Self {
        Self {
            tcx,
            pcx,
            pat,
            typing_env,
            self_ty,
            ty_vars: IndexVec::from_elem(RefCell::new(FxIndexSet::default()), &meta.ty_vars),
            const_vars: IndexVec::from_elem(RefCell::new(FxIndexSet::default()), &meta.const_vars),
            adt_matches: Default::default(),
        }
    }
}

impl<'pcx, 'tcx> MatchTy<'pcx, 'tcx> for MatchTyCtxt<'pcx, 'tcx> {
    fn pat(&self) -> &'pcx pat::RustItems<'pcx> {
        self.pat
    }
    fn pcx(&self) -> PatCtxt<'pcx> {
        self.pcx
    }
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
    fn typing_env(&self) -> ty::TypingEnv<'tcx> {
        self.typing_env
    }

    fn self_ty(&self) -> Option<ty::Ty<'tcx>> {
        self.self_ty
    }

    fn match_ty_var(&self, ty_var: pat::TyVar, ty: ty::Ty<'tcx>) -> bool {
        self.ty_vars[ty_var.idx].borrow_mut().insert(ty);
        true
    }
    #[instrument(level = "trace", skip(self), ret)]
    fn match_ty_const_var(&self, const_var: pat::ConstVar<'pcx>, konst: ty::Const<'tcx>) -> bool {
        //FIXME: handle more cases of `ty::ConstKind`
        if let ty::ConstKind::Value(value) = konst.kind()
            && self.match_ty(const_var.ty, value.ty)
        {
            let const_value = self.tcx.valtree_to_const_val(konst.to_value());
            self.const_vars[const_var.idx]
                .borrow_mut()
                .insert(mir::Const::from_value(const_value, value.ty));
            return true;
        }
        false
    }
    #[instrument(level = "trace", skip(self), ret)]
    fn match_const_var(&self, const_var: pat::ConstVar<'pcx>, konst: mir::Const<'tcx>) -> bool {
        if self.match_ty(const_var.ty, konst.ty()) {
            self.const_vars[const_var.idx].borrow_mut().insert(konst);
            return true;
        }
        false
    }
    fn match_adt_matches(&self, pat: Symbol, adt_match: AdtMatch<'tcx>) -> bool {
        self.adt_matches
            .borrow_mut()
            .entry(pat)
            .or_default()
            .entry(adt_match.adt.did())
            .or_insert(adt_match);
        true
    }

    fn adt_matched(&self, adt_pat: Symbol, adt: ty::AdtDef<'tcx>, f: impl FnOnce(&AdtMatch<'tcx>)) {
        let adt_matches = self.adt_matches.borrow();
        adt_matches
            .get(&adt_pat)
            .and_then(|adt_match| adt_match.get(&adt.did()))
            .map(f);
    }
}

pub(crate) trait MatchTy<'pcx, 'tcx> {
    fn self_ty(&self) -> Option<ty::Ty<'tcx>>;
    fn pat(&self) -> &'pcx pat::RustItems<'pcx>;
    fn pcx(&self) -> PatCtxt<'pcx>;
    fn tcx(&self) -> TyCtxt<'tcx>;
    fn typing_env(&self) -> ty::TypingEnv<'tcx>;

    #[must_use]
    fn match_ty_var(&self, ty_var: pat::TyVar, ty: ty::Ty<'tcx>) -> bool;
    #[must_use]
    fn match_ty_const_var(&self, const_var: pat::ConstVar<'pcx>, konst: ty::Const<'tcx>) -> bool;
    #[must_use]
    fn match_const_var(&self, const_var: pat::ConstVar<'pcx>, konst: mir::Const<'tcx>) -> bool;
    #[must_use]
    fn match_adt_matches(&self, pat: Symbol, adt_match: AdtMatch<'tcx>) -> bool;

    #[instrument(level = "trace", skip(self), ret)]
    fn match_ty(&self, ty_pat: pat::Ty<'pcx>, ty: ty::Ty<'tcx>) -> bool {
        let ty_pat_kind = ty_pat.kind().clone();
        let ty_kind = *ty.kind();
        let matched = match (ty_pat_kind, ty_kind) {
            (pat::TyKind::TyVar(ty_var), _)
                // FIXME: 
                // The following code relies on some assumptions:
                // - The predicate after the declaration of the meta variable is always like
                //   `is_all_safe_trait(self) && !is_primitive(self)`
                if ty_var.pred.clauses.iter().all(|clause|
                    clause.terms.iter().any(|term| {
                        if let PredicateKind::Ty(ty_pred) = term.kind
                            && term.args.iter().all(|arg| { matches!(arg, PredicateArg::SelfValue) }) {
                                let res = ty_pred(self.tcx(), self.typing_env(), ty);
                                if term.is_neg {
                                    !res
                                } else {
                                    res
                                }
                            } // for debugging 
                            else if let PredicateKind::Trivial(trivial) = term.kind {
                                if term.is_neg {
                                    !trivial()
                                } else {
                                    trivial()
                                }
                            }
                            else {
                                false
                            }
                    })
                ) =>
            {
                self.match_ty_var(ty_var, ty)
            },
            (pat::TyKind::Array(ty_pat, konst_pat), ty::Array(ty, konst)) => {
                self.match_ty(ty_pat, ty) && self.match_const(konst_pat, konst)
            },
            (pat::TyKind::Slice(ty_pat), ty::Slice(ty)) => self.match_ty(ty_pat, ty),
            (pat::TyKind::Tuple(tys_pat), ty::Tuple(tys)) => {
                tys_pat.len() == tys.len() && zip(tys_pat, tys).all(|(&ty_pat, ty)| self.match_ty(ty_pat, ty))
            },
            (pat::TyKind::Ref(region_pat, pat_ty, pat_mutblty), ty::Ref(region, ty, mutblty)) => {
                self.match_region(region_pat, region) && pat_mutblty == mutblty && self.match_ty(pat_ty, ty)
            },
            (pat::TyKind::RawPtr(ty_pat, mutability_pat), ty::RawPtr(ty, mutblty)) => {
                mutability_pat == mutblty && self.match_ty(ty_pat, ty)
            },
            (pat::TyKind::Uint(ty_pat), ty::Uint(ty)) => ty_pat == ty,
            (pat::TyKind::Int(ty_pat), ty::Int(ty)) => ty_pat == ty,
            (pat::TyKind::Float(ty_pat), ty::Float(ty)) => ty_pat == ty,
            // (pat::TyKind::Path(path_with_args), ty::Adt(adt, args)) => {
            //     self.match_path_with_args(path_with_args, adt.did(), args)
            // },
            // (pat::TyKind::Path(path_with_args), ty::FnDef(def_id, args)) => {
            //     self.match_path_with_args(path_with_args, def_id, args)
            // },
            (pat::TyKind::Def(def_id_pat, args_pat), ty::Adt(adt, args)) => {
                let def_id = adt.did();
                // trace!(?def_id_pat, ?def_id, ?args_pat, ?args, "match_ty def");
                self.match_generic_args(&args_pat, args, self.tcx().generics_of(def_id)) && def_id_pat == def_id
            },
            (pat::TyKind::Def(def_id_pat, args_pat), ty::FnDef(def_id, args)) => {
                self.match_generic_args(&args_pat, args, self.tcx().generics_of(def_id)) && def_id_pat == def_id
            },
            (pat::TyKind::Path(path_with_args), _) => {
                //FIXME: generics args are ignored.
                match path_with_args.path {
                    pat::Path::Item(path) => ty_res(self.pcx(), self.tcx(), path.0, path_with_args.args),
                    pat::Path::LangItem(item) => lang_item_res(self.pcx(), self.tcx(), item),
                    pat::Path::TypeRelative(_, _) => todo!(),
                }
                .map(|ty_pat| self.match_ty(ty_pat, ty))
                .unwrap_or(false)
            },
            (pat::TyKind::AdtPat(pat), ty::Adt(adt, _)) => {
                if let Some(adt_pat) = self.pat().get_adt(pat)
                    && let Some(adt_match) = self.match_adt(adt_pat, adt) {
                        self.match_adt_matches(pat, adt_match)
                } else {
                    false
                }
            },
            // (pat::TyKind::Alias(alias_kind_pat, path, args), ty::Alias(alias_kind, alias)) => {
            //     alias_kind_pat == alias_kind
            //         && self.match_path(path, alias.def_id)
            //         && self.match_generic_args(args, alias.args)
            // },
            (pat::TyKind::Bool, ty::Bool) => true,
            (pat::TyKind::Self_, _) => {
                self.self_ty() == Some(ty)
            },
            (pat::TyKind::Any, _) => true,
            (
                pat::TyKind::TyVar(_)
                | pat::TyKind::AdtPat(_)
                | pat::TyKind::Array(..)
                | pat::TyKind::Slice(_)
                | pat::TyKind::Tuple(_)
                | pat::TyKind::Ref(..)
                | pat::TyKind::RawPtr(..)
                | pat::TyKind::Uint(_)
                | pat::TyKind::Int(_)
                | pat::TyKind::Float(_)
                | pat::TyKind::Def(_, _)
                | pat::TyKind::Bool
                | pat::TyKind::Str
                | pat::TyKind::Char,
                ty::Bool
                | ty::Char
                | ty::Int(_)
                | ty::Uint(_)
                | ty::Float(_)
                | ty::Adt(..)
                | ty::Foreign(..)
                | ty::Str
                | ty::Array(..)
                | ty::Pat(..)
                | ty::Slice(_)
                | ty::RawPtr(..)
                | ty::Ref(..)
                | ty::FnDef(..)
                | ty::FnPtr(..)
                | ty::Dynamic(..)
                | ty::Closure(..)
                | ty::CoroutineClosure(..)
                | ty::Coroutine(..)
                | ty::CoroutineWitness(..)
                | ty::Never
                | ty::Tuple(_)
                | ty::Alias(..)
                | ty::Param(_)
                | ty::Bound(..)
                | ty::Placeholder(_)
                | ty::Infer(_)
                | ty::Error(_)
                | ty::UnsafeBinder(_),
            ) => false,
        };
        // debug!(?ty_pat, ?ty, matched, "match_ty");
        matched
    }

    #[instrument(level = "trace", skip(self), ret)]
    fn match_adt(&self, adt_pat: &pat::Adt<'pcx>, adt: ty::AdtDef<'tcx>) -> Option<AdtMatch<'tcx>> {
        MatchAdtCtxt::new(self.tcx(), self.pcx(), self.pat(), adt_pat).match_adt(adt)
    }

    #[instrument(level = "trace", skip(self), ret)]
    fn match_const(&self, konst_pat: pat::Const<'pcx>, konst: ty::Const<'tcx>) -> bool {
        match (konst_pat, konst.kind()) {
            (pat::Const::ConstVar(const_var), _) => self.match_ty_const_var(const_var, konst),
            //(pat::Const::Value(_value_pat), ty::Value(_ty, ty::ValTree::Leaf(_value))) => todo!(),
            (
                pat::Const::Value(_value_pat),
                ty::ConstKind::Value(ty::Value {
                    ty: _ty,
                    valtree: _valtree,
                }),
            ) if matches!(*_valtree, ValTreeKind::Leaf(_val)) => {
                todo!()
            },
            (
                // pat::Const::ConstVar(_)
                pat::Const::Value(_),
                ty::ConstKind::Param(_)
                | ty::ConstKind::Infer(_)
                | ty::ConstKind::Bound(..)
                | ty::ConstKind::Placeholder(_)
                | ty::ConstKind::Unevaluated(_)
                | ty::ConstKind::Value(..)
                | ty::ConstKind::Error(_)
                | ty::ConstKind::Expr(_),
            ) => false,
        }
    }

    #[instrument(level = "trace", skip(self), ret)]
    pub fn match_ty_const_var(&self, const_var: pat::ConstVar<'pcx>, konst: ty::Const<'tcx>) -> bool {
        //FIXME: handle more cases of `ty::ConstKind`
        match konst.kind() {
            ty::ConstKind::Param(param) => {
                let ty = param.find_ty_from_env(self.typing_env.param_env);
                self.match_ty(const_var.ty, ty) && {
                    // We can't convert a const generic param into a `mir::Const`
                    self.const_vars[const_var.idx].borrow_mut().insert(Const::Param(param));
                    true
                }
            },
            ty::ConstKind::Value(value) => {
                self.match_ty(const_var.ty, value.ty) && {
                    let const_value = self.tcx.valtree_to_const_val(value);
                    self.const_vars[const_var.idx]
                        .borrow_mut()
                        .insert(Const::MIR(mir::Const::from_value(const_value, value.ty)));
                    true
                }
            },
            _ => false,
        }
    }

    #[instrument(level = "trace", skip(self), ret)]
    pub fn match_const_var(&self, const_var: pat::ConstVar<'pcx>, konst: mir::Const<'tcx>) -> bool {
        if self.match_ty(const_var.ty, konst.ty()) {
            self.const_vars[const_var.idx].borrow_mut().insert(Const::MIR(konst));
            return true;
        }
        false
    }

    #[instrument(level = "debug", skip(self), ret)]
    fn match_region(&self, pat: pat::RegionKind, region: ty::Region<'tcx>) -> bool {
        // FIXME: implement region matching
        true
        // matches!(
        //     (pat, region.kind()),
        //     (pat::RegionKind::ReStatic, ty::RegionKind::ReStatic) | (pat::RegionKind::ReAny, _)
        // )
    }

    /// Match type path
    #[instrument(level = "trace", skip(self), ret)]
    fn match_path_with_args(
        &self,
        path_with_args: pat::PathWithArgs<'pcx>,
        def_id: DefId,
        args: ty::GenericArgsRef<'tcx>,
    ) -> bool {
        let generics = self.tcx().generics_of(def_id);
        self.match_path(path_with_args.path, def_id) && self.match_generic_args(&path_with_args.args, args, generics)
    }

    #[instrument(level = "debug", skip(self), ret)]
    fn match_path(&self, path: pat::Path<'pcx>, def_id: DefId) -> bool {
        let matched = match path {
            // pat::Path::Item(path) => matches!(self.match_item_path(path, def_id), Some([])),
            pat::Path::Item(path) => self.match_item_path_by_def_path(path, def_id),
            pat::Path::TypeRelative(ty, name) => {
                self.tcx().item_name(def_id) == name
                    && self
                        .tcx()
                        .opt_parent(def_id)
                        .is_some_and(|did| self.match_ty(ty, self.tcx().type_of(did).instantiate_identity()))
            },
            pat::Path::LangItem(lang_item) => self.tcx().is_lang_item(def_id, lang_item),
        };
        // debug!(?path, ?def_id, matched, "match_path");
        matched
    }

    /// Resolve definition path from `path`.
    // FIXME: when searching in the same crate, if with the same kind, an item path should always be resolved to the
    // same item, so this can be cached for performance.
    #[instrument(level = "trace", skip(self), ret)]
    fn match_item_path_by_def_path(&self, path: pat::ItemPath<'pcx>, def_id: DefId) -> bool {
        let kind = if let Some(kind) = PatItemKind::infer_from_def_kind(self.tcx().def_kind(def_id)) {
            kind
        } else {
            return false;
        };
        let res = def_path_res(self.tcx(), path.0, kind);
        trace!(?res);
        let mut res = res.into_iter().filter_map(|res| match res {
            Res::Def(_, id) => Some(id),
            _ => None,
        });
        match res.next() {
            Some(pat_id) => {
                // FIXME: there should be at most one item matching specific item kind
                assert!(res.next().is_none());

                trace!(?pat_id, ?def_id);

                pat_id == def_id
            },
            None => {
                let def_path = self.tcx().def_path(def_id);
                let def_path: Vec<_> = std::iter::once(self.tcx().crate_name(def_path.krate))
                    .chain(def_path.data.iter().map(|data| match data.data.name() {
                        DefPathDataName::Named(symbol) | DefPathDataName::Anon { namespace: symbol } => symbol,
                    }))
                    .collect();
                debug!(?path, ?def_id, ?kind, ?def_path, "fallback");
                path.0 == def_path
            },
        }
    }

    #[instrument(level = "trace", skip(self), ret)]
    fn match_item_path(&self, path: pat::ItemPath<'pcx>, def_id: DefId) -> Option<&'pcx [Symbol]> {
        let &[krate, ref in_crate @ ..] = path.0 else {
            // an empty `ItemPath`
            return None;
        };
        let def_path = self.tcx().def_path(def_id);
        let matched = match def_path.krate {
            LOCAL_CRATE => krate == kw::Crate,
            _ => self.tcx().crate_name(def_path.krate) == krate,
        };
        let mut pat_iter = in_crate.iter();
        use DefPathData::{Impl, TypeNs, ValueNs};
        let mut iter = def_path
            .data
            .iter()
            .filter(|data| matches!(data.data, Impl | TypeNs(_) | ValueNs(_)));
        let matched = matched
            && std::iter::zip(pat_iter.by_ref(), iter.by_ref())
                .all(|(&path, data)| data.data.get_opt_name().is_some_and(|name| name == path));
        // Check that `iter` (from `def_path`) is not longer than `pat_iter` (from `path`)
        let matched = matched && iter.next().is_none();
        debug!(?path, ?def_id, matched, "match_item_path");
        matched.then_some(pat_iter.as_slice())
    }

    #[instrument(level = "trace", skip(self), ret)]
    fn match_generic_args(
        &self,
        mut args_pat: &[pat::GenericArgKind<'pcx>],
        args: &'tcx [ty::GenericArg<'tcx>],
        generics: &'tcx ty::Generics,
    ) -> bool {
        if let Some(parent) = generics.parent
            && let Some((parent_args_pat, pat)) = args_pat.split_at_checked(generics.parent_count)
        {
            args_pat = pat;
            let args = &args[..generics.parent_count];
            let generics = self.tcx().generics_of(parent);

            if !self.match_generic_args(parent_args_pat, args, generics) {
                return false;
            }
        }
        trace!(?args_pat);
        // Is it necessary to call this function?
        let args_all = generics.own_args(args);
        trace!(?args_all);
        let args_no_default = generics.own_args_no_defaults(self.tcx(), args);
        trace!(?args_no_default);
        if args_pat.len() < args_no_default.len() || args_pat.len() > args_all.len() {
            false
        } else {
            // FIXME:
            // directly zip args_all[..args_pat.len()]?
            args_pat
                .iter()
                .zip(
                    args_no_default
                        .iter()
                        .chain(args_all[args_no_default.len()..args_pat.len()].iter()),
                )
                .all(|(pat, arg)| self.match_generic_arg(*pat, *arg))
        }
    }

    #[instrument(level = "trace", skip(self), ret)]
    fn match_generic_arg(&self, arg_pat: pat::GenericArgKind<'pcx>, arg: ty::GenericArg<'tcx>) -> bool {
        match (arg_pat, arg.unpack()) {
            (pat::GenericArgKind::Lifetime(region_pat), ty::GenericArgKind::Lifetime(region)) => {
                self.match_region(region_pat, region)
            },
            (pat::GenericArgKind::Type(ty_pat), ty::GenericArgKind::Type(ty)) => self.match_ty(ty_pat, ty),
            (pat::GenericArgKind::Const(konst_pat), ty::GenericArgKind::Const(konst)) => {
                self.match_const(konst_pat, konst)
            },
            (
                pat::GenericArgKind::Lifetime(_) | pat::GenericArgKind::Type(_) | pat::GenericArgKind::Const(_),
                ty::GenericArgKind::Lifetime(_) | ty::GenericArgKind::Type(_) | ty::GenericArgKind::Const(_),
            ) => false,
        }
    }

    fn adt_matched(&self, adt_pat: Symbol, adt: ty::AdtDef<'tcx>, f: impl FnOnce(&AdtMatch<'tcx>));

    fn for_variant_and_match(
        &self,
        adt_pat: Symbol,
        adt: ty::AdtDef<'tcx>,
        // variant_idx_pat: Option<Symbol>,
        // variant_idx: Option<VariantIdx>,
        f: impl FnOnce(&pat::Variant<'pcx>, &Candidates<FieldIdx>, &'tcx ty::VariantDef),
    ) {
        self.adt_matched(adt_pat, adt, |adt_match| {
            let adt_pat = self
                .pat()
                .get_adt(adt_pat)
                .unwrap_or_else(|| panic!("AdtPat `${adt_pat}` not found"));
            if adt_pat.is_enum() {
                todo!()
                // let (variant_pat, variant_index) =
                //     adt_pat.variant_and_index(variant_idx_pat.expect("variant_idx_pat is None"));
                // let variant = adt.variant(variant_idx.expect("variant_idx is None"));
                // let variant_match = adt_match.expect_enum().candidates[variant_index]
                //     .matched()
                //     .expect("variant not matched");
                // (variant_pat, variant_match, variant)
            } else {
                let variant_pat = adt_pat.non_enum_variant();
                let variant_match = &adt_match.expect_struct().candidates;
                let variant = adt.non_enum_variant();
                f(variant_pat, variant_match, variant);
            }
        })
    }
}
