/// Post-lowering use-site checks for `ops { ... }` references (R4–R5) and
/// `referenced_op_groups` collection.
///
/// These checks operate on the already-lowered `Pattern<'pcx>` IR.  They run
/// **after** `add_ops_block` and `add_pattern_item` have been called, so the
/// `ops_block.groups` map and all `RustItems`/`FnPatternBody` are in their
/// final state.
///
/// # Rules
///
/// | # | Rule |
/// |---|------|
/// | R4 | Every `$group::$op` operand resolves to a declared op group and op. |
/// | R5 | The number of arguments at the call site matches the op signature's arity. |
use rustc_data_structures::fx::FxHashSet;
use rustc_span::Symbol;

use crate::pat::{BasicBlockData, FnPatternBody, FnPatterns, Operand, OpsBlock, RustItems, TerminatorKind};

// ---------------------------------------------------------------------------
// Public error type
// ---------------------------------------------------------------------------

/// A use-site well-formedness violation for `$group::$op` references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpsUseError {
    pub message: String,
}

impl OpsUseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for OpsUseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Walk all function bodies in `items` looking for `Operand::OpRef` call
/// targets.  For each one:
///
/// 1. **(R4a)** Check that `group` is declared in `ops_block.groups`.
/// 2. **(R4b)** Check that `op` is declared within that group.
/// 3. **(R5)** Check that the call-site argument count matches the op signature's parameter count.
///
/// Also collects every group name referenced by a (potentially invalid) OpRef
/// into `referenced_groups`, which is used to populate
/// `RustItems::referenced_op_groups`.
///
/// Returns a (possibly empty) list of R4/R5 errors.
pub fn check_op_refs(
    items: &RustItems<'_>,
    ops_block: &OpsBlock<'_>,
    referenced_groups: &mut FxHashSet<Symbol>,
) -> Vec<OpsUseError> {
    let mut errors = Vec::new();
    check_fn_patterns(&items.fns, ops_block, referenced_groups, &mut errors);
    errors
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn check_fn_patterns(
    fns: &FnPatterns<'_>,
    ops_block: &OpsBlock<'_>,
    referenced_groups: &mut FxHashSet<Symbol>,
    errors: &mut Vec<OpsUseError>,
) {
    for fn_pat in fns {
        if let Some(body) = fn_pat.body {
            check_fn_body(body, ops_block, referenced_groups, errors);
        }
    }
}

fn check_fn_body(
    body: &FnPatternBody<'_>,
    ops_block: &OpsBlock<'_>,
    referenced_groups: &mut FxHashSet<Symbol>,
    errors: &mut Vec<OpsUseError>,
) {
    for bb_data in body.basic_blocks.iter() {
        check_basic_block(bb_data, ops_block, referenced_groups, errors);
    }
}

fn check_basic_block(
    bb_data: &BasicBlockData<'_>,
    ops_block: &OpsBlock<'_>,
    referenced_groups: &mut FxHashSet<Symbol>,
    errors: &mut Vec<OpsUseError>,
) {
    // We only need to look at terminators; OpRef appears as the `func` operand
    // of a Call terminator (never in a statement rvalue).
    if let Some(TerminatorKind::Call {
        func: Operand::OpRef { group, op },
        args,
        ..
    }) = &bb_data.terminator
    {
        // Record the group name (even if invalid — callers may want the
        // full list for diagnostics).
        referenced_groups.insert(*group);

        // R4a: group must be declared.
        let Some(op_group) = ops_block.groups.get(group) else {
            errors.push(OpsUseError::new(format!(
                "op group '{}' is not declared",
                group.as_str()
            )));
            return;
        };

        // R4b: op must be declared within the group.
        let Some(op_sig) = op_group.ops.get(op) else {
            errors.push(OpsUseError::new(format!(
                "op '{}' is not declared in op group '{}'",
                op.as_str(),
                group.as_str()
            )));
            return;
        };

        // R5: arity check.
        let expected = op_sig.params.len();
        let got = args.len();
        if expected != got {
            errors.push(OpsUseError::new(format!(
                "arity mismatch for op '{}::{}': expected {} arg{}, got {}",
                group.as_str(),
                op.as_str(),
                expected,
                if expected == 1 { "" } else { "s" },
                got
            )));
        }
    }
}
