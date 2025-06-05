use rustc_middle::mir::{Body, Const, PlaceRef};
use rustc_middle::ty::Ty;
use rustc_span::Span;

use super::LabelMap;
use super::non_local_meta_vars::{ConstVarIdx, PlaceVarIdx, TyVarIdx};

pub trait Matched<'tcx> {
    fn span(&self, labels: &LabelMap, body: &Body<'tcx>, name: &str) -> Span;
    fn type_meta_var(&self, idx: TyVarIdx) -> Ty<'tcx>;
    fn const_meta_var(&self, idx: ConstVarIdx) -> Const<'tcx>;
    fn place_meta_var(&self, idx: PlaceVarIdx) -> PlaceRef<'tcx>;
}
