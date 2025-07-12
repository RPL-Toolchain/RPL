use std::marker::PhantomData;
use std::sync::LazyLock;

use parser::{SpanWrapper, collect_elems_separated_by_comma, pairs};
use rustc_hash::FxHashMap;
use rustc_lint::{Level, Lint};
use rustc_span::Symbol;
use sync_arena::declare_arena;

use crate::context::MetaContext;
use crate::error::RPLMetaError;
use crate::utils::Ident;

declare_arena!(
    [
        [] _phantom: &'tcx (),
    ]
);

static ARENA: LazyLock<Arena<'static>> = LazyLock::new(Arena::default);

pub type DiagSymbolTables<'i> = FxHashMap<Symbol, DiagSymbolTable<'i>>;

#[derive(Default)]
pub struct DiagSymbolTable<'i> {
    diags: FxHashMap<Symbol, String>,
    /// The lints that are registered in this diag symbol table.
    ///
    /// # Note
    ///
    /// Keep this sorted by the `name` of the lint after collecting all lints,
    /// so that the lints are retrieved in later passes in a performant way.
    lints: Vec<&'static rustc_lint::Lint>,
    _phantom: PhantomData<&'i ()>,
}

impl<'i> DiagSymbolTable<'i> {
    pub fn collect_symbol_tables(
        mctx: &MetaContext<'i>,
        diags: impl Iterator<Item = &'i pairs::diagBlockItem<'i>>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> FxHashMap<Symbol, Self> {
        let mut diag_symbols = FxHashMap::default();
        for diag in diags {
            let name = diag.get_matched().0;
            let symbol_table = Self::collect_diag_symbol_table(mctx, diag, errors);
            _ = diag_symbols
                .try_insert(Symbol::intern(name.span.as_str()), symbol_table)
                .map_err(|entry| {
                    let ident = *entry.entry.key();
                    let err = RPLMetaError::SymbolAlreadyDeclared {
                        ident,
                        span: SpanWrapper::new(name.span, mctx.get_active_path()),
                    };
                    errors.push(err);
                });
        }
        diag_symbols
    }

    fn collect_diag_symbol_table(
        mctx: &MetaContext<'i>,
        diag: &'i pairs::diagBlockItem<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) -> Self {
        let mut diag_symbol_table = DiagSymbolTable::default();
        let (_, _, _, items, messages, _) = diag.get_matched();

        diag_symbol_table.get_lint(items, mctx, errors);

        if let Some(messages) = messages {
            let messages = collect_elems_separated_by_comma!(messages);
            for message in messages {
                let (ident, _, string) = message.get_matched();
                diag_symbol_table.add_diag(mctx, &ident.into(), string.span.to_string(), errors);
            }
        }

        diag_symbol_table.lints.sort_by_key(|lint| lint.name);
        diag_symbol_table.lints.windows(2).for_each(|pair| {
            if pair[0].name == pair[1].name {
                errors.push(RPLMetaError::DuplicateLint {
                    name: pair[0].name,
                    span: SpanWrapper::new(diag.span, mctx.get_active_path()),
                });
            }
        });

        diag_symbol_table
    }

    fn get_lint(
        &mut self,
        items: &'i pairs::diagItems<'i>,
        mctx: &MetaContext<'i>,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        let mut name = None;
        let mut level = None;
        for item in collect_elems_separated_by_comma!(items) {
            let (key, _, _, value) = item.get_matched();
            let key_str = key.span.as_str();
            match key_str {
                "name" => {
                    let value_str = value.diagMessageInner().span.as_str();
                    name = Some(format!("rpl::{}", value_str));
                },
                "level" => {
                    let value_str = value.diagMessageInner().span.as_str();
                    if let Some(level_) = Level::from_str(value_str) {
                        level = Some(level_);
                    } else {
                        return errors.push(RPLMetaError::InvalidPropertyInDiag {
                            property: "level",
                            value: value_str,
                            span: SpanWrapper::new(value.span, mctx.get_active_path()),
                        });
                    }
                },
                "primary" | "label" | "note" | "help" | "suggestion" => {
                    // These are not used in the symbol table, but we can collect them if needed.
                    // For now, we just ignore them.
                },
                _ => {
                    return errors.push(RPLMetaError::UnknownPropertyInDiag {
                        property: key_str,
                        span: SpanWrapper::new(key.span, mctx.get_active_path()),
                    });
                },
            }
        }

        let level = level.unwrap_or(Level::Deny);
        let name = if let Some(name) = name {
            ARENA.alloc_str(&name)
        } else {
            return errors.push(RPLMetaError::MissingPropertyInDiag {
                property: "name",
                span: SpanWrapper::new(items.span, mctx.get_active_path()),
            });
        };

        let lint = rustc_lint::Lint {
            name,
            default_level: level,
            ..rustc_lint::Lint::default_fields_for_macro()
        };

        let lint = ARENA.alloc(lint);
        self.lints.push(lint);
    }

    fn add_diag(
        &mut self,
        mctx: &MetaContext<'i>,
        ident: &Ident<'i>,
        message: String,
        errors: &mut Vec<RPLMetaError<'i>>,
    ) {
        _ = self
            .diags
            .try_insert(Symbol::intern(ident.span.as_str()), message)
            .map_err(|_entry| {
                let err = RPLMetaError::SymbolAlreadyDeclared {
                    ident: ident.name,
                    span: SpanWrapper::new(ident.span, mctx.get_active_path()),
                };
                errors.push(err);
            });
    }

    pub(crate) fn collect_lints(&self) -> &[&'static Lint] {
        &self.lints
    }

    pub fn get(&self, name: &str) -> Option<&'static Lint> {
        self.lints
            .binary_search_by(|lint| lint.name.strip_prefix("rpl::").unwrap_or(lint.name).cmp(name))
            .ok()
            .map(|idx| self.lints[idx])
    }

    pub fn lints(&self) -> &[&'static Lint] {
        &self.lints
    }
}
