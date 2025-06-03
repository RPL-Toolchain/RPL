use crate::RPLMetaError;
use crate::context::MetaContext;
use crate::symbol_table::{FnInner, ImplInner, NonLocalMetaSymTab};
use parser::pairs;
use rustc_data_structures::fx::FxHashMap;
use rustc_span::Symbol;
use std::sync::Arc;

use super::CheckFnCtxt;

pub(super) struct CheckImplCtxt<'i, 'r> {
    pub(super) meta_vars: Arc<NonLocalMetaSymTab>,
    pub(super) impl_def: &'r mut ImplInner<'i>,
    pub(super) imports: &'r FxHashMap<Symbol, &'i pairs::Path<'i>>,
    pub(super) errors: &'r mut Vec<RPLMetaError<'i>>,
}

impl<'i, 'r> CheckImplCtxt<'i, 'r> {
    pub(super) fn check_impl(&mut self, mctx: &MetaContext<'i>, rust_impl: &'i pairs::Impl<'i>) {
        let (_, _, _, _, fns, _) = rust_impl.get_matched();
        for rust_fn in fns.iter_matched() {
            let (fn_name, mut fn_def) = FnInner::parse_from(mctx, rust_fn.FnSig().FnName(), None, self.imports);
            let meta_vars = self.meta_vars.clone();
            CheckFnCtxt {
                meta_vars: meta_vars.clone(),
                impl_def: None, //FIXME
                fn_def: &mut fn_def,
                imports: self.imports,
                errors: self.errors,
            }
            .check_fn(mctx, rust_fn);
            if let Some(ident) = fn_name {
                self.impl_def.add_fn(ident, (fn_def, meta_vars).into()).unwrap(); //FIXME: handle errors
            }
        }
    }
}
