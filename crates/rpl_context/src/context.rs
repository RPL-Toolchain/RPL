use std::num::NonZero;
use std::ops::Deref;

use rpl_meta::idx::RPLIdx;
use rpl_meta::meta::collect_blocks;
use rpl_parser::pairs;
use rustc_data_structures::fx::FxHashMap;
use rustc_data_structures::sync::{Lock, Registry, WorkerLocal};
use rustc_hir as hir;
use rustc_middle::{mir, ty};
use rustc_span::Symbol;

use crate::pat::{self, Ty, TyKind, with_path};

pub struct PrimitiveTypes<'pcx> {
    pub u8: Ty<'pcx>,
    pub u16: Ty<'pcx>,
    pub u32: Ty<'pcx>,
    pub u64: Ty<'pcx>,
    pub u128: Ty<'pcx>,
    pub usize: Ty<'pcx>,
    pub i8: Ty<'pcx>,
    pub i16: Ty<'pcx>,
    pub i32: Ty<'pcx>,
    pub i64: Ty<'pcx>,
    pub i128: Ty<'pcx>,
    pub isize: Ty<'pcx>,
    pub bool: Ty<'pcx>,
    pub str: Ty<'pcx>,
}

impl<'pcx> PrimitiveTypes<'pcx> {
    fn new(arena: &'pcx crate::Arena<'pcx>) -> Self {
        Self {
            u8: Ty(arena.alloc(TyKind::Uint(ty::UintTy::U8))),
            u16: Ty(arena.alloc(TyKind::Uint(ty::UintTy::U16))),
            u32: Ty(arena.alloc(TyKind::Uint(ty::UintTy::U32))),
            u64: Ty(arena.alloc(TyKind::Uint(ty::UintTy::U64))),
            u128: Ty(arena.alloc(TyKind::Uint(ty::UintTy::U128))),
            usize: Ty(arena.alloc(TyKind::Uint(ty::UintTy::Usize))),
            i8: Ty(arena.alloc(TyKind::Int(ty::IntTy::I8))),
            i16: Ty(arena.alloc(TyKind::Int(ty::IntTy::I16))),
            i32: Ty(arena.alloc(TyKind::Int(ty::IntTy::I32))),
            i64: Ty(arena.alloc(TyKind::Int(ty::IntTy::I64))),
            i128: Ty(arena.alloc(TyKind::Int(ty::IntTy::I128))),
            isize: Ty(arena.alloc(TyKind::Int(ty::IntTy::Isize))),
            bool: Ty(arena.alloc(TyKind::Bool)),
            str: Ty(arena.alloc(TyKind::Str)),
        }
    }
}

#[derive(Clone, Copy)]
#[rustc_pass_by_value]
/// `PatCtxt` is a similar to `rustc_middle::ty::context`.
/// The central data structure of the rpl toolchain
/// A wrapper type for `PatternCtxt`, which is the structure that actually holds the data
/// `PatCtxt` Deref to `PatternCtxt`, and in practice, `PatCtxt` is passed around everywhere.
pub struct PatCtxt<'pcx> {
    pcx: &'pcx PatternCtxt<'pcx>,
}

impl<'pcx> Deref for PatCtxt<'pcx> {
    type Target = PatternCtxt<'pcx>;

    fn deref(&self) -> &Self::Target {
        self.pcx
    }
}

pub struct PatternCtxt<'pcx> {
    arena: &'pcx WorkerLocal<crate::Arena<'pcx>>,
    rpl_patterns: Lock<FxHashMap<RPLIdx, &'pcx pat::Pattern<'pcx>>>,
    pub primitive_types: PrimitiveTypes<'pcx>,
}

