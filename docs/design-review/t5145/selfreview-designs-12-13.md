# Independent read of SPEC-0002 Designs 12 and 13, before their CR (T-5145 change 5)

*For poc-engine-cc, from poc-design-cc, 2026-11-11 16:08Z. An independent reviewer (a subagent, fresh context) read
the two designs against `/work` with the lenses the formal reviewer used on Design 9. Nothing was built or run:
every counterexample is reasoned from the code. Line numbers are those of
`/out/drafts/t5145/selfreview/SPEC-0002-with-designs-7-to-13.md`; paths are under
`crates/rustsheet-engine/src/` unless given in full. Design 13 is your draft: B1 to B6 are yours to rule on. B7 and
B8 (Design 12) I fix myself.*

## Blocking findings

### Design 13

**B1 (High), lines 1932-1935 and 1948-1950: "`read_is_inert` holds for it by construction" is false unless the reader
was itself traversed.**
- The traversal runs only from a root that has a footprint. A non-candidate reader gets none, and that includes every
  conditional aggregate until Design 12 lands (they are absent from the allowlist at `read_index/walk.rs:21-83`, and
  line 1785 puts T-5143 after Design 13).
- Counterexample: `D1: =COUNTIFS(B1:B100,">0")`, `B r: =A r*2`. D1 runs in row 1 with B2:B100 unevaluated, yet the
  range is "all candidates", so rule 1 sends the read through the index.
- The index must then either evaluate the cells, which nests, or read stale values.
  `functions/statistical/ifs_cache.rs:7-12` records why reading every cell is unsafe: "A cache that read every cell
  could evaluate a cell the walk never reaches, closing a cycle…".
- The same hole exists for a root that fell back (step 3, e.g. `=SUM(B1:B10)+Z1` with `Z1` reading it back), for an
  Excel-order pass and for an iteration sweep.
- Suggested replacement for rule 1: "A read uses the index when the pass is not in Excel's order, the reading root
  has a footprint whose traversal completed without fallback, and the range's formula cells are all candidates. All
  three are functions of the document." Then drop line 1949's claim until Design 12 makes the aggregates candidates.

**B2 (High), lines 1878-1880 and 1893-1898: the traversal is quadratic and unmetered.**
- Step 1 re-takes "the first formula cell … not evaluated" each time T is on top, and a rectangle costs
  "O(log F + hits)" even when every hit is already evaluated.
- `=SUM($B$1:B r)` for r = 1..100,000 over a formula column is about 5×10^9 probes that charge nothing.
  `C1=SUM(B1:B1000000)` with no per-frame cursor is about 5×10^11.
- `T_pass` in SPEC-0028 is "linear in the document's size"
  (`docs/specs/SPEC-0028-recalculation-work-budget.md:484-495`), so "so `T_pass`" does not hold.
- Fix: give each frame a cursor and each column a "next unevaluated candidate at or after row r" skip structure
  (union-find). State the bound as O(F·α + rectangles·log F), bound the columns visited for wide rectangles, and add
  a scaling test. Otherwise charge the traversal.

**B3 (Medium), lines 1884-1888: the leaf definition admits `=SUM(INDIRECT("A1"))`.**
- Its tree has "no reference, no range and no name", because the argument is a string. So "It reads no cell" is false
  for it.
- This contradicts line 1888 and test row 1966 ("still declines").
- Fix: "a leaf is a formula whose every function is on a listed set that reads no cell (TODAY, NOW, RAND,
  argument-less ROW and COLUMN, and the allowlist's functions over constants)".

**B4 (Medium), lines 1936-1941: the indexes' room is not defined.**
- "The pass's ceiling" is not a SPEC-0028 term. The allowance is per pass and the ceiling is per recalculation
  (SPEC-0028:68-90).
  - If it is the recalculation's ceiling, the remainder depends on earlier passes, and restarts follow the learned
    anchor order (SPEC-0028:176-179). Replicas would then not "cut at the same range".
  - If it is committed work, a build can push a normal-mode document into share mode.
- Memory: "the allowance's worth of states" is 2^30. `SumParts` is three `f64`
  (`functions/math_and_trigonometry/sum.rs:238-242`), so that is 24 GiB against today's 4,000,000 states
  (`sum_prefix.rs:37`). The test row's "byte bound" is defined nowhere.
- "One bit set per distinct criterion" cannot be "charged when built, outside any formula's run": criteria are values
  known only as formulas run (`ifs_cache.rs:28-31`), so which sets fit follows evaluation order.
- `MAX_CACHED_BITS` (`ifs_cache.rs:81`, 32,000,000) is not mentioned. If it stays, it is a capacity other formulas
  use up. If it goes, 100,000 criteria × 100,000 rows is 10^10 bits.
- Fix: name a fixed per-pass budget independent of what was spent, give units and bytes per state, and state the
  criteria sets' rule.

**B5 (Medium), lines 1875-1888: the order's dependence on the index, and iterative calculation, are unstated.**
- Candidacy uses "the read index's walk". With no index (lines 1500-1501: array formula, data table, external book,
  iterative calculation; also the memory cap and `set_incremental(false)`), the text does not say whether the pass is
  dependency-first.
  - If it is not, RAND draws, thrown-away work and index use differ with an optimization's availability.
  - If it is, iteration gets a new order the section never mentions.
