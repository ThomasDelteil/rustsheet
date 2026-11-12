# Design 13: third read

Text read: `/out/drafts/t5145/selfreview/spec-with-change-5-v2.md`, lines 2319 to 2593, with Design 12 (2231 to
2317), Design 4 (502 to 710), Design 8 (770 to 911), Design 9 (1270 to 1282, 1350 to 1416), Design 11.1 and 11.3,
the Rollout (2764 to 2803) and SPEC-0028 section 11 (`/tmp/wt-design/docs/specs/SPEC-0028-recalculation-work-budget.md`,
461 to 712). Line numbers below are the v2 file's unless a file is named. Engine: `/tmp/wt-design-probe` at
`621a8367b`, paths under `crates/rustsheet-engine/src/`.

Design 13 is not implemented. **Ran** means today's engine on the finding's document, or a count on the list
structure with no engine. **Derived** means read off the text and the code lines cited. Each finding says which.

What I ran (all exit 0):
- `/tmp/review-c6/src/main.rs`, output `/tmp/review-c6/run.log`: probes P1 to P5 on today's engine. Built with
  `CARGO_TARGET_DIR=/tmp/review-c4/target CARGO_INCREMENTAL=0 cargo build --release --offline` in `/tmp/review-c6`.
- `/tmp/review-c6/traversal_orders.py`, output `/tmp/review-c6/traversal_orders.log`: probe counts of one frame under
  three readings of "placement order from T's cursor". No engine.

## 1. Blocking findings

### F1 (High): the record and the flatness it stands for have no lifetime under a restricted structural edit (Design 8) or a maintained formula write (T-5166)

**Lines 2479 to 2480**, "A full pass records which cells were complete, as it records the anchors' ranks
(`Incremental::anchor_rank`)", and **2509 to 2511**, "the record stands ... why one full pass's record serves every
restricted pass until the next". Against **Design 8, lines 880 to 881**: "The order of the cells that remain is
unchanged, so a full pass of the new state reaches each clean formula after the same cells it reached before: the
last full pass's flatness carries over."

Design 13 names no structural-edit rule (11.1, 11.2, 11.3 and 12 each have one). Under this pass a cell's turn
follows other cells' footprints, so Design 8's sentence is no longer true, and the record is keyed by place as the
ranks are. Design 8 refuses a workbook with anchors because the ranks are keyed by place (786 to 791) and drops the
spend map for the same reason (888 to 890). Nothing drops or shifts this record.

Counterexample (Z21 = Y21 = FALSE, K2 = 5, B2009 = 1):
- `A1: =B9*1`
- `A2: =IF(Z21,B3,0)+B9+K2`
- `B3: =IF(Y21,C3,1)`, `C3: =IF(Y21,2,B3)` (a static cycle no value closes)
- `B9: =B10+1` filled down to B2008

Derived, the design's full pass: A1 is a candidate whose footprint is the chain, so it takes B9 to B2008 early and
is complete. A2's first unevaluated formula cell in placement order is B3; B3 pushes C3, C3 finds B3 on the stack,
and A2, B3 and C3 are left (step 3). A2 runs at once and reads B9, which is evaluated. The pass is flat.

Delete row 1 through Design 8. No rectangle with a surviving reader changed, and the other formulas moved whole
with what they read (not rewritten: Design 8, 847 to 852; "moved without rewriting", 1907). The dirty set is empty,
so the edit is a skip (792 to 793) and `last_full_flat` stays set. In the new state nothing takes the chain early:
the old A2, now A1, is left by the cycle before its cursor reaches the chain, runs as today and starts B8 inside
itself.

