use std::fs::File;
use std::io;
use std::iter::Iterator;
use std::path::{Path, PathBuf};

use rpl_graphviz::{mir_cfg_to_graphviz, mir_ddg_to_graphviz};
use rustc_ast::token::{Token, TokenKind};
use rustc_ast::tokenstream::{TokenStreamIter, TokenTree};
use rustc_errors::{DiagArgValue, IntoDiagArg, MultiSpan};
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{self as hir};
use rustc_middle::hir::nested_filter::All;
use rustc_middle::ty::TyCtxt;
use rustc_middle::{mir, ty};
use rustc_span::symbol::kw;
use rustc_span::{ErrorGuaranteed, Span, Symbol};

pub fn visit_crate(tcx: TyCtxt<'_>) {
    let mut visitor = DebugVisitor::new(tcx);
    tcx.hir().walk_toplevel_module(&mut visitor);
    if !visitor.attrs.is_empty() {
        // tcx.dcx()
        //     .emit_err(crate::errors::AbortDueToDebugging::new(visitor.attrs));
        visitor.attrs.iter().for_each(|span| {
            tcx.dcx().emit_err(crate::errors::ErrorDueToDebugging { span: *span });
        });
    }
}

/// Dump or print HIR, or MIR for debugging.
///
/// Related attributes are:
/// - `#[rpl::dump_hir]`, which uses `std::fmt::Debug` for formatting.
/// - `#[rpl::print_hir]`, which uses `rustc_hir_pretty::id_to_string` for formatting.
/// - `#[rpl::dump_mir]`, which dumps the MIR of local or external functions.
///
/// # Example
/// ## HIR
/// ```ignore
/// #[rpl::dump_hir]
/// fn foo() {}
///
/// #[rpl::print_hir]
/// fn bar() {}
///
/// #[rpl::dump_hir]
/// trait Foo {}
///
/// #[rpl::print_hir]
/// trait Bar {}
///
/// fn test() {
///     #[rpl::dump_hir]
///     let x = 0;
///     #[rpl::print_hir]
///     let y = 0;
/// }
/// ```
/// ## MIR
/// ```ignore
/// #[rpl::dump_mir]
/// fn foo() {}
///
/// #[rpl::dump_mir]
/// fn test() {
///     let x = 0;
///     let y = 0;
/// }
///
/// fn external_functions() {
///     #[rpl::dump_mir]
///     let _ = std::alloc::alloc;
/// }
/// ```
struct DebugVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    attrs: Vec<Span>,
}

impl<'tcx> DebugVisitor<'tcx> {
    fn new(tcx: TyCtxt<'tcx>) -> Self {
        let attrs = Vec::new();
        Self { tcx, attrs }
    }
}

impl<'tcx> Visitor<'tcx> for DebugVisitor<'tcx> {
    type NestedFilter = All;

    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) -> Self::Result {
        self.debug_hir(item.hir_id());
        let _ = self.check_dump_mir_attrs(item.hir_id());
        intravisit::walk_item(self, item);
    }

    fn visit_trait_item(&mut self, item: &'tcx hir::TraitItem<'tcx>) -> Self::Result {
        self.debug_hir(item.hir_id());
        let _ = self.check_dump_mir_attrs(item.hir_id());
        intravisit::walk_trait_item(self, item)
    }

    fn visit_impl_item(&mut self, item: &'tcx hir::ImplItem<'tcx>) -> Self::Result {
        self.debug_hir(item.hir_id());
        let _ = self.check_dump_mir_attrs(item.hir_id());
        intravisit::walk_impl_item(self, item)
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) -> Self::Result {
        self.debug_hir(expr.hir_id);
        if let Ok(Some(attr)) = self.check_dump_mir_attrs(expr.hir_id) {
            self.debug_mir(expr, attr);
        }
        intravisit::walk_expr(self, expr)
    }

    fn visit_local(&mut self, local: &'tcx hir::LetStmt<'tcx>) -> Self::Result {
        let hir_id = self.tcx.parent_hir_id(local.hir_id);
        self.debug_hir(hir_id);
        if let Ok(Some(attr)) = self.check_dump_mir_attrs(hir_id) {
            let Some(init) = local.init else {
                self.tcx.dcx().emit_err(crate::errors::DumpMirExpectInit {
                    span: local.span,
                    missing: local.ty.map(|ty| ty.span).unwrap_or(local.pat.span).shrink_to_hi(),
                });
                return;
            };
            self.debug_mir(init, attr);
        }
        intravisit::walk_local(self, local)
    }

    fn visit_fn(
        &mut self,
        kind: intravisit::FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body_id: hir::BodyId,
        span: Span,
        def_id: LocalDefId,
    ) -> Self::Result {
        let hir_id = self.tcx.local_def_id_to_hir_id(def_id);
        if let Some((attr, DumpMirAllowed(true))) = self.get_dump_mir_attrs(hir_id) {
            let body = self.tcx.optimized_mir(def_id);
            dump_mir(self.tcx, body, span, &attr);
        }
        intravisit::walk_fn(self, kind, decl, body_id, def_id);
    }
}

