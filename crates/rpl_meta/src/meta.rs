use crate::context::MetaContext;
use crate::error::RPLMetaError;
use crate::idx::RPLIdx;
use crate::symbol_table::{DiagSymbolTable, SymbolTable};
use parser::pairs;
use rustc_data_structures::fx::FxHashMap;
use rustc_span::Symbol;
use std::path::Path;

/// Meta data of a single rpl file.
pub struct SymbolTables<'mcx> {
    /// Absolute path to the rpl file
    pub path: &'mcx Path,
    /// RPL Idx
    pub idx: RPLIdx,
    /// The name of the rpl file
    pub name: Symbol,
    /// The symbol table of the util block
    pub util_symbol_tables: FxHashMap<Symbol, SymbolTable<'mcx>>,
    /// The symbol table of the patt block
    pub patt_symbol_tables: FxHashMap<Symbol, SymbolTable<'mcx>>,
    /// The symbol table of the diag block
    pub diag_symbol_tables: FxHashMap<Symbol, DiagSymbolTable>,
    /// errors
    pub errors: Vec<RPLMetaError<'mcx>>,
}

impl<'mcx> SymbolTables<'mcx> {
    /// Collect the meta data of a parsed rpl file
    pub fn collect(path: &'mcx Path, main: &'mcx pairs::main<'mcx>, idx: RPLIdx, mctx: &MetaContext<'mcx>) -> Self {
        let mut errors = Vec::new();
        // Collect the pattern name of the rpl file.
        let name = Self::collect_rpl_pattern_name(main);
        // Collect the blocks.
        let (utils, patts, diags) = collect_blocks(main);
        // Collect the symbol table of the util blocks.
        let util_items = utils.iter().flat_map(|util| util.get_matched().3.iter_matched());
        let util_symbol_tables = SymbolTable::collect_symbol_tables(mctx, &[], util_items, &mut errors);
        // Collect the symbol table of the patt blocks.
        let patt_imports = patts.iter().flat_map(|patt| patt.get_matched().2.iter_matched());
        let patt_items = patts.iter().flat_map(|patt| patt.get_matched().3.iter_matched());
        let patt_symbol_tables =
            SymbolTable::collect_symbol_tables(mctx, &patt_imports.collect::<Vec<_>>(), patt_items, &mut errors);
        // Collect the symbol table of the diag blocks.
        let diag_items = diags.iter().flat_map(|diag| diag.get_matched().2.iter_matched());
        let diag_symbol_tables = DiagSymbolTable::collect_symbol_tables(mctx, diag_items, &mut errors);
        SymbolTables {
            path,
            name,
            idx,
            util_symbol_tables,
            patt_symbol_tables,
            diag_symbol_tables,
            errors,
        }
    }

    fn collect_rpl_pattern_name(main: &pairs::main<'mcx>) -> Symbol {
        let rpl_pattern = main.get_matched().1;
        let rpl_header = rpl_pattern.get_matched().0;
        let name = rpl_header.get_matched().1.span.as_str();
        Symbol::intern(name)
    }
}

impl<'mcx> SymbolTables<'mcx> {
    /// Show the errors of the symbol tables.
    pub fn show_error(&self, mut handler: impl FnMut(&RPLMetaError<'mcx>)) {
        if !self.errors.is_empty() {
            warn!(
                "{:?} generated {} error{}.",
                self.path,
                self.errors.len(),
                if self.errors.len() > 1 { "s" } else { "" }
            );

            for error in &self.errors {
                // FIXME: a better way to print the error
                handler(error);
            }
        } else {
            info!("No error found in {:?}", self.path);
        }
    }
}

pub fn collect_blocks<'mcx, 'i>(
    main: &'mcx pairs::main<'i>,
) -> (
    Vec<&'mcx pairs::utilBlock<'i>>,
    Vec<&'mcx pairs::pattBlock<'i>>,
    Vec<&'mcx pairs::diagBlock<'i>>,
) {
    let mut utils = Vec::new();
    let mut patts = Vec::new();
    let mut diags = Vec::new();

    let blocks = main.get_matched().1.get_matched().1;
    let blocks = blocks.iter_matched();

    for block in blocks {
        if let Some(util) = block.utilBlock() {
            utils.push(util);
        } else if let Some(patt) = block.pattBlock() {
            patts.push(patt);
        } else if let Some(diag) = block.diagBlock() {
            diags.push(diag);
        }
    }

    (utils, patts, diags)
}
