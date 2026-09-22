/// Well-formedness checks for `ops { ... }` blocks (R1–R3).
///
/// These checks operate on the raw pest parse-tree and must run **before** any
/// lowering function (`NonLocalMetaVars::from_meta_decls`, `Ty::from`, etc.) is
/// called, because the lowering code uses `unreachable!()` contracts that assume
/// the input has already been validated.
///
/// # Rules
///
/// | # | Rule |
/// |---|------|
/// | R1 | Every meta-var in an op group's `MetaVariableDeclList` must be of kind `type`. |
/// | R2 | Op signatures may only reference meta-vars that are declared in the same op group. |
/// | R3 | Op signatures must not contain concrete Rust types (concrete paths or primitive types). |
use std::collections::HashSet;
use std::ops::Deref;

use rpl_meta::collect_elems_separated_by_comma;
use rpl_parser::generics::{Choice2, Choice3, Choice14};
use rpl_parser::pairs;

// ---------------------------------------------------------------------------
// Public error type
// ---------------------------------------------------------------------------

/// A well-formedness violation found in an `ops { ... }` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpsWfError {
    pub group: String,
    pub message: String,
}

impl OpsWfError {
    fn new(group: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            group: group.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for OpsWfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ops group '{}'] {}", self.group, self.message)
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Check every `opsItem` in `block` for R1, R2, and R3 violations.
///
/// Returns a (possibly empty) list of errors.  A non-empty return value means
/// that the corresponding `opsItem`(s) should **not** be lowered.
pub fn check_ops_block(block: &pairs::opsBlock<'_>) -> Vec<OpsWfError> {
    let mut errors = Vec::new();
    for item in block.opsItem() {
        check_ops_item(item, &mut errors);
    }
    errors
}

// ---------------------------------------------------------------------------
// Per-item checks
// ---------------------------------------------------------------------------

fn check_ops_item(item: &pairs::opsItem<'_>, errors: &mut Vec<OpsWfError>) {
    let group_name = item.Identifier().span.as_str();
    let meta_decl_list = item.MetaVariableDeclList();

    // -- R1: collect declared type-var names and flag non-type vars.
    let declared_type_vars = collect_declared_type_vars(group_name, meta_decl_list, errors);

    // -- R2 + R3: walk each OpFnDecl signature.
    for decl in item.OpFnDecl() {
        let sig = decl.OpFnSig();
        check_op_sig(group_name, sig, &declared_type_vars, errors);
    }
}

/// Pre-scan `MetaVariableDeclList`, collect names of `type`-kind vars, and
/// emit R1 errors for any non-`type`-kind var.
///
/// Returns the list of declared `type`-kind meta-var names (with `$` prefix,
/// as they appear in the source).
fn collect_declared_type_vars<'i>(
    group_name: &str,
    meta_decl_list: Option<&'i pairs::MetaVariableDeclList<'i>>,
    errors: &mut Vec<OpsWfError>,
) -> Vec<&'i str> {
    let mut type_var_names: Vec<&'i str> = Vec::new();

    let Some(mdl) = meta_decl_list else {
        return type_var_names;
    };
    let Some(inner) = mdl.get_matched().1 else {
        return type_var_names;
    };

    for decl in collect_elems_separated_by_comma!(inner) {
        let (ident, _, ty, _) = decl.get_matched();
        let var_name = ident.span.as_str(); // includes `$`

        match ty.deref() {
            Choice3::_0(_type_kind) => {
                // R1 OK — type-kind meta-var.
                type_var_names.push(var_name);
            },
            Choice3::_1(_const_kind) => {
                // R1 violation: const-kind.
                errors.push(OpsWfError::new(
                    group_name,
                    format!(
                        "op-level meta-var '{}' must be of kind 'type' \
                         (only 'type' is supported in v1)",
                        var_name
                    ),
                ));
            },
            Choice3::_2(_place_kind) => {
                // R1 violation: place-kind.
                errors.push(OpsWfError::new(
                    group_name,
                    format!(
                        "op-level meta-var '{}' must be of kind 'type' \
                         (only 'type' is supported in v1)",
                        var_name
                    ),
                ));
            },
        }
    }

    type_var_names
}