struct DumpMirAllowed(bool);

fn find_attr<'a>(attrs: &'a [hir::Attribute], expected_attr: &str) -> Option<(&'a hir::AttrItem, Span)> {
    attrs.iter().find_map(|attr| {
        if let hir::AttrKind::Normal(normal_attr) = &attr.kind
            && normal_attr
                .path
                .segments
                .iter()
                .map(|ident| ident.as_str())
                .eq(expected_attr.split("::"))
        {
            return Some((normal_attr.as_ref(), attr.span));
        }
        None
    })
}

fn contains_attr(attrs: &[hir::Attribute], expected_attr: &str) -> Option<Span> {
    find_attr(attrs, expected_attr).map(|(_, span)| span)
}

macro_rules! dump_mir_options {
    ($($name:ident: $ty:ty = $default:expr),* $(,)?) => {
        #[derive(Debug)]
        struct DumpMirOptions {
            $( $name: $ty, )*
        }

        impl Default for DumpMirOptions {
            fn default() -> Self {
                Self {
                    $( $name: $default, )*
                }
            }
        }
        impl DumpMirOptions {
            $(
            #[allow(non_upper_case_globals)]
            const $name: &'static str = stringify!($name);
            )*
        }
    };
}

dump_mir_options! {
    include_extra_comments: bool = true,
    dump_cfg: bool = false,
    dump_ddg: bool = false,
}

struct DumpMirAttr {
    span: Span,
    options: DumpMirOptions,
}

fn contains_dump_mir(attrs: &[hir::Attribute]) -> Option<DumpMirAttr> {
    find_attr(attrs, DUMP_MIR).map(|(attr, span)| {
        let mut options = DumpMirOptions::default();
        match &attr.args {
            hir::AttrArgs::Empty => {},
            hir::AttrArgs::Delimited(delim_args) => {
                let mut trees = delim_args.tokens.iter();
                fn eat_ident(trees: &mut TokenStreamIter<'_>) -> Option<Symbol> {
                    match trees.next() {
                        Some(TokenTree::Token(token, _)) => token.ident().map(|(ident, _)| ident.name),
                        _ => None,
                    }
                }
                fn matches_token(token: Option<&TokenTree>, f: impl FnOnce(&Token) -> bool) -> bool {
                    matches!(token, Some(TokenTree::Token(token, _)) if f(token))
                }
                fn matches_token_kind(token: Option<&TokenTree>, kind: &TokenKind) -> bool {
                    matches_token(token, |token| &token.kind == kind)
                }
                fn eat_token_kind(trees: &mut TokenStreamIter<'_>, kind: TokenKind) -> Option<TokenKind> {
                    matches_token_kind(trees.next(), &kind).then_some(kind)
                }
                fn eat_eq_bool(trees: &mut TokenStreamIter<'_>) -> Option<bool> {
                    if !matches_token_kind(trees.peek(), &TokenKind::Eq) {
                        return None;
                    }
                    match trees.nth(1) {
                        Some(TokenTree::Token(token, _)) if token.is_bool_lit() => Some(token.is_ident_named(kw::True)),
                        _ => None,
                    }
                }
                while let Some(()) = try {
                    let name = eat_ident(&mut trees)?;
                    let value = eat_eq_bool(&mut trees).unwrap_or(true);
                    match name.as_str() {
                        DumpMirOptions::include_extra_comments => options.include_extra_comments = value,
                        DumpMirOptions::dump_cfg => options.dump_cfg = value,
                        DumpMirOptions::dump_ddg => options.dump_ddg = value,
                        _ => {},
                    }
                    eat_token_kind(&mut trees, TokenKind::Comma)?;
                } {}
            },
            hir::AttrArgs::Eq { .. } => {},
        };
        DumpMirAttr { span, options }
    })
}

