use error::{DynamicError, DynamicErrorBuilder};
use rpl_meta::collect_elems_separated_by_comma;
use rpl_meta::symbol_table::WithPath;
use rpl_meta::utils::Ident;
use rpl_parser::generics::{Choice2, Choice3, Choice4};
use rpl_parser::pairs;
use rustc_data_structures::fx::FxHashMap;
use rustc_middle::mir::Body;
use rustc_span::Symbol;
use std::ops::Deref;
use std::sync::Arc;

use crate::PatCtxt;

mod error;
mod item;
mod matched;
mod mir;
mod non_local_meta_vars;
mod pretty;
mod ty;
mod utils;

pub use item::*;
pub use matched::Matched;
pub use mir::*;
pub use non_local_meta_vars::*;
pub use ty::*;

pub type Label = Symbol;
pub type LabelMap = FxHashMap<Label, mir::Location>;

#[derive(Debug, Clone, Copy)]
pub enum PattOrUtil {
    Patt,
    Util,
}

pub enum PatternItem<'pcx> {
    RustItems(RustItems<'pcx>),
    RPLPatternOperation,
}

pub struct RustItems<'pcx> {
    pub pcx: PatCtxt<'pcx>,
    pub adts: FxHashMap<Symbol, Adt<'pcx>>,
    pub fns: FnPatterns<'pcx>,
    pub impls: FxHashMap<Symbol, Impl<'pcx>>,
}

impl<'pcx> RustItems<'pcx> {
    pub(crate) fn new(pcx: PatCtxt<'pcx>) -> Self {
        Self {
            pcx,
            adts: Default::default(),
            fns: Default::default(),
            impls: Default::default(),
        }
    }

    fn add_item(
        &mut self,
        pat_name: Option<Symbol>,
        item: WithPath<'pcx, &'pcx pairs::RustItem<'pcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'_>,
    ) {
        match &***item {
            Choice4::_0(rust_fn) => {
                let fn_name = Symbol::intern(rust_fn.FnSig().FnName().span.as_str());
                let fn_symbol_table = symbol_table.get_fn(fn_name).unwrap();
                self.add_fn(WithPath::new(item.path, rust_fn), meta, fn_symbol_table);
            },
            Choice4::_1(rust_struct) => self.add_struct(pat_name, with_path(item.path, rust_struct), symbol_table),
            Choice4::_2(rust_enum) => self.add_enum(pat_name, with_path(item.path, rust_enum), symbol_table),
            Choice4::_3(rust_impl) => self.add_impl(pat_name, with_path(item.path, rust_impl), meta, symbol_table),
        }
    }

    fn add_fn(
        &mut self,
        rust_fn: WithPath<'pcx, &'pcx pairs::Fn<'pcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        fn_symbol_table: &'pcx FnSymbolTable<'pcx>,
    ) {
        let fn_pat = FnPattern::from(rust_fn, self.pcx, fn_symbol_table, meta);
        let fn_pat = self.pcx.alloc_fn(fn_pat);
        let fn_name = fn_pat.name;
        match fn_name.as_str() {
            "_" => {
                // unnamed function, add it to the unnamed_fns
                self.fns.unnamed_fns.push(fn_pat);
            },
            _ => {
                // named function, add it to the named_fns
                self.fns.named_fns.insert(fn_name, fn_pat);
            },
        }
    }

    fn add_struct(
        &mut self,
        pat_name: Option<Symbol>,
        rust_struct: WithPath<'pcx, &'pcx pairs::Struct<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'pcx>,
    ) {
        let mut struct_inner = StructInner::default();
        if let Some(fields) = rust_struct.get_matched().4 {
            let fields = collect_elems_separated_by_comma!(fields);
            for field in fields {
                let (name, _, ty) = field.get_matched();
                let name = Symbol::intern(name.span.as_str());
                let ty = Ty::from(with_path(rust_struct.path, ty), self.pcx, symbol_table);
                let field = Field { ty };
                struct_inner.fields.insert(name, field);
            }
        }

        let struct_pat = Adt::new_struct(struct_inner);
        // let struct_pat = self.pcx.alloc_struct(struct_pat);
        if let Some(pat_name) = pat_name {
            self.adts.insert(pat_name, struct_pat);
        }
    }

