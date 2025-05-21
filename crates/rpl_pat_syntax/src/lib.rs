#![feature(box_patterns)]
#![feature(let_chains)]

use derive_more::{Deref, Display, From, IntoIterator};
use proc_macro2::Span;
use std::borrow::Borrow;
use std::marker::PhantomData;
use syn::punctuated::Punctuated;
use syn::{Ident, Token, token};
use syn_derive::{Parse, ToTokens};

#[macro_use]
mod to_tokens;
#[macro_use]
mod parse;

#[cfg(test)]
mod tests;

pub use parse::ParseError;

pub(crate) mod kw {
    // Metadata
    syn::custom_keyword!(meta);
    syn::custom_keyword!(ty);
    syn::custom_keyword!(place);
    syn::custom_keyword!(lang);
    syn::custom_keyword!(ctor);
    syn::custom_keyword!(mir);

    // export
    syn::custom_keyword!(export);
    syn::custom_keyword!(Statement);

    // Statement
    syn::custom_keyword!(drop);
    syn::custom_keyword!(switchInt);

    // Operand
    syn::custom_keyword!(copy);

    // place
    syn::custom_keyword!(of);
    syn::custom_keyword!(RET);

    // Rvalue
    syn::custom_keyword!(Len);
    syn::custom_keyword!(discriminant);
    syn::custom_keyword!(raw);

    // CastKind
    syn::custom_keyword!(PtrToPtr);
    syn::custom_keyword!(IntToInt);
    syn::custom_keyword!(Transmute);
    syn::custom_keyword!(PointerExposeProvenance);
    syn::custom_keyword!(PointerCoercion);

    syn::custom_keyword!(MutToConstPointer);
    syn::custom_keyword!(ArrayToPointer);
    syn::custom_keyword!(Unsize);

    syn::custom_keyword!(AsCast);
    syn::custom_keyword!(Implicit);

    // BinOp
    syn::custom_keyword!(Add);
    syn::custom_keyword!(Sub);
    syn::custom_keyword!(Mul);
    syn::custom_keyword!(Div);
    syn::custom_keyword!(Rem);
    syn::custom_keyword!(BitAnd);
    syn::custom_keyword!(BitOr);
    syn::custom_keyword!(Lt);
    syn::custom_keyword!(Gt);
    syn::custom_keyword!(Le);
    syn::custom_keyword!(Ge);
    syn::custom_keyword!(Eq);
    syn::custom_keyword!(Ne);
    syn::custom_keyword!(Offset);

    // NullOp
    syn::custom_keyword!(SizeOf);
    syn::custom_keyword!(AlignOf);
    syn::custom_keyword!(OffsetOf);

    // UnOp
    syn::custom_keyword!(Neg);
    syn::custom_keyword!(Not);
    syn::custom_keyword!(PtrMetadata);

    // Aggregate
    syn::custom_keyword!(from);
}

#[derive(Clone, Copy)]
pub struct Region {
    span: Span,
    pub kind: RegionKind,
}

#[derive(Clone, Copy, ToTokens, From)]
pub enum RegionKind {
    ReAny(Token!(_)),
    ReStatic(Token![static]),
}

#[derive(Default, Clone, Copy, ToTokens, Parse, From)]
pub enum Mutability {
    #[parse(peek = Token![mut])]
    Mut(Token![mut]),
    #[default]
    Not,
}

#[derive(Clone, Copy, ToTokens, Parse, From)]
pub enum PtrMutability {
    #[parse(peek = Token![const])]
    Const(Token![const]),
    #[parse(peek = Token![mut])]
    Mut(Token![mut]),
}

#[derive(ToTokens, Parse)]
pub struct TypeDecl {
    tk_type: Token![type],
    pub ident: Ident,
    #[parse(
        |input| if input.peek(Token![<]) {
            Err(input.error(ParseError::TypeWithGenericsNotSupported))
        } else {
            input.parse()
        }
    )]
    tk_eq: Token![=],
    pub ty: Type,
    tk_semi: Token![;],
}

#[derive(Parse, ToTokens)]
pub struct TypeArray {
    #[syn(bracketed)]
    bracket: token::Bracket,
    #[syn(in = bracket)]
    pub ty: Box<Type>,
    #[syn(in = bracket)]
    tk_semi: Token![;],
    #[syn(in = bracket)]
    pub len: Const,
}

/*
pub struct FnPtrArg {
    pub name: Option<(Ident, Token![:])>,
    pub ty: Type,
}
pub struct FnPtrVariadic {
    pub name: Option<(Ident, Token![:])>,
    tk_dots: Token![...],
    tk_comma: Option<Token![,]>,
}

pub struct TypeFnPtr {
    pub lifetimes: Option<syn::BoundLifetimes>,
    pub unsafety: Option<Token![unsafe]>,
    pub abi: Option<syn::Abi>,
    tk_fn: Token![fn],
    paren: token::Paren,
    pub inputs: Punctuated<FnPtrArg, Token![,]>,
    pub variadic: Option<syn::BareVariadic>,
    pub output: syn::ReturnType,
}
*/

pub struct TypeGroup {
    tk_group: token::Group,
    pub ty: Box<Type>,
}

#[derive(Clone, Copy, ToTokens, Parse, From)]
pub struct TypeNever {
    tk_bang: Token![!],
}

pub type TypeParen = Parenthesized<Box<Type>>;

pub struct GenericConst {
    brace: Option<token::Brace>,
    pub konst: Const,
}

