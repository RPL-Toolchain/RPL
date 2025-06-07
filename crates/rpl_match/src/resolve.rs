//! Resolve an item path.
//!
//! See <https://doc.rust-lang.org/nightly/nightly-rustc/src/clippy_utils/lib.rs.html#691>
use rpl_context::{PatCtxt, pat};
use rpl_resolve::{PatItemKind, def_path_res};
use rustc_hir::LangItem;
use rustc_hir::def::Res;
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;

#[instrument(level = "debug", skip(pcx, tcx), ret)]
pub fn ty_res<'tcx, 'pcx>(
    pcx: PatCtxt<'pcx>,
    tcx: TyCtxt<'tcx>,
    path: &[Symbol],
    args: pat::GenericArgsRef<'pcx>,
) -> Option<pat::Ty<'pcx>> {
    let res = def_path_res(tcx, path, PatItemKind::Type);
    let res: Vec<_> = res
        .into_iter()
        .filter_map(|res| match res {
            Res::Def(_, def_id) => pat::Ty::from_ty_lossy(pcx, tcx.type_of(def_id).instantiate_identity(), args),
            // Res::Def(_, def_id) => pat::Ty::from_ty_lossy(pcx, tcx.type_of(def_id).instantiate(tcx, args)),
            Res::PrimTy(prim_ty) => args.is_empty().then(|| pat::Ty::from_prim_ty(pcx, prim_ty)),
            Res::SelfTyParam { .. }
            | Res::SelfTyAlias { .. }
            | Res::SelfCtor(..)
            | Res::Local(_)
            | Res::ToolMod
            | Res::NonMacroAttr(..)
            | Res::Err => None,
        })
        .collect();
    //FIXME: implement `PartialEq` correctly for `pat::Ty` so that we can deduplicate `res`
    // res.dedup();
    if res.len() > 1 {
        info!(?res, "ambiguous type path");
    }
    res.first().copied()
}

pub fn lang_item_res<'pcx>(pcx: PatCtxt<'pcx>, tcx: TyCtxt<'_>, item: LangItem) -> Option<pat::Ty<'pcx>> {
    tcx.lang_items()
        .get(item)
        .map(|def_id| pat::Ty::from_def(pcx, def_id, pat::GenericArgsRef(&[])))
}