Ran (P1, today's engine on the document the delete leaves, whose root is left under the design too):
`P1 row-1 root deleted chain 2000 depth None: reader A1=Ok(Number(2006.0)) restricted_pass_allowed=false spent=71814`
and `depth Some(1): ... spent=72102`: the pass nests, and at depth limit 1 it throws away 288 units that no kept
pass's bound holds. `test/test_restricted_pass.rs:725-752` exists to forbid `last_full_flat` set for such a state.

Two consequences, both derived:
- With the record kept by place, place A1 held a complete cell. Write K1 = 6: the closure is A1, the record holds
  A1 as complete, its read of B8 is "inside the footprint ... not ahead" (2486 to 2488), and the pass is kept. This
  is the second read's B3(a) again, through a stale record.
- With the record shifted, A1 is a left root and that edit is rejected, but every edit that does not touch A1 is
  kept on a state whose full pass nests.

A formula write that T-5166 maintains without a full pass does the same with no structural edit (derived):
- In the document above, rewrite A1 from `=B9*1` to `=1`. The closure is A1, the pass is kept, and the full pass of
  the new state starts B9 inside A2, as after the delete. No handling of the record changes that.
- Taking the written cell out of the record is not enough. With `B5: =YEAR(Y5)`, `C7: =B5+D9`, `D9: =E9*2` and A1
  rewritten from `=1` to `=IF(Z1,C7,0)`, Z1 = FALSE, the pass that follows the write runs A1 as today, then B5,
  then C7, which the record holds as complete, and is kept. The full pass of the new state leaves C7 (B5 is not
  evaluated when A1's traversal reaches it), and at row 7 C7 starts D9 inside itself. The record still holds C7
  and D9 as complete.

T-5166's text is not in this file (CR-321604031); what decides this part is whether it clears Design 4's records
on a formula write. Design 9's `calculate` leans on the same records (1367 to 1369).

Suggested text, a new bullet after "The restricted pass follows the same order":
"**Lifetime.** The record, and the flatness it stands for, are functions of the formulas, their places and the
cells' kinds: under this pass a cell's turn follows other cells' footprints, not its place alone. A literal write
changes none of them. Whatever does, without a full pass, drops the record and declines restricted passes until
the next full pass: a formula write that Design 2's maintenance keeps off a full pass (T-5166), and a structural
edit that Design 8 restricts. Design 8's 'Why the pass stays exact' holds for natural order; its 'When' gains: the
last full pass took no cell early. Otherwise the edit runs a full pass." Add both documents to "What it must show",
with the mutation "the record kept across a restricted structural edit".

### F2 (Medium): rule 1 makes index use depend on history, which the design's own definition and SPEC-0028 section 11 exclude; and the anchors' order is no longer one of its inputs

**Lines 2517 to 2519**, "All four are functions of the document, of the anchors' order (`anchor_order`, which the
order of entry and past restarts set) and of whether the last evaluation ended in Excel's order", and **2549 to
2551**, "Which builds fit follows the order of the pass, and that order holds `anchor_order` wherever the workbook
has more than one anchor". Against **2335 to 2336**, "whether a read uses it is a function of the document alone",
and SPEC-0028 lines 539 to 543 ("a function of the document alone: not of the evaluation order") and 620 ("A new
charge adds no other exception").

- **Excel's order is history, and it already moves units.** `evaluation.rs:487-494`: the first pass of an
  evaluation runs in Excel's order when the evaluation before it ended so, whatever the document now holds. Ran
  (P4): `A1: =WEEKNUM(45000,1)`, `A2: =WEEKNUM(45000,B2)`, A3 to A40 `=A(r-1)+1`. Built with B2 = 1:
  `spent=720`. Built with B2 = 22, then B2 = 1, the same document: `spent=1176`; evaluated again: `spent=720`.
  Under rule 1 the 1176-unit evaluation reads through no index and the other does, so once a read that is not
  through an index charges its walk (SPEC-0028 rule 1) the two replicas differ in more than a thrown-away pass.
- **The anchors' order is stale text.** Derived. The sentence is the second read's B6 fix, written when anchors
  were roots with footprints. Since the ruling, an anchor runs no traversal and reads through no index (2384 to
  2386, 2525), and a cell an anchor starts is not alone on the stack. Index reads and builds are made by complete
  cells in the loop, after every anchor has run. Which plain cells the anchors evaluated on demand is a function of
  the final pass's values, not of the order that reached them. I found no path by which `anchor_order` changes
  index use or which builds fit in the pass that counts. If the authors know one, the text should name it.
  The test rows "Same units in two orders" and "The room's edge" are scoped by the same clause.

Suggested text: "All four are functions of the document: an anchor makes no index read and no build, and the loop
runs after every anchor has, so `anchor_order` is no input. The first condition is the exception. Whether a pass
is in Excel's order is left by the evaluation before it (`evaluation.rs`), so [one of:] a read that would use an
index but for the pass's order charges what the index read charges, the same units by either path; [or] an
evaluation that began in Excel's order and ends with nothing showing is not one whose units are compared, which is
a new exception to SPEC-0028 section 11 and needs its owner's ruling." Drop the `anchor_order` clause of rule 2.

### F3 (Medium): rule 1's third condition cannot be decided in a restricted pass from the lists that pass has

**Lines 2485 to 2486**, "The pass's lists hold its dirty cells only", **2432 to 2433**, "above 2 ms to build, they
are kept between passes", and **2443**, "Rule 1 of the index reads below asks candidacy of such a list". Rule 1
(2515 to 2516): "the range lies in a rectangle of that cell's footprint and its formula cells are all candidates".

A formula the walk does not model is always dirty, so it is in the restricted list. A clean dynamic anchor with a
modelled formula, its spill cells (11.3: "It files nothing") and a clean one-cell scalar anchor are non-candidates
that are not dirty. With `E1: =A1:A3*2` and `G5: =SUM(E1:E3)+B1`, a full pass reads E1:E3 by today's path (the
range holds an anchor and spill cells). After a write of B1 the restricted list holds G5 alone, the range shows no
formula cell, and the read goes through an index. A list under 2 ms is not kept, so there is nothing else to ask.

Derived; no unit differs today, since both paths charge a constant. They differ once a read that is not through
an index charges its walk, which is what this design is the precondition of. Building the full list in every
restricted pass is O(F log F), against "bounded by the closure".

Suggested text: "In a restricted pass a clean formula cell is a candidate unless its cell is an array anchor or a
spill cell (every formula the walk does not model is dirty). Rule 1 asks the cell's kind for the clean cells of a
range, once per range and pass, and the pass's list for the dirty ones." Add to "A kept list" row: "a restricted
pass and a full pass make the same reads through an index over a range that holds a clean anchor's spill".

### F4 (Medium): the build record is by key, and a running total's build grows

**Lines 2545 to 2549.** "A full pass records which builds it made and what each took of its room (their keys, not
their sets). A kept restricted pass adds the builds it made that the record does not hold ... A restricted pass
that would take the record past a room is rejected".

