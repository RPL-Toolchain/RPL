# Gap tests: the faithfulness boundary

These tests pin down the behaviours described in the paper appendix
(`app:witness`).  Each records the **current** behaviour together with a comment
saying what should happen after the corresponding fix.

| Test | Direction | Current behaviour | After fixing |
|---|---|---|---|
| `control_dep` | false positive | reports a flow from `x = 1` to a `return x` that no execution can reach | should become `//@check-pass` |
| `memory_alias` | false negative | does not match a write/read pair through two places that may alias | should report |
| `loop_carried` | positive, records the fold | matches a flow whose only edge is carried by the back edge | unchanged; counting stays inexpressible |

## Notes

- `control_dep` is the only item that falsifies the faithfulness equation itself:
  the static graph carries an edge that no execution realises.  Repairing it needs
  control-dependence edges *and* branch-condition reasoning --- the witness's `c`
  and `!c` are complements, which an edge alone does not record.
- `memory_alias` is a boundary of discriminating power, not of the equation: the
  pattern needs a single place, and `alpha` cannot prove that `p` and `q` may
  alias.
- `loop_carried` records that the loop-carried dependence **is** represented, folded
  onto one static edge.  The claim "the graph has no edge for it" is wrong; what is
  missing is the multiplicity.  Counting properties have no match/no-match test by
  construction, because the pattern language cannot express "at least two
  iterations".

## Verifying by hand

Add `#[rpl::dump_mir(dump_ddg)]` to the function of interest and run

    RPL_PATS=tests/ui/gaps/<name>/<name>.rpl cargo rpl -- -Zinline-mir=false

The DDG is written to `mir_dump/*.ddg.dot`.

## Running

    TESTNAME=gaps cargo test --test compile-test

Regenerate expected output after an intentional change with `RUSTC_BLESS=1`.
