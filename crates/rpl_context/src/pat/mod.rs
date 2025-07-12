use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

pub use error::DynamicError;
use error::DynamicErrorBuilder;
use rpl_constraints::Constraints;
use rpl_meta::collect_elems_separated_by_comma;
use rpl_meta::symbol_table::WithPath;
use rpl_meta::utils::Ident;
use rpl_parser::generics::{Choice2, Choice3, Choice4};
use rpl_parser::pairs;
use rustc_data_structures::fx::{FxHashMap, FxIndexMap};
use rustc_hir::FnDecl;
use rustc_middle::mir::Body;
use rustc_span::Symbol;
use rustc_span::source_map::SourceMap;

use crate::PatCtxt;
use crate::pat::table::ColumnType;

mod attr;
mod error;
mod item;
mod matched;
mod mir;
mod non_local_meta_vars;
mod pretty;
mod table;
mod ty;
mod utils;

pub use attr::PatAttr;
pub use item::*;
pub use matched::{Matched, MatchedMap};
pub use mir::*;
pub use non_local_meta_vars::*;
pub(crate) use table::TableHead;
pub use ty::*;

pub type Label = Symbol;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Spanned {
    Location(mir::Location),
    Local(mir::Local),
    Body,
    Output,
}

pub type LabelMap = FxHashMap<Label, Spanned>;

#[derive(Debug, Clone, Copy)]
pub enum PattOrUtil {
    Patt,
    Util,
}

pub enum PatternItem<'pcx> {
    RustItems(RustItems<'pcx>),
    RPLPatternOperation(PatternOperation<'pcx>),
}

impl<'pcx> PatternItem<'pcx> {
    pub fn meta(&self) -> &NonLocalMetaVars<'_> {
        match self {
            PatternItem::RustItems(items) => &items.meta,
            PatternItem::RPLPatternOperation(op) => &op.meta,
        }
    }
    pub(crate) fn diag_name(&self) -> Option<Symbol> {
        match self {
            PatternItem::RustItems(items) => items.attr.diag,
            PatternItem::RPLPatternOperation(op) => op.attr.diag,
        }
    }
    pub(crate) fn consts(&self) -> &FxHashMap<Symbol, &'pcx str> {
        match self {
            PatternItem::RustItems(items) => &items.attr.consts,
            PatternItem::RPLPatternOperation(op) => &op.attr.consts,
        }
    }
    pub(crate) fn table_head(&self) -> TableHead {
        match self {
            PatternItem::RustItems(items) => items.table_head(),
            PatternItem::RPLPatternOperation(op) => op.table_head(),
        }
    }
    pub fn expect_rust_items(&self) -> &RustItems<'pcx> {
        match self {
            PatternItem::RustItems(items) => items,
            PatternItem::RPLPatternOperation(_) => panic!("Expected RustItems, found PatternOperation"),
        }
    }
}

pub struct RustItems<'pcx> {
    pub pcx: PatCtxt<'pcx>,
    pub meta: Arc<NonLocalMetaVars<'pcx>>,
    pub adts: FxHashMap<Symbol, Adt<'pcx>>,
    pub fns: FnPatterns<'pcx>,
    pub impls: FxHashMap<Symbol, Impl<'pcx>>,
    pub attr: PatAttr<'pcx>,
}

impl<'pcx> RustItems<'pcx> {
    pub(crate) fn new(pcx: PatCtxt<'pcx>, meta: Arc<NonLocalMetaVars<'pcx>>, attr: PatAttr<'pcx>) -> Self {
        Self {
            pcx,
            meta,
            adts: Default::default(),
            fns: Default::default(),
            impls: Default::default(),
            attr,
        }
    }

    fn add_item(
        &mut self,
        pat_name: Option<Symbol>,
        item: WithPath<'pcx, &'pcx pairs::RustItemWithConstraint<'pcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'_>,
    ) {
        let path = item.path;
        let (attr, item, where_block) = item.get_matched();
        let constraints = Constraints::from_where_block_opt(attr.iter_matched(), where_block, path)
            .unwrap_or_else(|err| panic!("unexpected error in constraints:\n{err}"));
        match item.deref() {
            Choice4::_0(rust_fn) => {
                let fn_name = Symbol::intern(rust_fn.FnSig().FnName().span.as_str());
                let fn_symbol_table = symbol_table.get_fn(fn_name).unwrap();
                self.add_fn(WithPath::new(path, rust_fn), meta, fn_symbol_table, constraints);
            },
            Choice4::_1(rust_struct) => {
                self.add_struct(pat_name, with_path(path, rust_struct), meta, symbol_table, constraints)
            },
            Choice4::_2(rust_enum) => {
                self.add_enum(pat_name, with_path(path, rust_enum), meta, symbol_table, constraints)
            },
            Choice4::_3(rust_impl) => {
                self.add_impl(pat_name, with_path(path, rust_impl), meta, symbol_table, constraints)
            },
        }
    }

