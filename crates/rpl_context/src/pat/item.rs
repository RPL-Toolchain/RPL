use std::ops::Deref;
use std::sync::Arc;

use derive_more::derive::Debug;
use rpl_constraints::Constraints;
use rpl_constraints::attributes::{ExtraSpan, Safety, Visibility};
use rpl_meta::collect_elems_separated_by_comma;
use rpl_meta::symbol_table::{WithMetaTable, WithPath};
use rpl_parser::generics::Choice4;
use rpl_parser::pairs;
use rustc_data_structures::fx::{FxHashMap, FxIndexMap};
use rustc_hir::FnHeader;
use rustc_hir::def_id::LocalDefId;
use rustc_middle::mir;
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;

use super::utils::mutability_from_pair_mutability;
use super::{FnPatternBody, FnSymbolTable, NonLocalMetaVars, Path, RawDecleration, RawStatement, Ty};
use crate::PatCtxt;

pub type StructInner<'pcx> = Variant<'pcx>;
pub type Struct<'pcx> = WithMetaTable<'pcx, StructInner<'pcx>>;

pub type EnumInner<'pcx> = FxIndexMap<Symbol, Variant<'pcx>>;
pub type Enum<'pcx> = WithMetaTable<'pcx, EnumInner<'pcx>>;

#[derive(Debug)]
pub struct Adt<'pcx> {
    pub meta: Arc<NonLocalMetaVars<'pcx>>,
    pub kind: AdtKind<'pcx>,
    pub constraints: Constraints,
}

impl<'pcx> Adt<'pcx> {
    pub(crate) fn new_struct(
        inner: StructInner<'pcx>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        constraints: Constraints,
    ) -> Self {
        Self {
            meta,
            kind: AdtKind::Struct(inner),
            constraints,
        }
    }
    pub(crate) fn new_enum(
        inner: EnumInner<'pcx>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        constraints: Constraints,
    ) -> Self {
        Self {
            meta,
            kind: AdtKind::Enum(inner),
            constraints,
        }
    }
    pub fn non_enum_variant_mut(&mut self) -> &mut Variant<'pcx> {
        match &mut self.kind {
            AdtKind::Struct(variant) => variant,
            AdtKind::Enum(_) => panic!("cannot mutate non-enum variant of enum"),
        }
    }
    // pub fn add_variant(&mut self, name: Symbol) -> &mut Variant<'pcx> {
    //     match &mut self.kind {
    //         AdtKind::Struct(_) => panic!("cannot add variant to struct"),
    //         AdtKind::Enum(variants) => variants.entry(name).or_insert_with(Variant::default),
    //     }
    // }
    pub fn non_enum_variant(&self) -> &Variant<'pcx> {
        match &self.kind {
            AdtKind::Struct(variant) => variant,
            AdtKind::Enum(_) => panic!("cannot access non-enum variant of enum"),
        }
    }
    pub fn variant_and_index(&self, name: Symbol) -> (&Variant<'pcx>, usize) {
        match &self.kind {
            AdtKind::Struct(_) => panic!("expected enum"),
            AdtKind::Enum(variants) => {
                let (index, _, variant) = variants
                    .get_full(&name)
                    .unwrap_or_else(|| panic!("variant `${name}` not found"));
                (variant, index)
            },
        }
    }
    pub fn variant(&self, name: Symbol) -> &Variant<'pcx> {
        self.variant_and_index(name).0
    }
    pub fn variant_index(&self, name: Symbol) -> usize {
        self.variant_and_index(name).1
    }
    pub fn is_enum(&self) -> bool {
        matches!(self.kind, AdtKind::Enum(_))
    }
    pub fn is_struct(&self) -> bool {
        matches!(self.kind, AdtKind::Struct(_))
    }
}

#[derive(Debug)]
pub enum AdtKind<'pcx> {
    Struct(StructInner<'pcx>),
    Enum(FxIndexMap<Symbol, Variant<'pcx>>),
}

#[derive(Default, Debug)]
pub struct Variant<'pcx> {
    pub fields: FxIndexMap<Symbol, Field<'pcx>>,
}

