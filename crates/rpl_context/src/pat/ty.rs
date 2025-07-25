use std::ops::Deref;

use rpl_meta::symbol_table::{GetType, MetaVariable, TypeOrPath, WithPath};
use rpl_meta::{collect_elems_separated_by_comma, utils};
use rpl_parser::generics::{Choice2, Choice3, Choice4, Choice10, Choice12, Choice14};
use rpl_parser::pairs;
use rustc_data_structures::fx::FxHashSet;
use rustc_data_structures::packed::Pu128;
use rustc_hir::def_id::DefId;
use rustc_hir::{LangItem, PrimTy};
use rustc_middle::mir;
use rustc_middle::ty::{self};
use rustc_span::Symbol;

use super::FnSymbolTable;
use crate::PatCtxt;
use crate::cvt_prim_ty::CvtPrimTy;
use crate::pat::non_local_meta_vars::{ConstVar, TyVar};

// FIXME: Use interning for the types
#[derive(Clone, Copy)]
#[rustc_pass_by_value]
pub struct Ty<'pcx>(pub(crate) &'pcx TyKind<'pcx>);

impl<'pcx> Ty<'pcx> {
    pub fn kind(self) -> &'pcx TyKind<'pcx> {
        self.0
    }
    // FIXME: this may breaks uniqueness of `Ty`
    pub fn from_ty_lossy(pcx: PatCtxt<'pcx>, ty: ty::Ty<'_>, args: GenericArgsRef<'pcx>) -> Option<Self> {
        Some(pcx.mk_ty(TyKind::from_ty_lossy(pcx, ty, args)?))
    }
    pub fn from_prim_ty(pcx: PatCtxt<'pcx>, ty: PrimTy) -> Self {
        pcx.mk_ty(TyKind::from(ty))
    }
    pub fn from_def(pcx: PatCtxt<'pcx>, def_id: DefId, args: GenericArgsRef<'pcx>) -> Self {
        pcx.mk_ty(TyKind::Def(def_id, args))
    }

