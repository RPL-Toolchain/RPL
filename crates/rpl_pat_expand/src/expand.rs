use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote, quote_each_token, quote_token};
use syn::Ident;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::{Pair, Pairs, Punctuated};
use syntax::*;

use crate::SymbolTable;

const MACRO_RPL: &str = "rpl";

pub struct PatternDefFn {
    pub(crate) item_fn: syn::ItemFn,
}

impl Parse for PatternDefFn {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let item_fn = input.parse()?;
        Ok(PatternDefFn { item_fn })
    }
}

impl PatternDefFn {
    pub(crate) fn expand_macro_rpl(mut self) -> syn::Result<TokenStream> {
        let inputs = self.item_fn.sig.inputs.iter().collect::<Vec<_>>();
        let pcx = if let [syn::FnArg::Typed(patterns)] = inputs[..]
            && let box syn::Pat::Ident(ref patterns) = patterns.pat
        {
            Some(&patterns.ident)
        } else {
            None
        };
        for stmt in &mut self.item_fn.block.stmts {
            match stmt {
                syn::Stmt::Local(syn::Local {
                    init:
                        Some(syn::LocalInit {
                            expr: box syn::Expr::Macro(syn::ExprMacro { mac, .. }),
                            ..
                        }),
                    ..
                })
                | syn::Stmt::Macro(syn::StmtMacro { mac, .. })
                    if mac.path.is_ident(MACRO_RPL) =>
                {
                    mac.path = syn::parse_quote!(::rpl_macros::identity);
                    let pattern = syn::parse2(std::mem::take(&mut mac.tokens))?;
                    mac.tokens = crate::expand_pattern(&pattern, pcx)?;
                },
                _ => {},
            }
        }
        Ok(self.item_fn.into_token_stream())
    }
}

#[derive(Clone, Copy)]
struct ExpandCtxt<'pat> {
    pcx: &'pat Ident,
    symbols: &'pat SymbolTable<'pat>,
}

macro_rules! decl_pat_id {
    ($(#[$meta:meta])*
    $vis:vis enum $ident:ident {
        $( $variant:ident = $name:ident($($kind:literal)?), )*
    }) => {
        $(#[$meta])*
        $vis enum $ident<'pat> {
            $( #[allow(dead_code)] $variant(&'pat Ident), )*
        }

        impl<'pat> $ident<'pat> {
            $(
                #[track_caller]
                #[allow(dead_code)]
                fn ${concat(expect_, $name)}(self) -> &'pat Ident {
                    match self {
                        $ident::$variant(ident) => ident,
                        _ => panic!(concat!("expect ", $( $kind, " ",)? "pattern")),
                    }
                }
            )*
            fn as_ident(self) -> &'pat Ident {
                match self {
                    $( $ident::$variant(ident) => ident, )*
                }
            }
        }

        impl<'pat> ExpandPatCtxt<'pat> {
            $(
                #[allow(dead_code)]
                fn ${concat(get_, $name)}(&'pat self) -> Option<&'pat Ident> {
                    let mut ecx = self;
                    loop {
                        if let $ident::$variant(ident) = ecx.pat {
                            return Some(ident);
                        }
                        ecx = ecx.parent?;
                    }
                }
            )*
        }
    };
}

decl_pat_id! {
    #[derive(Clone, Copy)]
    enum PatId {
        Pattern = pattern(),
        Impl = impl("`impl`"),
        Fn = fn("`fn`"),
        Mir = mir("`mir`"),
        Enum = enum("`enum`"),
        Variant = variant("`variant`"),
    }
}

#[derive(Clone, Copy)]
struct ExpandPatCtxt<'pat> {
    pcx: &'pat Ident,
    pat: PatId<'pat>,
    meta: Option<&'pat Meta>,
    parent: Option<&'pat ExpandPatCtxt<'pat>>,
    symbols: &'pat SymbolTable<'pat>,
}

pub fn expand(pattern: PatternDefFn) -> TokenStream {
    pattern
        .expand_macro_rpl()
        .unwrap_or_else(syn::Error::into_compile_error)
}

static PARAM_PCX: &str = "pcx";

pub fn expand_pattern(pattern: &Pattern, pcx: Option<&Ident>) -> syn::Result<TokenStream> {
    let symbols = crate::check_pattern(pattern)?;
    let pcx = match pcx {
        None => &syn::Ident::new(PARAM_PCX, proc_macro2::Span::call_site()),
        Some(pcx) => pcx,
    };
    Ok(ExpandCtxt::new(pcx, &symbols).expand(pattern).to_token_stream())
}

trait ToSymbol: Sized + ToString {
    fn to_symbol(&self) -> IdentSymbol {
        IdentSymbol(self.to_string())
    }
}

impl<S: ToString> ToSymbol for S {}

trait ExpandIdent: Sized {
    fn with_suffix(&self, suffix: impl std::fmt::Display) -> Ident;
    // fn with_span(self, span: Span) -> Ident {
    //     let mut ident = self.into();
    //     ident.set_span(span);
    //     ident
    // }
    fn as_ty(&self) -> Ident {
        self.with_suffix("_ty")
    }
    fn as_const(&self) -> Ident {
        self.with_suffix("_const")
    }
    fn as_place(&self) -> Ident {
        //FIXME: use suffix "_place"?
        self.with_suffix("_local")
    }
    fn as_adt(&self) -> Ident {
        self.with_suffix("_adt")
    }
    fn as_fn(&self) -> Ident {
        self.with_suffix("_fn")
    }
    fn as_ty_var(&self) -> Ident {
        self.with_suffix("_ty_var")
    }
    fn as_const_var(&self) -> Ident {
        self.with_suffix("_const_var")
    }
    fn as_place_var(&self) -> Ident {
        self.with_suffix("_place_var")
    }
    fn as_local(&self) -> Ident {
        self.with_suffix("_local")
    }
}

impl ExpandIdent for &'static str {
    fn with_suffix(&self, suffix: impl std::fmt::Display) -> Ident {
        format_ident!("{self}{suffix}")
    }
}
impl ExpandIdent for syn::Token![self] {
    fn with_suffix(&self, suffix: impl std::fmt::Display) -> Ident {
        format_ident!("self{suffix}", span = self.span)
    }
}
impl ExpandIdent for Ident {
    fn with_suffix(&self, suffix: impl std::fmt::Display) -> Ident {
        format_ident!("{self}{suffix}", span = self.span())
    }
}
impl ExpandIdent for PlaceLocal {
    fn with_suffix(&self, suffix: impl std::fmt::Display) -> Ident {
        self.kind.with_suffix(suffix)
    }
}
impl ExpandIdent for PlaceLocalKind {
    fn with_suffix(&self, suffix: impl std::fmt::Display) -> Ident {
        match self {
            PlaceLocalKind::Return(return_value) => format_ident!("RET{suffix}", span = return_value.span),
            PlaceLocalKind::Local(ident) => ident.with_suffix(suffix),
            PlaceLocalKind::SelfValue(self_value) => self_value.with_suffix(suffix),
        }
    }
}

