# Dataflow predicates (`flows_to` / `may_panic` / Rudra UD)

RPL can express **source → sink** constraints in `where` blocks, on top of MIR CFG/DDG subgraph matching.

## Syntax

```rpl
p[...] = unsafe? fn _(..) -> _ {
    'src:
    // statement that defines / affects `$x`
    ...
    'sink:
    // statement that uses `$x` (or a Call site)
} where {
    flows_to($x, 'src, 'sink),
    may_panic('sink),
}
```

| Predicate | Arguments | Meaning |
|-----------|-----------|---------|
| `flows_to($x, 'src, 'sink)` | local/place + two labels | DDG path labeled with `$x` from `'src` to `'sink` (intermediate statements allowed). |
| `may_panic('sink)` | one label | Broader panic heuristic: MIR `Assert`; Call whose callee is Rudra-unresolvable; **plus** local-generic fallback / `Fn*` / closure / `dyn` / `Param` / `Alias`. Denylist (`mem::forget`, `ptr::read`/`write`/`copy*`, …) is never a sink. Prefer **not** using this for default Rudra UD. |
| `lifetime_bypass('loc)` / `strong_bypass` / `weak_bypass` | one label | Call whose callee is on Rudra's lifetime-bypass path table (strong ∪ weak / strong / weak). |
| `unresolvable_generic('loc)` | one label | Strict Rudra sink: `Instance::try_resolve` → `Ok(None)` / `Err` only (no local-generic fallback). |
| `generic_drop('loc)` | one label | Call is `ptr::drop_in_place` (Rudra `GENERIC_FN_LIST`). |
| `cfg_reaches('src, 'sink)` | two labels | MIR CFG reachability (intrablock statement order, or interblock via successors). **Not** DDG. When present, matcher skips DDG adjacency between pattern statements. |
| `bypass_on_copy('loc)` | one label | `ptr::read`/`write` on a `Copy` pointed-to type (Rudra skip). |
| `set_len_to_zero('loc)` | one label | `Vec::set_len` with constant `0` (Rudra skip; leak is safe). |

`where` attaches to a single `fn` item, not the outer multi-item `{ ... }` brace. Negation (`!pred`) is supported.

## vs subgraph matching

- **`match_ddg`**: requires **direct** DDG edges between matched pattern statements (skipped when `cfg_reaches` appears in `where`).
- **`flows_to`**: only **DDG reachability** of `$x` between two anchors.
- **`cfg_reaches`**: only **control-flow** reachability between two anchors (Rudra UD style).

## Engine

- [`DataDepGraph::flows_to`](../../crates/rpl_mir_graph/src/graph.rs)
- [`PredicateEvaluator`](../../crates/rpl_match/src/predicate_evaluator.rs) (`mir_ddg` / `mir_cfg` injected from MatchSession)
- Path tables: [`rudra_paths`](../../crates/rpl_match/src/rudra_paths.rs)

Prefer `-Z inline-mir=false` in UI tests when matching `Vec::set_len` / `ptr::*` as Calls (otherwise RPL may inline them to intrinsics/field writes).

## Default Panic Safety pattern

Default library entry: [`docs/patterns-pest/panic-safety.rpl`](../patterns-pest/panic-safety.rpl).

Open query (any Call → any Call) with:

```text
strong_bypass|weak_bypass('src)
∧ (unresolvable_generic('sink) ∨ generic_drop('sink))
∧ cfg_reaches('src, 'sink)
∧ !bypass_on_copy('src) ∧ !set_len_to_zero('src)
```

Strong → `deny`, weak → `warn`. Lint name: `panic_safety`.

## Example matrix

| Sample | Role | Location |
|--------|------|----------|
| Feature `flows_to` | Lock `flows_to` TP/TN | `tests/ui/features/flows_to/` |
| Feature `rudra_ud` | Rudra UD predicates TP/TN | `tests/ui/features/rudra_ud/` |
| CVE-2020-25795 | `copy_nonoverlapping` then `Iterator::next` | `tests/ui/cve/cve_2020_25795/minimal.rs` |
| CVE-2020-35923 | `NotNan` field write + `is_nan` (`may_panic`) | `docs/patterns-pest/cve/CVE-2020-35923.rpl` |
| Retain-like synth | `set_len(0)` then callback (**wider than Rudra**) | `tests/ui/features/panic_safety_retain/` |
| id-map | weak/strong bypass + sink | `tests/ui/cve/cve_2021_30455/` |

## Pattern tips (Panic Safety)

- Prefer **Rudra-style** open query above; do **not** use dual `fn $poison` / `fn $sink` metavars for “any Call → any Call”, and do **not** treat “`$f()` then `ptr::write`” as the bug shape (that is often the *safe* order).
- Any-call anchors: `_ = _(..);` (callee `_`, args `..`).
- **Qself**: `<I as Iterator>::next` is matched by `_ = _(..)` plus `unresolvable_generic`.
- **Decls before stmts**: all `let` locals must appear before non-decl MIR statements in a pattern block.
- **`copy_nonoverlapping`**: as a Rust call it is a Call terminator; the MIR statement form (`copy_nonoverlapping(_, _, _);`) is a separate keyword pattern.

## Limitations

- Root-local only for `flows_to`; no unwind-edge requirement.
- `unresolvable_generic` is stricter than historical `may_panic`; local wrappers that still `try_resolve` to `Some` are not UD sinks unless they are `generic_drop`.
- Function *names* in patterns do not filter which MIR items are searched.
