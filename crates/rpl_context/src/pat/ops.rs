use derive_more::derive::Debug;
use rustc_data_structures::fx::FxIndexMap;
use rustc_span::{Span, Symbol};

use crate::pat::{NonLocalMetaVars, Param, Ty};

/// A single function signature inside an op-group. Body-less.
#[derive(Debug)]
pub struct OpSignature<'pcx> {
    pub name: Symbol,
    pub params: Vec<Param<'pcx>>,
    pub ret: Option<Ty<'pcx>>,
    pub span: Span,
}

/// An abstract op group — a parameterized bundle of named MIR signatures.
#[derive(Debug)]
pub struct OpGroup<'pcx> {
    pub name: Symbol,
    pub meta_vars: NonLocalMetaVars<'pcx>,
    pub ops: FxIndexMap<Symbol, OpSignature<'pcx>>,
    pub span: Span,
}

/// The contents of the `ops { ... }` block.
#[derive(Debug, Default)]
pub struct OpsBlock<'pcx> {
    pub groups: FxIndexMap<Symbol, OpGroup<'pcx>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opsblock_default_is_empty() {
        let b = OpsBlock::default();
        assert!(b.groups.is_empty());
    }
}
