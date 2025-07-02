#![feature(rustc_private)]

//! Resolve an item path.
//!
//! See <https://doc.rust-lang.org/nightly/nightly-rustc/src/clippy_utils/lib.rs.html#691>
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;
#[macro_use]
extern crate tracing;

use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{CrateNum, LOCAL_CRATE, LocalDefId};
use rustc_hir::{ImplItemRef, ItemKind, Node, OwnerId, PrimTy, TraitItemRef};
use rustc_middle::ty::fast_reject::SimplifiedType;
use rustc_middle::ty::{FloatTy, IntTy, Mutability, TyCtxt, UintTy};
use rustc_span::Symbol;
use rustc_span::def_id::DefId;
use rustc_span::symbol::Ident;

/// Kind of an item path in pattern.
///
/// Matching a subset of [DefKind]. There 31 variants in [DefKind] currently.
///
/// Ignored as it's not visible from MIR:
///
/// - [DefKind::Macro]
///
/// Ignored as it's not accessible from root:
///
/// - [DefKind::TyParam]
/// - [DefKind::ConstParam]
/// - [DefKind::AnonConst]
/// - [DefKind::InlineConst]
/// - [DefKind::OpaqueTy]
/// - [DefKind::Field]
/// - [DefKind::LifetimeParam]
/// - [DefKind::GlobalAsm]
/// - [DefKind::Closure]
/// - [DefKind::SyntheticCoroutineBody]
///
/// Ignored as it's a container of items:
///
/// - [DefKind::ExternCrate]
/// - [DefKind::Use]
/// - [DefKind::ForeignMod]
/// - [DefKind::Impl]
#[derive(Debug, Clone, Copy)]
pub enum PatItemKind {
    // Type namespace
    /// [DefKind::Mod]
    Mod,
    /// [DefKind::Struct], [DefKind::Union], [DefKind::Enum], [DefKind::TyAlias],
    /// [DefKind::ForeignTy] and [DefKind::AssocTy]
    Type,
    /// [DefKind::Variant]
    Variant,
    /// [DefKind::Trait] and [DefKind::TraitAlias]
    Trait,
    /// [DefKind::Fn], [DefKind::AssocFn]
    Fn,
    /// [DefKind::Const], [DefKind::AssocConst]
    Const,
    /// [DefKind::Static]
    // FIXME: add fields to support more operations.
    Static,
    /// [DefKind::Ctor]
    Ctor,
}

impl PatItemKind {
    #[tracing::instrument(level = "trace", ret)]
    fn match_resolve(&self, res: &Res) -> bool {
        match self {
            Self::Mod => matches!(res, Res::Def(DefKind::Mod, _)),
            Self::Type => matches!(
                res,
                Res::Def(
                    DefKind::Struct
                        | DefKind::Union
                        | DefKind::Enum
                        | DefKind::TyAlias
                        | DefKind::ForeignTy
                        | DefKind::AssocTy,
                    _
                ) | Res::PrimTy(_)
            ),
            Self::Variant => matches!(res, Res::Def(DefKind::Variant, _)),
            Self::Trait => matches!(res, Res::Def(DefKind::Trait | DefKind::TraitAlias, _)),
            Self::Fn => matches!(res, Res::Def(DefKind::Fn | DefKind::AssocFn, _)),
            Self::Const => matches!(res, Res::Def(DefKind::Const | DefKind::AssocConst, _)),
            Self::Static => matches!(res, Res::Def(DefKind::Static { .. }, _)),
            Self::Ctor => matches!(res, Res::Def(DefKind::Ctor(..), _)),
        }
    }

    pub fn infer_from_def_kind(kind: DefKind) -> Option<Self> {
        Some(match kind {
            DefKind::Mod => Self::Mod,
            DefKind::Struct | DefKind::Union | DefKind::Enum => Self::Type,
            DefKind::Variant => Self::Variant,
            DefKind::Trait => Self::Trait,
            DefKind::TyAlias | DefKind::ForeignTy => Self::Type,
            DefKind::TraitAlias => Self::Trait,
            DefKind::AssocTy | DefKind::TyParam => Self::Type,
            DefKind::Fn => Self::Fn,
            DefKind::Const | DefKind::ConstParam => Self::Const,
            DefKind::Static { .. } => Self::Static,
            DefKind::Ctor(..) => Self::Ctor,
            DefKind::AssocFn => Self::Fn,
            DefKind::AssocConst => Self::Const,
            DefKind::Macro(..)
            | DefKind::ExternCrate
            | DefKind::Use
            | DefKind::ForeignMod
            | DefKind::AnonConst
            | DefKind::InlineConst
            | DefKind::OpaqueTy
            | DefKind::Field
            | DefKind::LifetimeParam
            | DefKind::GlobalAsm
            | DefKind::Impl { .. }
            | DefKind::Closure
            | DefKind::SyntheticCoroutineBody => None?,
        })
    }
}