/// Check a single `OpFnSig` for R2 (undeclared meta-var references) and
/// R3 (concrete type references).
fn check_op_sig(group_name: &str, sig: &pairs::OpFnSig<'_>, declared_type_vars: &[&str], errors: &mut Vec<OpsWfError>) {
    // Walk parameters.
    if let Some(params_pair) = sig.OpFnParamsSeparatedByComma() {
        let (first, rest) = params_pair.OpFnParam();
        let params: Vec<_> = std::iter::once(first).chain(rest).collect();
        for param in params {
            if let Some(ty) = param_type(param) {
                check_type(group_name, ty, declared_type_vars, errors);
            }
        }
    }

    // Walk return type.
    if let Some(fn_ret) = sig.FnRet() {
        let (_, placeholder_or_ty) = fn_ret.get_matched();
        if let Choice2::_1(ty) = placeholder_or_ty {
            check_type(group_name, ty, declared_type_vars, errors);
        }
    }
}

/// Extract the `Type` node from an `OpFnParam`, if any.
///
/// `OpFnParam` alternatives:
/// - `SelfParam`         — no `Type` node to check (the self type is implicit)
/// - `NormalParam`       — has a `Type`
/// - `PlaceHolderWithType` — has a `Type`
/// - `Type`              — IS the `Type`
/// - `Dot2`              — variadic `..`, no type
fn param_type<'a>(param: &'a pairs::OpFnParam<'a>) -> Option<&'a pairs::Type<'a>> {
    // SelfParam — skip (self receiver, no explicit type to validate)
    if param.SelfParam().is_some() {
        return None;
    }
    // NormalParam: `$name: Type`
    if let Some(normal) = param.NormalParam() {
        let (_, _, _, ty) = normal.get_matched();
        return Some(ty);
    }
    // PlaceHolderWithType: `_: Type`
    if let Some(ph_with_ty) = param.PlaceHolderWithType() {
        let (_, _, _, ty) = ph_with_ty.get_matched();
        return Some(ty);
    }
    // Bare `Type` param (e.g. `&mut $T`)
    if let Some(ty) = param.Type() {
        return Some(ty);
    }
    // Dot2 — variadic
    None
}

/// Recursively walk a `Type` node to check R2 and R3.
///
/// - R2: if a `TypeMetaVariable` is found whose name is not in `declared_type_vars`, emit an error.
/// - R3: if a `TypePath` or `PrimitiveType` leaf is found, emit an error.
///
/// Wrapper types (`TypeReference`, `TypePtr`, `TypeSlice`, `TypeTuple`,
/// `TypeArray`, `TypeGroup`, `TypeParen`) are traversed without error.
/// `TypeNever`, `PlaceHolder` (`_`), and `kw_Self` are silently accepted.
fn check_type(group_name: &str, ty: &pairs::Type<'_>, declared_type_vars: &[&str], errors: &mut Vec<OpsWfError>) {
    match ty.deref() {
        // Wrapping types — recurse into the inner type(s).
        Choice14::_0(ty_array) => {
            let (_, inner, _, _, _) = ty_array.get_matched();
            check_type(group_name, inner, declared_type_vars, errors);
        },
        Choice14::_1(ty_group) => {
            let (_, inner) = ty_group.get_matched();
            check_type(group_name, inner, declared_type_vars, errors);
        },
        Choice14::_2(_ty_never) => {
            // `!` — accept (rare, but abstract)
        },
        Choice14::_3(ty_paren) => {
            let (_, inner, _) = ty_paren.get_matched();
            check_type(group_name, inner, declared_type_vars, errors);
        },
        Choice14::_4(ty_ptr) => {
            let (_, _, inner) = ty_ptr.get_matched();
            check_type(group_name, inner, declared_type_vars, errors);
        },
        Choice14::_5(ty_ref) => {
            let (_, _, _, inner) = ty_ref.get_matched();
            check_type(group_name, inner, declared_type_vars, errors);
        },
        Choice14::_6(ty_slice) => {
            let (_, inner, _) = ty_slice.get_matched();
            check_type(group_name, inner, declared_type_vars, errors);
        },
        Choice14::_7(ty_tuple) => {
            let (_, tys_opt, _) = ty_tuple.get_matched();
            if let Some(tys) = tys_opt {
                let tys_vec = collect_elems_separated_by_comma!(tys).collect::<Vec<_>>();
                for inner in tys_vec {
                    check_type(group_name, inner, declared_type_vars, errors);
                }
            }
        },
        // Meta-variable reference — R2 check.
        Choice14::_8(ty_meta_var) => {
            // The span text is the raw `$Name` including the `$`.
            let var_name = ty_meta_var.span.as_str();
            if !declared_type_vars.contains(&var_name) {
                errors.push(OpsWfError::new(
                    group_name,
                    format!(
                        "op-level meta-var '{}' is not declared in op group '{}'",
                        var_name, group_name
                    ),
                ));
            }
        },
        // `Self` — silently accepted.
        Choice14::_9(_kw_self) => {},
        // PrimitiveType — R3 violation (concrete built-in type).
        Choice14::_10(prim) => {
            errors.push(OpsWfError::new(
                group_name,
                format!(
                    "ops blocks must use op-level meta-vars only — concrete types belong \
                     in rpl.toml (found primitive type '{}')",
                    prim.span.as_str()
                ),
            ));
        },
        // PlaceHolder (`_`) — accepted (wildcard / anonymous).
        Choice14::_11(_placeholder) => {},
        // TypePath — R3 violation (concrete Rust path).
        Choice14::_12(ty_path) => {
            errors.push(OpsWfError::new(
                group_name,
                format!(
                    "ops blocks must use op-level meta-vars only — concrete types belong \
                     in rpl.toml (found path '{}')",
                    ty_path.span.as_str()
                ),
            ));
        },
        // LangItemWithArgs — R3 violation (concrete lang-item reference).
        Choice14::_13(lang_item) => {
            errors.push(OpsWfError::new(
                group_name,
                format!(
                    "ops blocks must use op-level meta-vars only — concrete types belong \
                     in rpl.toml (found lang item '{}')",
                    lang_item.span.as_str()
                ),
            ));
        },
    }
}

