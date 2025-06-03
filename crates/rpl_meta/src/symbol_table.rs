use crate::check::CheckCtxt;
use crate::collect_elems_separated_by_comma;
use crate::context::MetaContext;
use crate::error::{RPLMetaError, RPLMetaResult};
use crate::utils::{Ident, Path};
use derive_more::derive::{Debug, From};
use parser::generics::{Choice3, Choice4};
use parser::pairs::TypeMetaVariable;
use parser::{SpanWrapper, pairs};
use pest_typed::Span;
use rpl_predicates::PredicateConjunction;
use rustc_hash::FxHashMap;
use rustc_span::Symbol;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Copy, From)]
pub enum TypeOrPath<'i> {
    Type(&'i pairs::Type<'i>),
    Path(&'i pairs::Path<'i>),
}

impl<'i> TypeOrPath<'i> {
    pub fn span(&self) -> Span<'i> {
        match self {
            Self::Type(ty) => ty.span,
            Self::Path(path) => path.span,
        }
    }
}

pub enum MetaVariableType {
    Type,
    Const,
    Place,
}

pub enum AdtPatType {
    Struct,
    Enum,
}

// the usize in the hashmap is the *-index of a non-local meta variable
#[derive(Default, Clone, Debug)]
pub struct NonLocalMetaSymTab {
    type_vars: FxHashMap<Symbol, (usize, PredicateConjunction)>,
    const_vars: FxHashMap<Symbol, (usize, PredicateConjunction)>,
    place_vars: FxHashMap<Symbol, (usize, PredicateConjunction)>,
}

