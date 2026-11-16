use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

pub use error::DynamicError;
use error::DynamicErrorBuilder;
use rpl_constraints::Constraints;
use rpl_constraints::attributes::Visibility;
use rpl_constraints::predicates::{PredicateArg, PredicateTerm};
use rpl_meta::collect_elems_separated_by_comma;
use rpl_meta::meta::PattSymbolTables;
use rpl_meta::symbol_table::WithPath;
use rpl_parser::generics::{Choice2, Choice3, Choice4, Choice9};
use rpl_parser::pairs;
use rustc_data_structures::fx::{FxHashMap, FxHashSet, FxIndexMap};
use rustc_hir::FnDecl;
use rustc_middle::mir::Body;
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

/// Expansion result for a sequence of raw statements: `(positive, negative)` alternatives.
type ExpandedStatements<'pcx> = (Vec<Vec<RawStatement<'pcx>>>, Vec<Vec<RawStatement<'pcx>>>);

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
    pub fn should_deduplicate(&self) -> bool {
        match self {
            PatternItem::RustItems(items) => items.attr.should_deduplicate(),
            PatternItem::RPLPatternOperation(op) => op.attr.should_deduplicate(),
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
                for fn_pat in &items.fns.all_fns {
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
                let (vis, name, _, ty) = field.get_matched();
                let name = Symbol::intern(name.span.as_str());
                let ty = Ty::from(with_path(rust_struct.path, ty), self.pcx, symbol_table);
                let field = Field {
                    ty,
                    vis: Visibility::parse(vis.as_ref()),
                };
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
                                let (vis, name, _, ty) = field.get_matched();
                                let name = Symbol::intern(name.span.as_str());
                                let ty = Ty::from(with_path(rust_enum.path, ty), self.pcx, symbol_table);
                                let field = Field {
                                    ty,
                                    vis: Visibility::parse(vis.as_ref()),
                                };
                                enum_variant.fields.insert(name, field);
                            }
                        }
                        variant.get_matched().0
                    },
                    Choice3::_1(variant) => {
                        let (name, _, ty, _) = variant.get_matched();
                        let name = Symbol::intern(name.span.as_str());
                        let ty = Ty::from(with_path(rust_enum.path, ty), self.pcx, symbol_table);
                        let field = Field {
                            ty,
                            vis: Visibility::Unspecified,
                        };
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
        // The negative items may legitimately carry extra columns (e.g. a label inside a
        // `without!` / `or!` fragment that is absent from the positive, as in
        // `without!('reset: _ = _(..))`). Union those columns into the head so the schema can
        // still refer to them, instead of requiring negative == positive.
        let mut head = head;
        for (_, item, _) in &self.negative {
            for (name, column) in item.table_head() {
                if let Some(existing) = head.get(&name) {
                    debug_assert_eq!(*existing, column, "conflicting column types for `{name}`");
                } else {
                    head.insert(name, column);
                }
            }
        }
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
    /// Counter for synthesizing unique util-pattern names during macro desugaring.
    next_macro_util: usize,
}