/// Resolves a def path like `std::vec::Vec`.
///
/// Can return multiple resolutions when there are multiple versions of the same crate, e.g.
/// `memchr::memchr` could return the functions from both memchr 1.0 and memchr 2.0.
///
/// Also returns multiple results when there are multiple paths under the same name e.g. `std::vec`
/// would have both a [`DefKind::Mod`] and [`DefKind::Macro`].
///
/// This function is expensive and should be used sparingly.
#[tracing::instrument(level = "trace", skip(tcx), ret)]
pub fn def_path_res(tcx: TyCtxt<'_>, path: &[Symbol], kind: PatItemKind) -> Vec<Res> {
    let (base, path) = match path {
        [primitive] => {
            let primitive = PrimTy::from_name(*primitive);
            match primitive {
                Some(prim) => {
                    // If the path is a primitive type, return it directly.
                    return vec![Res::PrimTy(prim)];
                },
                None => {
                    // If the path is not a primitive type, show an error.
                    warn!("Unknown primitive type: {:?}", primitive);
                    return Vec::new();
                },
            }
        },
        [base, path @ ..] => (base, path),
        [] => return Vec::new(),
    };

    // let base_sym = Symbol::intern(base);

    let local_crate = if tcx.crate_name(LOCAL_CRATE) == *base || "crate" == base.as_str() {
        Some(LOCAL_CRATE.as_def_id())
    } else {
        None
    };

    let crates = find_primitive_impls(tcx, *base)
        .chain(local_crate)
        .map(|id| Res::Def(tcx.def_kind(id), id))
        .chain(find_crates(tcx, *base))
        .collect();

    // trace!(?crates);

    let resolved = def_path_res_with_base(tcx, crates, path, kind);

    if resolved.is_empty() {
        // FIXME: consider a more elegant way to show errors.
        debug!("No items found for path {:?} with kind {:?}", path, kind);
    }

    resolved
}

/// Resolves a def path like `vec::Vec` with the base `std`.
///
/// This is lighter than [`def_path_res`], and should be called with [`find_crates`] looking up
/// items from the same crate repeatedly, although should still be used sparingly.
// #[instrument(level = "trace", skip(tcx), ret)]
pub(crate) fn def_path_res_with_base(
    tcx: TyCtxt<'_>,
    mut base: Vec<Res>,
    mut path: &[Symbol],
    kind: PatItemKind,
) -> Vec<Res> {
    while let [segment, rest @ ..] = path {
        path = rest;
        // let segment = Symbol::intern(segment);
        let segment = *segment;

        base = base
            .into_iter()
            .filter_map(|res| res.opt_def_id())
            .flat_map(|def_id| {
                let mut children = Vec::new();

                // Some items that may be contained in an `impl`.
                if matches!(
                    kind,
                    PatItemKind::Const | PatItemKind::Fn | PatItemKind::Type | PatItemKind::Variant
                ) {
                    // When the current def_id is e.g. `struct S`, check the impl items in
                    // `impl S { ... }`
                    children.extend(
                        tcx.inherent_impls(def_id)
                            .iter()
                            .flat_map(|&impl_def_id| item_children_by_name(tcx, impl_def_id, segment)),
                    );
                }

                children.extend(item_children_by_name(tcx, def_id, segment));

                children
            })
            .collect();

        // trace!(?segment, ?rest, ?base);
    }

    // trace!(?base);

    base.into_iter().filter(|res| kind.match_resolve(res)).collect()
}

// #[instrument(level = "trace", skip(tcx), ret)]
fn non_local_item_children_by_name(tcx: TyCtxt<'_>, def_id: DefId, name: Symbol) -> Vec<Res> {
    match tcx.def_kind(def_id) {
        DefKind::Mod | DefKind::Enum | DefKind::Trait => tcx
            .module_children(def_id)
            .iter()
            .filter(|item| item.ident.name == name)
            .map(|child| child.res.expect_non_local())
            .collect(),
        DefKind::Impl { .. } => tcx
            .associated_item_def_ids(def_id)
            .iter()
            .copied()
            .filter(|assoc_def_id| tcx.item_name(*assoc_def_id) == name)
            .map(|assoc_def_id| Res::Def(tcx.def_kind(assoc_def_id), assoc_def_id))
            .collect(),
        _ => Vec::new(),
    }
}