    pub fn from<'mcx>(
        ty: WithPath<'mcx, &'mcx pairs::Type<'mcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &impl GetType<'mcx>,
    ) -> Self {
        let p = ty.path;
        match ty.inner.deref() {
            Choice14::_0(ty_array) => {
                let (_, ty, _, len, _) = ty_array.get_matched();
                let ty = WithPath::new(p, ty);
                let ty = Self::from(ty, pcx, fn_sym_tab);
                match len {
                    Choice2::_0(len) => pcx.mk_array_ty(ty, IntValue::from_integer(len).into()),
                    Choice2::_1(len) => {
                        let (const_idx, const_ty, const_pred) =
                            fn_sym_tab.force_get_meta_var(WithPath::new(p, len)).expect_const();
                        let len = ConstVar::from(pcx, fn_sym_tab, const_idx, WithPath::new(p, const_ty), const_pred);
                        pcx.mk_array_ty(ty, len.into())
                    },
                }
            },
            Choice14::_1(ty_group) => {
                let (_, ty) = ty_group.get_matched();
                let ty = WithPath::new(p, ty);
                Self::from(ty, pcx, fn_sym_tab)
            },
            Choice14::_2(_ty_never) => {
                todo!("implement `TyKind::Never`")
            },
            Choice14::_3(ty_paren) => {
                let (_, ty, _) = ty_paren.get_matched();
                let ty = WithPath::new(p, ty);
                Self::from(ty, pcx, fn_sym_tab)
            },
            Choice14::_4(ty_ptr) => {
                let (_, mutability, ty) = ty_ptr.get_matched();
                let ty = WithPath::new(p, ty);
                let ty = Self::from(ty, pcx, fn_sym_tab);
                let mutability = if mutability.kw_mut().is_some() {
                    mir::Mutability::Mut
                } else {
                    mir::Mutability::Not
                };
                pcx.mk_raw_ptr_ty(ty, mutability)
            },
            Choice14::_5(ty_ref) => {
                let (_, region, mutability, ty) = ty_ref.get_matched();
                let ty = WithPath::new(p, ty);
                let ty = Self::from(ty, pcx, fn_sym_tab);
                let region = if let Some(region) = region {
                    RegionKind::from(region)
                } else {
                    RegionKind::ReAny
                };
                let mutability = if mutability.kw_mut().is_some() {
                    mir::Mutability::Mut
                } else {
                    mir::Mutability::Not
                };
                pcx.mk_ref_ty(region, ty, mutability)
            },
            Choice14::_6(ty_slice) => {
                let (_, ty, _) = ty_slice.get_matched();
                let ty = WithPath::new(p, ty);
                let ty = Self::from(ty, pcx, fn_sym_tab);
                pcx.mk_slice_ty(ty)
            },
            Choice14::_7(ty_tuple) => {
                let (_, tys, _) = ty_tuple.get_matched();
                let tys = if let Some(tys) = tys {
                    let tys = collect_elems_separated_by_comma!(tys).collect::<Vec<_>>();
                    tys.iter()
                        .map(|ty| {
                            let ty = WithPath::new(p, *ty);
                            Self::from(ty, pcx, fn_sym_tab)
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                };
                pcx.mk_tuple_ty(&tys)
            },
            Choice14::_8(ty_meta_var) => match fn_sym_tab.force_get_ty_meta_var(WithPath::new(p, ty_meta_var)) {
                MetaVariable::Type(idx, pred) => {
                    let ty_meta_var = TyVar {
                        idx: idx.into(),
                        name: Symbol::intern(ty_meta_var.span.as_str()),
                        pred,
                    };
                    pcx.mk_var_ty(ty_meta_var)
                },
                MetaVariable::Const(..) | MetaVariable::Place(..) => {
                    panic!("A non-type meta variable used as a type variable")
                },
                MetaVariable::AdtPat(_, name) => pcx.mk_adt_pat_ty(Symbol::intern(name)),
            },
            Choice14::_9(_ty_self) => pcx.mk_self_ty(),
            Choice14::_10(primitive_types) => pcx.mk_ty(TyKind::from_primitive_type(primitive_types)),
            Choice14::_11(_place_holder) => pcx.mk_any_ty(),
            Choice14::_12(ty_path) => Self::from_type_path(WithPath::new(p, ty_path), pcx, fn_sym_tab),
            Choice14::_13(lang_item) => {
                let lang_item = PathWithArgs::from_lang_item(WithPath::new(p, lang_item), pcx, fn_sym_tab);
                pcx.mk_path_ty(lang_item)
            },
        }
    }

    /// Creates a `Ty` from a `TypePath`, resolving any type variables or paths.
    ///
    /// # Note
    ///
    /// Also checks [`PathWithArgs::from_type_path`] for resolving type paths.
    /// Maybe reuse `PathWithArgs::from_type_path`?
    #[instrument(level = "trace", skip_all, ret)]
    fn from_type_path<'mcx>(
        ty_path: WithPath<'mcx, &'mcx pairs::TypePath<'mcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &impl GetType<'mcx>,
    ) -> Self {
        let p = ty_path.path;
        let (qself, path) = ty_path.get_matched();
        if qself.is_some() {
            todo!("qself is not supported yet");
        }
        // FIXME: imports logic incomplete?
        if let Some(ident) = utils::Path::from(path).as_ident() {
            // only check for the first time
            if let Ok(TypeOrPath::Type(ty)) = fn_sym_tab.get_type_or_path(&WithPath::new(p, ident)) {
                let ty = WithPath::new(p, ty);
                return Ty::from(ty, pcx, fn_sym_tab);
            }
        }
        let mut path = utils::Path::from(path);
        let mut used = FxHashSet::default();
        while let Some(ident) = path.leading_ident()
            && let Ok(TypeOrPath::Path(mapped)) = fn_sym_tab.get_type_or_path(&WithPath::new(p, ident))
        {
            if !used.insert(mapped) {
                break; // Avoid infinite loop
            }
            let mapped = utils::Path::from(mapped);
            path = path.replace_leading_ident(mapped);
        }
        let args = GenericArgsRef::from_path(WithPath::new(p, &path), pcx, fn_sym_tab);
        let path = Path::from(path, pcx);

        // fn_sym_tab.inner.get_type(mctx, path.ident(), errors);
        let path_with_args = PathWithArgs { path, args };
        pcx.mk_path_ty(path_with_args)
    }

    pub fn from_fn_ret<'mcx>(
        ty: WithPath<'mcx, &'mcx pairs::FnRet<'mcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &FnSymbolTable<'mcx>,
    ) -> Self {
        let p = ty.path;
        let (_, placeholder_or_ty) = ty.get_matched();
        match placeholder_or_ty {
            Choice2::_0(_) => pcx.mk_any_ty(),
            Choice2::_1(ty) => Self::from(WithPath::new(p, ty), pcx, fn_sym_tab),
        }
    }
}

