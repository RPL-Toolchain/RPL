use either::Either;
use rpl_match::resolve::{PatItemKind, def_path_res};
use rustc_hir::def::Res;
use rustc_hir::intravisit::Visitor;
use rustc_hir::{
    self as hir, BodyId, ImplItem, ImplItemKind, Item, ItemKind, Node, QPath, TraitFn, TraitItem, TraitItemKind,
};
use rustc_middle::ty::TyCtxt;
use rustc_span::{Span, Symbol};

// Make sure a mir statement is translated from a hir function
// Example: make sure the following code is translated from `std::mem::transmute`
// ```rust
// let _2: bool = move _1 as bool (Transmute);
// ```
// Similar to `rustc_trait_selection::error_reporting::traits::FindExprBySpan`
struct FindExprBySpanAndFnPath<'tcx> {
    span: Span, // the matched mir statement's span
    // path: Vec<Symbol>,
    resolved_res: Vec<Res>, // the path of the function to be translated
    tcx: TyCtxt<'tcx>,
    result: Option<&'tcx hir::Expr<'tcx>>,
}

impl<'tcx> FindExprBySpanAndFnPath<'tcx> {
    pub fn new(span: Span, path: &str, tcx: TyCtxt<'tcx>) -> Self {
        let path = path.split("::").map(Symbol::intern).collect::<Vec<_>>();
        let resolved_res = def_path_res(tcx, &path, PatItemKind::Fn);
        Self {
            span,
            resolved_res,
            tcx,
            result: None,
        }
    }
}

impl<'v> Visitor<'v> for FindExprBySpanAndFnPath<'v> {
    type NestedFilter = rustc_middle::hir::nested_filter::OnlyBodies;

    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    fn visit_expr(&mut self, ex: &'v hir::Expr<'v>) {
        if self.span == ex.span
            && let hir::ExprKind::Call(callee, _args) = ex.kind
            && let hir::ExprKind::Path(path) = callee.kind
            && let QPath::Resolved(_, path) = path
            && self.resolved_res.contains(&path.res)
        {
            self.result = Some(ex);
            debug!("mir_span: {:?}", self.span);
            debug!("expr_span: {:?}", ex.span);
            debug!("resolved_res: {:?}", self.resolved_res);
            debug!("path.res: {:?}", path.res);
        } else {
            hir::intravisit::walk_expr(self, ex);
        }
    }
}

pub fn get_body_id_from_hir_node(node: Node<'_>) -> Option<BodyId> {
    match node {
        Node::TraitItem(TraitItem {
            kind: TraitItemKind::Fn(_fn_sig, TraitFn::Provided(body_id)),
            ..
        }) => Some(*body_id),
        Node::ImplItem(ImplItem {
            kind: ImplItemKind::Fn(_fn_sig, body_id),
            ..
        }) => Some(*body_id),
        Node::Item(Item {
            kind:
                ItemKind::Fn {
                    sig: _,
                    generics: _,
                    body: body_id,
                    has_body: _,
                },
            ..
        }) => Some(*body_id),
        _ => None,
    }
}

pub fn translate_from_hir_function(
    tcx: TyCtxt<'_>,
    mir_location: rustc_middle::mir::Location,
    body: &rustc_middle::mir::Body<'_>,
    hir_fn_path: &str,
) -> bool {
    let mir_stmt = body.stmt_at(mir_location);
    let (span, scope) = match mir_stmt {
        Either::Left(stmt) => (stmt.source_info.span, stmt.source_info.scope),
        Either::Right(terminator) => (terminator.source_info.span, terminator.source_info.scope),
    };
    let Some(hir_id) = scope.lint_root(&body.source_scopes) else {
        return false;
    };
    let Some(body_id) = get_body_id_from_hir_node(tcx.hir_node(hir_id)) else {
        return false;
    };

    let mut expr_finder = FindExprBySpanAndFnPath::new(span, hir_fn_path, tcx);
    expr_finder.visit_expr(tcx.hir().body(body_id).value);
    let Some(expr) = expr_finder.result else {
        return false;
    };
    trace!("expr: {:?}", expr);
    true
}