A criteria set's size is fixed by its key. A prefix is not: it "keeps one state per stored cell of its start's
prefix" (2537) and grows as longer ranges are read. Take running totals `=SUM($B$1:Br)` in enough columns to pass
4,000,000 states. In a full pass the last start's prefix stops at some row r*, and reads past it "fall back to
today's paths" (2540). For a SUM that sets no flag (11.1's table, line 1862: "none"), so restricted passes run.
A restricted pass starts with an empty room: a dirty cell at row r* + 1000 builds that start's prefix to its own
row, and it fits. The record holds the key, so nothing is added and the pass is kept. The full pass of the new
state reads that cell by today's path.

Derived. The two paths charge alike today; they differ once today's path charges its walk. Also, "a full pass of
the new state could not make every build" (2548 to 2549) is not what the test shows: the record never loses a
key, so it is a superset. The rule is conservative, and the reason given is too strong.

Suggested text: "... adds the builds it made that the record does not hold, and for a build the record holds,
what it took beyond the record's figure (a prefix read further than the full pass built it). A restricted pass
that would take the record past a room is rejected: the record holds every build a full pass of the new state
makes, so within the room that pass makes them all." Add to "The room's edge" row: "running totals in enough
columns to pass the states' room: a restricted pass that reads past where the full pass's prefix stopped is
rejected".

### F5 (Medium): a charge outside any formula's run is new, and Design 4's per-cell rule does not carry it

**Lines 2541 to 2543**, "Its build is charged once to the pass, outside any formula's run". Against Design 4,
**lines 579 to 581**, "a restricted pass sets `committed` to `bound.committed - old(D) + new(D)`", and **614 to
616**, "a full pass of the new state commits each clean cell's recorded work plus each dirty cell's new work. Work
committed outside any cell's run is never taken off, so it can only raise the bound."