#[derive(Clone, Copy)]
pub enum RegionKind {
    ReAny,
    ReStatic,
}

impl RegionKind {
    pub fn from(region: &pairs::Region<'_>) -> RegionKind {
        match region.get_matched().1 {
            Choice2::_0(_) => RegionKind::ReAny,
            Choice2::_1(_) => RegionKind::ReStatic,
        }
    }
}

// FIXME:
// Can we make `TyVar` Copy?
#[derive(Clone)]
pub enum TyKind<'pcx> {
    TyVar(TyVar),
    AdtPat(Symbol),
    Array(Ty<'pcx>, Const<'pcx>),
    Slice(Ty<'pcx>),
    Tuple(&'pcx [Ty<'pcx>]),
    Ref(RegionKind, Ty<'pcx>, mir::Mutability),
    RawPtr(Ty<'pcx>, mir::Mutability),
    Path(PathWithArgs<'pcx>),
    Def(DefId, GenericArgsRef<'pcx>),
    Uint(ty::UintTy),
    Int(ty::IntTy),
    Float(ty::FloatTy),
    Bool,
    Str,
    Char,
    Self_,
    Any,
}

impl From<PrimTy> for TyKind<'_> {
    fn from(ty: PrimTy) -> Self {
        match ty {
            PrimTy::Int(int_ty) => TyKind::Int(CvtPrimTy::cvt(int_ty)),
            PrimTy::Uint(uint_ty) => TyKind::Uint(CvtPrimTy::cvt(uint_ty)),
            PrimTy::Float(float_ty) => TyKind::Float(CvtPrimTy::cvt(float_ty)),
            PrimTy::Str => TyKind::Str,
            PrimTy::Bool => TyKind::Bool,
            PrimTy::Char => TyKind::Char,
        }
    }
}

impl<'pcx> TyKind<'pcx> {
    //FIXME: this is incomplete
    //FIXME: add a new `TyKind` for resolved types, just like `rustc_middle::ty::TyKind`
    //FIXME: this may breaks uniqueness of `Ty`
    pub fn from_ty_lossy(pcx: PatCtxt<'pcx>, ty: ty::Ty<'_>, args: GenericArgsRef<'pcx>) -> Option<Self> {
        fn require_empty(args: GenericArgsRef<'_>) -> Option<GenericArgsRef<'_>> {
            if args.is_empty() { Some(args) } else { None }
        }
        Some(match ty.kind() {
            ty::TyKind::Bool => Self::Bool,
            ty::TyKind::Char => Self::Char,
            ty::TyKind::Int(int_ty) => Self::Int(*int_ty),
            ty::TyKind::Uint(uint_ty) => Self::Uint(*uint_ty),
            ty::TyKind::Float(float_ty) => Self::Float(*float_ty),
            ty::TyKind::Adt(def, _) => Self::Def(def.did(), args),
            ty::TyKind::Foreign(def_id) => Self::Def(*def_id, args),
            ty::TyKind::Str => Self::Str,
            ty::TyKind::Array(_, _) => None?, //FIXME
            ty::TyKind::Pat(_, _) => None?,   //FIXME
            ty::TyKind::Slice(ty) => Self::Slice(pcx.mk_ty(Self::from_ty_lossy(pcx, *ty, require_empty(args)?)?)),
            ty::TyKind::RawPtr(ty, mutability) => Self::RawPtr(
                pcx.mk_ty(Self::from_ty_lossy(pcx, *ty, require_empty(args)?)?),
                *mutability,
            ),
            ty::TyKind::Ref(_, _, _) => None?,           //FIXME
            ty::TyKind::FnDef(_, _) => None?,            //FIXME
            ty::TyKind::FnPtr(_, _) => None?,            //FIXME
            ty::TyKind::Dynamic(_, _, _) => None?,       //FIXME
            ty::TyKind::Closure(_, _) => None?,          //FIXME
            ty::TyKind::CoroutineClosure(_, _) => None?, //FIXME
            ty::TyKind::Coroutine(_, _) => None?,        //FIXME
            ty::TyKind::CoroutineWitness(_, _) => None?, //FIXME
            ty::TyKind::Never => None?,                  //FIXME
            ty::TyKind::Tuple(_) => None?,               //FIXME
            ty::TyKind::Alias(_, _) => None?,            //FIXME
            ty::TyKind::Param(_) => None?,               //FIXME
            ty::TyKind::Bound(_, _) => None?,            //FIXME
            ty::TyKind::Placeholder(_) => None?,         //FIXME
            ty::TyKind::Infer(_) => None?,               //FIXME
            ty::TyKind::Error(_) => None?,
            ty::TyKind::UnsafeBinder(_) => None?,
        })
    }

    pub fn from_primitive_type(prim_ty: &pairs::PrimitiveType) -> Self {
        match prim_ty.deref() {
            Choice12::_0(_u8) => TyKind::Uint(ty::UintTy::U8),
            Choice12::_1(_u16) => TyKind::Uint(ty::UintTy::U16),
            Choice12::_2(_u32) => TyKind::Uint(ty::UintTy::U32),
            Choice12::_3(_u64) => TyKind::Uint(ty::UintTy::U64),
            Choice12::_4(_usize) => TyKind::Uint(ty::UintTy::Usize),
            Choice12::_5(_i8) => TyKind::Int(ty::IntTy::I8),
            Choice12::_6(_i16) => TyKind::Int(ty::IntTy::I16),
            Choice12::_7(_i32) => TyKind::Int(ty::IntTy::I32),
            Choice12::_8(_i64) => TyKind::Int(ty::IntTy::I64),
            Choice12::_9(_isize) => TyKind::Int(ty::IntTy::Isize),
            Choice12::_10(_bool) => TyKind::Bool,
            Choice12::_11(_str) => TyKind::Str,
        }
    }
}

#[derive(Clone, Copy)]
pub enum GenericArgKind<'pcx> {
    Lifetime(RegionKind),
    Type(Ty<'pcx>),
    Const(Const<'pcx>),
}

impl<'pcx> GenericArgKind<'pcx> {
    fn from<'mcx>(
        arg: WithPath<'mcx, &'mcx pairs::GenericArgument<'mcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &impl GetType<'mcx>,
    ) -> GenericArgKind<'pcx> {
        let p = arg.path;
        match arg.inner.deref() {
            Choice3::_0(region) => RegionKind::from(region).into(),
            Choice3::_1(ty) => GenericArgKind::Type(Ty::from(WithPath::new(p, ty), pcx, fn_sym_tab)),
            Choice3::_2(konst) => GenericArgKind::Const(Const::from_gconst(konst)),
        }
    }
}

impl From<RegionKind> for GenericArgKind<'_> {
    fn from(region: RegionKind) -> Self {
        GenericArgKind::Lifetime(region)
    }
}

impl<'pcx> From<Ty<'pcx>> for GenericArgKind<'pcx> {
    fn from(ty: Ty<'pcx>) -> Self {
        GenericArgKind::Type(ty)
    }
}