#[derive(ToTokens, Parse, From)]
pub enum GenericArgument {
    /// A region argument.
    #[parse(peek = syn::Lifetime)]
    Region(Region),
    /// A type argument.
    #[parse(peek_func = |input| input.parse::<Type>().is_ok())]
    Type(Type),
    /// A const argument.
    Const(GenericConst),
}

#[derive(ToTokens, Parse)]
pub struct AngleBracketedGenericArguments {
    tk_colon2: Option<Token![::]>,
    tk_lt: Token![<],
    #[parse(parse::parse_angle_bracketed)]
    pub args: Punctuated<GenericArgument, Token![,]>,
    tk_gt: Token![>],
}

#[derive(ToTokens, Parse)]
pub enum ReturnType {
    #[parse(peek = Token![->])]
    Type(Token![->], Box<Type>),
    Default,
}

#[derive(Parse, ToTokens)]
pub struct ParenthesizedGenericArguments {
    #[syn(parenthesized)]
    paren: token::Paren,
    /// `(A, B)`
    #[syn(in = paren)]
    #[parse(Punctuated::parse_terminated)]
    pub inputs: Punctuated<Type, Token![,]>,
    /// `C`
    pub output: syn::ReturnType,
}

#[derive(ToTokens, Parse, From)]
pub enum PathArguments {
    /// The `<'a, T>` in `std::slice::iter<'a, T>`.
    #[parse(peek_func = AngleBracketedGenericArguments::peek)]
    AngleBracketed(AngleBracketedGenericArguments),
    // /// The `(A, B) -> C` in `Fn(A, B) -> C`.
    // #[parse(peek = token::Paren)]
    // Parenthesized(ParenthesizedGenericArguments),
    None,
}

#[derive(Parse, ToTokens)]
pub struct PathSegment {
    pub ident: Ident,
    pub arguments: PathArguments,
}

#[derive(Clone, Copy, Parse, ToTokens)]
pub struct PathCrate {
    tk_dollar: Token![$],
    pub tk_crate: Token![crate],
    colon: Token![::],
}

#[derive(Clone, Copy, ToTokens, Parse)]
pub enum PathLeading {
    #[parse(peek = Token![::])]
    Colon(Token![::]),
    #[parse(peek = Token![$])]
    Crate(PathCrate),
    None,
}

#[derive(ToTokens, Parse)]
pub struct Path {
    pub leading: PathLeading,
    #[parse(Punctuated::parse_separated_nonempty)]
    pub segments: Punctuated<PathSegment, Token![::]>,
}

impl Path {
    pub fn as_ident(&self) -> Option<&Ident> {
        let PathLeading::None = self.leading else { return None };
        if self.segments.len() != 1 || self.segments.trailing_punct() {
            return None;
        }
        self.ident()
    }
    pub fn ident(&self) -> Option<&Ident> {
        Some(&self.segments.last()?.ident)
    }
}

pub struct QSelf {
    tk_lt: Token![<],
    pub ty: Box<Type>,
    pub position: usize,
    tk_as: Option<Token![as]>,
    tk_gt: Token![>],
}

pub struct TypePath {
    pub qself: Option<QSelf>,
    pub path: Path,
}

#[derive(ToTokens, Parse)]
pub struct TypePtr {
    tk_star: Token![*],
    pub mutability: PtrMutability,
    pub ty: Box<Type>,
}

#[derive(ToTokens, Parse)]
pub struct TypeReference {
    tk_and: Token![&],
    #[parse(Region::parse_opt)]
    pub region: Option<Region>,
    pub mutability: Mutability,
    pub ty: Box<Type>,
}

#[derive(ToTokens, Parse)]
pub struct TypeSlice {
    #[syn(bracketed)]
    bracket: token::Bracket,
    #[syn(in = bracket)]
    pub ty: Box<Type>,
}

#[derive(ToTokens)]
pub struct TypeTuple {
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub tys: Punctuated<Type, Token![,]>,
}

#[derive(ToTokens, Parse)]
pub struct TypeVar {
    tk_dollar: Token![$],
    pub ident: Ident,
}

#[derive(ToTokens, Parse)]
pub struct ConstVar {
    tk_dollar: Token![$],
    pub ident: Ident,
}

#[derive(ToTokens, From)]
pub enum Type {
    /// A fixed size array type: `[T; n]`.
    Array(TypeArray),

    // /// A function pointer type: `fn(usize) -> bool`.
    // FnPtr(syn::TypeBareFn),
    /// A type contained within invisible delimiters.
    Group(TypeGroup),

    /// The never type: `!`.
    Never(TypeNever),

    /// A parenthesized type equivalent to the inner type.
    Paren(TypeParen),

    /// A path like `std::slice::Iter`, optionally qualified with a
    /// self-type as in `<Vec<T> as SomeTrait>::Associated`.
    Path(TypePath),

    /// A raw pointer type: `*const T` or `*mut T`.
    Ptr(TypePtr),

    /// A reference type: `&'a T` or `&'a mut T`.
    Reference(TypeReference),

    /// A dynamically sized slice type: `[T]`.
    Slice(TypeSlice),

    // /// A trait object type `dyn Bound1 + Bound2 + Bound3` where `Bound` is a
    // /// trait or a lifetime.
    // TraitObject(TypeTraitObject),
    /// A tuple type: `(A, B, C, String)`.
    Tuple(TypeTuple),

    /// A `TyVar` from `#[meta($T:ty)]` or other pattern.
    TyVar(TypeVar),

    /// A language item
    LangItem(LangItemWithArgs),

    /// The `Self` type
    SelfType(Token![Self]),