impl NonLocalMetaSymTab {
    pub fn add_non_local_meta_var<'i>(
        &mut self,
        mctx: &MetaContext<'i>,
        meta_var: Ident<'i>,
        meta_var_ty: &pairs::MetaVariableType<'i>,
        preds: PredicateConjunction,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        match meta_var_ty.deref() {
            Choice3::_0(_) => {
                let existed = self.type_vars.insert(meta_var.name, (self.type_vars.len(), preds));
                if existed.is_some() {
                    let err = RPLMetaError::NonLocalMetaVariableAlreadyDeclared {
                        meta_var: meta_var.name,
                        span: SpanWrapper::new(meta_var.span, mctx.get_active_path()),
                    };
                    errors.push(err);
                }
            },
            Choice3::_1(_) => {
                let existed = self.const_vars.insert(meta_var.name, (self.const_vars.len(), preds));
                if existed.is_some() {
                    let err = RPLMetaError::NonLocalMetaVariableAlreadyDeclared {
                        meta_var: meta_var.name,
                        span: SpanWrapper::new(meta_var.span, mctx.get_active_path()),
                    };
                    errors.push(err);
                }
            },
            Choice3::_2(_) => {
                let existed = self.place_vars.insert(meta_var.name, (self.place_vars.len(), preds));
                if existed.is_some() {
                    let err = RPLMetaError::NonLocalMetaVariableAlreadyDeclared {
                        meta_var: meta_var.name,
                        span: SpanWrapper::new(meta_var.span, mctx.get_active_path()),
                    };
                    errors.push(err);
                }
            },
        }
    }

    pub fn get_non_local_meta_var<'i>(
        &self,
        mctx: &MetaContext<'i>,
        meta_var: Ident<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> Option<MetaVariableType> {
        if self.type_vars.contains_key(&meta_var.name) {
            Some(MetaVariableType::Type)
        } else if self.const_vars.contains_key(&meta_var.name) {
            Some(MetaVariableType::Const)
        } else if self.place_vars.contains_key(&meta_var.name) {
            Some(MetaVariableType::Place)
        } else {
            let err = RPLMetaError::NonLocalMetaVariableNotDeclared {
                meta_var: meta_var.name,
                span: SpanWrapper::new(meta_var.span, mctx.get_active_path()),
            };
            errors.push(err);
            None
        }
    }

    #[allow(clippy::manual_map)]
    #[instrument(skip(self))]
    pub fn get_from_symbol(&self, symbol: Symbol) -> Option<(MetaVariableType, (usize, PredicateConjunction))> {
        if let Some(idx_and_preds) = self.type_vars.get(&symbol) {
            Some((MetaVariableType::Type, idx_and_preds.clone()))
        } else if let Some(idx_and_preds) = self.const_vars.get(&symbol) {
            Some((MetaVariableType::Const, idx_and_preds.clone()))
        } else if let Some(idx_and_preds) = self.place_vars.get(&symbol) {
            Some((MetaVariableType::Place, idx_and_preds.clone()))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct WithMetaTable<T> {
    pub meta_vars: Arc<NonLocalMetaSymTab>,
    pub inner: T,
}

impl<T> From<(T, Arc<NonLocalMetaSymTab>)> for WithMetaTable<T> {
    fn from(inner: (T, Arc<NonLocalMetaSymTab>)) -> Self {
        Self {
            meta_vars: inner.1,
            inner: inner.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[debug("{inner:?}")]
#[debug(bound(T: std::fmt::Debug))]
pub struct WithPath<'i, T> {
    pub path: &'i std::path::Path,
    pub inner: T,
}

impl<T> Deref for WithPath<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'i, T> WithPath<'i, T> {
    pub fn new(path: &'i std::path::Path, inner: T) -> Self {
        Self { path, inner }
    }
    pub(crate) fn with_ctx(ctx: &MetaContext<'i>, inner: T) -> Self {
        Self {
            path: ctx.get_active_path(),
            inner,
        }
    }
    pub fn map<U, F>(&self, f: F) -> WithPath<'i, U>
    where
        F: FnOnce(&T) -> U,
    {
        let path = self.path;
        let inner = f(&self.inner);
        WithPath { path, inner }
    }
}

impl<'i> From<&WithPath<'i, Ident<'i>>> for SpanWrapper<'i> {
    fn from(with_path: &WithPath<'i, Ident<'i>>) -> Self {
        SpanWrapper::new(with_path.inner.span, with_path.path)
    }
}

impl<'i> From<WithPath<'i, Ident<'i>>> for SpanWrapper<'i> {
    fn from(with_path: WithPath<'i, Ident<'i>>) -> Self {
        SpanWrapper::from(&with_path)
    }
}

#[macro_export]
macro_rules! map_inner {
    ($with_path:ident => $expr:expr) => {
        $crate::symbol_table::WithPath::map($with_path, |with_path| $expr)
    };
}

#[derive(Default)]
pub struct SymbolTable<'i> {
    // meta variables in p[$T: ty]
    pub meta_vars: Arc<NonLocalMetaSymTab>,
    imports: FxHashMap<Symbol, &'i pairs::Path<'i>>,
    structs: FxHashMap<Symbol, Struct<'i>>,
    enums: FxHashMap<Symbol, Enum<'i>>,
    fns: FxHashMap<Symbol, Fn<'i>>,
    unnamed_fns: Vec<Fn<'i>>,
    impls: FxHashMap<(&'i pairs::Type<'i>, Option<&'i pairs::ImplKind<'i>>), Impl<'i>>,
}

impl<'i> SymbolTable<'i> {
    pub fn add_enum(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: Ident<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> Option<&mut Enum<'i>> {
        self.enums
            .try_insert(ident.name, (EnumInner::new(), self.meta_vars.clone()).into())
            .map_err(|entry| {
                let adt = entry.entry.key();
                let err = RPLMetaError::SymbolAlreadyDeclared {
                    ident: *adt,
                    span: SpanWrapper::new(ident.span, mctx.get_active_path()),
                };
                errors.push(err);
            })
            .ok()
    }

    pub fn add_struct(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: Ident<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> Option<&mut Struct<'i>> {
        self.structs
            .try_insert(ident.name, (StructInner::new(), self.meta_vars.clone()).into())
            .map_err(|entry| {
                let adt = entry.entry.key();
                let err = RPLMetaError::SymbolAlreadyDeclared {
                    ident: *adt,
                    span: SpanWrapper::new(ident.span, mctx.get_active_path()),
                };
                errors.push(err);
            })
            .ok()
    }

    pub fn add_fn(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: &'i pairs::FnName<'i>,
        self_ty: Option<&'i pairs::Type<'i>>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> Option<&mut Fn<'i>> {
        let (fn_name, fn_def) = FnInner::parse_from(mctx, ident, self_ty, &self.imports);
        if let Some(fn_name) = fn_name {
            self.fns
                .try_insert(fn_name.name, (fn_def, self.meta_vars.clone()).into())
                .map_err(|entry| {
                    let ident = entry.entry.key();
                    let err = RPLMetaError::SymbolAlreadyDeclared {
                        ident: *ident,
                        span: SpanWrapper::new(fn_name.span, mctx.get_active_path()),
                    };
                    errors.push(err);
                })
                .ok()
        } else {
            self.unnamed_fns.push((fn_def, self.meta_vars.clone()).into());
            Some(self.unnamed_fns.last_mut().unwrap())
        }
        // match ident.deref() {
        //     Choice3::_0(_) => {
        //         self.unnamed_fns.push(
        //             (
        //                 FnInner::new(ident.span, mctx.get_active_path(), self_ty, &self.imports),
        //                 self.meta_vars.clone(),
        //             )
        //                 .into(),
        //         );
        //         Some(self.unnamed_fns.last_mut().unwrap())
        //     },
        //     Choice3::_1(ident) => {
        //         let ident = Ident::from(ident);
        //         self.fns
        //             .try_insert(
        //                 ident.name,
        //                 (
        //                     FnInner::new(ident.span, mctx.get_active_path(), self_ty,
        // &self.imports),                     self.meta_vars.clone(),
        //                 )
        //                     .into(),
        //             )
        //             .map_err(|_entry| {
        //                 let err = RPLMetaError::SymbolAlreadyDeclared {
        //                     ident: ident.name,
        //                     span: SpanWrapper::new(ident.span, mctx.get_active_path()),
        //                 };
        //                 errors.push(err);
        //             })
        //             .ok()
        //     },
        //     Choice3::_2(ident) => {
        //         let ident = Ident::from(ident);
        //         self.fns
        //             .try_insert(
        //                 ident.name,
        //                 (
        //                     FnInner::new(ident.span, mctx.get_active_path(), self_ty,
        // &self.imports),                     self.meta_vars.clone(),
        //                 )
        //                     .into(),
        //             )
        //             .map_err(|_entry| {
        //                 let err = RPLMetaError::SymbolAlreadyDeclared {
        //                     ident: ident.name,
        //                     span: SpanWrapper::new(ident.span, mctx.get_active_path()),
        //                 };
        //                 errors.push(err);
        //             })
        //             .ok()
        //     },
        // }
    }

    /// See [`SymbolTable::get_impl`].
    pub fn add_impl(
        &mut self,
        mctx: &MetaContext<'i>,
        impl_pat: &'i pairs::Impl<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> Option<&mut Impl<'i>> {
        self.impls
            .try_insert(
                (impl_pat.Type(), impl_pat.ImplKind()),
                (ImplInner::new(impl_pat), self.meta_vars.clone()).into(),
            )
            .map_err(|_| {
                let err = RPLMetaError::ImplAlreadyDeclared {
                    span: SpanWrapper::new(impl_pat.span, mctx.get_active_path()),
                };
                errors.push(err);
            })
            .ok()
    }

    pub fn contains_adt(&self, ident: &Ident<'_>) -> bool {
        self.structs.contains_key(&ident.name) || self.enums.contains_key(&ident.name)
    }

    pub fn get_adt(&self, symbol: Symbol) -> Option<(AdtPatType, Symbol)> {
        if self.structs.contains_key(&symbol) {
            Some((AdtPatType::Struct, symbol))
        } else if self.enums.contains_key(&symbol) {
            Some((AdtPatType::Enum, symbol))
        } else {
            None
        }
    }
}

impl<'i> SymbolTable<'i> {
    pub fn collect_symbol_tables(
        mctx: &MetaContext<'i>,
        pat_imports: &[&'i pairs::UsePath<'i>],
        pat_items: impl Iterator<Item = &'i pairs::RPLPatternItem<'i>>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> FxHashMap<Symbol, Self> {
        let mut symbol_tables = FxHashMap::default();
        for pat_item in pat_items {
            let CheckCtxt {
                name,
                imports: _,
                symbol_table: symbols,
                errors: error_vec,
            } = Self::collect_symbol_table(mctx, pat_imports, pat_item);
            errors.extend(error_vec);
            _ = symbol_tables.try_insert(name, symbols).map_err(|entry| {
                let name = entry.entry.key();
                let err = RPLMetaError::SymbolAlreadyDeclared {
                    ident: *name,
                    span: SpanWrapper::new(pat_item.Identifier().span, mctx.get_active_path()),
                };
                errors.push(err);
            });
        }
        symbol_tables
    }

    fn collect_symbol_table(
        mctx: &MetaContext<'i>,
        imports: &[&'i pairs::UsePath<'i>],
        pat_item: &'i pairs::RPLPatternItem<'i>,
    ) -> CheckCtxt<'i> {
        let pat_item_name = Symbol::intern(pat_item.Identifier().span.as_str());
        let mut cctx = CheckCtxt::new(pat_item_name);

        for import in imports {
            cctx.check_import(mctx, import);
        }
        cctx.check_pat_item(mctx, pat_item);
        cctx
    }
}

impl<'i> SymbolTable<'i> {
    pub fn get_fn(&self, name: Symbol) -> Option<&Fn<'i>> {
        // FIXME
        if name == Symbol::intern("_") {
            return self.unnamed_fns.last();
        }
        self.fns.get(&name)
    }
    /// See [`SymbolTable::add_impl`].
    pub fn get_impl(&self, ty: &'i pairs::Type<'i>, impl_kind: Option<&'i pairs::ImplKind<'i>>) -> Option<&Impl<'i>> {
        // FIXME: how to identify an impl?
        self.impls.get(&(ty, impl_kind))
    }
}

pub type Enum<'i> = WithMetaTable<EnumInner<'i>>;

pub struct EnumInner<'i> {
    variants: FxHashMap<Symbol, Variant<'i>>,
}

impl<'i> EnumInner<'i> {
    fn new() -> Self {
        Self {
            variants: FxHashMap::default(),
        }
    }
    pub fn add_variant(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: Ident<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> Option<&mut Variant<'i>> {
        self.variants
            .try_insert(ident.name, Variant::new())
            .map_err(|_entry| {
                let err = RPLMetaError::SymbolAlreadyDeclared {
                    ident: ident.name,
                    span: SpanWrapper::new(ident.span, mctx.get_active_path()),
                };
                errors.push(err);
            })
            .ok()
    }
}

pub struct Variant<'i> {
    fields: FxHashMap<Symbol, &'i pairs::Type<'i>>,
}