impl<'pcx> From<Const<'pcx>> for GenericArgKind<'pcx> {
    fn from(konst: Const<'pcx>) -> Self {
        GenericArgKind::Const(konst)
    }
}

#[derive(Clone, Copy)]
pub struct ItemPath<'pcx>(pub &'pcx [Symbol]);

#[derive(Clone, Copy)]
pub enum Path<'pcx> {
    /// Such as `std::vec::Vec`?
    Item(ItemPath<'pcx>),
    TypeRelative(Ty<'pcx>, Symbol),
    LangItem(LangItem),
}

impl<'pcx> Path<'pcx> {
    pub fn from_pairs(path: &pairs::Path<'_>, pcx: PatCtxt<'pcx>) -> Self {
        let path: rpl_meta::utils::Path<'_> = path.into();
        Self::from(path, pcx)
    }
    pub fn from(path: rpl_meta::utils::Path<'_>, pcx: PatCtxt<'pcx>) -> Self {
        let mut items: Vec<Symbol> = Vec::new();
        if let Some(leading) = path.leading
            && leading.get_matched().0.is_some()
        {
            items.push(Symbol::intern("crate"));
        }
        //FIXME: how about the path arguments?
        items.extend(path.segments.iter().map(|seg| Symbol::intern(seg.0.span.as_str())));
        ItemPath(pcx.mk_slice(&items)).into()
    }
}

impl<'pcx> From<ItemPath<'pcx>> for Path<'pcx> {
    fn from(item: ItemPath<'pcx>) -> Self {
        Path::Item(item)
    }
}