    #[instrument(level = "debug", skip(self, rust_fn, meta, fn_symbol_table))]
    fn add_fn(
        &mut self,
        rust_fn: WithPath<'pcx, &'pcx pairs::Fn<'pcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        fn_symbol_table: &'pcx FnSymbolTable<'pcx>,
        constraints: Constraints,
    ) {
        let fn_pat = FnPattern::from(rust_fn, self.pcx, fn_symbol_table, meta, constraints);
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

    #[instrument(level = "debug", skip(self, rust_struct, symbol_table))]
    fn add_struct(
        &mut self,
        pat_name: Option<Symbol>,
        rust_struct: WithPath<'pcx, &'pcx pairs::Struct<'pcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'pcx>,
        constraints: Constraints,
    ) {
        let mut struct_inner = StructInner::default();
        let name = rust_struct.MetaVariable();
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

        let struct_pat = Adt::new_struct(struct_inner, meta, constraints);
        // let struct_pat = self.pcx.alloc_struct(struct_pat);
        self.adts.insert(Symbol::intern(name.span.as_str()), struct_pat);
    }

    #[instrument(level = "debug", skip(self, rust_enum, symbol_table))]
    fn add_enum(
        &mut self,
        pat_name: Option<Symbol>,
        rust_enum: WithPath<'pcx, &'pcx pairs::Enum<'pcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'pcx>,
        constraints: Constraints,
    ) {
        let mut enum_inner = EnumInner::default();
        let name = rust_enum.MetaVariable();

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

        let enum_pat = Adt::new_enum(enum_inner, meta, constraints);
        // let struct_pat = self.pcx.alloc_struct(struct_pat);
        self.adts.insert(Symbol::intern(name.span.as_str()), enum_pat);
    }

    #[instrument(level = "debug", skip(self, rust_impl, meta, symbol_table))]
    fn add_impl(
        &mut self,
        pat_name: Option<Symbol>,
        rust_impl: WithPath<'pcx, &'pcx pairs::Impl<'pcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'pcx>,
        constraints: Constraints,
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
                let (rust_fn, where_block) = rust_fn.get_matched();
                // FIXME: attributes on associated functions are not supported yet
                let constraints = Constraints::from_where_block_opt(std::iter::empty(), where_block, p)
                    .expect("unexpected error in constraints");
                let fn_name = Symbol::intern(rust_fn.FnSig().FnName().span.as_str());
                let fn_sym_tab = impl_sym_tab.inner.get_fn(fn_name).unwrap();
                let fn_def = FnPattern::from(
                    WithPath::new(p, rust_fn),
                    self.pcx,
                    fn_sym_tab,
                    Arc::clone(&meta),
                    constraints,
                );
                (fn_name, fn_def)
            })
            .collect();
        let impl_pat = Impl {
            meta,
            ty,
            trait_id,
            fns,
            constraints,
        };
        debug!(ty = ?impl_pat.ty, trait_id = ?impl_pat.trait_id, fns = ?impl_pat.fns.keys());
        if let Some(pat_name) = pat_name {
            self.impls.insert(pat_name, impl_pat);
        }
    }

    #[instrument(level = "trace", skip(self), fields(adts = ?self.adts.keys()), ret)]
    pub fn get_adt(&self, adt: Symbol) -> Option<&Adt<'pcx>> {
        self.adts.get(&adt)
    }

    fn table_head(&self) -> TableHead {
        let mut columns = FxHashMap::default();

        self.meta.table_head(&mut columns);

        for name in self.adts.keys() {
            columns.try_insert(*name, ColumnType::Ty).unwrap();
        }

        // FIX: should self.attr be included in the table head?

        for pat in &self.fns {
            if pat.name.as_str().starts_with("$") {
                columns.try_insert(pat.name, ColumnType::Ty).unwrap();
            }
            for label in pat.expect_body().labels.keys() {
                columns.try_insert(*label, ColumnType::Label).unwrap();
            }
        }

        columns
    }

    pub fn post_process<M: Eq + Hash + Debug>(&self, iter: impl Iterator<Item = M>) -> impl Iterator<Item = M> {
        self.attr.post_process(iter)
    }
}