impl<'pat> ExpandCtxt<'pat> {
    fn new(pcx: &'pat Ident, symbols: &'pat SymbolTable<'pat>) -> Self {
        Self { pcx, symbols }
    }
    fn with_pat(self, pat: PatId<'pat>) -> ExpandPatCtxt<'pat> {
        ExpandPatCtxt {
            pcx: self.pcx,
            pat,
            meta: None,
            parent: None,
            symbols: self.symbols,
        }
    }
    fn expand<T>(self, value: T) -> Expand<'pat, T> {
        Expand { value, ecx: self }
    }
    fn expand_projections<'a>(&self, place: &'a Place) -> Expand<'_, Projections<'a>> {
        self.expand(Projections(place))
    }
    fn expand_punctuated<'a, U: 'a, P: 'a>(self, value: &'a Punctuated<U, P>) -> ExpandPunctPairs<'pat, 'a, U, P> {
        self.expand_pairs(value, Punctuated::pairs)
    }
    fn expand_pairs<'a, U: 'a, I, F: Fn(&'a U) -> I>(self, value: &'a U, to_pairs: F) -> ExpandPairs<'pat, &'a U, F> {
        ExpandPairs {
            pairs: value,
            to_pairs,
            ecx: self,
        }
    }
}

impl<'pat> ExpandPatCtxt<'pat> {
    fn expand<T>(self, value: T) -> ExpandPat<'pat, T> {
        ExpandPat { value, ecx: self }
    }
    fn with_pat(&'pat self, pat: PatId<'pat>) -> Self {
        ExpandPatCtxt {
            pcx: self.pcx,
            pat,
            meta: self.meta,
            parent: Some(self),
            symbols: self.symbols,
        }
    }
    fn with_opt_meta(&'pat self, meta: Option<&'pat Meta>) -> Self {
        ExpandPatCtxt {
            pcx: self.pcx,
            pat: self.pat,
            meta,
            parent: Some(self),
            symbols: self.symbols,
        }
    }
    fn expand_punctuated<'a, U: 'a, P: 'a>(self, value: &'a Punctuated<U, P>) -> ExpandPatPunctPairs<'pat, 'a, U, P> {
        self.expand_pairs(value, Punctuated::pairs)
    }
    #[allow(clippy::type_complexity)] // FIXME: return type too complex
    fn expand_punctuated_mapped<'a, U: 'a, V: 'a, P: 'a>(
        self,
        value: &'a Punctuated<U, P>,
        f: fn(&'a U) -> V,
    ) -> ExpandPatPunct<'pat, 'a, U, P, impl Fn(&'a Punctuated<U, P>) -> impl IntoIterator<Item = Pair<V, &'a P>>> {
        self.expand_pairs(value, move |value| value.pairs().map_value(f))
    }
    fn expand_pairs<'a, U: 'a, I, F: Fn(&'a U) -> I>(
        self,
        value: &'a U,
        to_pairs: F,
    ) -> ExpandPatPairs<'pat, &'a U, F> {
        ExpandPatPairs {
            pairs: value,
            to_pairs,
            ecx: self,
        }
    }
}

type ExpandPunct<'ecx, 'a, U, P, F> = ExpandPairs<'ecx, &'a Punctuated<U, P>, F>;
type ExpandPunctPairs<'ecx, 'a, U, P> = ExpandPunct<'ecx, 'a, U, P, fn(&'a Punctuated<U, P>) -> Pairs<'a, U, P>>;
type ExpandPatPunct<'ecx, 'a, U, P, F> = ExpandPatPairs<'ecx, &'a Punctuated<U, P>, F>;
type ExpandPatPunctPairs<'ecx, 'a, U, P> = ExpandPatPunct<'ecx, 'a, U, P, fn(&'a Punctuated<U, P>) -> Pairs<'a, U, P>>;

struct Expand<'ecx, T> {
    value: T,
    ecx: ExpandCtxt<'ecx>,
}

struct ExpandPat<'ecx, T> {
    value: T,
    ecx: ExpandPatCtxt<'ecx>,
}

struct ExpandPairs<'ecx, T, F> {
    pairs: T,
    to_pairs: F,
    ecx: ExpandCtxt<'ecx>,
}

struct ExpandPatPairs<'ecx, T, F> {
    pairs: T,
    to_pairs: F,
    ecx: ExpandPatCtxt<'ecx>,
}

impl<'a, T: 'a, I, F, U: 'a, P: 'a> ToTokens for ExpandPairs<'_, &'a T, F>
where
    F: Fn(&'a T) -> I,
    I: IntoIterator<Item = Pair<U, &'a P>>,
    for<'ecx> Expand<'ecx, U>: ToTokens,
    P: ToTokens,
{
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        for pair in (self.to_pairs)(self.pairs) {
            let (value, punct) = pair.into_tuple();
            let value = self.ecx.expand(value);
            quote_each_token!(tokens #value #punct);
        }
    }
}

impl<'a, T: 'a, I, F, U: 'a, P: 'a> ToTokens for ExpandPatPairs<'_, &'a T, F>
where
    F: Fn(&'a T) -> I,
    I: IntoIterator<Item = Pair<U, &'a P>>,
    for<'ecx> ExpandPat<'ecx, U>: ToTokens,
    P: ToTokens,
{
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        for pair in (self.to_pairs)(self.pairs) {
            let (value, punct) = pair.into_tuple();
            let value = self.ecx.expand(value);
            quote_each_token!(tokens #value #punct);
        }
    }
}

trait PairsExt<'a, T: 'a, P: 'a> {
    fn map_value<U>(self, f: impl FnMut(&'a T) -> U) -> impl Iterator<Item = Pair<U, &'a P>>;
}

impl<'a, T: 'a, P: 'a> PairsExt<'a, T, P> for Pairs<'a, T, P> {
    fn map_value<U>(self, mut f: impl FnMut(&'a T) -> U) -> impl Iterator<Item = Pair<U, &'a P>> {
        self.map(move |pair| {
            let (value, punct) = pair.into_tuple();
            Pair::new(f(value), punct)
        })
    }
}

impl<'ecx, T: Clone> ToTokens for ExpandPat<'ecx, T>
where
    Expand<'ecx, T>: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        Expand {
            value: self.value.clone(),
            ecx: ExpandCtxt {
                pcx: self.ecx.pcx,
                symbols: self.ecx.symbols,
            },
        }
        .to_tokens(tokens);
    }
}

impl ToTokens for Expand<'_, &Pattern> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandCtxt { pcx, .. } = self.ecx;
        let Pattern { items } = self.value;
        let pattern = format_ident!("pattern");
        let ecx_with_pat = self.ecx.with_pat(PatId::Pattern(&pattern));
        let items = items.iter().map(|item| ecx_with_pat.expand(item));
        quote_each_token!(tokens {
            let #pattern = #pcx.new_pattern();
            #(#items)*
            &*#pattern
        });
    }
}

