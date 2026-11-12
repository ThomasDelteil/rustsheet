# Designs 12 and 13: second read

Text read: `/out/drafts/t5145/selfreview/spec-with-change-5.md`, lines 2232 to 2477 (line numbers below are that
file's). Engine: `/tmp/wt-design-probe` at `621a8367b` (paths below are under `crates/rustsheet-engine/src/`).
Neither design is implemented, so a probe shows what today's engine does, and what the design would do is
derived from its text and that code. I say which is which in each finding.

Probes (all ran, exit 0):
- `/tmp/review-c5/src/main.rs`, output `/tmp/review-c5/run.log`. Built with
  `CARGO_TARGET_DIR=/tmp/review-c4/target CARGO_INCREMENTAL=0 cargo build --release --offline` in `/tmp/review-c5`.
- `/tmp/review-c5/traversal_sim.py`, output `/tmp/review-c5/traversal_sim.log` (a count of the traversal's probes on
  the structure lines 2370 to 2374 describe; no engine).

## 1. Blocking findings

### B1 (High): the traversal runs plain cells in the anchors' phase, and a restart there has no reader the log can place

**Lines 2340 to 2347**, with 2388 and 2396. "before the loop evaluates a root R, an anchor or a cell, it evaluates the
formula cells of R's static footprint that are not evaluated yet ... pop T and evaluate it at depth 0
(`evaluate_root_cell`)", "Anchors still run first, in `anchor_order`", and line 2391, "Values in normal mode do not
change".

Counterexample (probe M): Z1 = FALSE, E1 = 10, E2 = 20, A1 `=IF(Z1,C3,0)+E1:E2`, C3 `=G2+1`, G1 `=A2+SEQUENCE(2)`.
Today: one pass, A1:A2 = 10, 20, G1:G2 = 21, 22, C3 = 23
(`M Z1=FALSE evaluation 1: ... restricted_pass_allowed=true spent=72`, the same at evaluation 2).

Under the design A1 is an anchor root with a footprint, and C3 is a candidate in it (the unread IF branch), so C3
runs before A1, in the anchors' phase, before G1. C3 reads G2 before G1 laid it: G1's spill contradicts that read,
or C3 meets G1's stale spill cell, and the pass restarts (`evaluation.rs:653-661`, `941-987`). The restart's reader
is the root on whose behalf the read was recorded (`evaluation.rs:884-892`). The design does not say which root
that is for a cell the traversal runs, and both answers fail:
- **C3 itself** (what "evaluate it at depth 0 (`evaluate_root_cell`)" gives, as `run_pass` does for a root,
  `evaluation.rs:556-558`). `RestartLog::learn` records the fact and returns without moving anything, because C3 is
  in no anchor order (`restart_log.rs:101-104`). The next pass is the same pass. Nothing caps restarts
  (`restart_log.rs:16-21`, "with no cap needed"); they end when thrown-away work passes the allowance, 2^30 units
  (`recalc_budget.rs:582-594`), at 72 units a pass here. `evaluation.rs:498-500` records this class: "without it the
  same restart repeated forever".
- **A1.** The log learns "G1 before A1". In the next pass G1 reads A2 before A1 ran, the log is asked for "A1 before
  G1", finds the loop and marks both anchors circular (`restart_log.rs:91-99`). Probe M with Z1 = TRUE shows what
  today's engine stores once both facts are learned: `A1:A2=0,None G1:G2=0,None C3=1`, both anchors `r=(1, 1)`.
  With Z1 = FALSE no formula reads C3, so nothing is circular: the values change from 10, 20, 21, 22, 23.

Probe L is the plain case (A1 `=C3+E1:E2`, C3 `=G2+1`, G1 `=SEQUENCE(2)`): today one restart (108 units, then 54),
whose reader is A1 because C3 runs inside A1.

Suggested text, after step 1: "A cell the traversal runs before an anchor is recorded as read on that anchor's
behalf (`Evaluation::root` stays the anchor), since the restart log orders anchors only. When such a cell, and not
the anchor's own formula, asks for a restart, the log learns nothing from it: the pass starts again with that anchor
on step 3's fallback for the rest of the evaluation, its footprint read on demand as today. So the facts the log
keeps are those of today's pass." Add both documents to "What it must show" (probe M: 10, 20, 21, 22, 23), with the
mutation "a fact learned from a cell the traversal ran".

### B2 (High): rule 1's three conditions do not give its conclusion

**Lines 2419 to 2422.** "A read uses an index only when all three hold: ... the reading root has a footprint whose
traversal completed without fallback; and the range's formula cells are all candidates. ... The range's formula
cells were then evaluated before the reader ran."

(a) **The reader is not the root.** A1 `=Z1+1`; Z1 `=COUNTIF(Q1:Q100,">0")+YEAR(Y1)` (no candidate: YEAR is off the
allowlist, `read_index/walk.rs:21-85`); Q1:Q100 `=P r*2`. A1 is the root (`evaluation.rs:210-212`: the root is the
cell the driver evaluates, "on whose behalf every read of the pass is recorded"), it has a footprint, {Z1}, and its
traversal completes: Z1 is no candidate, so "it nests on demand, as today" (line 2350). Z1 then reads Q1:Q100 inside
A1. All three conditions hold, and no Q cell is evaluated. Probe A, today's engine:
`root A1, Z1, depth limit None: ... restricted_pass_allowed=false spent=1849`, and at depth limit 2 `spent=1898`:
the Q cells are started inside Z1 inside A1 and unwind there. The control (root A201 after Z200) is flat and spends
1849 at both limits. An index built from that range reads cells of the last recalculation.

(b) **A candidate can be evaluated and circular.** A1 `=B1`, B1 `=A1`, C1:C3 `=A1+1`, `+2`, `+3`, D5
`=SUM(C1:C3)+E5`. C1:C3 are candidates and no cycle's members; D5's traversal completes without fallback (A1's fell
back, earlier). Today a reader of a circular cell's reader keeps its value (`circular.rs:32-41`), and both caches
refuse such a range so that every reader makes the read (`range_values.rs:158-161`, `sum_prefix.rs:237-241`). Probe
B: `B after E5=7: C1:C3=11,12,13 E5=7 D5=41`. A read through an index marks nothing, and D5 becomes 43. The sentence
after the rule (line 2424) keeps "a cycle's member" on today's path, which C1:C3 are not, and it is not one of the
"all three".

Suggested text: "A read uses an index only when all four hold: the pass is not in Excel's order; the read is made
by a root's own formula, the root alone on the stack, and that root has a footprint whose traversal completed
without fallback; the range lies in a rectangle of that footprint and its formula cells are all candidates; and no
cell of the range is circular or a cycle's member (`read_is_inert` gives `None`), since reading such a cell marks
the reader." Add to the Cycles row: "a SUM of one range and a SUMIFS over readers of a cycle: today's values".

### B3 (High): "A read inside the footprint is never ahead" keeps restricted passes a full pass would not match

**Lines 2411 to 2415.** "A read inside the footprint is never ahead: a full pass evaluates the footprint's formula
cells before the root, from its own loop, as the last full pass did. Every clean formula cell is a candidate or an
anchor ... and an anchor runs before every cell that is none."

(a) **A root that fell back.** A1 `=IF(Z1,B5,0)`, B5 `=IF(Y1,C5,1)`, C5 `=IF(Y1,2,B5)`, Z1 = Y1 = FALSE. B5 and C5
are a static cycle no value closes, so in a full pass A1 falls back and is "evaluated as today, on demand" (line
2358). Probe E: `Z1=FALSE: ... restricted_pass_allowed=true` (flat). Write Z1 = TRUE: the closure is A1, the
restricted traversal follows dirty cells only and meets no cycle, A1 reads clean B5 inside its footprint, and the
rule keeps the pass. The full pass of that state starts B5 inside A1:
`Z1=TRUE: ... restricted_pass_allowed=false spent=54`, and at depth limit 1 `spent=72`: 18 units more, the run of
A1 that unwinds, which the kept pass's bound does not hold. The last full pass did not run B5 before A1.

(b) **An anchor root.** A1 `=IF(Z1=1,C1:C2,E1:E2)` and C1 `=E1:E2*2`, both anchors, A1 ranked first. C1 is no
candidate, so no traversal runs it before A1. Probe F: `Z1=0: ... restricted_pass_allowed=true`, then
`Z1=1: A1:A2=2,4 restricted_pass_allowed=false`, and again false on the next full pass. Today's rule rejects this
by rank (`read_index/pass.rs:79-93`); the restated rule drops the rank test for a read inside the footprint.

Either way `last_full_flat` stays set for a state whose full pass nests, which
`test/test_restricted_pass.rs:725-752` exists to forbid.

Suggested text: "A read inside the footprint is not ahead when the last full pass ran the footprint's formula cells
before the root: the root is no anchor, and its traversal in that pass completed without fallback (the full pass
keeps the roots that fell back; a formula write clears them). A dirty root that fell back keeps today's rule. So
does a dirty anchor: a clean anchor ranked after it is ahead (`clean_cell_value`'s rank test), inside its footprint
or not." Add both fixtures to "The restricted pass" row, at depth limit 1.

### B4 (Medium): a criteria set charged to the formula that meets it first is the order of the pass again; the room is a capacity other formulas use up

**Lines 2431 to 2433 and 2427 to 2431**, against Design 12, lines 2281 to 2282 and 2288 to 2290. "A criteria set is
built when its criterion is first met, as today (`ifs_cache.rs`), and is charged to the formula whose run builds it"
and "Which builds fit is a function of the document". Design 12 leans on "built once per pass from the document, so
a read through one charges a constant" and "an indexed call's value and charge are functions of the cells in its
filed rectangles and of its heads".

- SPEC-0028 section 11 defines an index as built "counted in `T_pass` or charged once outside any formula's run"
  (`/tmp/wt-design/docs/specs/SPEC-0028-recalculation-work-budget.md:536-538`). Line 2332 drops that clause and line
  2432 contradicts it.
- 11.1's obligation (lines 1853 to 1857): a formula's charge "may depend on what ran before it in the pass only
  through state every pass builds alike". Every pass starts with empty caches, a restricted one too
  (`evaluation.rs:530-531`). With C1 `=COUNTIF(A1:A100000,"x")` and C2 `=COUNTIF(A1:A100000,"x")+B2`, a full pass
  charges the build to C1 and a constant to C2; after a write of B2 the restricted pass runs C2 alone and charges
  the build to C2. T-5172's guard (`read_index.rs:794-798`) keeps a stop at C2's own limit as "a full pass stops
  there too", and a full pass does not. This is the search cache's case (Design 4, "A search").
- The room is filled by other formulas, by values a literal write changes: a set is keyed by its criterion's value
  and costs its text (`ifs_cache.rs:117-127`, `398-406`), a prefix costs a state per cell
  (`sum_prefix.rs:201-207`). A write that changes a criterion cell read by a dirty formula adds a set; in a full
  pass of the new state a clean formula later in the order no longer fits and falls back, with another charge and
  11.1's flag set. The restricted pass builds only the dirty formulas' sets, sees room, and is kept.

Not probed: today's criteria reads charge a constant (SPEC-0028 line 640), so today's engine shows no unit
difference. This rests on the text and the code cited.

Suggested text: "A criteria set is built when its criterion is first met; its build is charged once to the pass,
outside any formula's run (SPEC-0028 section 11), so a read through it charges the constant whichever formula met
the criterion first, in a full pass and in a restricted one. A full pass records what its builds took of each room.
A restricted pass whose own builds, added to that, would pass a room is rejected: a full pass of the new state could
not make every build." Design 12's first bullet of precondition 2 then stands as written.

### B5 (Medium): the kept list outlives spill cells, and candidacy changes inside the pass that builds it

**Lines 2377 to 2381.** "A kept list holds the candidates, so it lives only while they are the same cells: a formula
write, a structural edit, a name or sheet op, and a pass in which a cell became a dynamic anchor or stopped being one
... each drop it".

- Rule 1 counts a spill cell as a non-candidate (line 2423) and "asks candidacy of such a list" (line 2380). A
  literal write moves spill cells with no listed event. Probe C: `Z1=2: A1=ArrayFormula(Dynamic, r=(1, 2))
  A2=SpillCell A3=none A5=none`, then `Z1=5: A1=ArrayFormula(Dynamic, r=(1, 5)) ... A3=SpillCell A5=SpillCell`
  (`model.rs:1044-1069`). A kept list still answers "all candidates" for A3:A9; the same document loaded builds a
  list that does not. The two replicas differ in charge and in 11.1's flag, and one engine differs with itself by
  whether the build took 2 ms (line 2377): the pass must be the same with the list kept or rebuilt.
- A cell becomes an anchor when it runs (`formula_cells.rs:327-360`, called at `evaluation.rs:783`). Probe C:
  `name over one cell: B1 kind=CellFormula`; after `update_defined_name` alone, which reparses and evaluates
  (`new_empty.rs:270-273`), `B1 kind=ArrayFormula(Dynamic, r=(1, 3)) B2 kind=SpillCell`. The list that pass built at
  its start (the name op dropped the old one) holds B1 as a candidate, and B1 lays spill cells during it. Dropping
  the list after the pass repairs the next pass, not this one's reads and charges.

Suggested text: "A kept list holds the candidates and the spill cells, so it lives only while they are the same
cells: a formula write, a structural edit, a name or sheet op, and a pass that made a cell a dynamic anchor or laid
or removed a spill cell each drop it. From that point such a pass reads through no index and is recorded as having
made a read that is not through one (rule 3). An index read records its range as read, as today's caches do
(`range_values.rs:12-13`), so a spill laid in it later restarts the pass. A pass with the list kept and one with it
rebuilt are the same pass: the test compares them."

### B6 (Medium): "functions of the document" leaves out two things the model keeps

**Line 2421**, "All three are functions of the document", and **line 2431**, "Which builds fit is a function of the
document, because the order of the pass is."

- `anchor_order` is history: kept across evaluations, new anchors appended, restarts reorder it
  (`evaluation.rs:196-200`, `positions.rs:107-134`). Probe D, the same four cells typed in two orders:
  `typed A1 first: A1:A2=1,2 C1:C2=2,4 ... restricted_pass_allowed=true` and
  `typed C1 first: A1:A2=1,2 C1:C2=2,4 ... restricted_pass_allowed=false`, each after two further evaluations. Anchors
  are roots with footprints (line 2340), so the order in which they meet criteria and fill the room follows it.
  SPEC-0028 already says so: "the learned anchor order decides the restarts" (lines 493 to 495), and its exception 3
  compares units only between replicas with one history (lines 606 to 611). The rows "Same units in two orders" and
  "The room's edge" assert more than that.
- Whether a pass is in Excel's order is left by the evaluation before it (`evaluation.rs:487-494`): the first pass
  after the last order-showing call is removed still runs in Excel's order.

Suggested text: "All three are functions of the document, of the anchors' order (`anchor_order`, which the order of
entry and past restarts set) and of whether the last evaluation ended in Excel's order." and "Which builds fit
follows the order of the pass; with more than one anchor that order is `anchor_order`, so two replicas agree on it
only with one history (SPEC-0028 section 11, exception 3)." Scope the two test rows to documents built with one
anchor order, or give the room to roots that are no anchors.

### B7 (Medium): the leaf rule admits formulas that read cells

**Lines 2351 to 2354.** "a **leaf**, a formula whose every function is on a listed set that reads no cell: TODAY,
NOW, RAND and RANDBETWEEN, ROW and COLUMN with no argument, and the allowlist's functions over constants. It reads no
cell".

The test is on functions only. `=TODAY()+A1` has one function, and it is listed. TODAY and NOW take a time-zone
argument here and read it (`functions/date_and_time.rs:443-447`); RANDBETWEEN evaluates both bounds
(`functions/math_and_trigonometry/random.rs:56-61`). Probe H: B1 `=TODAY(C5)`, B2 `=NOW(C5)`, B3
`=RANDBETWEEN(C6,C6)`, with C5 and C6 formulas below them: `B1=44873 B2=44873.467... B3=3
restricted_pass_allowed=false`, and with C5 giving "No/Such" both are `#VALUE!`. Each read a formula cell and nested.
A leaf that is a dynamic anchor (`=TODAY()+{1,2}`) is also a candidate by line 2351, against line 2349.

Suggested text: "a **leaf**, a formula that is no dynamic anchor, whose tree holds no reference, range, name,
structured or spill reference, and whose every function is TODAY, NOW, RAND, RANDBETWEEN, ROW, COLUMN or on the
allowlist."

### B8 (Medium): the traversal's cost misses the columns a rectangle meets, and it is not `T_pass`

**Lines 2372 to 2376.** "a wide rectangle visits the list's columns that meet it ... The traversal costs
O(F·α(F) + R·log F), for F candidates and R rectangles walked; each rectangle's lookup is charged to the pass, so it
is `T_pass`".

A rectangle over c list columns costs c probes even when every cell is evaluated. Simulation of the structure as
described, W columns by 20 rows, each cell reading the whole row above (`/tmp/review-c5/traversal_sim.log`):

| W | F | R | probes | F·4 + R·log2 F | ratio |
|---|---|---|---|---|---|
| 100 | 2,000 | 1,900 | 190,000 | 28,835 | 6.6 |
| 400 | 8,000 | 7,600 | 3,040,000 | 130,540 | 23.3 |
| 800 | 16,000 | 15,200 | 12,160,000 | 276,280 | 44.0 |

Probes are R·W: doubling F quadruples them. The footprint is a superset, so `=IF(FALSE,SUM($A$1:$XFC$1),0)` costs
today a few nodes and here one probe per formula column, uncharged. `T_pass` is the term "linear in the document's
size" that charges nothing (SPEC-0028 lines 483 to 491, 536 to 538); this is neither linear nor, as "charged to the
pass" reads, uncharged.

Suggested text: "The traversal costs O(F·α(F) + Σ c·log F), c being the list columns a walked rectangle meets. [One
of:] Each column probe is charged to the pass allowance at a fitted weight, a new charge with its DEVIATION entry
(SPEC-0028 section 11 rule 1); [or] a per-sheet structure answers 'the next candidate not evaluated in this
rectangle' in O(log² F), and the bound is O(F·α(F) + R·log² F)." Add to "The traversal's cost" row: "and W columns
of formulas each reading the row above: the probe count is linear in the cells at W, 2W and 4W".