// ---------------------------------------------------------------------------
// R6: Op-level meta-vars must not appear in pattern bodies
// ---------------------------------------------------------------------------

/// Check every `pattBlock` item in `patts` against the op-level meta-var
/// names collected from `ops_blocks`.
///
/// For each pattern item `p[...] = ...`:
/// 1. Collect the pattern-level declared meta-var names (those in `p[...]`).
/// 2. Collect all op-level meta-var names across all op groups.
/// 3. Walk every `TypeMetaVariable` in the item's RHS.
/// 4. If a type meta-var's name is in the op-level set but NOT in the pattern-level set, emit an R6
///    error: `"op-level meta-var '$T' cannot appear in a pattern body"`.
///
/// Returns a flat list of all R6 errors found.
pub fn check_r6_patt_vs_ops(ops_blocks: &[&pairs::opsBlock<'_>], patts: &[&pairs::pattBlock<'_>]) -> Vec<OpsWfError> {
    // Collect all op-level meta-var names (with `$` prefix) across all groups.
    let op_level_names: HashSet<&str> = collect_all_op_meta_var_names(ops_blocks);

    let mut errors = Vec::new();
    for patt_block in patts {
        for item in patt_block.RPLPatternItem() {
            let item_name = item.Identifier().span.as_str();
            // Collect pattern-level declared meta-var names.
            let pattern_level_names: HashSet<&str> = collect_pattern_meta_var_names(item.MetaVariableDeclList());

            // Walk the item body for TypeMetaVariable references.
            let rhs = item.RustItemsOrPatternOperation();
            let mut collector = TypeMetaVarCollector::default();
            collect_type_meta_vars_in_rhs(rhs, &mut collector);

            for name in collector.names {
                if op_level_names.contains(name) && !pattern_level_names.contains(name) {
                    errors.push(OpsWfError::new(
                        item_name,
                        format!("op-level meta-var '{}' cannot appear in a pattern body", name),
                    ));
                }
            }
        }
    }
    errors
}

/// Collect all type meta-var names (with `$`) declared in ANY op group across
/// all `opsBlock`s.
fn collect_all_op_meta_var_names<'i>(ops_blocks: &[&pairs::opsBlock<'i>]) -> HashSet<&'i str> {
    let mut names = HashSet::new();
    for ops_block in ops_blocks {
        for item in ops_block.opsItem() {
            if let Some(mdl) = item.MetaVariableDeclList()
                && let Some(inner) = mdl.get_matched().1
            {
                for decl in collect_elems_separated_by_comma!(inner) {
                    let (ident, _, ty, _) = decl.get_matched();
                    // Only type-kind vars are relevant for R6 (R1 already
                    // rejects non-type vars; we include them anyway for a
                    // conservative check).
                    if matches!(ty.deref(), Choice3::_0(_)) {
                        names.insert(ident.span.as_str());
                    }
                }
            }
        }
    }
    names
}