impl<'pcx> Variant<'pcx> {
    pub fn add_field(&mut self, name: Symbol, ty: Ty<'pcx>) {
        self.fields.insert(name, Field { ty });
    }
    pub fn field_and_index(&self, name: Symbol) -> (&Field<'pcx>, usize) {
        let (index, _, field) = self
            .fields
            .get_full(&name)
            .unwrap_or_else(|| panic!("field `${name}` not found"));
        (field, index)
    }
    pub fn field(&self, name: Symbol) -> &Field<'pcx> {
        self.field_and_index(name).0
    }
    pub fn field_index(&self, name: Symbol) -> usize {
        self.field_and_index(name).1
    }
}

#[derive(Debug)]
pub struct Field<'pcx> {
    pub ty: Ty<'pcx>,
}

pub struct Impl<'pcx> {
    pub meta: Arc<NonLocalMetaVars<'pcx>>,
    pub(crate) ty: Ty<'pcx>,
    pub(crate) trait_id: Option<Path<'pcx>>,
    pub fns: FxHashMap<Symbol, FnPattern<'pcx>>,
    pub constraints: Constraints,
}

#[derive(Default)]
pub struct FnPatterns<'pcx> {
    /// fn some_name (..) -> _ { .. }
    pub named_fns: FxHashMap<Symbol, &'pcx FnPattern<'pcx>>,
    /// fn _ (..) -> _ { .. }
    pub unnamed_fns: Vec<&'pcx FnPattern<'pcx>>,
}

pub struct FnPattern<'pcx> {
    pub name: Symbol,
    pub meta: Arc<NonLocalMetaVars<'pcx>>,
    pub symbol_table: &'pcx FnSymbolTable<'pcx>,
    pub params: Params<'pcx>,
    pub ret: Option<Ty<'pcx>>,
    pub body: Option<&'pcx FnPatternBody<'pcx>>,
    pub constraints: Constraints,
}

impl<'pcx> FnPattern<'pcx> {
    #[instrument(level = "trace", skip(pair, pcx, fn_sym_tab))]
    pub fn from(
        pair: WithPath<'pcx, &'pcx pairs::Fn<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        mut constraints: Constraints,
    ) -> Self {
        let p = pair.path;
        let (sig, body) = pair.get_matched();
        let (safety, visibility, name, params, ret) = Self::from_sig(WithPath::new(p, sig), pcx, fn_sym_tab);
        constraints.attrs.add_safety(safety);
        constraints.attrs.add_visibility(visibility);

        let (decls, stmts) = if let Some(body) = body.MirBody() {
            let (decls, stmts) = body.get_matched();
            (decls.iter_matched().collect(), stmts.iter_matched().collect())
        } else {
            (Vec::new(), Vec::new())
        };

        let raw_stmts = stmts
            .into_iter()
            .map(|stmt| RawStatement::from(WithPath::new(p, stmt), pcx, fn_sym_tab));
        let raw_decls = decls
            .into_iter()
            .map(|decl| RawDecleration::from(WithPath::new(p, decl), pcx, fn_sym_tab));

        let mut builder = FnPatternBody::builder();
        builder.mk_locals(fn_sym_tab, pcx);
        builder.mk_raw_decls(raw_decls);
        builder.mk_raw_stmts(raw_stmts);
        let mir = builder.build(name, constraints.attrs.output_name);
        debug!(self_idx = ?mir.self_idx, return_idx = ?mir.return_idx, params_idx = ?mir.params_idx, "create mir pattern");
        let body = Some(pcx.mk_mir_pattern(mir));

        Self {
            meta,
            name,
            params,
            ret,
            body,
            constraints,
            symbol_table: fn_sym_tab,
        }
    }

    pub fn from_sig(
        sig: WithPath<'pcx, &'pcx pairs::FnSig<'_>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> (Safety, Visibility, Symbol, Params<'pcx>, Option<Ty<'pcx>>) {
        let p = sig.path;
        let (visibility, unsafety, _, fn_name, _, params_pair, _, ret) = sig.get_matched();
        let safety = Safety::parse(unsafety.as_ref());
        let visibility = Visibility::parse(visibility.as_ref());
        let fn_name = Symbol::intern(fn_name.span.as_str());
        let params = if let Some(params_pair) = params_pair {
            Params::from(WithPath::new(p, params_pair), pcx, fn_sym_tab)
        } else {
            Params::default()
        };
        let ret = ret
            .as_ref()
            .map(|ret| Ty::from_fn_ret(WithPath::new(p, ret), pcx, fn_sym_tab));
        (safety, visibility, fn_name, params, ret)
    }