impl DebugVisitor<'_> {
    fn debug_hir(&mut self, hir_id: hir::HirId) {
        let attrs = self.tcx.hir().attrs(hir_id);
        let span = self.tcx.hir().span(hir_id);
        if let Some(attr_span) = contains_attr(attrs, DUMP_HIR) {
            self.attrs.push(attr_span);
            self.tcx.dcx().emit_note(crate::errors::DumpOrPrintDiag {
                message: format!("{:#?}", self.tcx.hir_node(hir_id)),
                span,
                attr_span,
                kind: DumpOrPrintDiagKind::DumpHir,
            });
        }
        if let Some(attr_span) = contains_attr(attrs, PRINT_HIR) {
            self.attrs.push(attr_span);
            let mut message = rustc_hir_pretty::id_to_string(&self.tcx.hir(), hir_id);
            if message.is_empty() {
                message = self.tcx.hir().node_to_string(hir_id);
            } else {
                message = format!("{hir_id:?} (`{message}`)");
            }
            self.tcx.dcx().emit_note(crate::errors::DumpOrPrintDiag {
                message,
                span,
                attr_span,
                kind: DumpOrPrintDiagKind::PrintHir,
            });
        }
    }
    fn get_dump_mir_attrs(&self, hir_id: hir::HirId) -> Option<(DumpMirAttr, DumpMirAllowed)> {
        contains_dump_mir(self.tcx.hir().attrs(hir_id)).map(|attr| {
            let dump_mir_allowed = matches!(
                self.tcx.hir_node(hir_id),
                hir::Node::Stmt(hir::Stmt {
                    kind: hir::StmtKind::Let(_),
                    ..
                }) | hir::Node::TraitItem(hir::TraitItem {
                    kind: hir::TraitItemKind::Fn(..),
                    ..
                }) | hir::Node::ImplItem(hir::ImplItem {
                    kind: hir::ImplItemKind::Fn(..),
                    ..
                }) | hir::Node::Item(hir::Item {
                    kind: hir::ItemKind::Fn { .. },
                    ..
                }),
            );
            (attr, DumpMirAllowed(dump_mir_allowed))
        })
    }
    fn check_dump_mir_attrs(&mut self, hir_id: hir::HirId) -> Result<Option<DumpMirAttr>, ErrorGuaranteed> {
        if let Some((attr, DumpMirAllowed(allowed))) = self.get_dump_mir_attrs(hir_id) {
            self.attrs.push(attr.span);
            return if allowed {
                Ok(Some(attr))
            } else {
                Err(self
                    .tcx
                    .dcx()
                    .emit_err(crate::errors::DumpMirInvalid(self.tcx.hir().span_with_body(hir_id))))
            };
        }
        Ok(None)
    }
    fn debug_mir<'tcx>(&self, expr: &'tcx hir::Expr<'tcx>, attr: DumpMirAttr) {
        let (mut def_id, args) = if let hir::ExprKind::Closure(closure) = expr.kind {
            (closure.def_id.to_def_id(), None)
        } else if let &ty::FnDef(def_id, args) = self.tcx.typeck(expr.hir_id.owner.def_id).expr_ty(expr).kind() {
            (def_id, Some(args))
        } else {
            self.tcx.dcx().emit_err(crate::errors::DumpMirNotFnPath(expr.span));
            return;
        };

        let args = args.unwrap_or_else(|| ty::GenericArgs::identity_for_item(self.tcx, def_id));
        if let Ok(Some(instance)) = ty::Instance::try_resolve(
            self.tcx,
            // self.tcx.param_env_reveal_all_normalized(expr.hir_id.owner.def_id),
            ty::TypingEnv::post_analysis(self.tcx, expr.hir_id.owner.def_id),
            def_id,
            args,
        ) {
            def_id = instance.def.def_id();
            if !self.tcx.is_mir_available(def_id) {
                self.tcx.dcx().emit_err(crate::errors::DumpMirNotAvailable {
                    instance: instance.into(),
                    span: expr.span,
                });
                return;
            }
        }

        let body = self.tcx.optimized_mir(def_id);
        dump_mir(self.tcx, body, expr.span, &attr);
    }
}

static PRINT_HIR: &str = "rpl::print_hir";
static DUMP_HIR: &str = "rpl::dump_hir";
static DUMP_MIR: &str = "rpl::dump_mir";

pub(crate) enum DumpOrPrintDiagKind {
    DumpHir,
    PrintHir,
}

impl IntoDiagArg for DumpOrPrintDiagKind {
    fn into_diag_arg(self) -> DiagArgValue {
        match self {
            Self::DumpHir => "dump_hir",
            Self::PrintHir => "print_hir",
        }
        .into_diag_arg()
    }
}