impl<'pcx> Pattern<'pcx> {
    pub(crate) fn new(pcx: PatCtxt<'pcx>) -> Self {
        Self {
            pcx,
            patt_block: Default::default(),
            util_block: Default::default(),
            diag_block: Default::default(),
            next_macro_util: 0,
        }
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
                if let Some(rust_fn) = item.RustItem().Fn()
                    && fn_has_body_macro(rust_fn)
                {
                    self.add_fn_macro_op(pat_name, attr, with_path(p, item), symbol_table, meta, block_type);
                } else {
                    self.add_items(
                        pat_name,
                        attr,
                        with_path(p, std::iter::once(item)),
                        symbol_table,
                        meta,
                        block_type,
                    );
                }
            },
            Choice3::_1(items) => {
                // A brace block with exactly one `Fn` item whose body contains macros
                // desugars just like a single top-level function item.
                let inner = items.get_matched().1;
                let mut iter = inner.iter_matched();
                if let (Some(item), None) = (iter.next(), iter.next())
                    && let Some(rust_fn) = item.RustItem().Fn()
                    && fn_has_body_macro(rust_fn)
                {
                    self.add_fn_macro_op(pat_name, attr, with_path(p, item), symbol_table, meta, block_type);
                } else {
                    self.add_items(
                        pat_name,
                        attr,
                        with_path(p, inner.iter_matched()),
                        symbol_table,
                        meta,
                        block_type,
                    );
                }
            },
            Choice3::_2(patt_op) => {
                self.add_patt_op(pat_name, attr, with_path(p, patt_op), meta, block_type);
            },
        }
    }

    /// Desugar a single function whose body contains `without!` / `or!` macros into a
    /// [`PatternOperation`] over synthesized util patterns.
    fn add_fn_macro_op(
        &mut self,
        pat_name: Symbol,
        attr: impl Iterator<Item = &'pcx pairs::Attr<'pcx>>,
        item: WithPath<'pcx, &'pcx pairs::RustItemWithConstraint<'pcx>>,
        symbol_table: &'pcx rpl_meta::symbol_table::SymbolTable<'pcx>,
        meta: Arc<NonLocalMetaVars<'pcx>>,
        block_type: PattOrUtil,
    ) {
        let p = item.path;
        let (_, rust_item, where_block) = item.get_matched();
        let Choice4::_0(rust_fn) = rust_item.deref() else {
            unreachable!("`add_fn_macro_op` expects a function item");
        };

        let fn_name = rust_fn.FnSig().FnName().span.as_str();
        let fn_symbol_table = symbol_table.get_fn(fn_name).unwrap();

        let (sig, body) = rust_fn.get_matched();
        let (safety, visibility, name, _, _) = FnPattern::from_sig(WithPath::new(p, sig), self.pcx, fn_symbol_table);

        let (decls, stmts) = if let Some(body) = body.MirBody() {
            let (decls, stmts) = body.get_matched();
            (
                decls.iter_matched().collect::<Vec<_>>(),
                stmts.iter_matched().collect::<Vec<_>>(),
            )
        } else {
            (Vec::new(), Vec::new())
        };

        // Parse the `where`-block constraints once (they are shared, modulo fragment-label
        // predicates that only apply to the negative).
        let mut constraints = Constraints::from_where_block_opt(std::iter::empty(), where_block, p)
            .expect("unexpected error in constraints");
        constraints.attrs.add_safety(safety);
        constraints.attrs.add_visibility(visibility);

        // Lower declarations (shared across expansions) and statements (with `without!` / `or!`
        // markers) once.
        let raw_decls = decls
            .iter()
            .map(|decl| RawDecleration::from(WithPath::new(p, decl), self.pcx, fn_symbol_table))
            .collect::<Vec<_>>();
        let raw_stmts = stmts
            .into_iter()
            .map(|stmt| RawStatement::from(WithPath::new(p, stmt), self.pcx, fn_symbol_table))
            .collect::<Vec<_>>();

        // Labels that exist only inside `without!` / `or!` fragments.
        let fragment_labels = collect_fragment_labels(&raw_stmts);

        let (positive, negative) = expand_raw_stmts(raw_stmts);

        // The positive util patterns lack the fragment labels, so drop the `where` predicates
        // that mention them.
        let mut positive_constraints = constraints.clone();
        filter_predicates(&mut positive_constraints, &fragment_labels);

        let positive = positive
            .into_iter()
            .map(|stmts| {
                self.register_macro_util(
                    pat_name,
                    p,
                    sig,
                    &raw_decls,
                    stmts,
                    positive_constraints.clone(),
                    name,
                    fn_symbol_table,
                    &meta,
                )
            })
            .collect();
        let negative = negative
            .into_iter()
            .map(|stmts| {
                self.register_macro_util(
                    pat_name,
                    p,
                    sig,
                    &raw_decls,
                    stmts,
                    constraints.clone(),
                    name,
                    fn_symbol_table,
                    &meta,
                )
            })
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

    /// Lower one expansion of a macro-desugared function into a synthesized util pattern.
    #[allow(clippy::too_many_arguments)]
    fn register_macro_util(
        &mut self,
        pat_name: Symbol,
        p: &'pcx std::path::Path,
        sig: &'pcx pairs::FnSig<'pcx>,
        raw_decls: &[RawDecleration<'pcx>],
        stmts: Vec<RawStatement<'pcx>>,
        constraints: Constraints,
        name: Symbol,
        fn_symbol_table: &'pcx FnSymbolTable<'pcx>,
        meta: &Arc<NonLocalMetaVars<'pcx>>,
    ) -> (Symbol, &'pcx PatternItem<'pcx>, MatchedMap) {
        let util_name = Symbol::intern(&format!("@{}${}", pat_name.as_str(), self.next_macro_util));
        self.next_macro_util += 1;

        let (_, _, _, params, ret) = FnPattern::from_sig(WithPath::new(p, sig), self.pcx, fn_symbol_table);

        let raw_decls = raw_decls.to_vec();
        let raw_stmts = stmts;

        let fn_pat = FnPattern::build(
            name,
            params,
            ret,
            raw_decls,
            raw_stmts,
            self.pcx,
            fn_symbol_table,
            Arc::clone(meta),
            constraints,
        );
        let fn_pat = self.pcx.alloc_fn(fn_pat);

        let mut rust_items = RustItems::new(self.pcx, Arc::clone(meta), PatAttr::default());
        rust_items.fns.all_fns.push(fn_pat);
        match name.as_str() {
            "_" => rust_items.fns.unnamed_fns.push(fn_pat),
            _ => {
                rust_items.fns.named_fns.insert(name, fn_pat);
            },
        }

        let item = self.pcx.alloc_pattern_item(PatternItem::RustItems(rust_items));
        self.util_block.insert(util_name, item);
        let map = MatchedMap::new(meta, item.meta(), None);
        (util_name, item, map)
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

/// Detect whether a function body contains a `without!` / `or!` macro at any nesting level.
fn fn_has_body_macro(rust_fn: &pairs::Fn<'_>) -> bool {
    let (_, body) = rust_fn.get_matched();
    let Some(mir) = body.MirBody() else {
        return false;
    };
    let (_, stmts) = mir.get_matched();
    stmts.iter_matched().any(stmt_has_body_macro)
}

fn stmt_has_body_macro(stmt: &pairs::MirStmt<'_>) -> bool {
    match stmt.deref() {
        Choice9::_8(m) => is_body_macro(m.get_matched().0),
        Choice9::_5(loop_) => loop_has_body_macro(loop_),
        Choice9::_6(switch) => switch_has_body_macro(switch),
        _ => false,
    }
}

fn loop_has_body_macro(loop_: &pairs::MirLoop<'_>) -> bool {
    let (_, _, block) = loop_.get_matched();
    block.get_matched().1.iter_matched().any(stmt_has_body_macro)
}

fn switch_has_body_macro(switch: &pairs::MirSwitchInt<'_>) -> bool {
    let (_, _, _, _, _, _, targets, _) = switch.get_matched();
    targets.iter_matched().any(|target| {
        let (_, _, body) = target.get_matched();
        switch_body_has_body_macro(body)
    })
}

fn switch_body_has_body_macro(body: &pairs::MirSwitchBody<'_>) -> bool {
    match body.deref() {
        Choice4::_0(block) => block.get_matched().1.iter_matched().any(stmt_has_body_macro),
        Choice4::_1(_) => false,
        Choice4::_2(loop_) => loop_has_body_macro(loop_),
        Choice4::_3(switch) => switch_has_body_macro(switch),
    }
}

fn is_body_macro(mac: &pairs::MirMacro<'_>) -> bool {
    matches!(mac.deref(), Choice4::_1(_) | Choice4::_2(_))
}

/// Expand `without!` / `or!` markers (at any nesting level) into concrete statement sequences.
///
/// Returns `(positive, negative)`:
/// - `positive`: every `or!` alternative is combined (cartesian product), `without!` fragments
///   dropped.
/// - `negative`: for each `without!`, one body per `or!` combination with that fragment restored.
fn expand_raw_stmts(stmts: Vec<RawStatement<'_>>) -> ExpandedStatements<'_> {
    compose(stmts.into_iter().map(expand_raw_stmt).collect())
}

fn expand_raw_stmt(stmt: RawStatement<'_>) -> ExpandedStatements<'_> {
    match stmt {
        RawStatement::Without(frag) => {
            let (frag_pos, _) = expand_raw_stmts(frag);
            (vec![Vec::new()], frag_pos)
        },
        RawStatement::Or(alts) => {
            let mut positive = Vec::new();
            let mut negative = Vec::new();
            for alt in alts {
                let (alt_pos, alt_neg) = expand_raw_stmts(alt);
                positive.extend(alt_pos);
                negative.extend(alt_neg);
            }
            (positive, negative)
        },
        RawStatement::Loop(body) => {
            let (body_pos, body_neg) = expand_raw_stmts(body);
            (
                body_pos.into_iter().map(RawStatement::Loop).map(|s| vec![s]).collect(),
                body_neg.into_iter().map(RawStatement::Loop).map(|s| vec![s]).collect(),
            )
        },
        RawStatement::SwitchInt {
            label,
            operand,
            targets,
            otherwise,
        } => expand_switch_int(label, operand, targets, otherwise),
        RawStatement::Panic(label, frag) => {
            let (frag_pos, _) = expand_raw_stmts(frag);
            (
                frag_pos
                    .into_iter()
                    .map(|f| vec![RawStatement::Panic(label, f)])
                    .collect(),
                Vec::new(),
            )
        },
        other => (vec![vec![other]], Vec::new()),
    }
}

/// Compose a list of `(positive, negative)` expansion results via cartesian product.
fn compose(per: Vec<ExpandedStatements<'_>>) -> ExpandedStatements<'_> {
    fn product<'pcx>(lists: &[Vec<Vec<RawStatement<'pcx>>>]) -> Vec<Vec<RawStatement<'pcx>>> {
        let mut acc: Vec<Vec<RawStatement<'pcx>>> = vec![Vec::new()];
        for alts in lists {
            let mut next = Vec::new();
            for prefix in acc {
                for alt in alts {
                    let mut q = prefix.clone();
                    q.extend(alt.iter().cloned());
                    next.push(q);
                }
            }
            acc = next;
        }
        acc
    }

    let positives: Vec<Vec<Vec<RawStatement<'_>>>> = per.iter().map(|(p, _)| p.clone()).collect();
    let positive = product(&positives);

    let mut negative = Vec::new();
    for (i, (_, negs)) in per.iter().enumerate() {
        for neg in negs {
            let prefix = product(&positives[..i]);
            let suffix = product(&positives[i + 1..]);
            for pre in &prefix {
                for suf in &suffix {
                    let mut q = pre.clone();
                    q.extend(neg.iter().cloned());
                    q.extend(suf.iter().cloned());
                    negative.push(q);
                }
            }
        }
    }
    (positive, negative)
}

/// Expand the bodies of a `switchInt` (targets + otherwise) recursively.
fn expand_switch_int<'pcx>(
    label: Option<Label>,
    operand: Operand<'pcx>,
    targets: Vec<(IntValue, Vec<RawStatement<'pcx>>)>,
    otherwise: Option<Vec<RawStatement<'pcx>>>,
) -> ExpandedStatements<'pcx> {
    let n_targets = targets.len();
    let has_otherwise = otherwise.is_some();

    // Each body slot expands to (positive, negative) alternatives.
    let slots: Vec<ExpandedStatements<'pcx>> = targets
        .iter()
        .map(|(_, body)| expand_raw_stmts(body.clone()))
        .chain(otherwise.iter().map(|body| expand_raw_stmts(body.clone())))
        .collect();

    // Build a concrete `switchInt` from a list of concrete bodies (in slot order).
    let build = |bodies: Vec<Vec<RawStatement<'pcx>>>| -> RawStatement<'pcx> {
        let concrete_targets: Vec<(IntValue, Vec<RawStatement<'pcx>>)> = targets
            .iter()
            .enumerate()
            .map(|(i, (value, _))| (*value, bodies[i].clone()))
            .collect();
        let concrete_otherwise = has_otherwise.then(|| bodies[n_targets].clone());
        RawStatement::SwitchInt {
            label,
            operand: operand.clone(),
            targets: concrete_targets,
            otherwise: concrete_otherwise,
        }
    };

    // Cartesian product over slots, keeping each slot's chosen body separate.
    fn slot_product<'pcx>(slots: &[ExpandedStatements<'pcx>]) -> Vec<Vec<Vec<RawStatement<'pcx>>>> {
        let mut acc: Vec<Vec<Vec<RawStatement<'pcx>>>> = vec![Vec::new()];
        for (pos, _) in slots {
            let mut next = Vec::new();
            for prefix in acc {
                for body in pos {
                    let mut q = prefix.clone();
                    q.push(body.clone());
                    next.push(q);
                }
            }
            acc = next;
        }
        acc
    }

    let positive = slot_product(&slots)
        .into_iter()
        .map(|bodies| vec![build(bodies)])
        .collect();

    let mut negative = Vec::new();
    for (i, (_, negs)) in slots.iter().enumerate() {
        for neg in negs {
            let prefix = slot_product(&slots[..i]);
            let suffix = slot_product(&slots[i + 1..]);
            for pre in &prefix {
                for suf in &suffix {
                    let mut bodies = pre.clone();
                    bodies.push(neg.clone());
                    bodies.extend(suf.iter().cloned());
                    negative.push(vec![build(bodies)]);
                }
            }
        }
    }

    (positive, negative)
}