    #[instrument(level = "trace", skip(self, tcx, header), fields(self = ?self.name), ret)]
    pub fn filter(&self, tcx: TyCtxt<'_>, def_id: LocalDefId, header: Option<FnHeader>) -> bool {
        self.constraints.attrs.filter(tcx, def_id, header)
    }
    /// Returns the extra spans for this function pattern.
    #[instrument(level = "trace", skip(self, tcx), fields(self = ?self.name), ret)]
    pub fn extra_span<'tcx>(&self, tcx: TyCtxt<'tcx>, def_id: LocalDefId) -> Option<ExtraSpan<'tcx>> {
        let mut attr_map = ExtraSpan::default();
        if let Some(inline) = self.constraints.attrs.inline {
            let inline_ = Symbol::intern("inline");
            let attr = inline.check(tcx.get_attrs(def_id, inline_))?;
            _ = attr_map.try_insert(inline_, attr);
        }
        Some(attr_map)
    }
}

#[derive(Default)]
pub struct Params<'pcx> {
    params: Vec<Param<'pcx>>,
    pub non_exhaustive: bool,
}

impl<'pcx> std::ops::Deref for Params<'pcx> {
    type Target = [Param<'pcx>];
    fn deref(&self) -> &Self::Target {
        &self.params
    }
}

pub struct Param<'pcx> {
    pub mutability: mir::Mutability,
    pub ident: Symbol,
    pub ty: Ty<'pcx>,
}

impl<'pcx> Param<'pcx> {
    pub fn from(
        param: WithPath<'pcx, &'pcx pairs::FnParam<'_>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> (Option<Self>, bool) {
        let p = param.path;
        match param.inner.deref() {
            Choice4::_0(_self_param) => {
                // FIXME: implement self param
                (None, false)
            },
            Choice4::_1(normal) => {
                let (mutability, ident, _, ty) = normal.get_matched();
                let mutability = mutability_from_pair_mutability(mutability);
                let ident = Symbol::intern(ident.span.as_str());
                let ty = Ty::from(WithPath::new(p, ty), pcx, fn_sym_tab);
                (Some(Self { mutability, ident, ty }), false)
            },
            Choice4::_2(place_holder_with_type) => {
                let (mutability, placeholder, _, ty) = place_holder_with_type.get_matched();
                let mutability = mutability_from_pair_mutability(mutability);
                let ty = Ty::from(WithPath::new(p, ty), pcx, fn_sym_tab);
                (
                    Some(Self {
                        mutability,
                        ident: Symbol::intern(placeholder.span.as_str()),
                        ty,
                    }),
                    false,
                )
            },
            Choice4::_3(_ellpisis) => (None, true),
        }
    }
}

impl<'pcx> FnPatterns<'pcx> {
    pub fn get_fn_pat(&self, name: Symbol) -> Option<&'pcx FnPattern<'pcx>> {
        self.named_fns.get(&name).copied()
    }
    pub fn iter(&self) -> impl Iterator<Item = &'pcx FnPattern<'pcx>> {
        self.into_iter()
    }
}

impl<'pcx> FnPattern<'pcx> {
    pub fn expect_body(&self) -> &'pcx FnPatternBody<'pcx> {
        match self.body {
            Some(mir_body) => mir_body,
            _ => panic!("expected MIR body"),
        }
    }
}

impl<'pcx> Params<'pcx> {
    pub fn from(
        pair: WithPath<'pcx, &'pcx pairs::FnParamsSeparatedByComma<'_>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = pair.path;
        let params = collect_elems_separated_by_comma!(pair);
        let mut non_exhaustive: bool = false;
        let params = params
            .into_iter()
            .filter_map(|param| {
                let (param, ne) = Param::from(WithPath::new(p, param), pcx, fn_sym_tab);
                non_exhaustive |= ne;
                param
            })
            .collect();
        Self { params, non_exhaustive }
    }
}

impl<'s, 'pcx> IntoIterator for &'s FnPatterns<'pcx> {
    type Item = &'pcx FnPattern<'pcx>;

    type IntoIter = std::iter::Copied<
        std::iter::Chain<
            std::collections::hash_map::Values<'s, Symbol, &'pcx FnPattern<'pcx>>,
            std::slice::Iter<'s, &'pcx FnPattern<'pcx>>,
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.named_fns.values().chain(self.unnamed_fns.iter()).copied()
    }
}