// #[instrument(level = "trace", skip(tcx), ret)]
fn local_item_children_by_name(tcx: TyCtxt<'_>, local_id: LocalDefId, name: Symbol) -> Vec<Res> {
    let hir = tcx.hir();

    let root_mod;
    let item_kind = match tcx.hir_node_by_def_id(local_id) {
        Node::Crate(r#mod) => {
            root_mod = ItemKind::Mod(r#mod);
            &root_mod
        },
        Node::Item(item) => &item.kind,
        _ => return Vec::new(),
    };

    // trace!(?item_kind);

    let res = |ident: Ident, owner_id: OwnerId| {
        // trace!(?ident, ?name, ?owner_id);
        if ident.name == name {
            let def_id = owner_id.to_def_id();
            Some(Res::Def(tcx.def_kind(def_id), def_id))
        } else {
            None
        }
    };

    match item_kind {
        ItemKind::Mod(r#mod) => r#mod
            .item_ids
            .iter()
            .filter_map(|&item_id| {
                let item = hir.item(item_id);
                match item.kind {
                    ItemKind::ForeignMod { abi: _, items } => {
                        items.iter().find_map(|item| res(item.ident, item.id.owner_id))
                    },
                    _ => res(item.ident, item_id.owner_id),
                }
            })
            .collect(),
        ItemKind::Impl(r#impl) => r#impl
            .items
            .iter()
            .filter_map(|&ImplItemRef { ident, id, .. }| res(ident, id.owner_id))
            .collect(),
        ItemKind::Trait(.., trait_item_refs) => trait_item_refs
            .iter()
            .filter_map(|&TraitItemRef { ident, id, .. }| res(ident, id.owner_id))
            .collect(),
        _ => Vec::new(),
    }
}

// #[instrument(level = "trace", skip(tcx), ret)]
fn item_children_by_name(tcx: TyCtxt<'_>, def_id: DefId, name: Symbol) -> Vec<Res> {
    if let Some(local_id) = def_id.as_local() {
        local_item_children_by_name(tcx, local_id, name)
    } else {
        non_local_item_children_by_name(tcx, def_id, name)
    }
}

/// Finds the crates called `name`, may be multiple due to multiple major versions.
pub(crate) fn find_crates(tcx: TyCtxt<'_>, name: Symbol) -> Vec<Res> {
    tcx.crates(())
        .iter()
        .copied()
        .filter(move |&num| tcx.crate_name(num) == name)
        .filter(move |&num| {
            // Find crates that are
            // either has been included as a part of prelude
            // or directly depended by local crate
            matches!(name.as_str(), "std" | "core" | "alloc")
                || tcx.extern_crate(num).map(|krate| krate.is_direct()).unwrap_or(false)
        })
        .map(CrateNum::as_def_id)
        .map(|id| Res::Def(tcx.def_kind(id), id))
        .collect()
}

fn find_primitive_impls(tcx: TyCtxt<'_>, name: Symbol) -> impl Iterator<Item = DefId> + '_ {
    let ty = match name.as_str() {
        "bool" => SimplifiedType::Bool,
        "char" => SimplifiedType::Char,
        "str" => SimplifiedType::Str,
        "array" => SimplifiedType::Array,
        "slice" => SimplifiedType::Slice,
        // FIXME: rustdoc documents these two using just `pointer`.
        //
        // Maybe this is something we should do here too.
        "const_ptr" => SimplifiedType::Ptr(Mutability::Not),
        "mut_ptr" => SimplifiedType::Ptr(Mutability::Mut),
        "isize" => SimplifiedType::Int(IntTy::Isize),
        "i8" => SimplifiedType::Int(IntTy::I8),
        "i16" => SimplifiedType::Int(IntTy::I16),
        "i32" => SimplifiedType::Int(IntTy::I32),
        "i64" => SimplifiedType::Int(IntTy::I64),
        "i128" => SimplifiedType::Int(IntTy::I128),
        "usize" => SimplifiedType::Uint(UintTy::Usize),
        "u8" => SimplifiedType::Uint(UintTy::U8),
        "u16" => SimplifiedType::Uint(UintTy::U16),
        "u32" => SimplifiedType::Uint(UintTy::U32),
        "u64" => SimplifiedType::Uint(UintTy::U64),
        "u128" => SimplifiedType::Uint(UintTy::U128),
        "f32" => SimplifiedType::Float(FloatTy::F32),
        "f64" => SimplifiedType::Float(FloatTy::F64),
        _ => {
            return [].iter().copied();
        },
    };

    tcx.incoherent_impls(ty).iter().copied()
}