impl<'i> Variant<'i> {
    fn new() -> Self {
        Self {
            fields: FxHashMap::default(),
        }
    }
    pub fn add_field(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: Ident<'i>,
        ty: &'i pairs::Type<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        _ = self.fields.try_insert(ident.name, ty).map_err(|_entry| {
            let err = RPLMetaError::SymbolAlreadyDeclared {
                ident: ident.name,
                span: SpanWrapper::new(ident.span, mctx.get_active_path()),
            };
            errors.push(err);
        });
    }
}

pub type Fn<'i> = WithMetaTable<FnInner<'i>>;

pub struct FnInner<'i> {
    #[expect(unused)]
    span: Span<'i>,
    path: &'i std::path::Path,
    types: FxHashMap<Symbol, TypeOrPath<'i>>,
    // FIXME: remove it when `self` parameter is implemented
    self_value: Option<&'i pairs::Type<'i>>,
    ret_value: Option<&'i pairs::Type<'i>>,
    self_param: Option<&'i pairs::SelfParam<'i>>,
    self_ty: Option<&'i pairs::Type<'i>>,
    params: FxHashMap<Symbol, &'i pairs::Type<'i>>,
    locals: FxHashMap<Symbol, (usize, &'i pairs::Type<'i>)>,
    pub symbol_to_local_idx: FxHashMap<Symbol, usize>,
}