impl ToTokens for ExpandPat<'_, &Item> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Item { meta, kind } = self.value;
        self.ecx.with_opt_meta(meta.as_ref()).expand(kind).to_tokens(tokens);
    }
}

impl ToTokens for ExpandPat<'_, &ItemKind> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.value {
            ItemKind::Fn(fn_pat) => self.ecx.expand(fn_pat).to_tokens(tokens),
            ItemKind::Struct(struct_pat) => self.ecx.expand(struct_pat).to_tokens(tokens),
            ItemKind::Enum(enum_pat) => self.ecx.expand(enum_pat).to_tokens(tokens),
            ItemKind::Impl(_impl_pat) => todo!(),
        }
    }
}

impl ToTokens for ExpandPat<'_, &FnPat> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pcx, .. } = self.ecx;
        let FnPat { sig, body } = self.value;
        let fn_pat = match sig.ident.as_ident() {
            Some(ident) => ident.as_fn(),
            None => format_ident!("fn_pat", span = sig.ident.span()),
        };
        self.ecx.with_pat(PatId::Fn(&fn_pat)).expand(sig).to_tokens(tokens);
        match body {
            FnBody::Empty(_semi) => {},
            FnBody::Mir(mir_body) => {
                let mir_pat = format_ident!("mir_pat");
                let mir_body = self.ecx.with_pat(PatId::Mir(&mir_pat)).expand(&mir_body.mir);
                quote_each_token!(tokens
                    #mir_body
                    let #mir_pat = #pcx.mk_mir_pattern(#mir_pat);
                    #fn_pat.set_body(::rpl_context::pat::FnBody::Mir(#mir_pat));
                );
            },
        }
    }
}

impl ToTokens for ExpandPat<'_, &FnSig> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, meta, .. } = self.ecx;
        let fn_pat = pat.expect_fn();
        let pattern = self.ecx.get_pattern().unwrap();
        let FnSig { ident, params, ret, .. } = self.value;
        quote_each_token!(tokens let #fn_pat = #pattern.fns.);
        match ident {
            IdentPat::Underscore(_) => {
                quote_each_token!(tokens new_unnamed(););
            },
            IdentPat::Pat(_, ident) => {
                let name = self.ecx.expand(ident.to_symbol());
                quote_each_token!(tokens new_fn_pat(#name););
            },
            IdentPat::Ident(ident) => {
                let name = self.ecx.expand(ident.to_symbol());
                quote_each_token!(tokens new_fn(#name););
            },
        }
        if let Some(meta) = meta {
            self.ecx.expand(meta).to_tokens(tokens);
        }
        let ret = self.ecx.expand(ret);
        quote_each_token!(tokens #fn_pat.set_ret_ty(#ret););
        for param in params.punctuated.iter() {
            self.ecx.expand(param).to_tokens(tokens);
        }
    }
}

impl ToTokens for ExpandPat<'_, &FnParam> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let fn_pat = pat.expect_fn();
        let FnParam { kind, .. } = self.value;
        match kind {
            FnParamKind::SelfParam(_param) => {
                todo!()
            },
            FnParamKind::Param(NormalParam { ident: None, ty }) => {
                let ty = self.ecx.expand(ty);
                let mutability = self.ecx.expand(Mutability::Not);
                quote_each_token!(tokens #fn_pat.params.add_param(::rustc_span::symbol::kw::Empty, #mutability, #ty););
            },
            &FnParamKind::Param(NormalParam {
                ident: Some(ParamPat {
                    mutability, ref ident, ..
                }),
                ref ty,
            }) => {
                let ident = self.ecx.expand(ident.to_symbol());
                let ty = self.ecx.expand(ty);
                let mutability = self.ecx.expand(mutability);
                quote_each_token!(tokens #fn_pat.params.add_param(#ident, #mutability, #ty););
            },
        }
    }
}

impl ToTokens for ExpandPat<'_, &FnRet> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pcx, .. } = self.ecx;
        match &self.value {
            FnRet::Ret(ReturnType::Type(_, box ty)) => self.ecx.expand(ty).to_tokens(tokens),
            FnRet::Ret(ReturnType::Default) => {
                quote_each_token!(tokens #pcx.mk_tuple_ty(&[]));
            },
            FnRet::Any(..) => {
                quote_each_token!(tokens #pcx.mk_any_ty());
            },
        }
    }
}

impl ToTokens for ExpandPat<'_, &Struct> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, meta, .. } = self.ecx;
        let pattern = pat.expect_pattern();
        let Struct { ident, fields, .. } = &self.value;
        let ident_sym = self.ecx.expand(ident.to_symbol());
        let ident = ident.as_adt();
        quote_each_token!(tokens
            #[allow(non_snake_case)]
            let #ident = #pattern.new_struct(#ident_sym);
        );
        let ecx_with_ident = self.ecx.with_pat(PatId::Variant(&ident));
        if let Some(meta) = meta {
            ecx_with_ident.expand(meta).to_tokens(tokens);
        }
        quote_each_token!(tokens
            #[allow(non_snake_case)]
            let #ident = #ident.non_enum_variant_mut();
        );
        for field in fields.iter() {
            ecx_with_ident.expand(field).to_tokens(tokens);
        }
    }
}

impl ToTokens for ExpandPat<'_, &Field> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let variant_pat = pat.expect_variant();
        let Field { ident, ty, .. } = self.value;
        let ident = self.ecx.expand(ident.to_symbol());
        let ty = self.ecx.expand(ty);
        quote_each_token!(tokens #variant_pat.add_field(#ident, #ty););
    }
}

impl ToTokens for ExpandPat<'_, &Enum> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, meta, .. } = self.ecx;
        let pattern = pat.expect_pattern();
        let Enum { ident, variants, .. } = self.value;
        let ident_sym = self.ecx.expand(ident.to_symbol());
        quote_each_token!(tokens
            #[allow(non_snake_case)]
            let #ident = #pattern.new_enum(#ident_sym);
        );
        if let Some(meta) = meta {
            self.ecx.expand(meta).to_tokens(tokens);
        }
        for variant in variants.iter() {
            self.ecx.with_pat(PatId::Enum(ident)).expand(variant).to_tokens(tokens);
        }
    }
}