/// Collect labels that appear inside `without!` / `or!` fragments (and are therefore absent from
/// the positive desugaring). These labels are still referenced by the `where` block of the
/// *negative* pattern only.
fn collect_fragment_labels(stmts: &[RawStatement<'_>]) -> FxHashSet<Symbol> {
    fn collect_all(stmts: &[RawStatement<'_>], out: &mut FxHashSet<Symbol>) {
        for stmt in stmts {
            match stmt {
                RawStatement::Assign(label, ..)
                | RawStatement::Call(label, ..)
                | RawStatement::CallIgnoreRet(label, ..)
                | RawStatement::CopyNonOverlapping(label, ..)
                | RawStatement::Unreachable(label)
                | RawStatement::Drop(label, ..)
                | RawStatement::Move(label, ..) => {
                    if let Some(label) = label {
                        out.insert(*label);
                    }
                },
                RawStatement::Panic(label, frag) => {
                    out.insert(*label);
                    collect_all(frag, out);
                },
                RawStatement::Without(frag) => collect_all(frag, out),
                RawStatement::Or(alts) => {
                    for alt in alts {
                        collect_all(alt, out);
                    }
                },
                RawStatement::Loop(body) => collect_all(body, out),
                RawStatement::SwitchInt {
                    label,
                    targets,
                    otherwise,
                    ..
                } => {
                    if let Some(label) = label {
                        out.insert(*label);
                    }
                    for (_, body) in targets {
                        collect_all(body, out);
                    }
                    if let Some(body) = otherwise {
                        collect_all(body, out);
                    }
                },
                RawStatement::Break | RawStatement::Continue | RawStatement::Return => {},
            }
        }
    }

    fn visit_fragments(stmts: &[RawStatement<'_>], out: &mut FxHashSet<Symbol>) {
        for stmt in stmts {
            match stmt {
                RawStatement::Without(frag) => collect_all(frag, out),
                RawStatement::Or(alts) => {
                    for alt in alts {
                        collect_all(alt, out);
                    }
                },
                RawStatement::Loop(body) => visit_fragments(body, out),
                RawStatement::SwitchInt { targets, otherwise, .. } => {
                    for (_, body) in targets {
                        visit_fragments(body, out);
                    }
                    if let Some(body) = otherwise {
                        visit_fragments(body, out);
                    }
                },
                RawStatement::Panic(_, frag) => visit_fragments(frag, out),
                _ => {},
            }
        }
    }

    let mut labels = FxHashSet::default();
    visit_fragments(stmts, &mut labels);
    labels
}

/// Remove from the `where` block any predicate term referencing a fragment label (which is absent
/// from the positive desugaring). A negated term (`!pred(frag_label)`) is vacuously true when the
/// label is absent, so its whole clause is dropped; a non-negated term is vacuously false and is
/// merely removed from its clause.
fn filter_predicates(constraints: &mut Constraints, fragment_labels: &FxHashSet<Symbol>) {
    let refers_frag = |term: &PredicateTerm| {
        term.args
            .iter()
            .any(|arg| matches!(arg, PredicateArg::Label(l) if fragment_labels.contains(l)))
    };

    let mut preds = Vec::new();
    for mut conj in std::mem::take(&mut constraints.preds) {
        let mut clauses = Vec::new();
        for mut clause in conj.clauses.drain(..) {
            if clause.terms.iter().any(|t| t.is_neg && refers_frag(t)) {
                continue;
            }
            clause.terms.retain(|t| !refers_frag(t));
            if !clause.terms.is_empty() {
                clauses.push(clause);
            }
        }
        conj.clauses = clauses;
        if !conj.clauses.is_empty() {
            preds.push(conj);
        }
    }
    constraints.preds = preds;
}
