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