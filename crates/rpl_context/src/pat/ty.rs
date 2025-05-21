use std::fmt::Debug;

use rustc_data_structures::packed::Pu128;
use rustc_hir::def_id::DefId;
use rustc_hir::{LangItem, PrimTy};
use rustc_middle::mir;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Symbol;

use either::Either;

use crate::PatCtxt;
use crate::cvt_prim_ty::CvtPrimTy;

use super::{Place, PlaceBase};

rustc_index::newtype_index! {
    #[debug_format = "?T{}"]
    pub struct TyVarIdx {}
}

rustc_index::newtype_index! {
    #[debug_format = "?C{}"]
    pub struct ConstVarIdx {}
}

rustc_index::newtype_index! {
    #[debug_format = "?P{}"]
    pub struct PlaceVarIdx {}
}

// FIXME: Use interning for the types
#[derive(Clone, Copy)]
#[rustc_pass_by_value]
pub struct Ty<'pcx>(pub(crate) &'pcx TyKind<'pcx>);

impl<'pcx> Ty<'pcx> {
    pub fn kind(self) -> &'pcx TyKind<'pcx> {
        self.0
    }
    //FIXME: this may breaks uniqueness of `Ty`
    pub fn from_ty_lossy(pcx: PatCtxt<'pcx>, ty: ty::Ty<'_>, args: GenericArgsRef<'pcx>) -> Option<Self> {
        Some(pcx.mk_ty(TyKind::from_ty_lossy(pcx, ty, args)?))
    }
    pub fn from_prim_ty(pcx: PatCtxt<'pcx>, ty: PrimTy) -> Self {
        pcx.mk_ty(TyKind::from(ty))
    }
    pub fn from_def(pcx: PatCtxt<'pcx>, def_id: DefId, args: GenericArgsRef<'pcx>) -> Self {
        pcx.mk_ty(TyKind::Def(def_id, args))
    }
}

#[derive(Clone, Copy)]
pub enum RegionKind {
    ReAny,
    ReStatic,
}

#[derive(Clone, Copy)]
pub enum TyKind<'pcx> {
    TyVar(TyVar<'pcx>),
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
}

#[derive(Clone, Copy)]
pub enum GenericArgKind<'pcx> {
    Lifetime(RegionKind),
    Type(Ty<'pcx>),
    Const(Const<'pcx>),
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

impl<'pcx> From<(Ty<'pcx>, &str)> for Path<'pcx> {
    fn from((ty, path): (Ty<'pcx>, &str)) -> Self {
        (ty, Symbol::intern(path)).into()
    }
}

#[derive(Clone, Copy)]
pub struct GenericArgsRef<'pcx>(pub &'pcx [GenericArgKind<'pcx>]);

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
    //FIXME: handle type alias correctly.
    pub path: Path<'pcx>,
    pub args: GenericArgsRef<'pcx>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum IntTy {
    NegInt(ty::IntTy),
    Int(ty::IntTy),
    Uint(ty::UintTy),
    Bool,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct IntValue {
    pub value: Pu128,
    pub ty: IntTy,
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

pub type TyPred = for<'tcx> fn(TyCtxt<'tcx>, ty::TypingEnv<'tcx>, ty::Ty<'tcx>) -> bool;

#[derive(Debug, Clone, Copy)]
pub enum Const<'pcx> {
    ConstVar(ConstVar<'pcx>),
    Value(IntValue),
}

impl<'pcx> From<ConstVar<'pcx>> for Const<'pcx> {
    fn from(konst: ConstVar<'pcx>) -> Self {
        Const::ConstVar(konst)
    }
}

#[derive(Clone, Copy)]
pub struct TyVar<'pcx> {
    pub idx: TyVarIdx,
    pub pred: &'pcx [&'pcx [Either<TyPred, TyPred>]],
}

#[derive(Clone, Copy)]
pub struct PlaceVar<'pcx> {
    pub idx: PlaceVarIdx,
    pub ty: Ty<'pcx>,
}

impl PlaceVarIdx {
    pub fn into_place<'pcx>(self) -> Place<'pcx, PlaceBase> {
        //FIXME: update `Place` to hold a `PlaceVar`
        Place::from(self)
    }
}

impl Debug for PlaceVar<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.idx, f)
    }
}

#[derive(Clone, Copy)]
pub struct ConstVar<'pcx> {
    pub idx: ConstVarIdx,
    pub ty: Ty<'pcx>,
}

impl<'pcx> ConstVar<'pcx> {
    pub fn into_const(self) -> Const<'pcx> {
        Const::ConstVar(self)
    }
}