impl PatternCtxt<'_> {
    pub fn entered<T>(f: impl FnOnce(PatCtxt<'_>) -> T) -> T {
        let arena = &WorkerLocal::<crate::Arena<'_>>::default();
        let pcx = &PatternCtxt {
            arena,
            rpl_patterns: Default::default(),
            primitive_types: PrimitiveTypes::new(arena),
        };
        f(PatCtxt { pcx })
    }
    // only for unit tests
    pub fn entered_no_tcx<T>(f: impl FnOnce(PatCtxt<'_>) -> T) -> T {
        Registry::new(NonZero::new(1).unwrap()).register();
        rustc_span::create_session_if_not_set_then(rustc_span::edition::LATEST_STABLE_EDITION, |_| Self::entered(f))
    }
}

impl<'pcx> PatCtxt<'pcx> {
    /// Maps strings to their interned representation
    pub fn mk_symbols(self, syms: &[&str]) -> &'pcx [Symbol] {
        self.arena.alloc_from_iter(syms.iter().copied().map(Symbol::intern))
    }
    pub fn mk_slice<T: Copy>(self, slice: &[T]) -> &'pcx [T] {
        if slice.is_empty() {
            return &[];
        }
        self.arena.alloc_slice(slice)
    }
    // fn mk_generic_args(self, generics: &[pat::GenericArgKind<'pcx>]) -> pat::GenericArgsRef<'pcx> {
    //     pat::GenericArgsRef(self.mk_slice(generics))
    // }
    // pub fn mk_type_relative(self, ty: Ty<'pcx>, path: &str) -> pat::Path<'pcx> {
    //     pat::Path::TypeRelative(ty, Symbol::intern(path))
    // }
    pub fn mk_lang_item(self, item: &str) -> pat::Path<'pcx> {
        hir::LangItem::from_name(Symbol::intern(item))
            .unwrap_or_else(|| panic!("unknown language item \"{item}\""))
            .into()
    }
    // pub fn mk_item_path(self, path: &[&str]) -> pat::ItemPath<'pcx> {
    //     pat::ItemPath(self.mk_symbols(path))
    // }
    // pub fn mk_path_with_args(
    //     self,
    //     path: impl Into<pat::Path<'pcx>>,
    //     generics: &[pat::GenericArgKind<'pcx>],
    // ) -> pat::PathWithArgs<'pcx> {
    //     let path = path.into();
    //     let args = self.mk_generic_args(generics);
    //     pat::PathWithArgs { path, args }
    // }
    pub fn mk_path_ty(self, path_with_args: pat::PathWithArgs<'pcx>) -> Ty<'pcx> {
        self.mk_ty(TyKind::Path(path_with_args))
    }
    pub fn mk_adt_ty(self, path_with_args: pat::PathWithArgs<'pcx>) -> Ty<'pcx> {
        self.mk_path_ty(path_with_args)
    }
    pub fn mk_adt_pat_ty(self, pat: Symbol) -> Ty<'pcx> {
        self.mk_ty(TyKind::AdtPat(pat))
    }
    pub fn mk_array_ty(self, ty: Ty<'pcx>, len: pat::Const<'pcx>) -> Ty<'pcx> {
        self.mk_ty(TyKind::Array(ty, len))
    }
    pub fn mk_slice_ty(self, ty: Ty<'pcx>) -> Ty<'pcx> {
        self.mk_ty(TyKind::Slice(ty))
    }
    pub fn mk_unit_ty(self) -> Ty<'pcx> {
        self.mk_ty(TyKind::Tuple(&[]))
    }
    pub fn mk_tuple_ty(self, ty: &[Ty<'pcx>]) -> Ty<'pcx> {
        self.mk_ty(TyKind::Tuple(self.mk_slice(ty)))
    }
    pub fn mk_ref_ty(self, region: pat::RegionKind, ty: Ty<'pcx>, mutability: mir::Mutability) -> Ty<'pcx> {
        self.mk_ty(TyKind::Ref(region, ty, mutability))
    }
    pub fn mk_raw_ptr_ty(self, ty: Ty<'pcx>, mutability: mir::Mutability) -> Ty<'pcx> {
        self.mk_ty(TyKind::RawPtr(ty, mutability))
    }
    pub fn mk_fn(self, path_with_args: pat::PathWithArgs<'pcx>) -> Ty<'pcx> {
        self.mk_path_ty(path_with_args)
    }
    pub fn mk_var_ty(self, ty_var: pat::TyVar) -> Ty<'pcx> {
        self.mk_ty(TyKind::TyVar(ty_var))
    }
    pub fn mk_any_ty(self) -> Ty<'pcx> {
        self.mk_ty(TyKind::Any)
    }
    pub(crate) fn mk_ty(self, kind: TyKind<'pcx>) -> Ty<'pcx> {
        Ty(self.arena.alloc(kind))
    }
}

impl<'pcx> PatCtxt<'pcx> {
    pub fn new_pattern(self) -> &'pcx mut pat::Pattern<'pcx> {
        self.arena.alloc(pat::Pattern::new(self))
    }
    pub fn alloc_pattern_item(self, item: pat::PatternItem<'pcx>) -> &'pcx mut pat::PatternItem<'pcx> {
        self.arena.alloc(item)
    }
    pub fn mk_mir_pattern(self, pattern: pat::FnPatternBody<'pcx>) -> &'pcx pat::FnPatternBody<'pcx> {
        self.arena.alloc(pattern)
    }
    pub fn alloc_fn(self, pat: pat::FnPattern<'pcx>) -> &'pcx mut pat::FnPattern<'pcx> {
        self.arena.alloc(pat)
    }
    pub fn alloc_struct(self, pat: pat::Adt<'pcx>) -> &'pcx mut pat::Adt<'pcx> {
        self.arena.alloc(pat)
    }
    pub fn add_parsed_patterns<'mcx: 'pcx>(self, mctx: &'mcx rpl_meta::context::MetaContext<'mcx>) {
        for (id, syntax_tree) in mctx.syntax_trees.iter() {
            self.add_parsed_pattern(*id, syntax_tree, mctx);
        }
    }
    pub fn for_each_rpl_pattern(self, mut f: impl FnMut(RPLIdx, &'pcx pat::Pattern<'pcx>)) {
        for (&id, pattern) in self.rpl_patterns.lock().iter() {
            f(id, pattern);
        }
    }
    pub fn add_parsed_pattern<'mcx: 'pcx>(
        self,
        id: RPLIdx,
        main: &'pcx pairs::main<'pcx>,
        mctx: &'mcx rpl_meta::context::MetaContext<'mcx>,
    ) {
        let pattern = self.new_pattern();
        // FIXME: process utils
        let (utils, patts, diags) = collect_blocks(main);

        {
            let patt_items = utils.iter().flat_map(|patt| patt.get_matched().3.iter_matched());
            let patt_symbol_tables = &mctx.symbol_tables.get(&id).unwrap().util_symbol_tables;
            patt_items.for_each(|item| {
                pattern.add_pattern_item(
                    with_path(mctx.get_active_path(), item),
                    patt_symbol_tables,
                    pat::PattOrUtil::Util,
                );
            });
        }
        {
            let patt_items = patts.iter().flat_map(|patt| patt.get_matched().3.iter_matched());
            let patt_symbol_tables = &mctx.symbol_tables.get(&id).unwrap().patt_symbol_tables;
            patt_items.for_each(|item| {
                pattern.add_pattern_item(
                    with_path(mctx.get_active_path(), item),
                    patt_symbol_tables,
                    pat::PattOrUtil::Patt,
                );
            });

            for diag in diags {
                pattern.add_diag(with_path(mctx.get_active_path(), diag), patt_symbol_tables)
            }
        }

        self.rpl_patterns.lock().insert(id, pattern);
    }
}