impl<'i> FnInner<'i> {
    fn new(
        span: Span<'i>,
        path: &'i std::path::Path,
        self_ty: Option<&'i pairs::Type<'i>>,
        imports: &FxHashMap<Symbol, &'i pairs::Path<'i>>,
    ) -> Self {
        Self {
            span,
            path,
            types: imports.iter().map(|(&k, v)| (k, TypeOrPath::Path(v))).collect(),
            self_value: None,
            ret_value: None,
            self_param: None,
            self_ty,
            params: FxHashMap::default(),
            locals: FxHashMap::default(),
            symbol_to_local_idx: FxHashMap::default(),
        }
    }
    pub(crate) fn parse_from(
        mctx: &MetaContext<'i>,
        fn_name: &'i pairs::FnName<'i>,
        self_ty: Option<&'i pairs::Type<'i>>,
        imports: &FxHashMap<Symbol, &'i pairs::Path<'i>>,
    ) -> (Option<Ident<'i>>, Self) {
        match fn_name.deref() {
            Choice3::_0(_) => (None, Self::new(fn_name.span, mctx.get_active_path(), self_ty, imports)),
            Choice3::_1(ident) => {
                let ident = Ident::from(ident);
                (
                    Some(ident),
                    FnInner::new(ident.span, mctx.get_active_path(), self_ty, imports),
                )
            },
            Choice3::_2(ident) => {
                let ident = Ident::from(ident);
                (
                    Some(ident),
                    FnInner::new(ident.span, mctx.get_active_path(), self_ty, imports),
                )
            },
        }
    }
    fn add_type_impl(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: Ident<'i>,
        ty: TypeOrPath<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        _ = self.types.try_insert(ident.name, ty).map_err(|_entry| {
            let err = RPLMetaError::TypeOrPathAlreadyDeclared {
                type_or_path: ident.name,
                span: SpanWrapper::new(ident.span, mctx.get_active_path()),
            };
            errors.push(err);
        });
    }
    pub fn add_type(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: Ident<'i>,
        ty: &'i pairs::Type<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        self.add_type_impl(mctx, ident, TypeOrPath::Type(ty), errors);
    }
    fn get_type(&self, path: &'i std::path::Path, ident: &Ident<'i>) -> Result<TypeOrPath<'i>, RPLMetaError<'i>> {
        self.types
            .get(&ident.name)
            .copied()
            .ok_or_else(|| RPLMetaError::TypeOrPathNotDeclared {
                span: SpanWrapper::new(ident.span, path),
                type_or_path: ident.name,
            })
    }