### B9 (Medium): "both orders are flat" is false when the moved formula reads a formula the walk does not model

**Lines 2392 to 2393.** "an IF's unread branch can move a formula earlier; both orders are flat."

A1 `=IF(Z1,C3,0)`, B2 `=YEAR(Y2)`, C3 `=B2+1`, Z1 = FALSE. Probe G, today: `Z1=FALSE: ...
restricted_pass_allowed=true`: A1 does not read C3, B2 runs in row 2, C3 reads it evaluated. Under the design A1's
traversal runs C3 first; B2 is no candidate, so C3 starts it inside itself (lines 2350 and 2389). Probe G with
Z1 = TRUE shows that order today: `restricted_pass_allowed=false`. After 11.1 this workbook restricts at every
edit (B2 and C3 are in every closure and run in order); after Design 13 its full pass nests and every restricted
pass is declined. The row "Flat documents unchanged" does not cover it: a footprint reaches ahead.

Suggested text: "...so an IF's unread branch can move a formula earlier. The traversal does not take early a
candidate whose own footprint holds a formula cell that is no candidate and is not evaluated: that candidate keeps
today's place, so a full pass that is flat today stays flat." Add the fixture to the table.

### B10 (Medium): rule 2 removes the room that rule 1's "as today" path and two other readers draw on