    fn add_enum(
        &mut self,
        pat_name: Option<Symbol>,
        rust_enum: WithPath<'pcx, &'pcx pairs::Enum<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'pcx>,
    ) {
        let mut enum_inner = EnumInner::default();

        if let Some(variants) = rust_enum.EnumVariantsSeparatedByComma() {
            let variants = collect_elems_separated_by_comma!(variants);
            for variant in variants {
                let mut enum_variant = Variant::default();
                let identifier = match variant.deref() {
                    Choice3::_0(variant) => {
                        if let Some(fields) = variant.get_matched().2 {
                            let fields = collect_elems_separated_by_comma!(fields);
                            for field in fields {
                                let (name, _, ty) = field.get_matched();
                                let name = Symbol::intern(name.span.as_str());
                                let ty = Ty::from(with_path(rust_enum.path, ty), self.pcx, symbol_table);
                                let field = Field { ty };
                                enum_variant.fields.insert(name, field);
                            }
                        }
                        variant.get_matched().0
                    },
                    Choice3::_1(variant) => {
                        let (name, _, ty, _) = variant.get_matched();
                        let name = Symbol::intern(name.span.as_str());
                        let ty = Ty::from(with_path(rust_enum.path, ty), self.pcx, symbol_table);
                        let field = Field { ty };
                        enum_variant.fields.insert(name, field);
                        variant.get_matched().0
                    },
                    Choice3::_2(unit) => unit,
                };
                let ident = Ident::from(identifier);
                enum_inner.insert(ident.name, enum_variant);
            }
        }

        let struct_pat = Adt::new_enum(enum_inner);
        // let struct_pat = self.pcx.alloc_struct(struct_pat);
        if let Some(pat_name) = pat_name {
            self.adts.insert(pat_name, struct_pat);
        }
    }

    fn add_impl(
        &mut self,
        pat_name: Option<Symbol>,
        rust_impl: WithPath<'pcx, &'pcx pairs::Impl<'pcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'pcx>,
    ) {
        let p = rust_impl.path;
        let (_, impl_kind, ty, _, fns, _) = rust_impl.get_matched();
        let impl_sym_tab = symbol_table.get_impl(ty, impl_kind.as_ref()).unwrap();
        let ty = Ty::from(WithPath::new(p, ty), self.pcx, symbol_table);
        let trait_id = impl_kind
            .as_ref()
            .map(|impl_kind| Path::from_pairs(impl_kind.get_matched().0, self.pcx));
        let fns = fns
            .iter_matched()
            .map(|rust_fn| {
                let fn_name = Symbol::intern(rust_fn.FnSig().FnName().span.as_str());
                let fn_sym_tab = impl_sym_tab.inner.get_fn(fn_name).unwrap();
                let fn_def = FnPattern::from(WithPath::new(p, rust_fn), self.pcx, fn_sym_tab, Arc::clone(&meta));
                (fn_name, fn_def)
            })
            .collect();
        let impl_pat = Impl {
            meta,
            ty,
            trait_id,
            fns,
        };
        if let Some(pat_name) = pat_name {
            self.impls.insert(pat_name, impl_pat);
        }
    }

    pub fn get_adt(&self, adt: Symbol) -> Option<&Adt<'pcx>> {
        self.adts.get(&adt)
    }
}

/// Corresponds to a pattern file in RPL, not a pattern item.
pub struct Pattern<'pcx> {
    pub pcx: PatCtxt<'pcx>,
    pub patt_block: FxHashMap<Symbol, PatternItem<'pcx>>, // indexed by pat_name
    pub util_block: FxHashMap<Symbol, PatternItem<'pcx>>, // indexed by pat_name
    diag_block: FxHashMap<Symbol, DynamicErrorBuilder<'pcx>>,
}