    pub fn add_path(&mut self, mctx: &MetaContext<'i>, path: &'i pairs::Path<'i>, errors: &mut Vec<RPLMetaError<'i>>) {
        let ty_or_path = path.into();
        let path: Path<'i> = path.into();
        let ident = path.ident();
        self.add_type_impl(mctx, ident, ty_or_path, errors);
    }
    pub fn add_import(
        &mut self,
        mctx: &MetaContext<'i>,
        path: &'i pairs::Path<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        let ty_or_path = path.into();
        let path: Path<'i> = path.into();
        let ident = path.ident();
        self.add_type_impl(mctx, ident, ty_or_path, errors);
    }

    pub fn get_sorted_locals(&self) -> WithPath<'i, Vec<(Symbol, &'i pairs::Type<'i>)>> {
        let mut locals = self
            .locals
            .iter()
            .map(|(ident, (idx, ty))| (ident, (idx, ty)))
            .collect::<Vec<_>>();
        locals.sort_by_key(|(_, (idx, _))| *idx);
        WithPath::new(
            self.path,
            locals.into_iter().map(|(ident, (_, ty))| (*ident, *ty)).collect(),
        )
    }

    pub fn get_local_idx(&self, symbol: Symbol) -> usize {
        self.symbol_to_local_idx.get(&symbol).copied().unwrap() // should not panic
    }
}