impl ToTokens for ExpandPat<'_, &Variant> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let enum_pat = pat.expect_enum();
        let Variant { ident, fields, .. } = self.value;
        let ident_sym = self.ecx.expand(ident.to_symbol());
        let fields = fields
            .iter()
            .map(|field| self.ecx.with_pat(PatId::Variant(ident)).expand(field));
        quote_each_token!(tokens
            #[allow(non_snake_case)]
            let #ident = #enum_pat.add_variant(#ident_sym);
            #(#fields)*
        );
    }
}

impl ToTokens for ExpandPat<'_, &Mir> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        let Mir {
            declarations,
            statements,
        } = &self.value;
        let declarations = declarations.iter().map(|declaration| self.ecx.expand(declaration));
        let statements = statements.iter().map(|statement| self.ecx.expand(statement));
        quote_each_token!(tokens
            let mut #mir_pat = ::rpl_context::pat::MirPattern::builder();
            #(#declarations)* #(#statements)*
            let #mir_pat = #mir_pat.build();
        );
    }
}

impl ToTokens for ExpandPat<'_, &Predicate> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let Predicate { not, pred } = &self.value;
        if not.is_some() {
            quote_each_token!(tokens either::Either::Right(#pred),);
        } else {
            quote_each_token!(tokens either::Either::Left(#pred),);
        }
    }
}

impl ToTokens for ExpandPat<'_, &PredicateClause> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let preds = match &self.value {
            PredicateClause::Paren { preds, .. } | PredicateClause::Predicate { preds } => preds,
        };
        let preds = preds.iter().map(|pred| self.ecx.expand(pred));
        quote_each_token!(tokens &[
            #(#preds)*
        ],);
    }
}

impl ToTokens for ExpandPat<'_, &PredicateConjunction> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let PredicateConjunction { preds, .. } = &self.value;
        let preds = preds.iter().map(|pred| self.ecx.expand(pred));
        quote_each_token!(tokens &[
            #(#preds)*
        ]);
    }
}

impl ToTokens for ExpandPat<'_, &MetaItem> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pcx, pat, .. } = self.ecx;
        let pat = pat.as_ident();
        let MetaItem {
            export, ident, kind, ..
        } = &self.value;
        match kind {
            MetaKind::Ty(ty_var) => {
                let ty_ident = ident.as_ty();
                let ty_var_ident = ident.as_ty_var();

                if let Some(preds) = &ty_var.preds {
                    let preds = self.ecx.expand(&preds.value);
                    quote_each_token!(tokens
                        #[allow(non_snake_case)]
                        let #ty_var_ident = #pat.meta.new_ty_var(#preds);
                    );
                } else {
                    quote_each_token!(tokens
                        #[allow(non_snake_case)]
                        let #ty_var_ident = #pat.meta.new_ty_var(&[]);
                    );
                }
                quote_each_token!(tokens
                    #[allow(non_snake_case)]
                    let #ty_ident = #pcx.mk_var_ty(#ty_var_ident);
                );
                if let Some(Export { inner, .. }) = export {
                    let ident = &inner.ident;
                    quote_each_token!(tokens #ident = #ty_var_ident;);
                }
            },
            MetaKind::Const(const_var) => {
                let const_ident = ident.as_const();
                let const_var_ident = ident.as_const_var();
                let ty = self.ecx.expand(&const_var.ty);
                quote_each_token!(tokens
                    #[allow(non_snake_case)]
                    let #const_var_ident = #pat.meta.new_const_var(#ty);
                    #[allow(non_snake_case)]
                    #[allow(unused_variables)]
                    let #const_ident = #pcx.mk_var_const(#const_var_ident);
                );
                if let Some(Export { inner, .. }) = export {
                    let ident = &inner.ident;
                    quote_each_token!(tokens #ident = #const_var_ident;);
                }
            },
            MetaKind::Place(place_var) => {
                let place_ident = ident.as_place();
                let place_var_ident = ident.as_place_var();
                let ty = self.ecx.expand(&place_var.ty);
                quote_each_token!(tokens
                    #[allow(non_snake_case)]
                    let #place_var_ident = #pat.meta.new_place_var(#ty);
                    #[allow(non_snake_case)]
                    let #place_ident = #pcx.mk_var_place(#place_var_ident);
                );
                if let Some(Export { inner, .. }) = export {
                    let ident = &inner.ident;
                    quote_each_token!(tokens #ident = #place_var_ident;);
                }
            },
        }
    }
}

impl ToTokens for ExpandPat<'_, &Meta> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Meta { inner, .. } = self.value;
        for meta_item in inner.iter() {
            self.ecx.expand(meta_item).to_tokens(tokens);
        }
    }
}

impl ToTokens for ExpandPat<'_, &Declaration> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.value {
            Declaration::TypeDecl(type_decl) => self.ecx.expand(type_decl).to_tokens(tokens),
            Declaration::UsePath(use_path) => self.ecx.expand(use_path).to_tokens(tokens),
            Declaration::LocalDecl(local_decl) => self.ecx.expand(local_decl).to_tokens(tokens),
        }
    }
}

impl ToTokens for ExpandPat<'_, &Statement> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let Statement { export, kind, .. } = &self.value;
        if let Some(Export {
            inner: ExportItem { ident, .. },
            ..
        }) = export
        {
            quote_each_token!(tokens #ident =);
        }
        self.ecx.expand(kind).to_tokens(tokens);
    }
}

impl<End: Parse + ToTokens> ToTokens for ExpandPat<'_, &StatementKind<End>> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match &self.value {
            StatementKind::Assign(assign, _) => self.ecx.expand(assign).to_tokens(tokens),
            StatementKind::Call(call, _) => self.ecx.expand(call).to_tokens(tokens),
            StatementKind::Drop(drop, _) => self.ecx.expand(drop).to_tokens(tokens),
            StatementKind::Control(control, _) => self.ecx.expand(control).to_tokens(tokens),
            StatementKind::Loop(loop_) => self.ecx.expand(loop_).to_tokens(tokens),
            StatementKind::SwitchInt(switch_int) => self.ecx.expand(switch_int).to_tokens(tokens),
        }
    }
}

impl ToTokens for ExpandPat<'_, &CallIgnoreRet> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        let Call { func, operands } = &self.value.call;
        let func = self.ecx.expand(func);
        let operands = self.ecx.expand_punctuated(&operands.value);
        quote_each_token!(tokens #mir_pat.mk_fn_call(#func, #mir_pat.mk_list([#operands]), None); );
    }
}

impl ToTokens for ExpandPat<'_, &Control> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        match self.value {
            Control::Break(_, _) => {
                quote_each_token!(tokens #mir_pat.mk_break(););
            },
            Control::Continue(_, _) => {
                quote_each_token!(tokens #mir_pat.mk_continue(););
            },
        }
    }
}

impl ToTokens for ExpandPat<'_, &Loop> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        let statements = self
            .value
            .block
            .statements
            .iter()
            .map(|statement| self.ecx.expand(statement));
        quote_each_token!(tokens #mir_pat.mk_loop(|#mir_pat| { #(#statements)* }););
    }
}