Today a pass's committed sum grows only where a run settles (`recalc_budget.rs:650-657`), so Design 4's sentence
is complete. With this design a full pass also commits its builds. With
`C1: =COUNTIF(A1:A100000,G1)`, `C2: =COUNTIF(A1:A100000,H1)` and G1 = H1 = "x", a full pass makes one build. Write
H1 = "y": the restricted pass makes the build for "y". A full pass of the new state makes two. The per-cell rule
gives `bound - old(C2) + new(C2)`, which holds one build: the bound is one build short for each new criterion a
clean cell's old one outlives. The summing bound (`recalc_budget.rs:371-381`) adds the pass's whole committed sum
and stays an upper bound.

Derived; T-5167 is not on main.

Suggested text, in rule 2: "A build's charge is committed work outside any cell's run. Under Design 4's per-cell
spend a restricted pass adds to the bound the charge of each build it made that the record does not hold; the
summing bound adds every build it made." And amend Design 4, line 614: "... plus each dirty cell's new work, plus
its builds (Design 13)".

### F6 (Medium): the kept list's drop set misses a literal written over an anchor, a spill cell or a formula cell

**Lines 2433 to 2435.** "a formula write, a structural edit, a name or sheet op, and a pass that made a cell a
dynamic anchor or laid or removed a spill cell each drop it".

The write removes the spill cells, not a pass. Ran (P2), `A1: =SEQUENCE(5)`:
- `P2 literal over the anchor, before a pass: A1=literal A2=none A5=none`
- `P2 literal over a spill cell, before a pass: A1=ArrayFormula(Dynamic, r=(1, 1)) A2=none A3=literal A5=none`,
  and after the pass the same kinds, `A1 value=Ok(String("#SPILL!"))`: the pass lays and removes nothing.
- `P2 literal over a formula cell, before a pass: C1=literal`

None of the three is a formula write, and no pass lays or removes a spill cell. A kept list still holds A2 to A5
as spill cells, so rule 1 keeps a range over them off the index where a rebuilt list does not, and it holds C1 as
a candidate that is never evaluated. "A pass with the list kept and one with it rebuilt are the same pass" (2442)
fails. `exhaust_recalc` (`recalc_budget.rs:615-639`) also retires spills outside any pass. In the other direction,
every pass writes each spill cell again (`model.rs:1044-1066`), so "laid" read as an event drops the list at every
pass of a workbook that spills: the test has to be on the set of cells, not on the writes.

Suggested text: "... so it lives only while they are the same cells. Whatever makes the read index stale drops it
(Design 2: a written cell that was or became a formula, an array formula or a spill cell among them), and so does
a pass, or an exhausted recalculation, that leaves another set of anchors or spill cells than the list holds." Add
the three writes to the "A kept list" row.

### F7 (Medium): the probe count holds only for an order the text does not give, and fails for the one it names

**Lines 2346 to 2347**, "take the next formula cell of T's footprint, in placement order from T's cursor", **2405
to 2407**, "A frame makes one probe for each list column each of its rectangles meets, and one more for each cell
it takes. So the traversal makes at most R + E + F probes", and **2446 to 2447**, "a frame holds its cell and a
cursor, not its rectangles".

Elsewhere "placed before" and "placed after" are (sheet, row, column) order (2456, 2489). The lists are by column.
With a cell cursor and that order, the next cell is the least over every list column of every rectangle, at each
step. Ran (`traversal_orders.log`): one frame over n columns that each hold one formula cell, on a diagonal:

| n | E | 16 F | cells taken | probes, (row, column) order | probes, column by column | the text's count |
|---|---|---|---|---|---|---|
| 500 | 499 | 8,016 | 500 | 250,500 | 1,000 | 1,000 |
| 1,000 | 999 | 16,016 | 1,000 | 1,001,000 | 2,000 | 2,000 |
| 2,000 | 1,999 | 32,016 | 2,000 | 4,002,000 | 4,000 | 4,000 |

Keeping one running minimum per row sweep gives the same figures (250,000; 1,000,000; 4,000,000). A frame with
400 one-cell rectangles and one range of 8,000 cells: 3,208,401 probes against 8,401. Every layout is under the
cap, and the probes charge nothing. The row "The traversal's cost" does not catch it: where every formula reads the
row above, each probe finds nothing and the orders cost the same.

