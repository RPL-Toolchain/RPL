clear

set -uxo pipefail
# Note: -e is intentionally omitted — rpl-driver exits 1 when it fires a lint
# (this is the expected behaviour for test targets that contain flagged code).
# Use || true after invocations that are expected to produce diagnostics.


export RUSTC_ICE=0
export RUSTC_LOG_COLOR=always
export RUSTC_LOG="rpl=trace"
export RUST_BACKTRACE=1

# RPL_PATS="tests/ui/features/diff_pat/default.rpl" cargo +rpl-dbg uitest -- "tests/ui/features/diff_pat" 2>&1 | tee .ansi
# RPL_PATS="tests/ui/features/diff_pat/meta_var.rpl" cargo +rpl-dbg uitest -- "tests/ui/features/diff_pat" 2>&1 | tee .ansi
# RPL_PATS="tests/ui/features/diff_pat/label.rpl" cargo +rpl-dbg uitest -- "tests/ui/features/diff_pat" 2>&1 | tee .ansi
# RPL_PATS="tests/ui/features/diff_pat/label.rpl" cargo +rpl-dbg run --bin rpl-driver -- "tests/ui/features/diff_pat/test.rs" 2>&1 | tee .ansi
# RPL_PATS="tests/ui/features/diff_pat" cargo +rpl-dbg uitest -- "tests/ui/features/diff_pat" 2>&1 | tee .ansi

# Task 13: lock_unlock end-to-end ops test
# Exercises the abstract-ops feature: parser, lowering, OpRef IR, OpsConfig
# substitution, cartesian iteration, and matcher OpRef resolution against
# std::sync::Mutex.  The workspace rpl.toml provides the [[ops.sync]] instance.
# Expected: lint fires at m.lock() → rpl-driver exits 1 (|| true is intentional).
RPL_PATS="tests/features/ops/lock_unlock.rpl" \
  cargo run --bin rpl-driver -- "tests/features/ops/lock_unlock.rs" 2>&1 | tee .ansi || true

# ---------------------------------------------------------------------------
# Task 14: four end-to-end UI tests for abstract-ops properties
# ---------------------------------------------------------------------------

# folded: ops group declared in .rpl, NO [[ops.sync_folded]] in rpl.toml.
# Cartesian product of zero instances → zero matches → no diagnostics (exit 0).
RPL_PATS="tests/features/ops/folded.rpl" \
  cargo run --bin rpl-driver -- "tests/features/ops/folded.rs" 2>&1 | tee .ansi

# partial_bad: three [[ops.sync_pb]] entries, one malformed (missing unlock).
# C3 validation skips the third instance; the first (Mutex) still fires.
# Expected: lint at m.lock() → exit 1 (|| true is intentional).
RPL_PATS="tests/features/ops/partial_bad.rpl" \
  cargo run --bin rpl-driver -- "tests/features/ops/partial_bad.rs" 2>&1 | tee .ansi || true

# set_op_with_ops: util patterns p_lock / p_uncovered use ops; patt = p_lock - p_uncovered.
# lock_only() matches p_lock but NOT p_uncovered → diagnostic fires.
# lock_and_mark() matches BOTH → subtracted → no diagnostic.
# Expected: one lint at lock_only → exit 1 (|| true is intentional).
RPL_PATS="tests/features/ops/set_op_with_ops.rpl" \
  cargo run --bin rpl-driver -- "tests/features/ops/set_op_with_ops.rs" 2>&1 | tee .ansi || true

# two_groups: two op groups (sync_2g × logger_2g); cartesian-product expansion.
# Mutex × black_box combination in main() is the only one that fires (RwLock
# also has an instance, but no matching call site is present).
# Expected: one lint → exit 1 (|| true is intentional).
RPL_PATS="tests/features/ops/two_groups.rpl" \
  cargo run --bin rpl-driver -- "tests/features/ops/two_groups.rs" 2>&1 | tee .ansi || true

# ---------------------------------------------------------------------------
# Task 15: CVE-2025-68260 POC — abstract intrusive-list-under-lock pattern
# ---------------------------------------------------------------------------

# buggy.rs: mirrors the pre-fix Node::release shape (lock → transfer → unlock → drain).
# sync_cve bound to Mutex::lock + std::intrinsics::black_box (opaque unlock sink).
# intrusive_list_cve bound to crate::transfer_to_temp + crate::drain_temp.
# Expected: ops_cve_2025_68260 lint fires → exit 1 (|| true is intentional).
RPL_PATS="tests/features/ops_cve_2025_68260/pattern.rpl" \
  cargo run --bin rpl-driver -- "tests/features/ops_cve_2025_68260/buggy.rs" 2>&1 | tee .ansi || true

# fixed.rs: guard drops naturally (no black_box call) — no $sync_cve::$unlock match.
# Pattern cannot fire without an explicit unlock call site.
# Expected: NO ops_cve_2025_68260 lint → exit 0.
RPL_PATS="tests/features/ops_cve_2025_68260/pattern.rpl" \
  cargo run --bin rpl-driver -- "tests/features/ops_cve_2025_68260/fixed.rs" 2>&1 | tee .ansi