impl ToTokens for ExpandPat<'_, &SwitchInt> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        let operand = self.ecx.expand(&self.value.operand);
        let targets = self.value.targets.iter().map(|target| self.ecx.expand(target));
        quote_each_token!(tokens #mir_pat.mk_switch_int(#operand, |mut #mir_pat| { #(#targets)* }););
    }
}

impl ToTokens for ExpandPat<'_, &SwitchTarget> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        let SwitchTarget { value, body, .. } = self.value;
        let body = self.ecx.expand(body);
        let value = match value {
            SwitchValue::Bool(lit_bool) => Some(quote!(#lit_bool)),
            SwitchValue::Int(lit_int) => Some(quote!(#lit_int)),
            SwitchValue::Underscore(_) => None,
        };
        let body = quote!(|#mir_pat| { #body });
        match value {
            Some(value) => {
                quote_each_token!(tokens #mir_pat.mk_switch_target(#value, #body););
            },
            None => {
                quote_each_token!(tokens #mir_pat.mk_otherwise(#body););
            },
        }
    }
}

impl ToTokens for ExpandPat<'_, &SwitchBody> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.value {
            SwitchBody::Statement(statement, _) => self.ecx.expand(statement).to_tokens(tokens),
            SwitchBody::Block(block) => self.ecx.expand(block).to_tokens(tokens),
        }
    }
}
impl ToTokens for ExpandPat<'_, &Block> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for statement in &self.value.statements {
            self.ecx.expand(statement).to_tokens(tokens);
        }
    }
}

impl ToTokens for Expand<'_, &TypeDecl> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let TypeDecl { ident, ty, .. } = self.value;
        let ty_ident = ident.as_ty();
        let ty = self.ecx.expand(ty);
        quote_each_token!(tokens #[allow(non_snake_case)] let #ty_ident = #ty;);
    }
}
impl ToTokens for Expand<'_, &UsePath> {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        todo!()
    }
}

impl ToTokens for ExpandPat<'_, &LocalDecl> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        let LocalDecl {
            export,
            local,
            ty,
            init,
            ..
        } = self.value;
        let ty = self.ecx.expand(ty);
        let ident = local.as_local();
        let mk_local_or_self = match local.kind {
            PlaceLocalKind::Return(_) => format_ident!("mk_return"),
            PlaceLocalKind::Local(_) => format_ident!("mk_local"),
            PlaceLocalKind::SelfValue(_) => format_ident!("mk_self"),
        };
        quote_each_token!(tokens let #ident = #mir_pat.#mk_local_or_self(#ty); );
        if let Some(PunctAnd {
            value: rvalue_or_call, ..
        }) = init
        {
            self.ecx
                .expand(Assign {
                    export: export.as_ref(),
                    place: local,
                    rvalue_or_call,
                })
                .to_tokens(tokens);
        } else if let PlaceLocalKind::SelfValue(_) = local.kind
            && let Some(Export { inner, .. }) = export
        {
            let stmt = &inner.ident;
            quote_each_token!(tokens #stmt = #ident;);
        }
    }
}

struct Assign<'a, P = Place> {
    export: Option<&'a Export>,
    place: &'a P,
    rvalue_or_call: &'a RvalueOrCall,
}

impl ToTokens for ExpandPat<'_, &syntax::Assign> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ecx
            .expand(Assign {
                // We have handled the export in its parent statement
                export: None,
                place: &self.value.place,
                rvalue_or_call: &self.value.rvalue_or_call,
            })
            .to_tokens(tokens);
    }
}

impl<'a, P> ToTokens for ExpandPat<'_, Assign<'a, P>>
where
    for<'ecx> ExpandPat<'ecx, &'a P>: ToTokens,
{
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        let Assign {
            export,
            place,
            rvalue_or_call,
        } = self.value;
        if let Some(Export { inner, .. }) = export {
            let stmt = &inner.ident;
            quote_each_token!(tokens #stmt =);
        }
        let place = self.ecx.expand(place);
        match rvalue_or_call {
            RvalueOrCall::Rvalue(rvalue) => {
                let rvalue = self.ecx.expand(rvalue);
                quote_each_token!(tokens #mir_pat.mk_assign(#place, #rvalue); );
            },
            RvalueOrCall::Call(Call { func, operands }) => {
                let func = self.ecx.expand(func);
                let operands = self.ecx.expand_punctuated(&operands.value);
                quote_each_token!(tokens #mir_pat.mk_fn_call(#func, #mir_pat.mk_list([#operands]), Some(#place)); );
            },
        }
    }
}

impl ToTokens for ExpandPat<'_, &Drop> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        let Drop { place, .. } = self.value;
        let place = self.ecx.expand(place);
        quote_each_token!(tokens #mir_pat.mk_drop(#place); );
    }
}

trait RegionExt {
    fn kind(&self) -> RegionKind;
}

impl RegionExt for Option<Region> {
    fn kind(&self) -> RegionKind {
        self.map_or(RegionKind::ReAny(Default::default()), |region| region.kind)
    }
}

impl ToTokens for Expand<'_, &Path> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandCtxt { pcx, .. } = self.ecx;
        let mut iter = self.value.segments.iter();
        let mut path = TokenStream::new();
        if let PathLeading::Crate(_) = self.value.leading {
            quote_each_token!(path "crate",);
        }
        let mut gen_args = TokenStream::new();
        for segment in iter.by_ref() {
            let ident = segment.ident.to_string();
            quote_each_token!(path #ident,);
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &segment.arguments {
                let trailing_punct = args.trailing_punct();
                let args = self.ecx.expand_punctuated(args);
                quote_each_token!(gen_args #args);
                if !trailing_punct {
                    quote_token!(, gen_args);
                }
            }
        }
        quote_each_token!(tokens #pcx.mk_path_with_args(
            #pcx.mk_item_path(&[#path]), &[#gen_args]
        ));
        if let Some(_rem) = iter.next() {
            todo!();
        }
    }
}

impl ToTokens for Expand<'_, &GenericArgument> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        match self.value {
            GenericArgument::Region(Region { kind, .. }) => self.ecx.expand(*kind).to_tokens(tokens),
            GenericArgument::Type(ty) => self.ecx.expand(ty).to_tokens(tokens),
            GenericArgument::Const(GenericConst { konst, .. }) => self.ecx.expand(konst).to_tokens(tokens),
        }
        quote_each_token!(tokens.into());
    }
}