impl<'pcx> Pattern<'pcx> {
    pub(crate) fn new(pcx: PatCtxt<'pcx>) -> Self {
        Self {
            pcx,
            patt_block: Default::default(),
            util_block: Default::default(),
            diag_block: Default::default(),
        }
    }

    pub fn get_diag<'tcx>(
        &self,
        pat_name: Symbol,
        label_map: &LabelMap,
        body: &Body<'tcx>,
        matched: &impl Matched<'tcx>,
    ) -> DynamicError {
        // FIXME: raise an error if the diag related to the pattern is not found
        self.diag_block.get(&pat_name).unwrap().build(label_map, body, matched)
    }
}

impl<'pcx> Pattern<'pcx> {
    pub fn add_pattern_item(
        &mut self,
        pat_item: WithPath<'pcx, &'pcx pairs::RPLPatternItem<'pcx>>,
        symbol_tables: &'pcx FxHashMap<Symbol, rpl_meta::symbol_table::SymbolTable<'_>>,
        block_type: PattOrUtil,
    ) {
        let p = pat_item.path;
        let (name, meta_decls, _, item_or_patt_op, _) = pat_item.get_matched();
        let name = Symbol::intern(name.span.as_str());
        let symbol_table = symbol_tables.get(&name).unwrap();
        let meta = Arc::new(NonLocalMetaVars::from_meta_decls(
            meta_decls.as_ref().map(|meta_decls| with_path(p, meta_decls)),
            self.pcx,
            symbol_table,
        ));
        self.add_item_or_patt_op(name, with_path(p, item_or_patt_op), symbol_table, meta, block_type);
    }

    fn add_item_or_patt_op(
        &mut self,
        pat_name: Symbol,
        item_or_patt_op: WithPath<'pcx, &'pcx pairs::RustItemsOrPatternOperation<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'_>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        block_type: PattOrUtil,
    ) {
        let p = item_or_patt_op.path;
        match &***item_or_patt_op {
            Choice3::_2(_patt_op) => {
                // FIXME: process the patt operation
                todo!()
            },
            _ => {
                let item = item_or_patt_op.RustItem();
                let items = item_or_patt_op.RustItems();
                let items = if let Some(items) = items {
                    items.get_matched().1.iter_matched().collect::<Vec<_>>()
                } else {
                    // unwrap here is safe because the `RustItem` or `RustItems` is not `None`
                    vec![item.unwrap()]
                };
                let rpl_pattern_item = match block_type {
                    PattOrUtil::Patt => self
                        .patt_block
                        .entry(pat_name)
                        .or_insert_with(|| PatternItem::RustItems(RustItems::new(self.pcx))),
                    PattOrUtil::Util => self
                        .util_block
                        .entry(pat_name)
                        .or_insert_with(|| PatternItem::RustItems(RustItems::new(self.pcx))),
                };
                let rpl_rust_items = match rpl_pattern_item {
                    PatternItem::RustItems(rust_items) => rust_items,
                    _ => unreachable!(),
                };
                for item in items {
                    rpl_rust_items.add_item(Some(pat_name), with_path(p, item), meta.clone(), symbol_table);
                }
            },
        }
    }
    pub fn add_diag(
        &mut self,
        diag: &'pcx pairs::diagBlock<'_>,
        symbol_tables: &'pcx FxHashMap<Symbol, rpl_meta::symbol_table::SymbolTable<'_>>,
    ) {
        for item in diag.get_matched().2.iter_matched() {
            let (ident, _, _, _, _, _) = item.get_matched();
            let name = Symbol::intern(ident.span.as_str());
            let symbol_table = symbol_tables.get(&name).unwrap();
            let diag = DynamicErrorBuilder::from_item(item, &symbol_table.meta_vars);
            let prev = self.diag_block.insert(name, diag);
            debug_assert!(prev.is_none(), "Duplicate diagnostic for {:?}", name); //FIXME: raise an error
        }
    }
}