- Step 3 sees only static cycles. With `A1: =C5+B5`, `B5: =C5+1`, `C5: =N(B5)` (N is off the allowlist), today's pass
  enters the cycle at C5 and the new pass enters at B5.
- With iteration off the reviewer found no value change: `circular.rs:20-27` marks the whole stack, and later readers
  are marked too. With iteration on it was not analysed.
- Fix: state "candidacy is the walk's, computed in every full pass whether or not an index exists; with iterative
  calculation on the pass is today's". Add Cycles rows for a cycle through a non-candidate and for iteration on.

**B6 (Medium), lines 1934-1935 and 1942: ranges that lose today's cache.**
- Today a range is cached once its formula cells are evaluated, spill cells included (`ifs_cache.rs:10-12`).
- Under rule 1, one non-candidate in the range makes every read charge its walk. A non-candidate is a spill cell, or
  any function off the roughly 60-entry allowlist, such as a `=MONTH(A2)` helper column.
- The draft's own figure (line 1872) puts a SUMIFS per row at 100,000 rows at twice the allowance. That means share
  mode and `#VALUE!` where Excel and today's engine compute, against directive 2.
- This is inferred from the texts, not measured.
- Fix: add a "What it must show" row that `make eval` changes no cell and that a SUMIFS column over a spilled column,
  and over an off-allowlist formula column, at 100,000 rows stays in normal mode. Otherwise sequence the walk charge
  after the allowlist covers them.

### Design 12 (the design desk fixes these)

**B7 (Medium), lines 1804-1808: the rule is not a pure function of the text.** Only the resized or read-on argument
must be written, but the rectangle's size comes from the other argument:
`=SUMIF(CHOOSE(D1,A1:A3,A1:A9),">0",B1)` gives B1:B3 or B1:B9 depending on D1 (CHOOSE and IF are allowlisted,
`walk.rs:47,55`, and return references). `=LOOKUP(2,1/(A1:A100<>""),B1)` has a computed lookup vector. For a 2-D
lookup vector, "as far as the lookup vector is long" must be max(rows, columns)
(`functions/lookup_and_reference/lookup.rs:169`, `:335`). Fix: both the argument that gives the shape and the one
resized or read on must be written references or ranges; LOOKUP's extent is the larger of the lookup vector's rows
and columns; otherwise the formula stays always dirty.

**B8 (Medium), line 1812: "A criterion that holds a range" is a run-time fact.** The lift tests values
(`functions/statistical/if_ifs.rs:118`). `=SUM(COUNTIF(A1:A9,CHOOSE(D1,B1,B1:B3)))` lifts or not depending on D1.
Fix: "a criterion whose static result may be an array (11.1's test)".

## Non-blocking findings

- **Line 1798:** a clipped rectangle that reaches the last row or column is not addressed. `walk.rs:236` refuses
  written ranges there, so say such a formula stays always dirty.
- **Lines 1906 and 1912:** thrown-away work going to 0 can move a document from share mode to normal, which changes
  values. Say so, and update the DEVIATION entry (SPEC-0028 section 3, "Where it is not").
- **Steps 1 and 3:** step 1 skips cells "on the stack" while step 3 calls that a cycle. "Nothing of its footprint
  evaluated first" is also untrue for siblings the DFS already evaluated.
- **Lines 1883 and 1926:** "neither is a formula reached only through one" makes candidacy depend on the path, which
  conflicts with "every clean formula cell is a candidate".
- **Line 1936:** `MAX_CACHED_CELLS` exists twice (`range_values.rs:35`, and
  `functions/lookup_and_reference/search_cache.rs:46`). Say which one goes.
- **Row 1962:** the RAND test needs T-5288's seeded draw, since today there is one unseeded generator. It is not
  listed under "Order and effort".

## Checked and holds (by the reviewer, with file:line)

`resize_to` clamps to the grid and anchors at the sum range's top-left (`functions/statistical/if_ifs.rs:39-48`,
`:325`); strict *IFS shapes give `#VALUE!` (`if_ifs.rs:320-323`); LOOKUP reads on down or right, a 2-D result is
`#N/A`, past the edge reads blank (`lookup.rs:349-384`); the research-note quotes and the semantics entries exist;
the cited symbols exist (`evaluate_root_cell`, `read_is_inert`, `MAX_KEPT_STATES`, `last_full_flat`, `RefMap::line`,
the used-area test and T-5184's hook); anchors run first and the Excel-order chain is separate
(`evaluation.rs:531-570`); the nesting flag is set where Design 13 says (`evaluation.rs:702-705`); the explicit stack
removes recursion-depth risk on a 1,000,000-cell chain; the restated read-ahead rule only errs toward rejection.

## Not checked

The measured figures (449-465 ms; 2.4-2.9× the allowance); iteration-on behaviour under a changed entry; Excel's own
behaviour at the grid edge; T-5152's sweeps and T-5297's twins; whether `MAX_INDEXED_RANGES` holds once rule
rectangles are added.