    /// any type
    Any(Token![_]),
}

#[derive(ToTokens, Parse, From, Display)]
#[display("${kind}")]
pub struct PlaceLocal {
    tk_dollar: Token![$],
    pub kind: PlaceLocalKind,
}

#[derive(ToTokens, Parse, From, Display)]
pub enum PlaceLocalKind {
    #[parse(peek = kw::RET)]
    #[display("RET")]
    Return(kw::RET),
    #[parse(peek = Ident)]
    #[display("{_0}")]
    Local(Ident),
    #[parse(peek = Token![self])]
    #[display("self")]
    SelfValue(Token![self]),
}

impl PlaceLocalKind {
    pub fn span(&self) -> Span {
        match self {
            PlaceLocalKind::Return(ret) => ret.span,
            PlaceLocalKind::Local(ident) => ident.span(),
            PlaceLocalKind::SelfValue(self_value) => self_value.span,
        }
    }
}

#[derive(ToTokens)]
pub struct PlaceParen {
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub place: Box<Place>,
}

#[derive(ToTokens, Parse)]
pub struct PlaceDeref {
    tk_star: Token![*],
    pub place: Box<Place>,
}

#[derive(ToTokens)]
pub struct PlaceField {
    pub place: Box<Place>,
    tk_dot: Token![.],
    /// Some if it is a pattern field
    tk_dollar: Option<Token![$]>,
    pub field: syn::Member,
}

impl PlaceField {
    pub fn is_pattern(&self) -> bool {
        self.tk_dollar.is_some()
    }
}

#[derive(ToTokens)]
pub struct PlaceIndex {
    pub place: Box<Place>,
    #[syn(bracketed)]
    bracket: token::Bracket,
    #[syn(in = bracket)]
    pub index: PlaceLocal,
}

#[derive(ToTokens)]
pub struct PlaceConstIndex {
    pub place: Box<Place>,
    #[syn(bracketed)]
    bracket: token::Bracket,
    #[syn(in = bracket)]
    pub from_end: Option<Token![-]>,
    #[syn(in = bracket)]
    pub index: syn::Index,
    #[syn(in = bracket)]
    kw_of: kw::of,
    #[syn(in = bracket)]
    pub min_length: syn::Index,
}

#[derive(ToTokens)]
pub struct PlaceSubslice {
    pub place: Box<Place>,
    #[syn(bracketed)]
    bracket: token::Bracket,
    #[syn(in = bracket)]
    pub from: Option<syn::Index>,
    #[syn(in = bracket)]
    tk_colon: Token![:],
    #[syn(in = bracket)]
    pub from_end: Option<Token![-]>,
    #[syn(in = bracket)]
    pub to: syn::Index,
}

#[derive(ToTokens)]
pub struct PlaceDowncast {
    pub place: Box<Place>,
    tk_as: Token![as],
    /// `Some` if it is a pattern downcast
    tk_dollar: Option<Token![$]>,
    pub variant: Ident,
}

impl PlaceDowncast {
    pub fn is_pattern(&self) -> bool {
        self.tk_dollar.is_some()
    }
}

#[derive(ToTokens, From)]
pub enum Place {
    /// `$local`
    Local(PlaceLocal),
    /// `(place)`
    Paren(PlaceParen),
    /// `*place`
    Deref(PlaceDeref),
    /// `place.field` or `place.$field`
    Field(PlaceField),
    /// `place[index]`
    Index(PlaceIndex),
    /// `place[const_index]`
    ConstIndex(PlaceConstIndex),
    /// `place[from..to]`
    Subslice(PlaceSubslice),
    /// `place as Variant` or `place as $Variant`
    DownCast(PlaceDowncast),
}

impl Borrow<PlaceLocal> for Place {
    fn borrow(&self) -> &PlaceLocal {
        self.local()
    }
}

impl Place {
    pub fn local(&self) -> &PlaceLocal {
        match self {
            Place::Local(local) => local,
            Place::Paren(PlaceParen { box place, .. })
            | Place::Deref(PlaceDeref { box place, .. })
            | Place::Field(PlaceField { box place, .. })
            | Place::Index(PlaceIndex { box place, .. })
            | Place::ConstIndex(PlaceConstIndex { box place, .. })
            | Place::Subslice(PlaceSubslice { box place, .. })
            | Place::DownCast(PlaceDowncast { box place, .. }) => place.local(),
        }
    }
    pub fn into_local(self) -> PlaceLocal {
        match self {
            Place::Local(local) => local,
            Place::Paren(PlaceParen { box place, .. })
            | Place::Deref(PlaceDeref { box place, .. })
            | Place::Field(PlaceField { box place, .. })
            | Place::Index(PlaceIndex { box place, .. })
            | Place::ConstIndex(PlaceConstIndex { box place, .. })
            | Place::Subslice(PlaceSubslice { box place, .. })
            | Place::DownCast(PlaceDowncast { box place, .. }) => place.into_local(),
        }
    }
}

#[derive(ToTokens, Parse)]
pub enum Const {
    #[parse(peek = syn::Lit)]
    Lit(syn::Lit),
    #[parse(peek = Token![$])]
    ConstVar(ConstVar),
    Path(TypePath),
}