The count holds when the frame goes rectangle by rectangle, column by column, top to bottom (the third column of
the table, and my derivation for left frames and cycles in part 3).

Suggested text: "... take the next formula cell of T's footprint that is not evaluated, in the frame's order: its
rectangles in the walk's order (`instantiate`), a rectangle's list columns from left to right, a column from top
to bottom. The cursor is that triple and only moves forward. This is not placement order, and nothing rests on
which of a footprint's cells is taken first." Add to the cost row: "one formula over n columns that each hold one
formula cell on a diagonal: 2n probes at n = 500, 1,000 and 2,000".

### F8 (Medium): a full pass can be flat while every restricted pass after it is rejected, where today's are declined

**Lines 2353**, "When the root R is left, it is evaluated at once, as today, its footprint read on demand", with
**2372** ("the cells it already evaluated stay evaluated") and **2488 to 2489**, "A root that was left reads ahead
when it reads a clean formula cell placed after it, inside its footprint or not."

Z1 = FALSE, Y9 = 45000, E5 = 4, `A1: =B5+IF(Z1,B9,0)`, `B5: =E5*2`, `B9: =YEAR(Y9)`.
Ran (P5), today: `A1=Ok(Number(8.0)) ... restricted_pass_allowed=false`. A1 starts B5 inside itself, so the edit
path declines without walking a closure (11.1, lines 1873 to 1875).

Derived, the design: A1's traversal takes B5 early, then meets B9, which is no candidate, and A1 is left. A1 runs
at once and reads B5 evaluated. The pass is flat, and restricted passes are allowed. A1 reads an always-dirty cell,
so it is in every closure, and it is a left root: its read of B5, clean and placed after it, is a read ahead. Every
edit runs a closure walk, a restricted pass, a put-back and then the full pass.

The same happens with no left root: `A1: =SUM(B5:B100)` over formulas and `A2: =TODAY()-B5`. A1 takes the column
early, A2 reads B5 evaluated, the pass is flat, and A2 is in every closure and is rejected ("an always-dirty root
that reads a clean cell placed after it is still rejected", line 2588). Design 4 names this cost (522 to 523:
"more than with incremental recalculation off").

Suggested text, in "The restricted pass follows the same order": "A full pass in which a cell that ran with no
traversal (a root that was left, a formula the walk does not model) read a formula cell placed after it records
that, as it records nesting, and declines restricted passes: every restricted pass that holds that cell would be
rejected." Or, to keep those passes: record each formula cell's turn in the full pass, run a restricted
pass's cells in that order, and call a read ahead when the clean cell's turn is later than the reader's. Add P5 to
the "Restricted pass" row as "declined, not rejected".

### F9 (Medium): the cap refuses a common layout whose traversal is cheap, and takes the whole workbook off the indexes

**Lines 2419 to 2421.** "When E is more than 16 for each formula cell (E > 16 F), no root is traversed: the pass is
today's, in natural order, no read uses an index, and no restricted pass traverses."

E counts the list columns between a rectangle's left and right edge ("two binary searches a rectangle"), so a
column counts when it holds a formula anywhere on the sheet. A table of literals with W columns, a total under
each column and `=SUM(Br:<last>r)` on each of n rows has F = n + W and E = n(W - 1). It is refused when
n(W - 17) > 16 W: from 289 rows at 18 columns, 55 rows at 24, 36 rows at 31. A gradebook with 30 score columns, an
average row and a SUM and an AVERAGE on each row is refused from 19 rows (58 n > 16 (2 n + 30)). Its traversal is
60 probes a row, all among a row's neighbours (9 to 24 ns measured, line 2423).

The refusal is of the workbook, and "no read uses an index" makes every conditional aggregate in it a read that
sets 11.1's flag (rule 3). So after Design 12 one such sheet leaves the workbook with no restricted pass.

Derived; arithmetic on the text's definition. Not counted on a corpus: the 336 workbooks under the worktree are
function tests. What decides the severity is the corpus count the text already plans (2424 to 2425); this is the
family to count.