impl<'pcx> From<(Ty<'pcx>, Symbol)> for Path<'pcx> {
    fn from((ty, path): (Ty<'pcx>, Symbol)) -> Self {
        Path::TypeRelative(ty, path)
    }
}

// impl<'pcx> From<(Ty<'pcx>, &str)> for Path<'pcx> {
//     fn from((ty, path): (Ty<'pcx>, &str)) -> Self {
//         (ty, Symbol::intern(path)).into()
//     }
// }

#[derive(Clone, Copy)]
pub struct GenericArgsRef<'pcx>(pub &'pcx [GenericArgKind<'pcx>]);

impl<'pcx> GenericArgsRef<'pcx> {
    pub fn from_path_pairs(
        args: WithPath<'pcx, &'pcx pairs::Path<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx impl GetType<'pcx>,
    ) -> Self {
        let path = args.path;
        let args = rpl_meta::utils::Path::from(args.inner);
        let args = WithPath::new(path, &args);
        Self::from_path(args, pcx, fn_sym_tab)
    }
    // FIXME: this function accumulates generic arguments from all path segments
    pub fn from_path<'mcx>(
        args: WithPath<'mcx, &rpl_meta::utils::Path<'mcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &impl GetType<'mcx>,
    ) -> Self {
        let p = args.path;
        let path: &rpl_meta::utils::Path<'_> = args.inner;
        let mut items: Vec<GenericArgKind<'_>> = Vec::new();
        path.segments.iter().for_each(|seg| {
            Self::from_generic_arguments(p, seg.1.iter().copied(), pcx, fn_sym_tab)
                .iter()
                .for_each(|arg| {
                    items.push(*arg);
                });
        });
        GenericArgsRef(pcx.mk_slice(&items))
    }

    pub fn from_generic_arguments<'mcx>(
        p: &'mcx std::path::Path,
        args: impl Iterator<Item = &'mcx pairs::GenericArgument<'mcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &impl GetType<'mcx>,
    ) -> Self {
        let args = args
            .into_iter()
            .map(|arg| GenericArgKind::from(WithPath::new(p, arg), pcx, fn_sym_tab))
            .collect::<Vec<_>>();
        GenericArgsRef(pcx.mk_slice(&args))
    }
}

impl<'pcx> std::ops::Deref for GenericArgsRef<'pcx> {
    type Target = [GenericArgKind<'pcx>];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl From<LangItem> for Path<'_> {
    fn from(lang_item: LangItem) -> Self {
        Path::LangItem(lang_item)
    }
}

#[derive(Clone, Copy)]
pub struct PathWithArgs<'pcx> {
    pub path: Path<'pcx>,
    pub args: GenericArgsRef<'pcx>,
}

impl<'pcx> PathWithArgs<'pcx> {
    /// Creates a `PathWithArgs` from a `Path`, resolving any type variables or paths.
    ///
    /// # Note
    ///
    /// Also checks [`Ty::from_type_path`] for resolving type paths.
    #[instrument(level = "trace", skip_all, ret)]
    pub fn from_path(
        path: WithPath<'pcx, &'pcx pairs::Path<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = path.path;
        let mut path = utils::Path::from(path.inner);
        let mut used = FxHashSet::default();
        while let Some(ident) = path.leading_ident()
            && let Ok(type_or_path) = fn_sym_tab.get_type_or_path(&WithPath::new(p, ident))
            && let Some(mapped) = type_or_path.try_as_path()
        {
            if !used.insert(mapped) {
                // Avoid infinite loop
                // FIXME: can Vec<Vec<T>> be resolved?
                break;
            }
            let mapped = utils::Path::from(mapped);
            path = path.replace_leading_ident(mapped);
        }
        let args = GenericArgsRef::from_path(WithPath::new(p, &path), pcx, fn_sym_tab);
        let path = Path::from(path, pcx);
        Self { path, args }
    }

