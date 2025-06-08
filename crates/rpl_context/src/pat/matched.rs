use std::collections::HashMap;

use rpl_meta::collect_elems_separated_by_comma;
use rpl_parser::generics::{Choice2, Choice3};
use rpl_parser::pairs;
use rustc_index::IndexVec;
use rustc_middle::mir::{Body, Const, PlaceRef};
use rustc_middle::ty::Ty;
use rustc_span::{Span, Symbol};

use super::non_local_meta_vars::{ConstVarIdx, PlaceVarIdx, TyVarIdx};
use crate::pat::NonLocalMetaVars;

pub trait Matched<'tcx> {
    fn span(&self, body: &Body<'tcx>, name: &str) -> Span;
    fn type_meta_var(&self, idx: TyVarIdx) -> Ty<'tcx>;
    fn const_meta_var(&self, idx: ConstVarIdx) -> Const<'tcx>;
    fn place_meta_var(&self, idx: PlaceVarIdx) -> PlaceRef<'tcx>;
}

/// - Key: indices/names in destination
/// - Value: indices/names in source
#[derive(Debug, PartialEq, Eq)]
pub struct MatchedMap {
    pub ty_vars: IndexVec<TyVarIdx, TyVarIdx>,
    pub const_vars: IndexVec<ConstVarIdx, ConstVarIdx>,
    pub place_vars: IndexVec<PlaceVarIdx, PlaceVarIdx>,
    pub labels: HashMap<Symbol, Symbol>,
}

impl MatchedMap {
    #[instrument(level = "trace", skip(configuration), ret)]
    pub fn new(
        target: &NonLocalMetaVars<'_>,
        source: &NonLocalMetaVars<'_>,
        configuration: &pairs::MetaVariableAssignList<'_>,
    ) -> Self {
        let mut vars: HashMap<Symbol, Symbol> = HashMap::new();
        let mut labels: HashMap<Symbol, Symbol> = HashMap::new();

        if let Some(configuration) = configuration.MetaVariableAssignsSeparatedByComma() {
            let assigns = collect_elems_separated_by_comma!(configuration);
            for assign in assigns {
                match &**assign {
                    Choice2::_0(assign) => {
                        let (source_var, _, target_var) = assign.get_matched();
                        match target_var {
                            Choice3::_0(_target_label) => todo!(),
                            Choice3::_1(target_var) => {
                                let target_var = Symbol::intern(target_var.span.as_str());
                                let source_var = Symbol::intern(source_var.span.as_str());
                                vars.try_insert(target_var, source_var).unwrap();
                            },
                            Choice3::_2(_) => todo!(),
                        }
                    },
                    Choice2::_1(assign) => {
                        let (source_label, _, target_label) = assign.get_matched();
                        let target_label = Symbol::intern(target_label.LabelName().span.as_str());
                        let source_label = Symbol::intern(source_label.LabelName().span.as_str());
                        labels.try_insert(source_label, target_label).unwrap();
                    },
                }
            }
        }
        trace!(vars = ?vars, labels = ?labels);
        MatchedMap {
            ty_vars: target
                .ty_vars
                .iter_enumerated()
                .map(|(idx, var)| {
                    source
                        .ty_vars
                        .iter()
                        .find_map(|source_var| {
                            (&source_var.name == vars.get(&var.name).unwrap_or(&var.name)).then_some(idx)
                        })
                        .unwrap()
                })
                .collect(),
            const_vars: target
                .const_vars
                .iter_enumerated()
                .map(|(idx, var)| {
                    source
                        .const_vars
                        .iter()
                        .find_map(|source_var| {
                            (&source_var.name == vars.get(&var.name).unwrap_or(&var.name)).then_some(idx)
                        })
                        .unwrap()
                })
                .collect(),
            place_vars: target
                .place_vars
                .iter_enumerated()
                .map(|(idx, var)| {
                    source
                        .place_vars
                        .iter()
                        .find_map(|source_var| {
                            (&source_var.name == vars.get(&var.name).unwrap_or(&var.name)).then_some(idx)
                        })
                        .unwrap()
                })
                .collect(),
            labels,
        }
    }
}