impl<'i> FnInner<'i> {
    pub fn add_self_param(
        &mut self,
        mctx: &MetaContext<'i>,
        self_param: &'i pairs::SelfParam<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        if self.self_param.is_some() {
            errors.push(RPLMetaError::SelfAlreadyDeclared {
                span: SpanWrapper::new(self_param.span, mctx.get_active_path()),
            });
        }
        self.self_param = Some(self_param);
    }
    pub fn add_param(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: Ident<'i>,
        ty: &'i pairs::Type<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        _ = self.params.try_insert(ident.name, ty).map_err(|_entry| {
            let err = RPLMetaError::SymbolAlreadyDeclared {
                ident: ident.name,
                span: SpanWrapper::new(ident.span, mctx.get_active_path()),
            };
            errors.push(err);
        });
    }
    pub fn add_local(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: Ident<'i>,
        ty: &'i pairs::Type<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        let len = self.locals.len();
        if let std::collections::hash_map::Entry::Vacant(e) = self.locals.entry(ident.name) {
            e.insert((len, ty));
            self.symbol_to_local_idx.insert(ident.name, len);
        } else {
            let err = RPLMetaError::SymbolAlreadyDeclared {
                ident: ident.name,
                span: SpanWrapper::new(ident.span, mctx.get_active_path()),
            };
            errors.push(err);
        }
    }
    pub fn add_place_local(
        &mut self,
        mctx: &MetaContext<'i>,
        local: &'i pairs::MirPlaceLocal<'i>,
        ty: &'i pairs::Type<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        match local.deref() {
            Choice4::_0(_place_holder) => {},
            Choice4::_1(self_value) => {
                self.self_value = Some(ty);
                //FIXME: does this reasonable to be an arbitrary idx?
                self.add_local(mctx, self_value.into(), ty, errors);
            },
            Choice4::_2(_ret_value) => self.ret_value = Some(ty),
            Choice4::_3(ident) => self.add_local(mctx, ident.into(), ty, errors),
        }
    }
    fn get_local_impl(&self, ident: Ident<'i>) -> Option<&'i pairs::Type<'i>> {
        self.locals
            .get(&ident.name)
            .map(|(_idx, ty)| ty)
            .or_else(|| self.params.get(&ident.name))
            .copied()
    }
    pub fn get_local(
        &self,
        mctx: &MetaContext<'i>,
        ident: Ident<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> Option<&'i pairs::Type<'i>> {
        self.get_local_impl(ident).or_else(|| {
            let err = RPLMetaError::SymbolNotDeclared {
                ident: ident.name,
                span: SpanWrapper::new(ident.span, mctx.get_active_path()),
            };
            errors.push(err);
            None
        })
    }
    pub fn get_place_local(
        &self,
        mctx: &MetaContext<'i>,
        local: &'i pairs::MirPlaceLocal<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> Option<&'i pairs::Type<'i>> {
        match local.deref() {
            Choice4::_0(_place_holder) => None,
            Choice4::_2(_ret_value) => self.ret_value.or_else(|| {
                errors.push(RPLMetaError::RetNotDeclared {
                    span: SpanWrapper::new(local.span, mctx.get_active_path()),
                });
                None
            }),

            Choice4::_3(ident) => self.get_local(mctx, ident.into(), errors),
            Choice4::_1(_) if self.self_value.is_none() && self.self_param.is_none() => {
                errors.push(RPLMetaError::SelfNotDeclared {
                    span: SpanWrapper::new(local.span, mctx.get_active_path()),
                });
                None
            },
            Choice4::_1(_) => self.self_value.or(self.self_ty).or_else(|| {
                errors.push(RPLMetaError::SelfTypeOutsideImpl {
                    span: SpanWrapper::new(local.span, mctx.get_active_path()),
                });
                None
            }),
        }
    }
}