    /// Creates a `PathWithArgs` from a `TypePath`, resolving any type variables or paths.
    ///
    /// # Note
    ///
    /// Also checks [`Ty::from_type_path`] for resolving type paths.
    #[instrument(level = "trace", skip_all, ret)]
    pub fn from_type_path(
        path: WithPath<'pcx, &'pcx pairs::TypePath<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = path.path;
        let (qself, path) = path.get_matched();
        if qself.is_some() {
            todo!("qself is not supported yet");
        }
        Self::from_path(WithPath::new(p, path), pcx, fn_sym_tab)
    }

    pub fn from_lang_item<'mcx>(
        lang_item: WithPath<'mcx, &'mcx pairs::LangItemWithArgs<'mcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &impl GetType<'mcx>,
    ) -> Self {
        let p = lang_item.path;
        let (_, _, _, _, lang_item, _, args) = lang_item.get_matched();
        let lang_item = LangItem::from_name(rustc_span::Symbol::intern(lang_item.span.as_str().trim_matches('"')))
            .unwrap_or_else(|| panic!("Unknown lang item {:?}", lang_item.span.as_str()));
        let args = if let Some(args) = args {
            GenericArgsRef::from_generic_arguments(
                p,
                collect_elems_separated_by_comma!(args.GenericArgumentsSepretatedByComma()),
                pcx,
                fn_sym_tab,
            )
        } else {
            GenericArgsRef(&[])
        };
        let path = Path::LangItem(lang_item);
        Self { path, args }
    }