**Lines 2434 to 2435.** "The `MAX_CACHED_CELLS` of `range_values.rs` goes with the index, which replaces what it
caches".

The index does not replace what that budget bounds. Rule 1 keeps today's cache for a range that holds a
non-candidate (lines 2422 to 2425, and the row "Today's cache kept"), and that path is `range_values`
(`range_values.rs:192`, `237`, `252`). The same budget bounds operator range operands (`cast.rs:399`), SUMPRODUCT's
block numbers (`functions/math_and_trigonometry/sumproduct.rs:507`, `562`; `ifs_cache.rs:299`), the value index
(`ifs_cache.rs:332`) and gathered values (`ifs_cache.rs:579`); see `range_values.rs:3-6`. Without it those are
unbounded (directive 5). The text also does not say what room today's prefix and match-set paths keep once
`MAX_KEPT_STATES` and `MAX_CACHED_BITS` are "the indexes' room": shared, the order of `read_is_inert` decides which
index builds fit.

Suggested text: "The `MAX_CACHED_CELLS` of `range_values.rs` stays, with today's path (rule 1) and its other
readers (operator range operands, SUMPRODUCT's masked blocks). The indexes' room is apart from today's caches'
rooms, so no read that is not through an index uses up an index's room; the bound is then twice 96 MB and 4 MB."

## 2. Non-blocking notes (Low)

- **N1, lines 2313 to 2314.** "a result range extended by 4" should be "to 4 cells". Probe J:
  `=LOOKUP(99,A1:B4,H1)` reads K1 (`A24=11000`), so H1 is filed as H1:K1, extended by 3; `=LOOKUP(99,A1:D2,F1:F2)`
  reads F4 (`A23=400`), extended by 2.
- **N2, lines 2428 to 2429 and 2467.** 96 MB and 4 MB count states and bits. Today an entry with no state counts as
  one state (`sum_prefix.rs:54-56`, `72-74`) and a set's entry as one word (`ifs_cache.rs:399-402`); keys, table
  slots and vector headers are outside the figure. Say "of states" and "of bits" in rule 2 as the test row does, or
  count the entries.
- **N3, rule 2.** State where the room ends: a set over 100,000 cells costs (1,563 + 1) x 64 = 100,096 bits
  (`ifs_cache.rs:400-402`), so 319 criteria fit. Problem 2's own "SUMIFS per row" with a key per row leaves the
  index at row 320 and declines as today. Three prefixes of a full column fit 4,000,000 states.
- **N4, line 2386.** "Every candidate is evaluated with its static inputs evaluated" holds for its candidate inputs
  only; lines 2389 and 2357 say otherwise for the rest.
- **N5, lines 2348 to 2349.** A Ctrl+Shift+Enter anchor is "no dynamic anchor", so a candidate by the letter, while
  `indexed_formula` refuses it (`read_index.rs:594`). Say which.
- **N6, Design 12, lines 2258 to 2260.** "written references or ranges": say whether a name (11.2), a parenthesized
  range or `@` counts. LOOKUP's array form is not named; it reads within its array (`lookup.rs:253-259`; probe J
  `A26=40`), so it needs no rule, only precondition 1.