Suggested text: "When E > 16 F and E is more than 2^20, no root is traversed ... Under the cap the traversal
makes at most R + F + max(16 F, 2^20) probes." Both terms are functions of the formulas, so replicas still refuse
alike. Or measure E against the sheet's stored cells, which is SPEC-0028's own measure of `T_pass`, once rule 5's
canonical storage holds.

### F10 (Medium): three places still say what the ruling on anchors withdrew, and one of them is a correctness rule

- **Rollout, lines 2781 to 2783**: "after it a workbook whose totals sit above their data, or whose anchors read
  formulas, is restricted, and searches and conditional aggregates no longer decline". Against 2384 to 2395
  (anchors keep today's evaluation, "such a workbook's restricted passes stay declined") and 2565 ("The search
  cache's row stays").
- **Line 2447**: "Design 9's cones and Design 10's `precedents` use the same lists and the same traversal", with
  **Design 9, lines 1278 to 1279**: "The traversal is Design 13's, started at the targets and kept to owed cells."
  This traversal now starts at no anchor, leaves a frame at the first cell it cannot take, runs in a restricted
  pass only for a root the record holds, and is refused whole by the wide count. A cone must hold "every owed
  precedent of its cells" (1282, and 1400 to 1401). Derived: with `A5: =K5*2`, `A9: =IF(Z1,B12,0)+A5` and the
  static cycle `B12: =IF(Y1,C12,1)`, `C12: =IF(Y1,2,B12)`, A9 is a left root. After a write of K5, A5 and A9 are
  owed. A root the record does not hold gets no traversal (2486 to 2487), so the cone of A9 is A9 alone. A5 is
  placed before A9, so the read is not ahead, and `compute` returns A9 from the stale A5. For an anchor target
  the read-ahead rule rejects the cone pass instead, so every `compute` of an anchor that reads an owed formula is
  a full pass.
- **Design 10, line 1682**: "of the kind Design 13 builds for its pass, here with spill cells listed too". Design
  13's lists now hold the spill cells (2433).

Suggested text: line 2447, "Design 9's cones and Design 10's `precedents` use the same lists. Their walk is not
this traversal: it enters every formula cell a footprint holds, an anchor, a spill cell's anchor and a left cell
included, the wide count does not refuse it, and its bound is its caller's (Design 10's budget; past it a cone is
everything owed)." Design 9, line 1278: "The walk uses Design 13's lists". Rollout: "after it a workbook whose
totals sit above their data is restricted, and conditional aggregates that read through an index no longer
decline (with Design 12)".

## 2. Non-blocking notes (Low)

- **L1, lines 2437 to 2441.** A cell that becomes an anchor in the loop can cost one restart that today's pass does
  not make: a candidate taken early reads where it will spill. Ran (P3), with a reader of C2 placed after the
  promoted C1: `spent=39`; placed before it: `spent=78` (the pass ran twice), `spent=39` at the next evaluation.
  With `A1: =IF(Z1,D5,0)` and `D5: =C2+1` the traversal makes D5 the early reader (derived). The values are the
  same, and the promoted cell joins `anchor_order` after the restart (`evaluation.rs:498-503`), so it ends. Say so.
  A promoted root has also had a traversal by the time it turns into an anchor ("a root R that is no anchor" is
  asked before it runs, `formula_cells.rs:327-362`); I found no harm in it, only the restart above.
- **L2, lines 2467 to 2468.** "its cells are left" is not always so. With `A1: =IF(Z1,B5,0)`,
  `B5: =IF(Y1,B3+C5,1)`, `B3: =YEAR(Y3)` and `C5: =IF(Y1,2,B5)`, B5 is left at B3 before C5 is pushed, and C5,
  whose footprint is evaluated when the loop reaches it, is complete. I found no harm: every read that closes the
  cycle is still made on demand by a left cell.