    pub fn from_path_or_lang_item(
        path_or_lang_item: WithPath<'pcx, &'pcx pairs::PathOrLangItem<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = path_or_lang_item.path;
        match path_or_lang_item.inner.deref() {
            Choice2::_0(path) => Self::from_path(WithPath::new(p, path), pcx, fn_sym_tab),
            Choice2::_1(lang_item) => Self::from_lang_item(WithPath::new(p, lang_item), pcx, fn_sym_tab),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum IntTy {
    NegInt(ty::IntTy),
    Int(ty::IntTy),
    Uint(ty::UintTy),
    Bool,
}

impl IntTy {
    fn from(suffix: &pairs::IntegerSuffix<'_>) -> Self {
        match suffix.deref() {
            Choice10::_0(_u8) => IntTy::Uint(ty::UintTy::U8),
            Choice10::_1(_u16) => IntTy::Uint(ty::UintTy::U16),
            Choice10::_2(_u32) => IntTy::Uint(ty::UintTy::U32),
            Choice10::_3(_u64) => IntTy::Uint(ty::UintTy::U64),
            Choice10::_4(_usize) => IntTy::Uint(ty::UintTy::Usize),
            Choice10::_5(_i8) => IntTy::Int(ty::IntTy::I8),
            Choice10::_6(_i16) => IntTy::Int(ty::IntTy::I16),
            Choice10::_7(_i32) => IntTy::Int(ty::IntTy::I32),
            Choice10::_8(_i64) => IntTy::Int(ty::IntTy::I64),
            Choice10::_9(_isize) => IntTy::Int(ty::IntTy::Isize),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct IntValue {
    pub value: Pu128,
    pub ty: IntTy,
}

impl IntValue {
    #[allow(clippy::from_str_radix_10)]
    pub fn from_integer(int: &pairs::Integer<'_>) -> Self {
        let (lit, ty) = int.get_matched();
        let value = match lit {
            Choice4::_0(dec) => {
                let dec_str = dec.span.as_str();
                let dec_str = &dec_str.replace('_', "");
                u128::from_str_radix(dec_str, 10)
                    .unwrap_or_else(|err| panic!("invalid decimal integer {:?}: {}", dec_str, err))
                    .into()
            },
            Choice4::_1(bin) => {
                let bin_str = bin.span.as_str();
                let bin_str = &bin_str.replace('_', "");
                u128::from_str_radix(bin_str, 2)
                    .unwrap_or_else(|err| panic!("invalid binary integer {:?}: {}", bin_str, err))
                    .into()
            },
            Choice4::_2(oct) => {
                let oct_str = oct.span.as_str();
                let oct_str = &oct_str.replace('_', "");
                u128::from_str_radix(oct_str, 8)
                    .unwrap_or_else(|err| panic!("invalid octal integer {:?}: {}", oct_str, err))
                    .into()
            },
            Choice4::_3(hex) => {
                let hex_str = hex.span.as_str();
                let hex_str = &hex_str.replace('_', "");
                u128::from_str_radix(hex_str, 16)
                    .unwrap_or_else(|err| panic!("invalid hexadecimal integer {:?}: {}", hex_str, err))
                    .into()
            },
        };
        let ty = if let Some(ty) = ty {
            IntTy::from(ty)
        } else {
            IntTy::Uint(ty::UintTy::Usize)
        };
        Self { value, ty }
    }

    pub fn from_switch_int_value(value: &pairs::MirSwitchValue<'_>) -> Option<Self> {
        match value.deref() {
            Choice3::_0(bool) => Some(Self::from_bool(bool)),
            Choice3::_1(integer) => Some(Self::from_integer(integer)),
            Choice3::_2(_) => None,
        }
    }

    pub fn from_bool(value: &pairs::Bool<'_>) -> Self {
        let value = if value.kw_true().is_some() { Pu128(1) } else { Pu128(0) };
        Self { value, ty: IntTy::Bool }
    }
}

impl IntValue {
    pub fn normalize(self, pointer_bytes: u64) -> Pu128 {
        use IntTy::{Bool, Int, NegInt, Uint};
        use ty::IntTy::{I8, I16, I32, I64, I128, Isize};

        let IntValue { ty, value } = self;
        let mask: u128 = match ty {
            NegInt(I8) => u8::MAX.into(),
            NegInt(I16) => u16::MAX.into(),
            NegInt(I32) => u32::MAX.into(),
            NegInt(I64) => u64::MAX.into(),
            NegInt(I128) => u128::MAX,
            NegInt(Isize) => match pointer_bytes {
                2 => u128::from(u16::MAX),
                4 => u128::from(u32::MAX),
                8 => u128::from(u64::MAX),
                _ => panic!("unsupported pointer size: {pointer_bytes}"),
            },
            Int(_) | Uint(_) | Bool => return value,
        };
        Pu128((value.get() ^ mask).wrapping_add(1) & mask)
    }
}

macro_rules! impl_uint {
    ($($ty:ident => $variant:ident),* $(,)?) => {$(
        impl From<$ty> for IntValue {
            fn from(value: $ty) -> Self {
                Self {
                    value: Pu128(value as u128),
                    ty: IntTy::Uint(ty::UintTy::$variant),
                }
            }
        }
    )* };
}

macro_rules! impl_int {
    ($($ty:ident => $variant:ident),* $(,)?) => {$(
        impl From<$ty> for IntValue {
            fn from(value: $ty) -> Self {
                let ty = if value < 0 { IntTy::NegInt } else { IntTy::Int };
                Self {
                    value: Pu128(value.unsigned_abs() as u128),
                    ty: ty(ty::IntTy::$variant),
                }
            }
        }
    )* };
}

impl_uint!(u8 => U8, u16 => U16, u32 => U32, u64 => U64, u128 => U128, usize => Usize);
impl_int!(i8 => I8, i16 => I16, i32 => I32, i64 => I64, i128 => I128, isize => Isize);

impl From<bool> for IntValue {
    fn from(value: bool) -> Self {
        Self {
            value: Pu128(value.into()),
            ty: IntTy::Bool,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Const<'pcx> {
    ConstVar(ConstVar<'pcx>),
    Value(IntValue),
}

impl Const<'_> {
    pub fn from(konst: &pairs::Konst<'_>) -> Self {
        match konst.deref() {
            Choice2::_0(lit) => match lit.deref() {
                Choice3::_0(int) => Self::Value(IntValue::from_integer(int)),
                _ => todo!("unsupported literal in Const: {:?}", lit),
            },
            Choice2::_1(_ty_path) => todo!(),
        }
    }

    pub fn from_integer(int: &pairs::Integer<'_>) -> Self {
        Self::Value(IntValue::from_integer(int))
    }

    pub fn from_gconst(konst: &pairs::GenericConst<'_>) -> Self {
        let konst = match konst.deref() {
            Choice2::_0(konst_with_brace) => konst_with_brace.get_matched().1,
            Choice2::_1(konst) => konst,
        };
        Self::from(konst)
    }
}

impl From<IntValue> for Const<'_> {
    fn from(value: IntValue) -> Self {
        Self::Value(value)
    }
}

impl<'pcx> From<ConstVar<'pcx>> for Const<'pcx> {
    fn from(value: ConstVar<'pcx>) -> Self {
        Self::ConstVar(value)
    }
}