- **N7, lines 2416 to 2417.** A static cycle no value closes (probe E's B5 and C5) is flat today and restricts when
  the edit does not flip the branch. Under step 3 every restricted pass that holds both cells meets the static cycle
  and is rejected. Say so under "What it guarantees".
- **N8, line 2383.** "bounded by the formulas, their footprints and the stack's F cells": say that a frame holds its
  cell and a cursor, not its rectangles. A filled column shares one parsed formula (`read_index.rs:677-680`), so F
  stacked frames that each hold a footprint are not bounded by the document's size.
- **N9, rule 1.** Today's caches also ask `pass_cache_safe` (`evaluation.rs:401-408`): a read made while the run
  unwinds or the pass is abandoned gets empty values (`evaluation.rs:577-582`). An index built in such a run must
  not be kept.

## 3. Checked and found to hold

- SUMIF and AVERAGEIF resize from the sum range's top-left cell, clipped to the grid: `if_ifs.rs:39-48`, `329`;
  probe J `A20=600`, `A21=600`.
- LOOKUP's lookup vector is searched along its first row when wider than tall, else its first column, so its length
  is the larger side: `lookup.rs:169`. Read-on is down for a column, right for a row or one cell: `lookup.rs:335`,
  `364-384`; probe J `A22`, `A23`, `A24`.
- A two-dimensional result vector is `#N/A` and reads no cell: `lookup.rs:332-334`, `363`; probe J `A25`.
- LOOKUP's lookup vector and written result range go through `range_search_vector` and its cut at the last used row;
  a row is cut only when it reaches the last column: `lookup_table.rs:102-128`, `596-627`, `635-657`; `lookup.rs:340`.
- *IFS and COUNTIFS shapes are strict: `if_ifs.rs:243-247`, `322-326`. Their walked blocks stay inside their ranges:
  `if_ifs.rs:526-548`, `617-625`.
- Whether a call lifts is a fact of its values: `if_ifs.rs:122`, `lift/driver.rs:41-43`. The static result covers
  the listed criteria: a one-cell range is a 1x1 array and INDEX is `Unknown`
  (`expressions/parser/static_analysis.rs:431-436`, `829-830`).
- `instantiate` refuses a range to the last row or column and a reference in the last column:
  `read_index/walk.rs:222`, `236`.
- Design 12's citations exist: `docs/excel-semantics/criteria.md:380`, `lookup.md:230`, `lookup.md:1683`,
  `docs/research/recalc-modes-and-dependency-tracing-2026-11-11.md:41`, `356` (read in `/tmp/wt-design`).
- Design 12's structural rule on six cases (a row or column inserted inside, just past and before the rule's
  rectangle; the sum range on another sheet; LOOKUP's one-cell result): each case that changes a value is in D0. By
  reading; Design 8 is not on main.
- Constants: `MAX_KEPT_STATES` 4,000,000 (`sum_prefix.rs:38`); `SumParts` is three `f64`
  (`functions/math_and_trigonometry/sum.rs:237-242`); `MAX_CACHED_BITS` 32,000,000, "4 MB" (`ifs_cache.rs:81-82`);
  `MAX_CACHED_CELLS` in both files (`range_values.rs:35`, `search_cache.rs:48`).
- Both caches follow `read_is_inert`: `sum_prefix.rs:237`, `range_values.rs:158-182`, `228-236`.
- Problem 1: the total above its data nests and the one below is flat, with the same units. Probe I:
  `total in C1: ... restricted_pass_allowed=false spent=180012`, `total in C10001: ... true spent=180012`.
- "2.4 to 2.9 times" and "2 times" are SPEC-0028's: lines 641 to 642.
- The N example: both entries mark the same cells and each keeps its stored value: `circular.rs:20-41`, `47-55`,
  `evaluation.rs:681-692`. By reading, not run.
- `evaluate_root_cell`, `last_full_flat` and the anchors' ranks exist as cited:
  `functions/logical/lambda_scope.rs:181`, `recalc_budget.rs:384`, `read_index/pass.rs:34-42`.
- Design 4's rule today is natural order, anchors by rank: `read_index/pass.rs:79-93`. A clean cell is not
  `Evaluated` in a restricted pass: `read_index/pass.rs:66-98`.
- The evaluator stores a cell's kind with its value: `model.rs:911-936`, `1048-1055`.
- An evaluated cell stays evaluated in this pass, so "passed once" holds: the two removals are Excel's order and
  iteration only (`evaluation.rs:730`, `iteration.rs:241`).
- With no argument TODAY, NOW, RAND, ROW and COLUMN read the clock, the generator or the cell's own place, and no
  cell: `functions/date_and_time.rs:443-445`, `functions/math_and_trigonometry/random.rs:44-49`,
  `functions/lookup_and_reference/mod.rs:494-513`.
- Lens 1 (a change undone inside a pass): nothing beyond B3 and B5. Lens 2 (nodes against walks): B8, and nothing in
  Design 12, where the resized rectangle replaces the written sum range as the one walked (`if_ifs.rs:329`,
  `607`), so `MAX_INDEXED_RANGES`'s count holds.

## 4. Not checked

- Anything Design 13 does at run time: there is no implementation. Each probe shows today's engine on the
  finding's document, and the design's behaviour is derived from its text; B4's unit difference cannot be shown
  today.
- The native timings of line 2329 (449 to 465 ms, 36 ms), the 2 ms threshold and the 60,000-formula book.
- Excel itself: no oracle run. Nothing here is a claim about Excel beyond what the code's cited evidence says.
- The facade (`crates/rustsheet`): `writes_only_literals`, the edit path, change capture.
- Design 12 against Designs 8 and 10 as built (neither is on main), T-5184's hook and the used-area oracle beyond
  their existence (`test/test_used_area_input.rs:397`, `885`).
- Designs 1 to 11, except where 12 or 13 lean on them.
- The crate's test suites; only the probe binary was built.