- **L3, lines 2346 and 2357 to 2358.** Say when a spill cell counts as evaluated (when its anchor is,
  `range_values.rs:168-180`), and that a Ctrl+Shift+Enter anchor is a root with no traversal ("a root R that is
  no anchor" reads as a dynamic one). Line 2401 says "a list of formula positions", 2433 "the candidates and the
  spill cells".
- **L4, line 2580.** "`=IF(FALSE,SUM($A$1:$XFC$1),0)` filled down beside 800 formula columns is refused alike"
  depends on the counts. Beside 800 columns of one formula each it is refused from 17 rows (800 x 17 = 13,600
  against 16 x 817 = 13,072) and not at 16. Beside the W = 800 layout of the same row the workbook is refused
  without it. Give the rows.
- **L5, lines 2485 to 2486.** "the traversal's work is bounded by the closure": by the closure's rectangles and
  the dirty list columns they meet, which is at most E, not the closure's size. The restricted pass has no cap of
  its own.
- **L6, lines 2421 to 2422.** "E and F are functions of the formulas and of which columns hold one": if the lists
  hold spill cells, also of the spills' extents. A literal that moves a spill is a full pass (11.3), so the
  consequence holds.
- **L7, lines 2538 to 2540.** 319 criteria sets leave SPEC-0028's own lower shape, a per-row SUMIFS over 100,000
  rows (its lines 649 to 653), outside the index for 99.7% of its rows. Today's path answers those rows from the
  range's value index (`ifs_cache.rs:147-155`, `311`), which this design does not make an index. Charged as walks
  that shape is still twice the allowance. Say whether the value index joins.
- **L8, rule 2.** The record only grows between full passes. A workbook near the room (300 sets of 100,096 bits)
  rejects one restricted pass in about 19 edits of a criterion cell. The same drift as Design 4's summing bound.
- **L9, lines 2351 to 2352.** A left cell gets no traversal when the loop reaches it. A second traversal there
  would be sound (every cell placed before it has run) and would cost one more frame a cell. As written,
  `C7: =B5+SUM(D8:D9999)` loses the pass's benefit when any earlier formula names C7 in an unread branch.
- **L10, lines 2379 to 2382.** With no index every full pass walks every formula's footprint twice (the count of
  E, then the traversal). Measure it with the lists; a workbook past the index's memory cap pays it at every edit.
- **L11, line 2354.** Say whether a root with no footprint (a formula the walk does not model) is "complete". It
  has no rectangle, so rules 1 and 2 never ask, but the record's definition admits it.
- **L12, line 2582.** "replicas with one history and both entry orders": say entry orders of what.

## 3. Checked and found to hold

**Area 1, "left" and "complete", for edits of literals (what restricts on main).** I could not break it. Tried:
- *A left/complete decision that follows values or history.* In a flat pass no cell is started on demand, so the
  evaluated set at each root's turn follows the formulas, places and kinds alone. A cell an anchor starts on demand
  makes the pass nest, and no restricted pass follows.
- *The guard of rule 3.* Every formula cell of a complete root's footprint was placed before the root and ran at
  its place, or was taken early and is in the record. A clean cell's footprint holds no dirty cell (it would be in
  the closure), so a traversal kept to dirty cells reaches every dirty cell the full one reaches.
- *A kept pass whose full pass nests.* A complete cell reads inside its footprint (`walk.rs:21-85`, `205-253`). A
  left root, a formula the walk does not model and an anchor keep today's rule, and a cell placed before a reader
  has run by the reader's turn in either order.
- *Static cycles*, entered from each member and from a reader above: the cell that first reads into the cycle by
  value is today's, with the same stack (`circular.rs:20-41`).
- *Depth and work limits.* A cell taken early runs at depth 0 from an explicit stack. T-5172's guard is by sums,
  not by order (`read_index.rs:837-847`).
- *A record gone stale between two restricted passes of literal writes.* None found; F1 is the other kinds of edit.
- *A document flat today that this pass makes not flat.* None found, given L3's reading of a spill cell.
- Every cell left through a formula the walk does not model reads it transitively, so it is in every closure. Only
  cells left through a static cycle can be clean. F1 and F8 rest on this.

**Area 2, builds by key.** For criteria sets the rule is sound: a clean cell's builds are in the record (from the
full pass, or added by the kept pass that last ran it), a dirty cell's are this pass's, so the record and the new
builds hold every build of the new full pass. A restricted pass cannot fail a build a full pass makes: its room
starts empty (`evaluation.rs:530-531`). A rejected pass adds nothing. Stale keys after other edits only add.
The figures: 4,000,000 states (`sum_prefix.rs:38`), 32,000,000 bits (`ifs_cache.rs:82`), 32,000,000 / 100,096 =
319, three full-column prefixes. F4 and F5 are the two exceptions.