pub enum Visibility {
    Public,
    Private,
}

pub type Struct<'i> = WithMetaTable<StructInner<'i>>;

pub type StructInner<'pcx> = Variant<'pcx>;

pub type Impl<'i> = WithMetaTable<ImplInner<'i>>;

pub struct ImplInner<'i> {
    #[allow(dead_code)]
    trait_: Option<&'i pairs::Path<'i>>,
    #[allow(dead_code)]
    ty: &'i pairs::Type<'i>,
    fns: FxHashMap<Symbol, Fn<'i>>,
}

impl<'i> ImplInner<'i> {
    pub fn new(impl_pat: &'i pairs::Impl<'i>) -> Self {
        let impl_pat = impl_pat.get_matched();
        let trait_ = impl_pat.1.as_ref().map(|trait_| trait_.get_matched().0);
        Self {
            trait_,
            ty: impl_pat.2,
            fns: FxHashMap::default(),
        }
    }
}

impl<'i> ImplInner<'i> {
    pub fn add_fn(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: Ident<'i>,
        fn_def: Fn<'i>,
    ) -> RPLMetaResult<'i, &mut Fn<'i>> {
        self.fns
            .try_insert(ident.name, fn_def)
            .map_err(|_entry| RPLMetaError::MethodAlreadyDeclared {
                span: SpanWrapper::new(ident.span, mctx.get_active_path()),
            })
    }
}

impl<'i> ImplInner<'i> {
    pub fn get_fn(&self, name: Symbol) -> Option<&Fn<'i>> {
        self.fns.get(&name)
    }
}

#[derive(Default)]
pub struct DiagSymbolTable {
    diags: FxHashMap<Symbol, String>,
}

impl DiagSymbolTable {
    pub fn collect_symbol_tables<'i>(
        mctx: &MetaContext<'i>,
        diags: impl Iterator<Item = &'i pairs::diagBlockItem<'i>>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> FxHashMap<Symbol, DiagSymbolTable> {
        let mut diag_symbols = FxHashMap::default();
        for diag in diags {
            let name = diag.get_matched().0;
            let symbol_table = Self::collect_diag_symbol_table(mctx, diag, errors);
            _ = diag_symbols
                .try_insert(Symbol::intern(name.span.as_str()), symbol_table)
                .map_err(|entry| {
                    let ident = entry.entry.key();
                    let err = RPLMetaError::SymbolAlreadyDeclared {
                        ident: *ident,
                        span: SpanWrapper::new(name.span, mctx.get_active_path()),
                    };
                    errors.push(err);
                });
        }
        diag_symbols
    }

    fn collect_diag_symbol_table<'i>(
        mctx: &MetaContext<'i>,
        diag: &'i pairs::diagBlockItem<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> DiagSymbolTable {
        let mut diag_symbol_table = DiagSymbolTable::default();
        let (_, _, _, _tldr, messages, _) = diag.get_matched();
        if let Some(messages) = messages {
            let messages = collect_elems_separated_by_comma!(messages);
            for message in messages {
                let (ident, _, string) = message.get_matched();
                diag_symbol_table.add_diag(mctx, ident.into(), string.span.to_string(), errors);
            }
        }
        diag_symbol_table
    }

    pub fn add_diag<'i>(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: Ident<'i>,
        message: String,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        _ = self.diags.try_insert(ident.name, message).map_err(|_entry| {
            let err = RPLMetaError::SymbolAlreadyDeclared {
                ident: ident.name,
                span: SpanWrapper::new(ident.span, mctx.get_active_path()),
            };
            errors.push(err);
        });
    }
}