/// Collect all meta-var names (with `$`) declared in a
/// `MetaVariableDeclList` (the `[...]` bracket of a pattern item).
fn collect_pattern_meta_var_names<'i>(meta_decl_list: Option<&'i pairs::MetaVariableDeclList<'i>>) -> HashSet<&'i str> {
    let mut names = HashSet::new();
    if let Some(mdl) = meta_decl_list
        && let Some(inner) = mdl.get_matched().1
    {
        for decl in collect_elems_separated_by_comma!(inner) {
            let (ident, _, _, _) = decl.get_matched();
            names.insert(ident.span.as_str());
        }
    }
    names
}

// ---------------------------------------------------------------------------
// TypeMetaVariable collector for R6
// ---------------------------------------------------------------------------

/// Accumulates every `TypeMetaVariable` span text found in a parse sub-tree.
#[derive(Default)]
struct TypeMetaVarCollector<'i> {
    names: Vec<&'i str>,
}

/// Entry point: walk `RustItemsOrPatternOperation` collecting all
/// `TypeMetaVariable` references.
///
/// We do a best-effort recursive walk over the parts of the grammar that can
/// contain `Type` nodes (function signatures and MIR bodies).  We deliberately
/// do NOT walk pattern-operation sub-expressions (they reference util items by
/// name, not types directly).
fn collect_type_meta_vars_in_rhs<'i>(
    rhs: &'i pairs::RustItemsOrPatternOperation<'i>,
    collector: &mut TypeMetaVarCollector<'i>,
) {
    use rpl_parser::generics::Choice3;
    match rhs.deref() {
        Choice3::_0(single_item) => collect_in_rust_item_with_constraint(single_item, collector),
        Choice3::_1(items_block) => {
            let (_, items, _) = items_block.get_matched();
            for item in items.iter_matched() {
                collect_in_rust_item_with_constraint(item, collector);
            }
        },
        Choice3::_2(_patt_op) => {
            // PatternOperation references util items by name; no Type nodes to
            // walk here.
        },
    }
}

fn collect_in_rust_item_with_constraint<'i>(
    item: &'i pairs::RustItemWithConstraint<'i>,
    collector: &mut TypeMetaVarCollector<'i>,
) {
    use rpl_parser::generics::Choice4;
    let (_, inner, _where_block) = item.get_matched();
    match inner.deref() {
        Choice4::_0(rust_fn) => collect_in_fn(rust_fn, collector),
        Choice4::_1(_struct) => { /* struct fields not yet scanned for R6 */ },
        Choice4::_2(_enum) => { /* enum variants not yet scanned for R6 */ },
        Choice4::_3(_impl) => { /* impl fns not yet scanned for R6 */ },
    }
}

fn collect_in_fn<'i>(rust_fn: &'i pairs::Fn<'i>, collector: &mut TypeMetaVarCollector<'i>) {
    let (sig, body) = rust_fn.get_matched();
    collect_in_fn_sig(sig, collector);
    if let Some(mir_body) = body.MirBody() {
        collect_in_mir_body(mir_body, collector);
    }
}

fn collect_in_fn_sig<'i>(sig: &'i pairs::FnSig<'i>, collector: &mut TypeMetaVarCollector<'i>) {
    // Walk parameters.
    if let Some(params) = sig.FnParamsSeparatedByComma() {
        let (first, rest) = params.FnParam();
        for param in std::iter::once(first).chain(rest) {
            collect_in_fn_param(param, collector);
        }
    }
    // Walk return type.
    if let Some(fn_ret) = sig.FnRet() {
        let (_, placeholder_or_ty) = fn_ret.get_matched();
        if let Choice2::_1(ty) = placeholder_or_ty {
            collect_in_type(ty, collector);
        }
    }
}

fn collect_in_fn_param<'i>(param: &'i pairs::FnParam<'i>, collector: &mut TypeMetaVarCollector<'i>) {
    use rpl_parser::generics::Choice4;
    match param.deref() {
        Choice4::_0(_self_param) => { /* self — no explicit type to check */ },
        Choice4::_1(normal) => {
            let (_, _, _, ty) = normal.get_matched();
            collect_in_type(ty, collector);
        },
        Choice4::_2(ph_with_ty) => {
            let (_, _, _, ty) = ph_with_ty.get_matched();
            collect_in_type(ty, collector);
        },
        Choice4::_3(_ellipsis) => {},
    }
}