fn dump_mir<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, span: Span, attr: &DumpMirAttr) {
    let blocks = dump_mir_blocks(body);
    let locals_and_source_scopes = dump_mir_locals_and_source_scopes(body);

    let def_id = body.source.def_id();
    let files = dump_mir_to_file(tcx, body, &attr.options).map_or(Vec::new(), |path| {
        let mut files = vec![crate::errors::DumpMirFile {
            file: path.display().to_string(),
            content: "MIR",
        }];
        if attr.options.dump_cfg
            && let Ok(path) = dump_mir_cfg_to_file(body, &path)
        {
            files.push(crate::errors::DumpMirFile {
                file: path.display().to_string(),
                content: "control flow graph",
            });
        }
        if attr.options.dump_ddg
            && let Ok(path) = dump_mir_ddg_to_file(body, &path)
        {
            files.push(crate::errors::DumpMirFile {
                file: path.display().to_string(),
                content: "data dependency graph",
            });
        }
        files
    });
    tcx.dcx().emit_note(crate::errors::DumpMir {
        span,
        def_id: def_id.into(),
        files,
        attr_span: attr.span,
        locals_and_source_scopes,
        blocks,
    });
}

fn dump_mir_to_file<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, options: &DumpMirOptions) -> io::Result<PathBuf> {
    use filepath::FilePath;
    let mut file = mir::pretty::create_dump_file(tcx, "mir", false, "dump_mir", &"", body)?;
    mir::pretty::write_mir_fn(
        tcx,
        body,
        &mut |_, _| Ok(()),
        &mut file,
        mir::pretty::PrettyPrintMirOptions {
            include_extra_comments: options.include_extra_comments,
        },
    )?;
    file.get_ref().path()
}

fn dump_mir_cfg_to_file(body: &mir::Body<'_>, path: &Path) -> std::io::Result<PathBuf> {
    let mut path = path.to_path_buf();
    path.add_extension("cfg.dot");
    let mut file = File::create(&path)?;
    mir_cfg_to_graphviz(body, &mut file, &Default::default())?;
    Ok(path)
}

fn dump_mir_ddg_to_file(body: &mir::Body<'_>, path: &Path) -> std::io::Result<PathBuf> {
    let mut path = path.to_path_buf();
    path.add_extension("ddg.dot");
    let mut file = File::create(&path)?;
    mir_ddg_to_graphviz(body, &mut file, &Default::default())?;
    Ok(path)
}

fn dump_mir_locals_and_source_scopes(body: &mir::Body<'_>) -> crate::errors::DumpMirLocalsAndSourceScopes {
    let mut multi_span = MultiSpan::from_span(body.span);
    for (local, local_decl) in body.local_decls.iter_enumerated() {
        let scope = local_decl.source_info.scope;
        let ty = local_decl.ty;
        #[allow(rustc::untranslatable_diagnostic)]
        multi_span.push_span_label(local_decl.source_info.span, format!("{local:?}: {ty}; // {scope:?}"));
    }
    for (ss, scope_data) in body.source_scopes.iter_enumerated() {
        #[allow(rustc::untranslatable_diagnostic)]
        multi_span.push_span_label(scope_data.span, format!("{ss:?}"));
        if let Some((inlined, _)) = scope_data.inlined {
            #[allow(rustc::untranslatable_diagnostic)]
            multi_span.push_span_label(scope_data.span, inlined.to_string());
        }
    }
    crate::errors::DumpMirLocalsAndSourceScopes { multi_span }
}

fn dump_mir_blocks(body: &mir::Body<'_>) -> Vec<crate::errors::DumpMirBlock> {
    body.basic_blocks.iter_enumerated().map(dump_mir_block).collect()
}

fn dump_mir_block((bb, block_data): (mir::BasicBlock, &mir::BasicBlockData<'_>)) -> crate::errors::DumpMirBlock {
    let mut block = format!("{bb:?}: {{\n");
    let indent = "    ";
    let mut multi_span = MultiSpan::from_span(block_data.terminator().source_info.span);
    for (dbg, source_info) in block_data
        .statements
        .iter()
        .map(|stmt| (stmt as &dyn std::fmt::Debug, stmt.source_info))
        .chain(std::iter::once({
            let term = block_data.terminator();
            (&term.kind as &dyn std::fmt::Debug, term.source_info)
        }))
    {
        let scope = source_info.scope;
        block.push_str(indent);
        let label = format!("{dbg:?}; // {scope:?}");
        block.push_str(&label);
        block.push('\n');
        #[allow(rustc::untranslatable_diagnostic)]
        multi_span.push_span_label(source_info.span, label);
    }
    block.push('}');
    crate::errors::DumpMirBlock { block, multi_span }
}

/*
impl crate::errors::AbortDueToDebugging {
    fn new(spans: Vec<Span>) -> Self {
        let suggs = spans.iter().copied().map(Into::into).collect();
        Self {
            span: spans.into(),
            suggs,
        }
    }
}

impl From<Span> for crate::errors::AbortDueToDebuggingSugg {
    fn from(span: Span) -> Self {
        Self { span }
    }
}
*/

impl From<Span> for crate::errors::ErrorDueToDebugging {
    fn from(span: Span) -> Self {
        Self { span }
    }
}