static PRIMITIVES: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "bool", "str",
];

pub(crate) fn ident_is_primitive(ident: &Ident) -> bool {
    PRIMITIVES.contains(&ident.name.to_string().as_str())
}

pub(crate) fn str_is_primitive(ident: &str) -> bool {
    PRIMITIVES.contains(&ident)
}

pub enum TypeVariable {
    MetaVariable(MetaVariableType, usize, PredicateConjunction),
    AdtPat(AdtPatType, Symbol),
}

impl From<(MetaVariableType, (usize, PredicateConjunction))> for TypeVariable {
    fn from((kind, (idx, pred)): (MetaVariableType, (usize, PredicateConjunction))) -> Self {
        TypeVariable::MetaVariable(kind, idx, pred)
    }
}

impl From<(AdtPatType, Symbol)> for TypeVariable {
    fn from((kind, symbol): (AdtPatType, Symbol)) -> Self {
        TypeVariable::AdtPat(kind, symbol)
    }
}

pub trait GetType<'i> {
    fn get_type<'s>(&'s self, ident: &WithPath<'i, Ident<'i>>) -> Result<TypeOrPath<'s>, RPLMetaError<'i>>;
    fn get_type_var<'s>(&'s self, ty_meta_var: &TypeMetaVariable<'i>) -> TypeVariable;
}

impl<'i> GetType<'i> for Fn<'i> {
    fn get_type(&self, ident: &WithPath<'i, Ident<'i>>) -> Result<TypeOrPath<'i>, RPLMetaError<'i>> {
        FnInner::get_type(&self.inner, ident.path, &ident.inner)
    }
    fn get_type_var(&self, ty_meta_var: &TypeMetaVariable) -> TypeVariable {
        let (kind, (idx, pred)) = self
            .meta_vars
            .get_from_symbol(Symbol::intern(ty_meta_var.span.as_str()))
            .unwrap_or_else(|| {
                panic!(
                    "Type variable `{}` not found in symbol table {:?}",
                    ty_meta_var.span.as_str(),
                    self.meta_vars
                )
            });
        // unwrap should be safe here because of the meta pass.
        TypeVariable::MetaVariable(kind, idx, pred)
    }
}

impl<'i> GetType<'i> for WithMetaTable<&'_ FnInner<'i>> {
    fn get_type(&self, ident: &WithPath<'i, Ident<'i>>) -> Result<TypeOrPath<'i>, RPLMetaError<'i>> {
        FnInner::get_type(self.inner, ident.path, &ident.inner)
    }
    fn get_type_var(&self, ty_meta_var: &TypeMetaVariable) -> TypeVariable {
        let (kind, (idx, pred)) = self
            .meta_vars
            .get_from_symbol(Symbol::intern(ty_meta_var.span.as_str()))
            .unwrap(); // unwrap should be safe here because of the meta pass.
        TypeVariable::MetaVariable(kind, idx, pred)
    }
}

impl<'i> GetType<'i> for SymbolTable<'i> {
    fn get_type(&self, ident: &WithPath<'i, Ident<'i>>) -> Result<TypeOrPath<'i>, RPLMetaError<'i>> {
        self.imports
            .get(&ident.name)
            .copied()
            .map(TypeOrPath::Path)
            .ok_or_else(move || RPLMetaError::TypeOrPathNotDeclared {
                span: ident.into(),
                type_or_path: ident.name,
            })
    }
    fn get_type_var(&self, ty_meta_var: &TypeMetaVariable) -> TypeVariable {
        let symbol = Symbol::intern(ty_meta_var.span.as_str());
        self.meta_vars
            .get_from_symbol(symbol)
            .map(TypeVariable::from)
            .or_else(|| self.get_adt(symbol).map(TypeVariable::from))
            .unwrap()
        // unwrap should be safe here because of the meta pass.
    }
}