/// `positive` is a list of positive pattern items, `negative` is a list of negative pattern items,
/// they are joined together to form a pattern operation.
///
/// `(positive_1 | positive_2 | ... | positive_n) & !(negative_1 | negative_2 | ... | negative_m)`
pub struct PatternOperation<'pcx> {
    pub pcx: PatCtxt<'pcx>,
    pub meta: Arc<NonLocalMetaVars<'pcx>>,
    pub positive: Vec<(Symbol, &'pcx PatternItem<'pcx>, MatchedMap)>,
    pub negative: Vec<(Symbol, &'pcx PatternItem<'pcx>, MatchedMap)>,
    pub attr: PatAttr<'pcx>,
}

impl PatternOperation<'_> {
    fn table_head(&self) -> TableHead {
        let head = self.positive.first().unwrap().1.table_head();
        debug_assert!(
            self.positive.iter().all(|(_, item, _)| item.table_head() == head),
            "All positive pattern items should have the same table head"
        );
        debug_assert!(
            self.negative.iter().all(|(_, item, _)| item.table_head() == head),
            "All negative pattern items should have the same table head as the positive one"
        );
        head
    }

    pub fn post_process<M: Eq + Hash + Debug>(&self, iter: impl Iterator<Item = M>) -> impl Iterator<Item = M> {
        self.attr.post_process(iter)
    }
}

