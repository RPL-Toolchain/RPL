use std::cell::OnceCell;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

pub use error::DynamicError;
use error::DynamicErrorBuilder;
use rpl_constraints::Constraints;
use rpl_constraints::predicates::PredicateConjunction;
use rpl_meta::collect_elems_separated_by_comma;
use rpl_meta::meta::PattSymbolTables;
use rpl_meta::symbol_table::{GetType, MetaVariable, TypeOrPath, WithPath};
use rpl_meta::utils::self_param_ty;
use rpl_parser::generics::{Choice2, Choice3, Choice4};
use rpl_parser::pairs;
use rustc_data_structures::fx::{FxHashMap, FxHashSet, FxIndexMap};
use rustc_hir::FnDecl;
use rustc_middle::mir::{Body, Mutability as MirMutability};
use rustc_span::Symbol;
use rustc_span::source_map::SourceMap;

use crate::PatCtxt;
use crate::pat::table::ColumnType;
use crate::pat::utils::Ident;

mod attr;
mod error;
mod item;
mod matched;
mod mir;
mod non_local_meta_vars;
mod ops;
pub mod ops_resolved;
pub mod ops_uses;
pub mod ops_wf;
mod pretty;
mod table;
mod ty;
mod utils;

pub use attr::PatAttr;
pub use item::*;
pub use matched::{Matched, MatchedMap};
pub use mir::*;
pub use non_local_meta_vars::*;
pub use ops::*;
pub use ops_resolved::ResolvedOpBindings;
pub use ops_uses::{OpsUseError, check_op_refs};
pub use ops_wf::{OpsWfError, check_ops_block, check_r6_patt_vs_ops};
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

    /// Collect label → [`Spanned`] kinds for diagnostic primary validation.
    fn label_map(&self) -> FxHashMap<Symbol, Spanned> {
        match self {
            PatternItem::RustItems(items) => {
                let mut map = FxHashMap::default();
                for fn_pat in &items.fns {
                    if let Some(body) = fn_pat.body {
                        map.extend(body.labels.iter().map(|(&k, &v)| (k, v)));
                    }
                }
                for impl_pat in items.impls.values() {
                    for fn_pat in impl_pat.fns.values() {
                        if let Some(body) = fn_pat.body {
                            map.extend(body.labels.iter().map(|(&k, &v)| (k, v)));
                        }
                    }
                }
                map
            },
            PatternItem::RPLPatternOperation(op) => {
                let mut map = FxHashMap::default();
                for (_, item, matched_map) in &op.positive {
                    for (label, spanned) in item.label_map() {
                        let mapped = *matched_map.labels.get(&label).unwrap_or(&label);
                        map.insert(mapped, spanned);
                    }
                }
                map
            },
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
    /// The set of op-group names referenced by `Operand::OpRef` anywhere in
    /// this pattern's function bodies.  Populated by
    /// [`Pattern::check_and_populate_op_refs`] after lowering completes.
    ///
    /// Stored in a `OnceCell` so it can be initialised via a shared `&Self`
    /// reference (as held by `util_block`, where items are arena-allocated and
    /// shared across multiple `PatternOperation`s).  Reads before population
    /// return an empty set via `get_or_init`.
    pub(crate) referenced_op_groups: OnceCell<FxHashSet<Symbol>>,
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
            referenced_op_groups: OnceCell::new(),
        }
    }

    /// Returns the set of op-group names referenced by `$group::$op` operands
    /// anywhere in this pattern's function bodies.
    ///
    /// This is populated by [`Pattern::check_and_populate_op_refs`] after all
    /// pattern items and the ops block have been lowered.  The matcher (Task
    /// 11) consumes this to know which op-group instances to iterate.
    ///
    /// If called before population, returns a (durable) empty set via
    /// `OnceCell::get_or_init`; subsequent population attempts on an
    /// already-initialised cell are silently ignored, which preserves the
    /// invariant that the first observed value is the final one.
    pub fn referenced_op_groups(&self) -> &FxHashSet<Symbol> {
        self.referenced_op_groups.get_or_init(FxHashSet::default)
    }

    fn add_item(
        &mut self,
        pat_name: Option<Symbol>,
        item: WithPath<'pcx, &'pcx pairs::RustItemWithConstraint<'pcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'pcx>,
    ) {
        let path = item.path;
        let (attr, item, where_block) = item.get_matched();
        let constraints = Constraints::from_where_block_opt(attr.iter_matched(), where_block, path)
            .unwrap_or_else(|err| panic!("unexpected error in constraints:\n{err}"));
        match item.deref() {
            Choice4::_0(rust_fn) => {
                let fn_name = rust_fn.FnSig().FnName().span.as_str();
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
        self.fns.all_fns.push(fn_pat);
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
    fn add_struct<'mcx>(
        &mut self,
        pat_name: Option<Symbol>,
        rust_struct: WithPath<'mcx, &'mcx pairs::Struct<'mcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        symbol_table: &rpl_meta::symbol_table::SymbolTable<'mcx>,
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
    fn add_enum<'mcx>(
        &mut self,
        pat_name: Option<Symbol>,
        rust_enum: WithPath<'mcx, &'mcx pairs::Enum<'mcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        symbol_table: &'mcx rpl_meta::symbol_table::SymbolTable<'mcx>,
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
    fn add_impl<'mcx: 'pcx>(
        &mut self,
        pat_name: Option<Symbol>,
        rust_impl: WithPath<'pcx, &'pcx pairs::Impl<'pcx>>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        symbol_table: &'mcx rpl_meta::symbol_table::SymbolTable<'mcx>,
        constraints: Constraints,
    ) {
        let p = rust_impl.path;
        let (_, _, impl_kind, ty, _, fns, _) = rust_impl.get_matched();
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
                let fn_name = rust_fn.FnSig().FnName().span.as_str();
                let fn_sym_tab = impl_sym_tab.inner.get_fn(fn_name).unwrap();
                let fn_def = FnPattern::from(
                    WithPath::new(p, rust_fn),
                    self.pcx,
                    fn_sym_tab,
                    Arc::clone(&meta),
                    constraints,
                );
                (Symbol::intern(fn_name), fn_def)
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

    /// Returns the union of op-group names referenced by any operand
    /// (positive or negative) of this set-op, recursing into nested
    /// `PatternOperation`s.
    ///
    /// This is consumed by the matcher (Task 11) to know which op-group
    /// instances to iterate over for cartesian-product expansion.
    pub fn referenced_op_groups(&self) -> FxHashSet<Symbol> {
        let mut all = FxHashSet::default();
        for (_name, item, _map) in self.positive.iter().chain(self.negative.iter()) {
            match item {
                PatternItem::RustItems(rust_items) => {
                    for &g in rust_items.referenced_op_groups() {
                        all.insert(g);
                    }
                },
                PatternItem::RPLPatternOperation(inner) => {
                    for g in inner.referenced_op_groups() {
                        all.insert(g);
                    }
                },
            }
        }
        all
    }
}

/// Corresponds to a pattern file in RPL, not a pattern item.
pub struct Pattern<'pcx> {
    pub pcx: PatCtxt<'pcx>,
    pub patt_block: FxIndexMap<Symbol, PatternItem<'pcx>>, // indexed by pat_name
    pub util_block: FxIndexMap<Symbol, &'pcx PatternItem<'pcx>>, // indexed by pat_name
    pub ops_block: OpsBlock<'pcx>,
    diag_block: FxHashMap<Symbol, DynamicErrorBuilder<'pcx>>,
    /// R4/R5 errors discovered during `check_and_populate_op_refs`.
    /// Stored here so callers (tests, driver) can inspect them after lowering.
    pub(crate) op_ref_errors: Vec<ops_uses::OpsUseError>,
}

impl<'pcx> Pattern<'pcx> {
    pub(crate) fn new(pcx: PatCtxt<'pcx>) -> Self {
        Self {
            pcx,
            patt_block: Default::default(),
            util_block: Default::default(),
            ops_block: OpsBlock::default(),
            diag_block: Default::default(),
            op_ref_errors: Default::default(),
        }
    }

    /// Returns the R4/R5 use-site errors found during `check_and_populate_op_refs`.
    pub fn op_ref_errors(&self) -> &[ops_uses::OpsUseError] {
        &self.op_ref_errors
    }

    #[instrument(level = "debug", skip(self, source_map, body, decl))]
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
        symbol_tables: &'pcx PattSymbolTables<'_>,
        block_type: PattOrUtil,
    ) {
        let p = pat_item.path;
        let (_, attr, name, meta_decls, _, item_or_patt_op) = pat_item.get_matched();
        let name = name.span.as_str();
        let symbol_table = symbol_tables.get(&name).unwrap();
        let meta = Arc::new(NonLocalMetaVars::from_meta_decls(
            meta_decls.as_ref().map(|meta_decls| with_path(p, meta_decls)),
            self.pcx,
            symbol_table,
        ));
        let name = Symbol::intern(name);
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
    fn add_patt_op<'mcx: 'pcx>(
        &mut self,
        pat_name: Symbol,
        attr: impl Iterator<Item = &'mcx pairs::Attr<'mcx>>,
        patt_op: WithPath<'mcx, &'mcx pairs::PatternOperation<'mcx>>,
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

    /// Lower an `opsBlock` pest pair into `self.ops_block`.
    ///
    /// Runs well-formedness checks R1–R3 on the raw parse tree before any
    /// lowering so that the `unreachable!()` contracts in `OpsMetaLookup`
    /// cannot be triggered by malformed input.  Groups that fail a check are
    /// skipped; callers should surface the returned errors to the user.
    ///
    /// For each (valid) `opsItem` in the block we:
    /// 1. Extract the group name (bare, no leading `$`).
    /// 2. Lower the `MetaVariableDeclList` into `NonLocalMetaVars` using a minimal `GetType`
    ///    implementation backed by the item's own type-var declarations.
    /// 3. Lower each `OpFnDecl` into an `OpSignature` (name, params, ret).
    /// 4. Build an `OpGroup` and insert it into `self.ops_block.groups`.
    ///
    /// Returns the list of well-formedness errors found (if any).
    pub fn add_ops_block<'mcx: 'pcx>(
        &mut self,
        ops_block: WithPath<'mcx, &'mcx pairs::opsBlock<'mcx>>,
    ) -> Vec<ops_wf::OpsWfError> {
        // R1–R3: pre-validate before touching any lowering code.
        let wf_errors = ops_wf::check_ops_block(ops_block.inner);
        // Collect group names that have errors so we can skip them below.
        let bad_groups: std::collections::HashSet<&str> = wf_errors.iter().map(|e| e.group.as_str()).collect();

        let p = ops_block.path;
        for item in ops_block.opsItem() {
            // -- 1. Group name (bare Identifier, no `$`).
            let group_name = Symbol::intern(item.Identifier().span.as_str());
            let _span = item.span; // TODO(task-6): replace DUMMY_SP with a real rustc Span

            // Skip groups that failed R1/R2/R3.
            if bad_groups.contains(group_name.as_str()) {
                continue;
            }

            // -- 2. Pre-scan MetaVariableDeclList to build OpsMetaLookup.
            //    We need the lookup both for `NonLocalMetaVars::from_meta_decls`
            //    (const/place var types) and for `Ty::from` on param types
            //    that reference type meta-variables like `$T`.
            let meta_decl_list = item.MetaVariableDeclList();
            let lookup = OpsMetaLookup::from_meta_decl_list(meta_decl_list);

            // -- 3. Lower the MetaVariableDeclList into NonLocalMetaVars.
            let meta =
                NonLocalMetaVars::from_meta_decls(meta_decl_list.map(|mdl| WithPath::new(p, mdl)), self.pcx, &lookup);

            // -- 4. Lower each OpFnDecl into OpSignature.
            let mut ops: FxIndexMap<Symbol, OpSignature<'pcx>> = FxIndexMap::default();
            for decl in item.OpFnDecl() {
                let sig = decl.OpFnSig();
                // Op name: FnName is PlaceHolder | MetaVariable | Identifier.
                // In practice op fn names are MetaVariables like `$lock`.
                let op_name_raw = sig.FnName().span.as_str();
                let op_name = Symbol::intern(op_name_raw.trim_start_matches('$'));

                // Lower parameters.
                let params: Vec<Param<'pcx>> = if let Some(params_pair) = sig.OpFnParamsSeparatedByComma() {
                    let (first, rest) = params_pair.OpFnParam();
                    std::iter::once(first)
                        .chain(rest)
                        .filter_map(|param| lower_op_fn_param(p, param, self.pcx, &lookup))
                        .collect()
                } else {
                    Vec::new()
                };

                // Lower return type.
                // We inline the logic of Ty::from_fn_ret because that function
                // takes &FnSymbolTable specifically; here we use our OpsMetaLookup.
                let ret = sig.FnRet().map(|fn_ret| {
                    let (_, placeholder_or_ty) = fn_ret.get_matched();
                    match placeholder_or_ty {
                        Choice2::_0(_) => self.pcx.mk_any_ty(),
                        Choice2::_1(ty) => Ty::from(WithPath::new(p, ty), self.pcx, &lookup),
                    }
                });

                let op_sig = OpSignature {
                    name: op_name,
                    params,
                    ret,
                    span: rustc_span::DUMMY_SP,
                }; // TODO(task-6): replace DUMMY_SP with a real rustc Span
                ops.insert(op_name, op_sig);
            }

            // -- 5. Build OpGroup and insert.
            let group = OpGroup {
                name: group_name,
                meta_vars: meta,
                ops,
                span: rustc_span::DUMMY_SP,
            }; // TODO(task-6): replace DUMMY_SP with a real rustc Span
            self.ops_block.groups.insert(group_name, group);
        }
        wf_errors
    }

    /// Run R4/R5 use-site checks on all `RustItems` in `patt_block` and
    /// `util_block`, populate `referenced_op_groups` on each `RustItems`, and
    /// store discovered errors in `self.op_ref_errors`.
    ///
    /// Call this **after** both `add_ops_block` and all `add_pattern_item`
    /// calls have completed.
    ///
    /// `referenced_op_groups` is now a `OnceCell`, so population goes through
    /// `OnceCell::set` via a shared reference.  This works uniformly for
    /// `patt_block` (owned `PatternItem`s) and `util_block` (arena-allocated
    /// items shared across multiple `PatternOperation`s) — no `unsafe` is
    /// needed.  A second `set` on an already-initialised cell returns `Err`
    /// and is ignored: by construction, each `RustItems` is visited once here
    /// and not observed by any reader until after this function returns.
    pub fn check_and_populate_op_refs(&mut self) {
        let mut all_errors = Vec::new();

        let process = |rust_items: &RustItems<'pcx>, errors: &mut Vec<ops_uses::OpsUseError>| {
            let mut referenced = FxHashSet::default();
            let errs = ops_uses::check_op_refs(rust_items, &self.ops_block, &mut referenced);
            // `set` returns Err iff the cell was already initialised; that
            // would indicate this function ran twice, which is a logic bug
            // but not a soundness one — silently keep the prior value.
            let _ = rust_items.referenced_op_groups.set(referenced);
            errors.extend(errs);
        };

        for (_name, item) in &self.patt_block {
            if let PatternItem::RustItems(rust_items) = item {
                process(rust_items, &mut all_errors);
            }
        }
        for (_name, item_ref) in &self.util_block {
            if let PatternItem::RustItems(rust_items) = *item_ref {
                process(rust_items, &mut all_errors);
            }
        }

        self.op_ref_errors = all_errors;
    }

    pub fn add_diag<'mcx: 'pcx>(
        &mut self,
        diag: WithPath<'mcx, &'mcx pairs::diagBlock<'mcx>>,
        diag_symbol_tables: &rpl_meta::meta::DiagSymbolTables<'mcx>,
        symbol_tables: &PattSymbolTables<'mcx>,
    ) {
        let mut items = FxHashMap::default();
        for item in diag.get_matched().2.iter_matched() {
            let (_, ident, _, _, _, _, _) = item.get_matched();
            let name = Symbol::intern(ident.span.as_str());
            let prev = items.insert(name, item);
            debug_assert!(prev.is_none(), "Duplicate diagnostic for {:?}", name); //FIXME: raise an error
        }

        for (name, pat_item) in &self.patt_block {
            let symbol_table = symbol_tables.get(&name.as_str()).unwrap();

            let diag_name = pat_item.diag_name().unwrap_or(*name);
            if let Some(diag_item) = items.get(&diag_name) {
                let labels = symbol_table.labels().map(Symbol::intern);
                let label_map = pat_item.label_map();
                let diag = DynamicErrorBuilder::<'pcx>::from_item(
                    WithPath::new(diag.path, diag_item),
                    &symbol_table.meta_vars,
                    pat_item.consts(),
                    &labels.collect(),
                    &label_map,
                    diag_symbol_tables
                        .get(&diag_name.as_str())
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

    /// Primary label names for a pattern's diagnostic, in declaration order.
    pub fn primary_labels(&self, pat_name: Symbol) -> Option<&[&str]> {
        self.diag_block.get(&pat_name).map(|diag| diag.primary_labels())
    }
}

/// Minimal `GetType` implementation for lowering ops-item signatures.
///
/// Ops items do not import Rust types or paths — they only use type meta-variables
/// (e.g. `$T`, `$U`) declared in the item's own `MetaVariableDeclList`.
/// This struct is populated by pre-scanning that list and maps each declared
/// type-variable name to its 0-based index.
struct OpsMetaLookup<'i> {
    /// (bare_name_with_dollar, index)
    type_vars: Vec<(&'i str, usize)>,
}

impl<'i> OpsMetaLookup<'i> {
    /// Build an `OpsMetaLookup` by scanning the `MetaVariableDeclList` of one
    /// `opsItem`. Only `$T: type` style (type-kind) declarations are collected;
    /// const and place vars are skipped (they would panic if their types
    /// contained path identifiers, but that scenario is not supported yet).
    ///
    /// # Index-counter alignment with `NonLocalMetaVars`
    ///
    /// `NonLocalMetaVars::from_meta_decls` partitions declarations into three
    /// separate buckets (type / const / place) and then pushes each bucket into
    /// its own `IndexVec` in three independent passes — so type-var indices in
    /// `NonLocalMetaVars` always start at 0 and count only type-kind decls.
    /// This function counts `idx` the same way (incrementing only for
    /// type-kind decls), so the `MetaVariable::Type(idx, …)` values we emit
    /// for downstream consumers are aligned.  This is **Path A** from the
    /// code-review checklist: the counter is correct as-is and must *not* be
    /// changed to be unconditional.
    fn from_meta_decl_list(meta_decl_list: Option<&'i pairs::MetaVariableDeclList<'i>>) -> Self {
        let mut type_vars = Vec::new();
        if let Some(mdl) = meta_decl_list
            && let Some(inner) = mdl.get_matched().1
        {
            let decls = collect_elems_separated_by_comma!(inner).collect::<Vec<_>>();
            let mut idx = 0usize;
            for decl in &decls {
                let (ident, _, ty, _) = decl.get_matched();
                if matches!(ty.deref(), Choice3::_0(_)) {
                    // Type meta-variable: retain name with $ prefix for matching.
                    // idx counts only type-kind decls — see alignment note above.
                    type_vars.push((ident.span.as_str(), idx));
                    idx += 1;
                }
            }
        }
        Self { type_vars }
    }
}

impl<'i> GetType<'i> for OpsMetaLookup<'i> {
    fn get_type_or_path(
        &self,
        ident: &WithPath<'i, &pairs::Identifier<'i>>,
    ) -> Result<TypeOrPath<'i>, rpl_meta::RPLMetaError<'i>> {
        // Ops signatures must only reference meta-variables declared in their
        // MetaVariableDeclList — bare Rust path types are not permitted.
        // Reaching this branch means resolver check R1 (Task 6) was not run
        // or failed to reject the invalid signature before lowering.
        unreachable!(
            "internal: ops signatures must use only op-level meta-vars; \
             bare path type `{}` at {:?} should have been rejected by \
             resolver check R1 before lowering",
            ident.span.as_str(),
            ident.path
        )
    }

    fn force_get_meta_var(&self, ident: WithPath<'i, &pairs::MetaVariable<'i>>) -> MetaVariable<'i> {
        let name = ident.inner.span.as_str();
        // The meta variable includes the `$` prefix in its span text.
        if let Some((_, idx)) = self.type_vars.iter().find(|(n, _)| *n == name) {
            MetaVariable::Type(*idx, PredicateConjunction::default())
        } else {
            // Reaching this branch means the meta-variable was used in a
            // signature but not declared in the ops item's MetaVariableDeclList.
            // Resolver check R1 (Task 6) must reject this before lowering.
            unreachable!(
                "internal: meta-variable `{}` at {:?} is not declared in this \
                 ops item; this should have been rejected by resolver check R1 \
                 before lowering",
                name, ident.path
            )
        }
    }
}

/// Lower a single `OpFnParam` into a `Param`, or return `None` for the
/// variadic `..` case (which sets `non_exhaustive` rather than adding a param).
///
/// Handles the five alternatives of `OpFnParam`:
/// - `SelfParam` → self parameter with auto-inferred type
/// - `NormalParam` → `$name: Type`
/// - `PlaceHolderWithType` → `_: Type`
/// - `Type` → anonymous parameter with inferred type (e.g. `&mut $T`)
/// - `Dot2` → variadic `..`; returns `None`
fn lower_op_fn_param<'mcx, 'pcx: 'mcx>(
    p: &'mcx std::path::Path,
    param: &'mcx pairs::OpFnParam<'mcx>,
    pcx: PatCtxt<'pcx>,
    lookup: &OpsMetaLookup<'mcx>,
) -> Option<Param<'pcx>> {
    use utils::mutability_from_pair_mutability;

    if let Some(self_param) = param.SelfParam() {
        let (ty, mutability) = self_param_ty(self_param);
        let ty = Ty::from(WithPath::new(p, ty), pcx, lookup);
        return Some(Param {
            mutability,
            ident: Symbol::intern("self"),
            ty,
        });
    }

    if let Some(normal) = param.NormalParam() {
        let (mutability, ident, _, ty) = normal.get_matched();
        let mutability = mutability_from_pair_mutability(mutability);
        let ident = Symbol::intern(ident.span.as_str());
        let ty = Ty::from(WithPath::new(p, ty), pcx, lookup);
        return Some(Param { mutability, ident, ty });
    }

    if let Some(place_holder_with_type) = param.PlaceHolderWithType() {
        let (mutability, _placeholder, _, ty) = place_holder_with_type.get_matched();
        let mutability = mutability_from_pair_mutability(mutability);
        let ty = Ty::from(WithPath::new(p, ty), pcx, lookup);
        return Some(Param {
            mutability,
            ident: Symbol::intern("_"),
            ty,
        });
    }

    if let Some(ty_only) = param.Type() {
        // Bare `Type` parameter: no explicit name, synthesize `_`.
        let ty = Ty::from(WithPath::new(p, ty_only), pcx, lookup);
        return Some(Param {
            mutability: MirMutability::Not,
            ident: Symbol::intern("_"),
            ty,
        });
    }

    // Dot2 / `..` — variadic; callers can set non_exhaustive if needed.
    // For ops signatures we simply drop it (no body to match against).
    None
}