impl ToTokens for Expand<'_, &Type> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandCtxt { pcx, symbols, .. } = self.ecx;
        match self.value {
            Type::Array(TypeArray { box ty, len, .. }) => {
                let ty = self.ecx.expand(ty);
                let len = self.ecx.expand(len);
                quote_each_token!(tokens #pcx.mk_array_ty(#ty, #len));
            },
            Type::Group(TypeGroup { box ty, .. }) | Type::Paren(TypeParen { value: box ty, .. }) => {
                self.ecx.expand(ty).to_tokens(tokens);
            },
            Type::Never(_) => todo!(),
            Type::Path(TypePath { qself: None, path }) if let Some(ident) = path.as_ident() => {
                if crate::is_primitive(ident) {
                    quote_each_token!(tokens #pcx.primitive_types.#ident);
                } else {
                    ident.as_ty().to_tokens(tokens);
                }
            },
            Type::Path(TypePath { path, .. }) => {
                let path = self.ecx.expand(path);
                quote_each_token!(tokens #pcx.mk_path_ty(#path));
            },
            Type::Ptr(TypePtr { mutability, box ty, .. }) => {
                let ty = self.ecx.expand(ty);
                let mutability = self.ecx.expand(*mutability);
                quote_each_token!(tokens #pcx.mk_raw_ptr_ty(#ty, #mutability));
            },
            Type::Reference(TypeReference {
                region,
                mutability,
                box ty,
                ..
            }) => {
                let region = self.ecx.expand(region.kind());
                let ty = self.ecx.expand(ty);
                let mutability = self.ecx.expand(*mutability);
                quote_each_token!(tokens #pcx.mk_ref_ty(#region, #ty, #mutability));
            },
            Type::Slice(TypeSlice { box ty, .. }) => {
                let ty = self.ecx.expand(ty);
                quote_each_token!(tokens #pcx.mk_slice_ty(#ty));
            },
            Type::Tuple(TypeTuple { tys, .. }) => {
                let tys = self.ecx.expand_punctuated(tys);
                quote_each_token!(tokens #pcx.mk_tuple_ty(&[#tys]));
            },
            Type::TyVar(TypeVar { ident, .. }) => {
                if symbols.contains_adt(ident) {
                    let adt_pat = self.ecx.expand(ident.to_symbol());
                    quote_each_token!(tokens #pcx.mk_adt_pat_ty(#adt_pat));
                } else {
                    ident.as_ty().to_tokens(tokens);
                }
            },
            Type::LangItem(lang_item) => {
                let lang_item = self.ecx.expand(lang_item);
                quote_each_token!(tokens #pcx.mk_adt_ty(#lang_item));
            },
            Type::SelfType(_) => todo!(),
            Type::Any(_) => {
                quote_each_token!(tokens #pcx.mk_any_ty());
            },
        }
    }
}

impl ToTokens for Expand<'_, &LangItemWithArgs> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandCtxt { pcx, .. } = self.ecx;
        let LangItemWithArgs { item, args, .. } = self.value;
        let mut gen_args = TokenStream::new();
        if let Some(args) = args {
            let args = self.ecx.expand_punctuated(&args.args);
            quote_each_token!(gen_args #args);
        }
        quote_each_token!(tokens #pcx.mk_path_with_args(
            #pcx.mk_lang_item(#item), &[#gen_args]
        ));
    }
}

impl ToTokens for ExpandPat<'_, &Rvalue> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        quote_each_token!(tokens ::rpl_context::pat::Rvalue::);
        match self.value {
            Rvalue::Any(_) => {
                quote_each_token!(tokens Any);
            },
            Rvalue::Use(RvalueUse { operand, .. }) => {
                let operand = self.ecx.expand(operand);
                quote_each_token!(tokens Use(#operand));
            },
            Rvalue::Repeat(RvalueRepeat { operand, len, .. }) => {
                let operand = self.ecx.expand(operand);
                quote_each_token!(tokens Repeat(#operand, #len));
            },
            Rvalue::Ref(RvalueRef {
                region,
                mutability,
                place,
                ..
            }) => {
                let region = self.ecx.expand(region.kind());
                let mutability = self.ecx.expand(BorrowKind(*mutability));
                let place = self.ecx.expand(place);
                quote_each_token!(tokens Ref(#region, #mutability, #place));
            },
            Rvalue::RawPtr(RvalueRawPtr { mutability, place, .. }) => {
                let mutability = self.ecx.expand(*mutability);
                let place = self.ecx.expand(place);
                quote_each_token!(tokens RawPtr(#mutability, #place));
            },
            Rvalue::Len(RvalueLen { place, .. }) => {
                let place = self.ecx.expand(place);
                quote_each_token!(tokens Len(#place));
            },
            Rvalue::Cast(RvalueCast {
                operand, ty, cast_kind, ..
            }) => {
                let operand = self.ecx.expand(operand);
                let ty = self.ecx.expand(ty);
                let cast_kind = self.ecx.expand(*cast_kind);
                quote_each_token!(tokens Cast(#cast_kind, #operand, #ty));
            },
            Rvalue::BinaryOp(RvalueBinOp { op, lhs, rhs, .. }) => {
                let op = self.ecx.expand(*op);
                let lhs = self.ecx.expand(lhs);
                let rhs = self.ecx.expand(rhs);
                quote_each_token!(tokens BinaryOp(#op, Box::new([#lhs, #rhs])));
            },
            Rvalue::NullaryOp(RvalueNullOp { op, ty, .. }) => {
                let op = self.ecx.expand(*op);
                let ty = self.ecx.expand(ty);
                quote_each_token!(tokens NullaryOp(#op, #ty));
            },
            Rvalue::UnaryOp(RvalueUnOp { op, operand, .. }) => {
                let op = self.ecx.expand(*op);
                let operand = self.ecx.expand(operand);
                quote_each_token!(tokens UnaryOp(#op, #operand));
            },
            Rvalue::Discriminant(RvalueDiscriminant { place, .. }) => {
                let place = self.ecx.expand(place);
                quote_each_token!(tokens Discriminant(#place));
            },
            Rvalue::Aggregate(aggregate) => {
                let aggregate = self.ecx.expand(aggregate);
                quote_each_token!(tokens Aggregate(#aggregate));
            },
        }
    }
}

impl ToTokens for ExpandPat<'_, &Operand> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        quote_each_token!(tokens ::rpl_context::pat::Operand::);
        match self.value {
            Operand::Any(_) => quote_token!(Any tokens),
            Operand::AnyMultiple(_) => todo!(),
            Operand::Copy(OperandCopy { place, .. }) => {
                let place = self.ecx.expand(place);
                quote_each_token!(tokens Copy(#place));
            },
            Operand::Move(OperandMove { place, .. }) => {
                let place = self.ecx.expand(place);
                quote_each_token!(tokens Move(#place));
            },
            Operand::Constant(konst) => {
                let konst = self.ecx.expand(&konst.kind);
                quote_each_token!(tokens Constant(#konst));
            },
        }
    }
}

impl ToTokens for ExpandPat<'_, &FnOperand> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        quote_each_token!(tokens ::rpl_context::pat::Operand::);
        match self.value {
            FnOperand::Copy(Parenthesized {
                value: OperandCopy { place, .. },
                ..
            }) => {
                let place = self.ecx.expand(place);
                quote_each_token!(tokens Copy(#place));
            },
            FnOperand::Move(Parenthesized {
                value: OperandMove { place, .. },
                ..
            }) => {
                let place = self.ecx.expand(place);
                quote_each_token!(tokens Move(#place));
            },
            FnOperand::Type(TypePath { qself: None, path }) => {
                let path = self.ecx.expand(path);
                quote_each_token!(tokens Constant(#mir_pat.mk_zeroed(#path)));
            },
            FnOperand::Type(TypePath {
                qself: Some(_qself),
                path: _,
            }) => {
                todo!()
            },
            FnOperand::LangItem(lang_item) => {
                let lang_item = self.ecx.expand(lang_item);
                quote_each_token!(tokens Constant(#mir_pat.mk_zeroed(#lang_item)));
            },
            FnOperand::FnPat(_, fn_pat) => {
                let fn_pat = self.ecx.expand(fn_pat.to_symbol());
                quote_each_token!(tokens FnPat(#fn_pat));
            },
        }
    }
}

impl ToTokens for ExpandPat<'_, &ConstOperandKind> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        match self.value {
            ConstOperandKind::Lit(lit) => self.ecx.expand(lit).to_tokens(tokens),
            ConstOperandKind::ConstVar(const_var) => {
                let konst = const_var.ident.as_const_var();
                quote_each_token!(tokens #mir_pat.mk_const_var(#konst));
            },
            ConstOperandKind::Type(TypePath { qself: None, path }) => {
                // todo!();
                let path = self.ecx.expand(path);
                quote_each_token!(tokens #mir_pat.mk_zeroed(#path));
            },
            ConstOperandKind::Type(TypePath {
                qself: Some(_qself),
                path: _,
            }) => {
                todo!();
            },
            ConstOperandKind::LangItem(lang_item) => {
                let lang_item = self.ecx.expand(lang_item);
                quote_each_token!(tokens #mir_pat.mk_zeroed(#lang_item));
            },
        }
    }
}

impl ToTokens for Expand<'_, &Const> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.value {
            Const::Lit(lit) => self.ecx.expand(lit).to_tokens(tokens),
            Const::ConstVar(const_var) => const_var.ident.as_const().to_tokens(tokens),
            Const::Path(TypePath { qself: None, path }) => {
                todo!("{}", path.to_token_stream());
                // let path = self.expand(path);
                // quote_each_token!(tokens ::rpl_context::pat::ConstOperand::ZeroSized(
                //     #patterns.pcx.mk_item_path(&[#path]),
                // ));
            },
            Const::Path(TypePath {
                qself: Some(_qself),
                path: _,
            }) => {
                todo!();
            },
        }
    }
}
impl ToTokens for Expand<'_, &syn::Lit> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        match self.value {
            syn::Lit::Str(_lit_str) => todo!(),
            syn::Lit::ByteStr(_lit_byte_str) => todo!(),
            syn::Lit::CStr(_lit_cstr) => todo!(),
            syn::Lit::Byte(_lit_byte) => todo!(),
            syn::Lit::Char(_lit_char) => todo!(),
            syn::Lit::Int(lit_int) => {
                quote_each_token!(tokens ::rpl_context::pat::ConstOperand::ScalarInt(#lit_int.into()));
            },
            syn::Lit::Float(_lit_float) => todo!(),
            syn::Lit::Bool(_lit_bool) => todo!(),
            syn::Lit::Verbatim(_literal) => todo!(),
            _ => todo!(),
        }
    }
}

#[derive(Clone, Copy)]
struct Projections<'a>(&'a Place);

impl ToTokens for Expand<'_, Projections<'_>> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let mut place = self.value.0;
        let mut projections = Vec::new();
        loop {
            let mut proj = quote!(::rpl_context::pat::PlaceElem::);
            let inner = match place {
                Place::Local(_) => break,
                Place::Paren(PlaceParen { place: box inner, .. }) => {
                    place = inner;
                    continue;
                },
                Place::Deref(PlaceDeref { box place, .. }) => {
                    quote_each_token!(proj Deref,);
                    place
                },
                Place::Field(
                    place_field @ PlaceField {
                        box place,
                        field: syn::Member::Named(field),
                        ..
                    },
                ) if place_field.is_pattern() => {
                    let field = self.ecx.expand(field.to_symbol());
                    quote_each_token!(proj FieldPat(#field),);
                    place
                },
                Place::Field(PlaceField { box place, field, .. }) => {
                    let field = self.ecx.expand(field);
                    quote_each_token!(proj Field(#field),);
                    place
                },
                Place::Index(PlaceIndex { box place, index, .. }) => {
                    let index = index.as_local();
                    quote_each_token!(proj Index(#index),);
                    place
                },
                &Place::ConstIndex(PlaceConstIndex {
                    box ref place,
                    from_end,
                    index: syn::Index { index, .. },
                    min_length: syn::Index { index: min_length, .. },
                    ..
                }) => {
                    let from_end = from_end.is_some();
                    quote_each_token!(proj ConstIndex {
                        offset: #index,
                        min_length: #min_length,
                        from_end: #from_end
                    },);
                    place
                },
                &Place::Subslice(PlaceSubslice {
                    box ref place,
                    ref from,
                    from_end,
                    to: syn::Index { index: to, .. },
                    ..
                }) => {
                    let from = from.as_ref().map_or(0, |from| from.index);
                    let from_end = from_end.is_some();
                    quote_each_token!(proj Subslice { from: #from, to: #to, from_end: #from_end },);
                    place
                },
                Place::DownCast(place_downcast @ PlaceDowncast { box place, variant, .. })
                    if place_downcast.is_pattern() =>
                {
                    let variant = self.ecx.expand(variant.to_symbol());
                    quote_each_token!(proj DowncastPat(#variant),);
                    place
                },
                Place::DownCast(PlaceDowncast { box place, variant, .. }) => {
                    let variant = self.ecx.expand(variant.to_symbol());
                    quote_each_token!(proj Downcast(#variant),);
                    place
                },
            };
            projections.push(proj);
            place = inner;
        }
        projections.reverse();
        quote_each_token!(tokens #(#projections)*);
    }
}

impl ToTokens for Expand<'_, &syn::Member> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        quote_each_token!(tokens ::rpl_context::pat::FieldAcc::);
        match self.value {
            syn::Member::Named(name) => {
                let name = self.ecx.expand(name.to_symbol());
                quote_each_token!(tokens Named(#name));
            },
            &syn::Member::Unnamed(syn::Index { index, .. }) => {
                quote_each_token!(tokens Unnamed(#index.into()));
            },
        }
    }
}

impl ToTokens for Expand<'_, &PlaceLocal> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let local = self.value.as_local();
        quote_each_token!(tokens #local.into_place());
    }
}

impl ToTokens for Expand<'_, &Place> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandCtxt { pcx, .. } = self.ecx;
        if let Place::Local(local) = self.value {
            self.ecx.expand(local).to_tokens(tokens);
        } else {
            let local = self.value.local().as_local();
            let projections = self.ecx.expand_projections(self.value);
            quote_each_token!(tokens ::rpl_context::pat::Place::new(#local, #pcx.mk_slice(&[#projections])));
        }
    }
}

#[derive(Clone)]
struct IdentSymbol(String);

impl ToTokens for Expand<'_, IdentSymbol> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ident = self.value.0.as_str();
        quote_each_token!(tokens ::rustc_span::Symbol::intern(#ident));
    }
}

impl ToTokens for ExpandPat<'_, &RvalueAggregate> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let ExpandPatCtxt { pat, .. } = self.ecx;
        let mir_pat = pat.expect_mir();
        quote_each_token!(tokens ::rpl_context::pat::AggKind::);
        match self.value {
            RvalueAggregate::Array(AggregateArray { operands, .. }) => {
                // let ty = self.expand(ty);
                let operands = self.ecx.expand_punctuated(&operands.operands);
                quote_each_token!(tokens Array, #mir_pat.mk_list([#operands]));
            },
            RvalueAggregate::Tuple(AggregateTuple { operands }) => {
                let operands = self.ecx.expand_punctuated(&operands.value);
                quote_each_token!(tokens Tuple, #mir_pat.mk_list([#operands]));
            },
            RvalueAggregate::AdtStruct(AggregateAdtStruct {
                adt,
                fields: StructFields { fields, .. },
            }) => {
                let adt = self.ecx.expand(adt);
                let operands = self.ecx.expand_punctuated_mapped(fields, |f| &f.operand);
                let fields = self.ecx.expand_punctuated_mapped(fields, |f| f.ident.to_symbol());
                quote_each_token!(tokens
                    Adt(#adt, #mir_pat.mk_list([#fields]).into()),
                    #mir_pat.mk_list([#operands]),
                );
            },
            RvalueAggregate::AdtTuple(AggregateAdtTuple { adt, fields, .. }) => {
                let adt = self.ecx.expand(adt);
                let operands = self.ecx.expand_punctuated(&fields.value);
                quote_each_token!(tokens
                    Adt(#adt, ::rpl_context::pat::AggAdtKind::Tuple),
                    #mir_pat.mk_list([#operands]),
                );
            },
            RvalueAggregate::AdtUnit(AggregateAdtUnit { adt }) => {
                let adt = self.ecx.expand(adt);
                quote_each_token!(tokens Adt(#adt, ::rpl_context::pat::AggAdtKind::Unit), Box::new([]));
            },
            RvalueAggregate::RawPtr(AggregateRawPtr {
                ty: TypePtr { mutability, box ty, .. },
                ptr,
                metadata,
                ..
            }) => {
                let ty = self.ecx.expand(ty);
                let mutability = self.ecx.expand(*mutability);
                let ptr = self.ecx.expand(ptr);
                let metadata = self.ecx.expand(metadata);
                quote_each_token!(tokens RawPtr(#ty, #mutability), #mir_pat.mk_list([#ptr, #metadata]));
            },
        }
    }
}

impl ToTokens for Expand<'_, &PathOrLangItem> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.value {
            PathOrLangItem::Path(path) => self.ecx.expand(path).to_tokens(tokens),
            PathOrLangItem::LangItem(lang_item) => self.ecx.expand(lang_item).to_tokens(tokens),
        }
    }
}

impl ToTokens for Expand<'_, RegionKind> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        quote_each_token!(tokens ::rpl_context::pat::RegionKind::);
        match self.value {
            RegionKind::ReAny(_) => quote_token!(ReAny tokens),
            RegionKind::ReStatic(_) => quote_token!(ReStatic tokens),
        }
    }
}

#[derive(Clone, Copy)]
struct BorrowKind(Mutability);

impl ToTokens for Expand<'_, BorrowKind> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        quote_each_token!(tokens ::rustc_middle::mir::BorrowKind::);
        match self.value.0 {
            Mutability::Not => {
                quote_each_token!(tokens Shared);
            },
            Mutability::Mut(_) => {
                quote_each_token!(tokens Mut { kind: ::rustc_middle::mir::MutBorrowKind::Default });
            },
        }
    }
}

impl ToTokens for Expand<'_, Mutability> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        quote_each_token!(tokens ::rustc_middle::mir::Mutability::);
        match self.value {
            Mutability::Not => quote_token!(Not tokens),
            Mutability::Mut(_) => quote_token!(Mut tokens),
        }
    }
}

impl ToTokens for Expand<'_, PtrMutability> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        quote_each_token!(tokens ::rustc_middle::mir::Mutability::);
        match self.value {
            PtrMutability::Const(_) => quote_token!(Not tokens),
            PtrMutability::Mut(_) => quote_token!(Mut tokens),
        }
    }
}

impl ToTokens for Expand<'_, CastKind> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        match self.value {
            CastKind::PtrToPtr(kw) => {
                quote_each_token!(tokens ::rustc_middle::mir::CastKind::#kw);
            },
            CastKind::IntToInt(kw) => {
                quote_each_token!(tokens ::rustc_middle::mir::CastKind::#kw);
            },
            CastKind::Transmute(kw) => {
                quote_each_token!(tokens ::rustc_middle::mir::CastKind::#kw);
            },
            CastKind::PointerExposeProvenance(kw) => {
                quote_each_token!(tokens ::rustc_middle::mir::CastKind::#kw);
            },
            CastKind::PointerCoercion {
                kw, coercion, source, ..
            } => {
                quote_each_token!(tokens
                    ::rustc_middle::mir::CastKind::#kw(
                        rustc_middle::ty::adjustment::PointerCoercion::#coercion,
                        ::rustc_middle::mir::CoercionSource::#source
                    )
                );
            },
        }
    }
}

impl ToTokens for Expand<'_, BinOp> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let value = self.value;
        quote_each_token!(tokens ::rustc_middle::mir::BinOp::#value);
    }
}

impl ToTokens for Expand<'_, UnOp> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let value = self.value;
        quote_each_token!(tokens ::rustc_middle::mir::UnOp::#value);
    }
}

impl ToTokens for Expand<'_, NullOp> {
    fn to_tokens(&self, mut tokens: &mut TokenStream) {
        let value = self.value;
        quote_each_token!(tokens ::rustc_middle::mir::NullOp::#value);
    }
}