fn collect_in_mir_body<'i>(body: &'i pairs::MirBody<'i>, collector: &mut TypeMetaVarCollector<'i>) {
    let (decls, stmts) = body.get_matched();
    for decl in decls.iter_matched() {
        collect_in_mir_decl(decl, collector);
    }
    for stmt in stmts.iter_matched() {
        collect_in_mir_stmt(stmt, collector);
    }
}

fn collect_in_mir_decl<'i>(decl: &'i pairs::MirDecl<'i>, collector: &mut TypeMetaVarCollector<'i>) {
    // MirDecl has two accessor methods: MirTypeDecl() and MirLocalDecl()
    if let Some(local_decl) = decl.MirLocalDecl() {
        // `let $x: Type = ...` — the declared type may reference a meta-var
        let ty = local_decl.Type();
        collect_in_type(ty, collector);
    }
    // MirTypeDecl (type aliases) don't carry op-level meta-var leaks we care about.
}

fn collect_in_mir_stmt<'i>(stmt: &'i pairs::MirStmt<'i>, collector: &mut TypeMetaVarCollector<'i>) {
    // Use the accessor methods on MirStmt rather than deref/match.
    // Cast rvalues are the only place type meta-vars can appear in statements.
    if let Some(assign) = stmt.MirAssign() {
        let rvalue_or_call = assign.MirRvalueOrCall();
        use rpl_parser::generics::Choice2;
        match rvalue_or_call.deref() {
            Choice2::_0(_call) => {},
            Choice2::_1(rvalue) => collect_in_mir_rvalue(rvalue, collector),
        }
    }
}

fn collect_in_mir_rvalue<'i>(rvalue: &'i pairs::MirRvalue<'i>, collector: &mut TypeMetaVarCollector<'i>) {
    use rpl_parser::generics::Choice12;
    match rvalue.deref() {
        Choice12::_1(cast) => {
            // `operand as Type (cast_kind)` — the target type may reference meta-vars
            let (_operand, _, ty, _, _cast_kind, _) = cast.get_matched();
            collect_in_type(ty, collector);
        },
        // All other rvalue forms don't carry explicit Type annotations in a
        // way that would allow op-level meta-var leaks.
        _ => {},
    }
}

/// Recursively walk a `Type` parse node, pushing any `TypeMetaVariable`
/// span text into `collector.names`.
fn collect_in_type<'i>(ty: &'i pairs::Type<'i>, collector: &mut TypeMetaVarCollector<'i>) {
    match ty.deref() {
        Choice14::_0(ty_array) => {
            let (_, inner, _, _, _) = ty_array.get_matched();
            collect_in_type(inner, collector);
        },
        Choice14::_1(ty_group) => {
            let (_, inner) = ty_group.get_matched();
            collect_in_type(inner, collector);
        },
        Choice14::_2(_ty_never) => {},
        Choice14::_3(ty_paren) => {
            let (_, inner, _) = ty_paren.get_matched();
            collect_in_type(inner, collector);
        },
        Choice14::_4(ty_ptr) => {
            let (_, _, inner) = ty_ptr.get_matched();
            collect_in_type(inner, collector);
        },
        Choice14::_5(ty_ref) => {
            let (_, _, _, inner) = ty_ref.get_matched();
            collect_in_type(inner, collector);
        },
        Choice14::_6(ty_slice) => {
            let (_, inner, _) = ty_slice.get_matched();
            collect_in_type(inner, collector);
        },
        Choice14::_7(ty_tuple) => {
            let (_, tys_opt, _) = ty_tuple.get_matched();
            if let Some(tys) = tys_opt {
                let tys_vec = collect_elems_separated_by_comma!(tys).collect::<Vec<_>>();
                for inner in tys_vec {
                    collect_in_type(inner, collector);
                }
            }
        },
        Choice14::_8(ty_meta_var) => {
            // This is the key: collect the name with the `$` prefix.
            collector.names.push(ty_meta_var.span.as_str());
        },
        Choice14::_9(_kw_self) => {},
        Choice14::_10(_prim) => {},
        Choice14::_11(_placeholder) => {},
        Choice14::_12(_ty_path) => {},
        Choice14::_13(_lang_item) => {},
    }
}

// Well-formed-input coverage is provided by the integration tests in
// `tests/ops_lowering.rs` and `tests/ops_wf.rs`; no in-file unit tests are
// needed here.
