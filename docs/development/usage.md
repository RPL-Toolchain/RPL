# Usage

## The Pattern Language

### Notes

- When the operand has a `Copy` type, operator `Copy` or `Move` are considered equivalent.
- Use `#[deduplicate]` on pattern items to avoid duplication (it costs, so take your care).
- For fn items in patterns, `pub fn` only matches public functions, `pub(restricted) fn` only matches non-public functions, and `fn` matches all functions.
- For fn items in patterns, `unsafe fn` only matches unsafe functions, `fn` only matches safe functions, and `unsafe? fn` matches all functions.
- For fn items in patterns,  `#[inline] fn` only matches functions annotated with `#[inline]` or `#[inline(always)]`, `#[inline(always)] fn` only matches functions annotated with `#[inline(always)]`, `#[inline(never)] fn` only matches functions annotated with `#[inline(never)]`, `#[inline(any)] fn` only matches functions not annotated with `#[inline(never)]`, and `fn` matches all functions.
- Use `#[output = "foo"]` on fn items in patterns to bind its output span with `foo`.
- `fn $foo` binds `$foo` with the span of the function.

### Multi-function and type matching (MatchSession)

A `RustItems` block is one **session**: the driver indexes crate items, then assigns each pattern **slot** (struct/enum, named `fn`, `fn _`, or `impl` method) a `DefId` while a shared environment (**SharedEnv**) stays consistent.

- **Outer vs inner.** Outer search assigns slots (all ADT slots first, then function slots). Inner matching runs `CheckMirCtxt::check_with` on one MIR `Body` at a time, with SharedEnv pre-bound. Type vars, non-local consts, `AdtPat → AdtDef`, and ADT field indices are shared; places, locals, and statement locations stay per-slot.
- **Required vs optional.** Named `fn`s, every ADT, and impl methods are required: an empty candidate domain fails the prefix (no result). `fn _` is optional (skip, then try unused defs). An all-`fn _` assignment with nothing filled is discarded.
- **Same `impl`.** Methods in one pattern `impl $Name { fn $a; fn $b }` must come from the same rustc `impl` (`impl_of_method`). Distinct pattern `impl` blocks are not pinned to each other except through SharedEnv.
- **Signature-only slots** (empty MIR body in the `.rpl` file) still run `match_ty` on parameters and return type; they do not run the statement graph. An omitted `ret` is unconstrained (not forced to `()`).
- **Constraints.** After a slot's inner match succeeds, `fn` (and `impl`) constraints are evaluated on that slot's `Matched` overlaid with SharedEnv. A metavar **mentioned** in those constraints that is still unbound fails; unused type vars that the constraints never mention do not.
- **Permutations.** Assigning `$f1`/`$f2` to two functions in either order yields two `SessionResult`s. UI tests often need two `//~|` ERROR annotations per site. Diagnostics emit once per MIR function slot in a result: spans come from that slot's normalized match; type/const metavars prefer SharedEnv.
- **Pattern operations.** `p - q` (and `p + r - q`) filters positive session results against negatives when **mapped SharedEnv** is equal and **alignable slot DefIds** match (at least the primary function, plus any `MatchSlot` present on both sides). MIR statement/place equality is not the subtract key.
- **Truncation.** Result count is capped by `DEFAULT_MAX_SESSION_RESULTS` (256). Hitting the cap is not treated as a complete search: `rpl_match` logs a tracing warning and the driver emits one rustc warning per truncated pattern item. Raise the cap via `SessionConfig::max_results` in a custom driver.