#[derive(ToTokens, Parse)]
pub struct LangItemWithArgs {
    tk_pound: Token![#],
    #[syn(bracketed)]
    bracket: token::Bracket,
    #[syn(in = bracket)]
    kw_lang: kw::lang,
    #[syn(in = bracket)]
    tk_eq: Token![=],
    #[syn(in = bracket)]
    pub item: syn::LitStr,
    #[parse(AngleBracketedGenericArguments::parse_opt)]
    pub args: Option<AngleBracketedGenericArguments>,
}

#[derive(Parse, ToTokens)]
pub enum ConstOperandKind {
    #[parse(peek = syn::Lit)]
    Lit(syn::Lit),
    #[parse(peek = Token![#])]
    LangItem(LangItemWithArgs),
    #[parse(peek = Token![$])]
    ConstVar(ConstVar),
    Type(TypePath),
}

#[derive(ToTokens, Parse)]
pub struct ConstOperand {
    tk_const: Token![const],
    pub kind: ConstOperandKind,
}

#[derive(ToTokens, Parse)]
pub struct OperandCopy {
    kw_copy: kw::copy,
    pub place: Place,
}

#[derive(ToTokens, Parse)]
pub struct OperandMove {
    tk_move: Token![move],
    pub place: Place,
}

#[derive(Parse, ToTokens, From)]
pub enum Operand {
    #[parse(peek = Token![_])]
    Any(Token![_]),
    #[parse(peek = Token![..])]
    AnyMultiple(Token![..]),
    #[parse(peek = Token![move])]
    Move(OperandMove),
    #[parse(peek = kw::copy)]
    Copy(OperandCopy),
    #[parse(peek = Token![const])]
    Constant(ConstOperand),
}

#[derive(ToTokens, From)]
pub enum FnOperand {
    Copy(Parenthesized<OperandCopy>),
    Move(Parenthesized<OperandMove>),
    Type(TypePath),
    LangItem(LangItemWithArgs),
    FnPat(Token![$], Ident),
}

#[derive(Parse, ToTokens)]
pub struct Parenthesized<T: quote::ToTokens, P: parse::ParseFn<T> = parse::ParseParse> {
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    #[parse(P::parse)]
    pub value: T,
    #[parse(|_| Ok(PhantomData))]
    #[to_tokens(|_, _| {})]
    _parse: PhantomData<P>,
}

pub struct RvalueUse {
    paren: Option<token::Paren>,
    pub operand: Operand,
}

impl From<Parenthesized<OperandMove>> for RvalueUse {
    fn from(operand: Parenthesized<OperandMove>) -> Self {
        Self {
            paren: Some(operand.paren),
            operand: Operand::Move(operand.value),
        }
    }
}

impl From<Parenthesized<OperandCopy>> for RvalueUse {
    fn from(operand: Parenthesized<OperandCopy>) -> Self {
        Self {
            paren: Some(operand.paren),
            operand: Operand::Copy(operand.value),
        }
    }
}

#[derive(ToTokens, Parse)]
pub struct RvalueRepeat {
    #[syn(bracketed)]
    bracket: token::Bracket,
    #[syn(in = bracket)]
    pub operand: Operand,
    #[syn(in = bracket)]
    tk_semi: Token![;],
    #[syn(in = bracket)]
    pub len: syn::LitInt,
}

#[derive(ToTokens, Parse)]
pub struct RvalueRef {
    tk_and: Token![&],
    #[parse(Region::parse_opt)]
    pub region: Option<Region>,
    pub mutability: Mutability,
    pub place: Place,
}

#[derive(ToTokens, Parse)]
pub struct RvalueRawPtr {
    tk_and: Token![&],
    kw_raw: kw::raw,
    pub mutability: PtrMutability,
    pub place: Place,
}

#[derive(ToTokens, Parse)]
pub struct RvalueLen {
    kw_len: kw::Len,
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub place: Place,
}

// /// See [`rustc_hir::hir::Safety`]
// #[derive(ToTokens, Parse)]
// pub enum Safety {
//     Unsafe,
//     Safe,
// }

/// See [`rustc_middle::ty::adjustment::PointerCoercion`]
#[derive(Clone, Copy, ToTokens, Parse)]
pub enum PointerCoercion {
    // ReifyFnPointer,
    // UnsafeFnPointer,
    // ClosureFnPointer(Safety),
    #[parse(peek = kw::MutToConstPointer)]
    MutToConstPointer(kw::MutToConstPointer),
    #[parse(peek = kw::ArrayToPointer)]
    ArrayToPointer(kw::ArrayToPointer),
    #[parse(peek = kw::Unsize)]
    Unsize(kw::Unsize),
    // DynStar,
}

/// See [`rustc_middle::mir::CoercionSource`]
#[derive(Clone, Copy, ToTokens, Parse)]
pub enum CoercionSource {
    #[parse(peek = kw::AsCast)]
    AsCast(kw::AsCast),
    #[parse(peek = kw::Implicit)]
    Implicit(kw::Implicit),
}

#[derive(Clone, Copy, ToTokens, Parse, From)]
pub enum CastKind {
    #[parse(peek = kw::PtrToPtr)]
    PtrToPtr(kw::PtrToPtr),
    #[parse(peek = kw::IntToInt)]
    IntToInt(kw::IntToInt),
    #[parse(peek = kw::Transmute)]
    Transmute(kw::Transmute),
    #[parse(peek = kw::PointerExposeProvenance)]
    PointerExposeProvenance(kw::PointerExposeProvenance),
    /// See [`rustc_middle::mir::CastKind::PointerCoercion`]
    /// See <https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.CastKind.html#variant.PointerCoercion>
    #[parse(peek = kw::PointerCoercion)]
    PointerCoercion {
        kw: kw::PointerCoercion,
        #[syn(parenthesized)]
        paren: token::Paren,
        #[syn(in = paren)]
        coercion: PointerCoercion,
        #[syn(in = paren)]
        comma: Token![,],
        #[syn(in = paren)]
        source: CoercionSource,
    },
}

#[derive(ToTokens, Parse)]
pub struct RvalueCast {
    pub operand: Operand,
    tk_as: Token![as],
    pub ty: Type,
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub cast_kind: CastKind,
}

#[derive(Clone, Copy, Parse, ToTokens, From)]
#[rustfmt::skip]
pub enum BinOp {
    #[parse(peek = kw::Add)] Add(kw::Add),
    #[parse(peek = kw::Sub)] Sub(kw::Sub),
    #[parse(peek = kw::Mul)] Mul(kw::Mul),
    #[parse(peek = kw::Div)] Div(kw::Div),
    #[parse(peek = kw::Rem)] Rem(kw::Rem),
    #[parse(peek = kw::BitAnd)] BitAnd(kw::BitAnd),
    #[parse(peek = kw::BitOr)] BitOr(kw::BitOr),
    #[parse(peek = kw::Lt)] Lt(kw::Lt),
    #[parse(peek = kw::Gt)] Gt(kw::Gt),
    #[parse(peek = kw::Le)] Le(kw::Le),
    #[parse(peek = kw::Ge)] Ge(kw::Ge),
    #[parse(peek = kw::Eq)] Eq(kw::Eq),
    #[parse(peek = kw::Ne)] Ne(kw::Ne),
    #[parse(peek = kw::Offset)] Offset(kw::Offset),
}

#[derive(ToTokens, Parse)]
pub struct RvalueBinOp {
    pub op: BinOp,
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub lhs: Operand,
    #[syn(in = paren)]
    tk_comma: Token![,],
    #[syn(in = paren)]
    pub rhs: Operand,
}

#[derive(Clone, Copy, ToTokens, Parse, From)]
#[rustfmt::skip]
pub enum NullOp {
    #[parse(peek = kw::SizeOf)] SizeOf(kw::SizeOf),
    #[parse(peek = kw::AlignOf)] AlignOf(kw::AlignOf),
    // OffsetOf(kw::OffsetOf),
}

#[derive(ToTokens, Parse)]
pub struct RvalueNullOp {
    pub op: NullOp,
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub ty: Type,
}

#[derive(Clone, Copy, ToTokens, Parse, From)]
#[rustfmt::skip]
pub enum UnOp {
    #[parse(peek = kw::Neg)] Neg(kw::Neg),
    #[parse(peek = kw::Not)] Not(kw::Not),
    #[parse(peek = kw::PtrMetadata)] PtrMetadata(kw::PtrMetadata),
}

#[derive(ToTokens, Parse)]
pub struct RvalueUnOp {
    pub op: UnOp,
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub operand: Operand,
}

#[derive(ToTokens, Parse)]
pub struct RvalueDiscriminant {
    kw_discr: kw::discriminant,
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub place: Place,
}

#[derive(ToTokens, Parse)]
pub struct AggregateArray {
    // bracket: token::Bracket,
    // pub ty: Box<Type>,
    // tk_semi: Token![;],
    // tk_underscore: Token![_],
    // kw_from: kw::from,
    pub operands: BracketedOperands,
}

#[derive(ToTokens, Parse, From)]
pub struct AggregateTuple {
    #[parse(ParenthesizedOperands::parse_tuple_like)]
    pub operands: ParenthesizedOperands,
}

#[derive(ToTokens, Parse)]
pub struct StructField {
    pub ident: Ident,
    tk_colon: Token![:],
    pub operand: Operand, /* FIXME _marker: std::marker::PhantomData `::` <u8> */
}

#[derive(ToTokens, Parse)]
pub struct StructFields {
    #[syn(braced)]
    brace: token::Brace,
    #[syn(in = brace)]
    pub fields: PunctuatedWithEnd<StructField>,
}

#[derive(ToTokens, Parse, From)]
pub enum PathOrLangItem {
    #[parse(peek = Token![#])]
    LangItem(LangItemWithArgs),
    Path(Path),
}

#[derive(ToTokens, Parse)]
pub struct AggregateAdtStruct {
    pub adt: PathOrLangItem,
    pub fields: StructFields,
}

#[derive(ToTokens, Parse)]
pub struct AggregateAdtUnit {
    pub adt: PathOrLangItem,
}

#[derive(ToTokens, Parse)]
pub struct Ctor {
    pub pound: Token![#],
    #[syn(bracketed)]
    bracket: token::Bracket,
    #[syn(in = bracket)]
    pub kw_ctor: kw::ctor,
}

#[derive(ToTokens, Parse)]
pub struct AggregateAdtTuple {
    ctor: Ctor,
    pub adt: Path,
    pub fields: ParenthesizedOperands,
}

#[derive(ToTokens, Parse)]
pub struct AggregateRawPtr {
    pub ty: TypePtr,
    kw_from: kw::from,
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub ptr: Operand,
    #[syn(in = paren)]
    tk_comma: Token![,],
    #[syn(in = paren)]
    pub metadata: Operand,
}

#[derive(ToTokens, From)]
pub enum RvalueAggregate {
    Array(AggregateArray),
    Tuple(AggregateTuple),
    AdtStruct(AggregateAdtStruct),
    AdtTuple(AggregateAdtTuple),
    AdtUnit(AggregateAdtUnit),
    RawPtr(AggregateRawPtr),
}

#[derive(ToTokens, From)]
pub enum Rvalue {
    Any(Token![_]),
    Use(RvalueUse),
    Repeat(RvalueRepeat),
    Ref(RvalueRef),
    RawPtr(RvalueRawPtr),
    Len(RvalueLen),
    Cast(RvalueCast),
    BinaryOp(RvalueBinOp),
    NullaryOp(RvalueNullOp),
    UnaryOp(RvalueUnOp),
    Discriminant(RvalueDiscriminant),
    Aggregate(RvalueAggregate),
    // ShallowInitBox(Operand<'tcx>, Ty<'tcx>),
    // CopyForDeref(CopyForDerefValue),
}

pub type ParenthesizedOperands = Parenthesized<Punctuated<Operand, Token![,]>, parse::PunctuatedParseTerminated>;

#[derive(Parse, ToTokens)]
pub struct BracketedOperands {
    #[syn(bracketed)]
    bracket: token::Bracket,
    #[syn(in = bracket)]
    #[parse(Punctuated::parse_terminated)]
    pub operands: Punctuated<Operand, Token![,]>,
}

#[derive(ToTokens, Parse)]
pub struct Call {
    pub func: FnOperand,
    pub operands: ParenthesizedOperands,
}

pub struct Macro<K, C, P = parse::ParseParse> {
    kw: K,
    tk_bang: Token![!],
    delim: syn::MacroDelimiter,
    pub content: C,
    _parse: PhantomData<P>,
}

#[derive(ToTokens, From)]
pub enum RvalueOrCall {
    Rvalue(Rvalue),
    Call(Call),
}

#[derive(ToTokens, Parse)]
pub struct UsePath {
    tk_use: Token![use],
    pub path: Path,
    tk_semi: Token![;],
}

#[derive(ToTokens, Parse)]
pub struct LocalInit {
    tk_eq: Token![=],
    pub rvalue_or_call: RvalueOrCall,
}

/// A local declaration with an optional initializer.
#[derive(Parse, ToTokens)]
pub struct LocalDecl {
    #[parse(Export::parse_opt)]
    pub export: Option<Export>,
    tk_let: Token![let],
    tk_mut: Option<Token![mut]>,
    // FIXME: change it to `ident: Ident`
    pub local: PlaceLocal,
    tk_colon: Token![:],
    pub ty: Type,
    #[parse(PunctAnd::parse_opt)]
    pub init: Option<PunctAnd<Token![=], RvalueOrCall>>,
    tk_semi: Token![;],
}

impl LocalDecl {
    pub fn is_mut(&self) -> bool {
        self.tk_mut.is_some()
    }
}

#[derive(ToTokens, Parse)]
pub struct Assign {
    pub place: Place,
    tk_eq: Token![=],
    pub rvalue_or_call: RvalueOrCall,
}

#[derive(ToTokens, Parse)]
pub struct CallIgnoreRet {
    tk_underscore: Token![_],
    tk_eq: Token![=],
    pub call: Call,
}

#[derive(ToTokens, Parse)]
pub struct Drop {
    kw_drop: kw::drop,
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub place: Place,
}

#[derive(ToTokens, Parse, From)]
pub enum Declaration {
    #[parse(peek = Token![type])]
    TypeDecl(TypeDecl),
    #[parse(peek = Token![use])]
    UsePath(UsePath),
    LocalDecl(LocalDecl),
}

#[derive(ToTokens, Parse)]
pub struct Loop {
    pub label: Option<syn::Label>,
    tk_loop: Token![loop],
    pub block: Block,
}

#[derive(ToTokens, Parse)]
pub enum Control {
    #[parse(peek = Token![break])]
    Break(Token![break], Option<syn::Label>),
    #[parse(peek = Token![continue])]
    Continue(Token![continue], Option<syn::Label>),
}

#[derive(ToTokens, Parse, IntoIterator, Deref)]
pub struct Many<T: quote::ToTokens, P: parse::ParseFn<T> = parse::ParseParse>(
    #[parse(P::parse_many)]
    #[to_tokens(|tokens, elems: &Vec<T>| elems.iter().for_each(|elem| elem.to_tokens(tokens)))]
    #[into_iterator(owned, ref, ref_mut)]
    #[deref]
    pub Vec<T>,
    #[parse(|_| Ok(PhantomData))]
    #[to_tokens(|_, _| {})]
    PhantomData<P>,
);

// type Attributes = Many<syn::Attribute, parse::AttributeParseOuter>;

#[derive(ToTokens, Parse)]
pub struct Block {
    #[syn(braced)]
    brace: token::Brace,
    #[syn(in = brace)]
    pub statements: Many<Statement>,
}

#[derive(ToTokens, Parse)]
pub enum SwitchBody {
    #[parse(peek = token::Brace)]
    Block(Block),
    Statement(StatementKind<syn::parse::Nothing>, Token![,]),
}

#[derive(ToTokens, Parse)]
pub struct SwitchTarget {
    #[parse(Export::parse_opt)]
    pub export: Option<Export>,
    pub value: SwitchValue,
    tk_arrow: Token![=>],
    pub body: SwitchBody,
}

#[derive(ToTokens, From)]
pub enum SwitchValue {
    Bool(syn::LitBool),
    Int(syn::LitInt),
    Underscore(Token![_]),
}

#[derive(Parse, ToTokens)]
pub struct SwitchInt {
    kw_switch_int: kw::switchInt,
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub operand: Operand,
    #[syn(braced)]
    brace: token::Brace,
    #[syn(in = brace)]
    pub targets: Many<SwitchTarget>,
}

#[derive(Parse, ToTokens)]
pub enum StatementKind<End: quote::ToTokens + syn::parse::Parse = Token![;]> {
    #[parse(peek = Token![_])]
    Call(CallIgnoreRet, End),
    #[parse(peek_func = |input| input.peek(kw::drop) && input.peek2(token::Paren))]
    Drop(Drop, End),
    #[parse(peek_func = Control::peek)]
    Control(Control, End),
    #[parse(peek = Token![loop])]
    Loop(Loop),
    #[parse(peek = kw::switchInt)]
    SwitchInt(SwitchInt),
    Assign(Assign, End),
}

#[derive(ToTokens, Parse)]
pub struct Statement {
    #[parse(Export::parse_opt)]
    pub export: Option<Export>,
    pub kind: StatementKind,
}

#[derive(ToTokens, Parse)]
pub struct Predicate {
    pub not: Option<token::Not>,
    pub pred: syn::Path,
}

#[derive(ToTokens, Parse)]
pub enum PredicateClause {
    #[parse(peek = token::Paren)]
    Paren {
        #[syn(parenthesized)]
        paren: token::Paren,
        #[syn(in = paren)]
        #[parse(Punctuated::parse_terminated)]
        preds: Punctuated<Predicate, syn::token::OrOr>,
    },
    Predicate {
        #[parse(Punctuated::parse_separated_nonempty)]
        preds: Punctuated<Predicate, syn::token::OrOr>,
    },
}

#[derive(ToTokens, Parse)]
pub struct PredicateConjunction {
    #[parse(Punctuated::parse_separated_nonempty)]
    pub preds: Punctuated<PredicateClause, syn::token::AndAnd>,
}

/// A type variable from `#[meta($T:ty)]`, `#[meta($T:ty where pred)]` or other pattern.
#[derive(ToTokens, Parse)]
pub struct TyVar {
    kw_ty: kw::ty,
    #[parse(PunctAnd::parse_opt)]
    pub preds: Option<PunctAnd<Token![where], PredicateConjunction>>,
}

/// A type variable from `#[meta($c:const(ty))]`, `#[meta($c:const(ty) = pred)]` or other pattern.
#[derive(ToTokens, Parse)]
pub struct ConstMetaVar {
    kw_const: Token![const],
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub ty: Type,
    #[parse(PunctAnd::parse_opt)]
    pub preds: Option<PunctAnd<Token![where], PredicateConjunction>>,
}

/// A place variable from `#[meta($p:place(ty))]` or other pattern.
#[derive(ToTokens, Parse)]
pub struct PlaceMetaVar {
    kw_ty: kw::place,
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub ty: Type,
}

#[derive(ToTokens, Parse, From)]
pub enum MetaKind {
    #[parse(peek = kw::ty)]
    Ty(TyVar),
    #[parse(peek = Token![const])]
    Const(ConstMetaVar),
    #[parse(peek = kw::place)]
    Place(PlaceMetaVar),
}

#[derive(ToTokens, Parse)]
pub struct MetaItem {
    #[parse(Export::parse_opt)]
    pub export: Option<Export>,
    tk_dollar: Token![$],
    pub ident: Ident,
    tk_colon: Token![:],
    pub kind: MetaKind,
}

pub type Meta = Attribute<kw::meta, Punctuated<MetaItem, Token![,]>, parse::PunctuatedParseTerminated>;

#[derive(Parse, ToTokens)]
pub struct Mir {
    pub declarations: Many<Declaration>,
    pub statements: Many<Statement>,
}

#[derive(Clone, Copy, Parse, ToTokens)]
pub enum ExportKind {
    #[parse(peek = kw::Statement)]
    Statement(kw::Statement),
}

#[derive(Parse, ToTokens)]
pub struct Attribute<
    P: syn::parse::Parse + quote::ToTokens,
    I: quote::ToTokens,
    Parse: parse::ParseFn<I> = parse::ParseParse,
> {
    tk_pound: Token![#],
    #[syn(bracketed)]
    bracket: token::Bracket,
    #[syn(in = bracket)]
    path: P,
    #[syn(in = bracket)]
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    #[parse(Parse::parse)]
    pub inner: I,
    #[parse(|_| Ok(PhantomData))]
    #[to_tokens(|_, _| {})]
    _parse: PhantomData<Parse>,
}

pub type Export = Attribute<kw::export, ExportItem>;

#[derive(Parse, ToTokens)]
pub struct ExportItem {
    pub ident: Ident,
    #[parse(PunctAnd::parse_opt)]
    pub kind: Option<PunctAnd<Token![:], ExportKind>>,
}

#[derive(Parse, ToTokens)]
pub enum IdentPat {
    #[parse(peek = Token![_])]
    Underscore(Token![_]),
    #[parse(peek = Token![$])]
    Pat(Token![$], Ident),
    #[parse(peek = Ident)]
    Ident(Ident),
}

impl IdentPat {
    pub fn as_ident(&self) -> Option<&Ident> {
        match self {
            IdentPat::Ident(ident) | IdentPat::Pat(_, ident) => Some(ident),
            _ => None,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            IdentPat::Underscore(underscore) => underscore.span,
            IdentPat::Pat(_, ident) => ident.span(),
            IdentPat::Ident(ident) => ident.span(),
        }
    }
}

#[derive(Parse, ToTokens)]
pub struct ParamPat {
    pub mutability: Mutability,
    tk_dollar: Token![$],
    pub ident: Ident,
    tk_colon: Token![:],
}

#[derive(Parse, ToTokens)]
pub struct NormalParam {
    #[parse(ParamPat::parse_opt)]
    pub ident: Option<ParamPat>,
    pub ty: Type,
}

#[derive(Parse, ToTokens)]
pub struct PunctAnd<P: syn::parse::Parse + quote::ToTokens, T: syn::parse::Parse + quote::ToTokens> {
    punct: P,
    pub value: T,
}

#[derive(Parse, ToTokens)]
pub struct SelfParam {
    pub reference: Option<Token![&]>,
    pub mutability: Mutability,
    pub tk_self: Token![self],
    #[parse(PunctAnd::parse_opt)]
    pub ty: Option<PunctAnd<Token![:], Type>>,
}

#[derive(Parse, ToTokens)]
pub enum FnParamKind {
    #[parse(peek_func = SelfParam::peek)]
    SelfParam(SelfParam),
    Param(NormalParam),
}

#[derive(Parse, ToTokens)]
pub struct FnParam {
    #[parse(Export::parse_opt)]
    pub export: Option<Export>,
    pub kind: FnParamKind,
}

#[derive(ToTokens, IntoIterator, Deref)]
pub struct PunctuatedWithEnd<T: quote::ToTokens, P: quote::ToTokens = Token![,], End: quote::ToTokens = Token![..]> {
    #[into_iterator(owned, ref, ref_mut)]
    #[deref]
    pub punctuated: Punctuated<T, P>,
    pub end: Option<End>,
}

#[derive(Parse, ToTokens)]
pub enum FnRet {
    #[parse(peek_func = |input| input.peek(Token![->]) && input.peek3(Token![_]))]
    Any(Token![->], Token![_]),
    Ret(ReturnType),
}

#[derive(Parse, ToTokens)]
pub struct FnSig {
    tk_fn: Token![fn],
    pub ident: IdentPat,
    #[syn(parenthesized)]
    paren: token::Paren,
    #[syn(in = paren)]
    pub params: PunctuatedWithEnd<FnParam>,
    pub ret: FnRet,
}

#[derive(Parse, ToTokens)]
pub struct MirBody {
    tk_eq: Token![=],
    kw_mir: kw::mir,
    tk_bang: Token![!],
    #[syn(braced)]
    brace: token::Brace,
    #[syn(in = brace)]
    pub mir: Mir,
}

#[derive(Parse, ToTokens)]
pub enum FnBody {
    #[parse(peek = Token![;])]
    Empty(Token![;]),
    #[parse(peek = Token![=])]
    Mir(MirBody),
    // Hir(HirBody),
}

#[derive(Parse, ToTokens)]
pub struct FnPat {
    pub sig: FnSig,
    pub body: FnBody,
}

#[derive(Parse, ToTokens)]
pub struct Field {
    tk_dollar: Token![$],
    pub ident: Ident,
    tk_colon: Token![:],
    pub ty: Type,
}

#[derive(Parse, ToTokens)]
pub struct Variant {
    pub ident: Ident,
    #[syn(braced)]
    brace: token::Brace,
    #[syn(in = brace)]
    #[parse(Punctuated::parse_terminated)]
    pub fields: Punctuated<Field, Token![,]>,
}

#[derive(Parse, ToTokens)]
pub struct Struct {
    tk_struct: Token![struct],
    tk_dollar: Token![$],
    pub ident: Ident,
    #[syn(braced)]
    brace: token::Brace,
    #[syn(in = brace)]
    #[parse(Punctuated::parse_terminated)]
    pub fields: Punctuated<Field, Token![,]>,
}

#[derive(Parse, ToTokens)]
pub struct Enum {
    tk_enum: Token![enum],
    tk_dollar: Token![$],
    pub ident: Ident,
    #[syn(braced)]
    brace: token::Brace,
    #[syn(in = brace)]
    #[parse(Punctuated::parse_terminated)]
    pub variants: Punctuated<Variant, Token![,]>,
}

#[derive(Parse, ToTokens)]
pub enum ImplKind {
    #[parse(peek_func = |input| input.parse::<Path>().is_ok() && input.peek(Token![for]))]
    Trait(Path, Token![for]),
    Inherent,
}

impl ImplKind {
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            ImplKind::Trait(path, _) => Some(path),
            ImplKind::Inherent => None,
        }
    }
}

#[derive(Parse, ToTokens)]
pub enum ImplItemKind {
    Fn(FnPat),
}

#[derive(Parse, ToTokens)]
pub struct ImplItem {
    #[parse(Meta::parse_opt)]
    pub meta: Option<Meta>,
    pub kind: ImplItemKind,
}

#[derive(Parse, ToTokens)]
pub struct Impl {
    tk_impl: Token![impl],
    pub kind: ImplKind,
    pub ty: Type,
    #[syn(braced)]
    pub brace: token::Brace,
    #[syn(in = brace)]
    pub items: Many<ImplItem>,
}

#[derive(Parse, ToTokens)]
pub enum ItemKind {
    #[parse(peek = Token![fn])]
    Fn(FnPat),
    #[parse(peek = Token![struct])]
    Struct(Struct),
    #[parse(peek = Token![enum])]
    Enum(Enum),
    #[parse(peek = Token![impl])]
    Impl(Impl),
}

#[derive(Parse, ToTokens)]
pub struct Item {
    #[parse(Meta::parse_opt)]
    pub meta: Option<Meta>,
    pub kind: ItemKind,
}

#[derive(Parse, ToTokens)]
pub struct Pattern {
    pub items: Many<Item>,
}
