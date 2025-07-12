use std::path::Path;
use std::sync::RwLock;

use parser::pairs;
use rustc_data_structures::fx::FxHashMap;
use rustc_index::IndexVec;

use crate::arena::Arena;
use crate::idx::RPLIdx;
use crate::meta::SymbolTables;

/// Provides a context for the meta data of the RPL multi-files/modularity.
pub struct MetaContext<'mcx> {
    arena: &'mcx Arena<'mcx>,
    pub path2id: FxHashMap<&'mcx Path, RPLIdx>,
    pub id2path: IndexVec<RPLIdx, &'mcx Path>,
    pub contents: IndexVec<RPLIdx, &'mcx str>,
    pub syntax_trees: IndexVec<RPLIdx, &'mcx pairs::main<'mcx>>,
    pub symbol_tables: IndexVec<RPLIdx, SymbolTables<'mcx>>,
    active_path: RwLock<Option<&'mcx Path>>,
    pub(crate) lints: Vec<&'static rustc_lint::Lint>,
}

mod test {
    const fn _check_sync<T: Sync>() {}

    #[test]
    fn test_check_sync() {
        _check_sync::<super::MetaContext<'_>>();
    }
}

impl<'mcx> MetaContext<'mcx> {
    pub fn new(arena: &'mcx Arena<'mcx>) -> Self {
        Self {
            arena,
            path2id: FxHashMap::default(),
            id2path: IndexVec::new(),
            contents: IndexVec::new(),
            syntax_trees: IndexVec::new(),
            symbol_tables: IndexVec::new(),
            active_path: RwLock::new(None),
            lints: Vec::new(),
        }
    }

    /// Request a tree id for the given path.
    /// If the path already has an id, return it.
    /// Otherwise, create a new id, insert it into the path2id map, and return it.
    pub fn request_rpl_idx(&mut self, path: &'mcx Path) -> RPLIdx {
        if let Some(&id) = self.path2id.get(path) {
            id
        } else {
            // FIXME: Is this allocation necessary?
            let path = self.arena.alloc(path);
            let id: RPLIdx = self.path2id.len().into();
            self.path2id.insert(path, id);
            debug_assert_eq!(self.id2path.next_index(), id);
            self.id2path.push(path);
            id
        }
    }

    /// Set the active path.
    pub fn set_active_path(&self, path: Option<&'mcx Path>) {
        *self.active_path.write().unwrap() = path;
    }

    /// Get the active path.
    pub fn get_active_path(&self) -> &'mcx Path {
        self.active_path
            .read()
            .unwrap()
            .unwrap_or_else(|| panic!("Active path is not set."))
    }

    pub(crate) fn alloc_str(&self, s: &str) -> &'mcx str {
        self.arena.alloc_str(s)
    }

    pub(crate) fn alloc_ast(&self, value: pairs::main<'mcx>) -> &'mcx pairs::main<'mcx> {
        self.arena.alloc(value)
    }

    pub(crate) fn collect_lints(&self) -> impl Iterator<Item = &'static rustc_lint::Lint> {
        self.symbol_tables
            .iter()
            .flat_map(|symbol_table| symbol_table.collect_lints())
    }
    /// Register the lints in the lint store.
    pub fn register_lints(&self, lint_store: &mut rustc_lint::LintStore) {
        lint_store.register_lints(&self.lints);
    }
}