**Area 3, the lists and the anchors' phase.** `sync_anchor_order` takes every dynamic anchor
(`positions.rs:107-146`), so after the phase every dynamic spill cell's anchor is evaluated, a marked or blocked
one keeps no spill cells, and T-5297's one-cell anchor is among them. A restart, a share-mode re-run and the switch
to Excel's order each restore the spills and run the phase again (`evaluation.rs:465-519`, `positions.rs:60-77`).
An index read that records its range restarts the pass when a spill lands in it (`range_values.rs:12-13`,
`evaluation.rs:941-987`). A read over a cell that is promoted cannot come before that cell's run, since rule 1
asks for the range's cells evaluated, so "from that point" covers it. F6, F3 and L1 are what I found. I read
`evaluation.rs`, `positions.rs` and `formula_cells.rs` whole, and of `model.rs` the two spill writers (976 to
1095) and what P2 shows of a write.

**Area 4, the count.** For the by-column order of F7: a candidate is a frame once (left is sticky, a left root
gets no traversal), a frame spends one probe per list column per rectangle and one per cell taken, and a frame
that meets a left cell or a cycle stops there. So R + E + F holds, left frames and cycles included. E > 16 F for
the W by 20 layout is 19 W (W - 1) > 320 W: not at W = 17, from W = 18. Figures against the logs: 190,000,
3,040,000 and 12,160,000 probes (`/tmp/review-c5/traversal_sim.log`, `/out/probes/d13-probe-bench/run.log`);
290 ms and 18 us a formula at W = 800; 9 to 24 ns and 170 to 360 ns a probe (9.0 to 23.8 and 172.1 to 361.9).

**Area 5, anchors.** Rule 1 (2525), rule 3 (2555 to 2557), "What it guarantees" (2452 to 2454, 2469), 11.1's
"What it does not give", 11.3 (2152 to 2155), 11.4 (2212 to 2215) and Design 12 no longer traverse an anchor or
read through an index for one. A promoted cell is covered apart from L1. F10 lists what is left.

**Cited symbols and figures.** `an_anchor_that_reads_a_formula_cell_declines_restricted_passes`
(`test/test_one_cell_anchors.rs:138`), `now_in_timezone_arg` (`functions/date_and_time.rs:437`), `indexed_formula`
(`read_index.rs:581`), `clean_cell_value` and `anchor_rank` (`read_index/pass.rs:34-98`), `pass_cache_safe`
(`evaluation.rs:401`), `evaluate_root_cell` (`functions/logical/lambda_scope.rs:181`), `read_is_inert`
(`range_values.rs:158`), `resize_to` (`functions/statistical/if_ifs.rs:39`),
`work_that_follows_the_used_area_is_always_in_the_closure` (`test/test_used_area_input.rs:397`), SPEC-0028
"Where it is not" (line 173) and its 2.4 to 2.9 and 2 times (648 to 649). The measured examples against
`/tmp/review-c5/run.log`: 10, 20, 21, 22, 23 at 72 units twice; 54 and 72 units; D5 keeps 41; G flat with
Z1 = FALSE; H nests; K1 for `=LOOKUP(99,A1:B4,H1)`; A3 to A5 turned into spill cells; the name redefined.

## 4. Not checked

- Anything Design 13 does at run time. Every "derived" above is the text and the code read together.
- Excel. No oracle run; nothing here is a claim about Excel.
- F9 on a corpus, and the 2 ms threshold and the 60,000-formula book.
- The native timings of line 2332 (449 to 465 ms, 36 ms).
- T-5166's amendment (CR-321604031), Design 8 and T-5167 as built: none is on main. F1 and F5 rest on their texts
  in this file.
- The Taskei numbers, T-5298's status, and the facade (`crates/rustsheet`).
- Design 12 beyond the sentences that lean on Design 13.
- The crate's test suites; only the probe binary was built.