/// Corresponds to a pattern file in RPL, not a pattern item.
pub struct Pattern<'pcx> {
    pub pcx: PatCtxt<'pcx>,
    pub patt_block: FxIndexMap<Symbol, PatternItem<'pcx>>, // indexed by pat_name
    pub util_block: FxIndexMap<Symbol, &'pcx PatternItem<'pcx>>, // indexed by pat_name
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
        source_map: &SourceMap,
        fn_name: Option<Symbol>,
        body: &Body<'tcx>,
        decl: &FnDecl<'tcx>,
        matched: &impl Matched<'tcx>,
    ) -> Result<Box<DynamicError>, Box<DynamicError>> {
        Ok(Box::new(
            self.diag_block
                .get(&pat_name)
                .ok_or_else(|| Box::new(DynamicError::default_diagnostic(pat_name, body.span)))?
                .build(source_map, fn_name, body, decl, matched),
        ))
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
        let (attr, name, meta_decls, _, item_or_patt_op) = pat_item.get_matched();
        let name = Symbol::intern(name.span.as_str());
        let symbol_table = symbol_tables.get(&name).unwrap();
        let meta = Arc::new(NonLocalMetaVars::from_meta_decls(
            meta_decls.as_ref().map(|meta_decls| with_path(p, meta_decls)),
            self.pcx,
            symbol_table,
        ));
        self.add_item_or_patt_op(
            name,
            attr.iter_matched(),
            with_path(p, item_or_patt_op),
            symbol_table,
            meta,
            block_type,
        );
    }

    #[instrument(level = "debug", skip(self, attr, item_or_patt_op, symbol_table, meta), fields(patt_block = ?self.patt_block.keys(), util_block = ?self.util_block.keys()))]
    fn add_item_or_patt_op(
        &mut self,
        pat_name: Symbol,
        attr: impl Iterator<Item = &'pcx pairs::Attr<'pcx>>,
        item_or_patt_op: WithPath<'pcx, &'pcx pairs::RustItemsOrPatternOperation<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'_>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        block_type: PattOrUtil,
    ) {
        let p = item_or_patt_op.path;
        match &***item_or_patt_op {
            Choice3::_0(item) => {
                self.add_items(
                    pat_name,
                    attr,
                    with_path(p, std::iter::once(item)),
                    symbol_table,
                    meta,
                    block_type,
                );
            },
            Choice3::_1(items) => {
                self.add_items(
                    pat_name,
                    attr,
                    with_path(p, items.get_matched().1.iter_matched()),
                    symbol_table,
                    meta,
                    block_type,
                );
            },
            Choice3::_2(patt_op) => {
                self.add_patt_op(pat_name, attr, with_path(p, patt_op), meta, block_type);
            },
        }
    }

    fn patt_op(
        &self,
        meta: &NonLocalMetaVars<'pcx>,
        pat_cfg: &'pcx pairs::PatternConfiguration<'pcx>,
    ) -> (Symbol, &'pcx PatternItem<'pcx>, MatchedMap) {
        let name = Ident::from(pat_cfg.Identifier()).name;
        let item = *self.util_block.get(&name).unwrap();
        let map = MatchedMap::new(
            meta,
            item.meta(),
            pat_cfg
                .MetaVariableAssignList()
                .and_then(|list| list.MetaVariableAssignsSeparatedByComma()),
        );
        (name, item, map)
    }

    #[instrument(level = "debug", skip(self, attr, patt_op, meta), fields(patt_block = ?self.patt_block.keys(), util_block = ?self.util_block.keys()))]
    fn add_patt_op(
        &mut self,
        pat_name: Symbol,
        attr: impl Iterator<Item = &'pcx pairs::Attr<'pcx>>,
        patt_op: WithPath<'pcx, &'pcx pairs::PatternOperation<'pcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        block_type: PattOrUtil,
    ) {
        let patt_op = patt_op.PatternExpression();
        let (pos, pos_, neg) = patt_op.get_matched();
        let positive = std::iter::once(pos)
            .chain(pos_.iter_matched().map(|pos_| pos_.get_matched().1))
            .map(|pos| self.patt_op(&meta, pos))
            .collect();
        let negative = neg
            .iter_matched()
            .map(|negative| self.patt_op(&meta, negative.get_matched().1))
            .collect();
        let attr = PatAttr::parse_all(attr);
        let pat_ops = PatternOperation {
            pcx: self.pcx,
            meta,
            positive,
            negative,
            attr,
        };
        match block_type {
            PattOrUtil::Patt => self
                .patt_block
                .entry(pat_name)
                .or_insert(PatternItem::RPLPatternOperation(pat_ops)),
            PattOrUtil::Util => *self
                .util_block
                .entry(pat_name)
                .or_insert_with(|| self.pcx.alloc_pattern_item(PatternItem::RPLPatternOperation(pat_ops))),
        }
        .table_head();
    }

    #[instrument(level = "debug", skip(self, attr, items, symbol_table, meta))]
    fn add_items(
        &mut self,
        pat_name: Symbol,
        attr: impl Iterator<Item = &'pcx pairs::Attr<'pcx>>,
        items: WithPath<'pcx, impl Iterator<Item = &'pcx pairs::RustItemWithConstraint<'pcx>>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'_>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        block_type: PattOrUtil,
    ) {
        let p = items.path;
        match block_type {
            PattOrUtil::Patt => {
                self.patt_block.entry(pat_name).or_insert_with(|| {
                    let attr = PatAttr::parse_all(attr);
                    let mut rpl_rust_items = RustItems::new(self.pcx, meta.clone(), attr);
                    for item in items.inner {
                        rpl_rust_items.add_item(Some(pat_name), with_path(p, item), meta.clone(), symbol_table);
                    }
                    PatternItem::RustItems(rpl_rust_items)
                });
            },
            PattOrUtil::Util => {
                self.util_block.entry(pat_name).or_insert_with(|| {
                    let attr = PatAttr::parse_all(attr);
                    let mut rpl_rust_items = RustItems::new(self.pcx, meta.clone(), attr);
                    for item in items.inner {
                        rpl_rust_items.add_item(Some(pat_name), with_path(p, item), meta.clone(), symbol_table);
                    }
                    self.pcx.alloc_pattern_item(PatternItem::RustItems(rpl_rust_items))
                });
            },
        };
    }

    pub fn add_diag(
        &mut self,
        diag: WithPath<'pcx, &'pcx pairs::diagBlock<'_>>,
        diag_symbol_tables: &'pcx rpl_meta::symbol_table::DiagSymbolTables<'_>,
        symbol_tables: &'pcx FxHashMap<Symbol, rpl_meta::symbol_table::SymbolTable<'_>>,
    ) {
        let mut items = FxHashMap::default();
        for item in diag.get_matched().2.iter_matched() {
            let (ident, _, _, _, _, _) = item.get_matched();
            let name = Symbol::intern(ident.span.as_str());
            let prev = items.insert(name, item);
            debug_assert!(prev.is_none(), "Duplicate diagnostic for {:?}", name); //FIXME: raise an error
        }

        for (name, pat_item) in &self.patt_block {
            let symbol_table = symbol_tables.get(name).unwrap();

            let diag_name = pat_item.diag_name().unwrap_or(*name);
            if let Some(diag_item) = items.get(&diag_name) {
                let labels = symbol_table.labels();
                let diag = DynamicErrorBuilder::from_item(
                    WithPath::new(diag.path, diag_item),
                    &symbol_table.meta_vars,
                    pat_item.consts(),
                    &labels.collect(),
                    diag_symbol_tables
                        .get(&diag_name)
                        .unwrap_or_else(|| panic!("No diagnostic symbol table found for {diag_name}")),
                )
                .unwrap_or_else(|err| panic!("{err}"));
                let prev = self.diag_block.insert(*name, diag);
                debug_assert!(prev.is_none(), "Duplicate diagnostic for {:?}", name); //FIXME: raise an error
            } else {
                warn!("No diagnostic found for pattern item {:?} ({:?})", name, diag_name);
            }
        }
    }
}
