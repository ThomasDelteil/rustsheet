# Design 13: fourth check (the fold of the third read)

Text: `/out/drafts/t5145/selfreview/spec-with-change-5-v3.md`, lines 2324 to 2666, and the hunks of
`change-5-v2-to-v3.diff` outside it (Design 4 at 612, Design 8 at 879, Design 9 at 1278, Design 10 at 1684, the
Rollout at 2851). Line numbers are the v3 file's. Engine: `/tmp/wt-design-probe`, unchanged, paths under
`crates/rustsheet-engine/src/`. Design 13 is not implemented: **ran** is today's engine or a count with no engine,
**derived** is the text and the code read together.

Ran, exit 0: `/tmp/review-c6/src/main.rs` with argument `p7`, output `/tmp/review-c6/run-p7.log` (and `p6`,
`run-p6.log`, a variant that shows nothing). I read `/out/probes/d13-probe-bench/two_axis_count.py` and its log.

## 1. Blocking findings

### G1 (Medium): the Excel-order rule does not make the pass a function of the document: a stopped run marks the watch

**Lines 2400 to 2404.** "a pass in Excel's order that the evaluation began with, and in which the order does not
show, is thrown away ... So the order of the pass that counts is the one an engine that just loaded the document
would run: a function of the document".

In a pass in Excel's order a run that stops at a read goes on with an empty value, and the chain puts back the
state it changed but not the watch (`excel_order.rs:505` and `520` save and restore `order_state` alone; the doc
comment at 47 to 49 says a stopped run "leaves its marks"). So the order can show in that pass through a call the default pass
never makes.

Ran (P7): `A1: =20+2`, `A5: =WEEKNUM(45000,23-A1)`, `C1: =WEEKNUM(45000,D1)`.
- Built with D1 = 1: `A5=Ok(Number(11.0)) [restricted_pass_allowed=true spent=66]`.
- Built with D1 = 22, then D1 = 1, the same document, three evaluations:
  `A5=Ok(Number(11.0)) [restricted_pass_allowed=false spent=96]` each time.

A5 runs before A1 in Excel's order, stops, and calls WEEKNUM with type 23, which marks `state_read`
(`functions/date_and_time/weeknum.rs:243`). Under the new rule the order "shows" in the pass this replica begins
with, so the pass is not thrown away: it counts, with no traversal and no index read, at every evaluation from
then on. The loaded replica runs the dependency-first pass. Rule 1's first condition is still history.

Suggested text: "... a pass in Excel's order that the evaluation began with, and in which no run that completed
marked the order watch, is thrown away. A stopped run's marks do not count: the chain puts the watch back with
the state when a run stops (`run_excel_order`). A completed run makes the calls the default pass makes until the
first call that reads or takes the state, so the two passes agree on whether the order shows." That last sentence
needs its own check by the evaluator's owner. The simpler rule is that no evaluation begins in Excel's order, at
the price of one thrown-away pass at every evaluation of a workbook whose order shows. Add the document to the
"Excel's order" row.

### G2 (Medium): where the last full pass took no cell early, the record cannot be "left as it is"

**Lines 2562 to 2566.** "Design 2's and Design 8's arguments then stand as written, with the record left as it is
until the next full pass."

For flatness the branch holds; I could not break it (part 2). Two parts of the record do not carry over. Derived.
- **Turns.** A turn is "its place in the order the pass ran them" (2517). A cell that became a formula has none,
  and after a structural edit a place holds the turn of whatever stood there before, or none. Rules 1 to 3 do not
  say where a dirty cell with no turn runs, or whether a clean one is later. With `A1: =IF(Z1,C9,0)`, Z1 = FALSE
  and C9 empty, write `=YEAR(Y9)` into C9 (restricted, kept), then Z1 = TRUE. The closure is A1 and C9. If the
  cell with no turn runs first, A1 reads C9 already run and the pass is kept; a full pass of that state leaves A1
  at C9 and starts C9 inside it. That is a kept literal edit on a state whose full pass nests.
- **Complete cells.** After Design 12: `A1: =COUNTIF(B1:B200,">0")+IF(Z1,C5,0)`, B1:B200 literals, C5 empty,
  Z1 = FALSE. The full pass takes no cell early, A1 is complete and reads through an index. Write `=YEAR(Y5)` into
  C5: the pass is restricted and kept. A full pass of the new state leaves A1 at C5, so its COUNTIF reads by
  today's path, sets 11.1's flag (rule 3) and declines restricted passes. The engine goes on restricting at every
  edit (A1 now reads an always-dirty cell), with A1 held as complete and reading through an index.

Suggested text: "... Design 2's and Design 8's arguments then stand as written. A turn is then the anchor's rank,
and after the anchors the cell's place, so a cell that became a formula, or that a structural edit moved, has one
and the record keeps none of its own. The complete cells do not carry over: such an edit clears them, and until
the next full pass no read uses an index." Add to the "Lifetime" row: the two documents above.

### G3 (Medium): a frame that resumes instantiates and sorts its whole footprint, so the work is not bounded by the probes

**Lines 2353 to 2355**, "T's rectangles in the walk's order (`instantiate`)", with **2481 to 2482**, "a frame
holds its cell and a cursor, not its rectangles". This wording was my suggestion in F7.

`instantiate` places every reference, then sorts and removes duplicates (`read_index/walk.rs:205-253`). A frame
that holds no rectangles calls it once for each cell it takes. Derived: one formula of 8,192 characters holds about
2,700 references; with one of them a range over 1,000,000 formula cells not yet evaluated, the frame resumes
1,000,000 times, 2.7 x 10^9 placements and as many sorts of 2,700. E is 0, so the cap does not refuse it, the
probes are within the count, and nothing is charged.

Suggested text: "... T's rectangles in the order of its formula's references (the walk's leaves, before
`instantiate` sorts them), each placed when the cursor reaches it, so a frame that resumes places one reference.
A reference written twice is walked twice, and R counts references." Add to the cost row: "one formula of 2,000
references and a range of n formula cells: the placements are linear in n".

### G4 (Medium): the turns are a record written for every formula at every full pass, and Design 4's rulings for such a record are not carried

**Lines 2517 to 2520.** "A full pass records each formula cell's turn ... With the turn it records whether the
cell was complete." This is the option I proposed in F8.

Design 4 measured a per-formula record kept at every full pass at 64 to 82% on the million-formula chain with a
map (lines 525 to 529), and ruled that nothing is tracked with incremental off or on the system clock (530 to
536) and that the record counts toward T-5153's cap (626 to 627). The text says none of the three for the turns.
A restricted pass needs them by cell, so they are a keyed store, not the lists. Cost, not soundness. Derived.

Suggested text, in rule 1: "The turns are kept only while a restricted pass could follow (incremental on and a
fixed clock, as per-cell spend is), in a dense store by formula ordinal that counts toward T-5153's cap; past the
cap none are kept and restricted passes are declined. A full pass that took no cell early keeps none (G2's rule
gives them). The update is measured on T-5165's bench before the code lands, with the lists."

### G5 (Medium): rule 1's test of a range's candidates leaves out the leaf, and the restricted pass's version of it splits the two passes

**Lines 2578 to 2581.** "a formula cell is one when the walk models it. In a restricted pass every formula the
walk does not model is dirty and on 11.1's list, so a full pass and a restricted pass decide alike".

Step 2 makes a leaf a candidate though the walk does not model it (2372 to 2375), and a leaf is on 11.1's list.
Over a column of `=ROW()` or `=RAND()` cells, the lists' kinds say all candidates and the sentence says none; a
restricted pass that decides by the list says none. Derived.

Suggested text: "... a formula cell is one when it is a candidate of step 2: the walk models it, or it is a leaf.
The test reads the cell's kind and its formula, in a full pass and in a restricted one, not 11.1's list."

### Small
- **Design 9, lines 1399 to 1401**, still says "the read-ahead test compares natural order" and that the roots
  run "sorted as `edit_closure` returns them". Both are by turn now.
- **Rule 1 of the restricted pass.** Say that a spill cell has its anchor's turn (11.3 reads a clean anchor's
  spill cell by the anchor's rank).
- **Line 2405.** The thrown-away pass can also send this replica to share mode where the loaded one stays normal
  (`recalc_budget.rs:582-594`). The text's "work of the kind a restart throws away" covers it; say it.

## 2. Checked and could not break

- **The Excel-order rule against `evaluate_passes`** (`evaluation.rs:454-519`). It cannot loop: the thrown-away
  pass is the one the evaluation began with, so a round runs at most that pass, a default pass and one pass in
  Excel's order. `excel_order` is read at six places, all inside a pass or by `can_restrict` after it
  (`evaluation.rs:488-494`, `564`; `read_index/pass.rs:49`; `read_index.rs:744`; `excel_order.rs:376-387`;
  `tailed_array.rs:529`), so a thrown-away pass leaves nothing behind. For the WEEKNUM document of the text the
  rule does what it says (720 and 1,176 units match my P4). G1 is the exception.
- **"Took no cell early" and flatness.** Tried formula writes and structural edits in sequence, each restricted
  under that branch. Every by-value read stays a read of a cell placed before its reader or run in the anchors'
  phase, and under that condition any pass of this design is flat: a root reads what was placed before it, and a
  cell taken early reads only its footprint. A write that puts a total above its data is rejected by the turn
  rule and runs a full pass. With a turn by place for the cells G2 names, I found no kept literal edit on a
  state whose full pass nests; without it, G2's first document is one.
- **The other branch.** Both edits of the bullet, the skip Design 8 makes, and the `=IF(Z1,C7,0)` write each end
  in a full pass. Design 8's new sentence agrees.
- **Turns for literal writes.** Anchors by rank is today's rule; dirty cells run in turn order, so a dirty cell
  read with an earlier turn has run; the three documents of rule 2 come out as the text says, P5 kept. The
  exactness paragraph holds with a left cell's own traversal: the decisions still follow formulas, places and
  kinds alone.
- **The count 2 (R + E) + F.** The axis of a rectangle is chosen from the lists' lines, which do not change in a
  pass, so a resumed frame chooses as before; an evaluated cell is passed in both lists; a cell is pushed by
  another's traversal once and is a root once. The log's figures: one probe a rectangle for the row-above layout
  (1,900, 7,600, 15,200), one a formula for the table with totals, 2n on the diagonal (a tie, by columns), and
  10,500 against 10,600 readers of a 100 by 100 block (E = 1,039,500 and 1,049,400 against 2^20 = 1,048,576).
  Under the cap the probes are linear in R + F beyond a constant 2^21. G3 is about work that is not probes.
- **F3, F4, F5, F6, F10, and L1 to L12 as folded.** The candidacy test is asked of cells in both passes (G5
  apart); the record takes a prefix's growth and Design 4's sentence names the builds; the kept lists' drop set
  matches P2; the Rollout, Design 9's cone and Design 10's lists say what Design 13 now does.

Not checked: Excel; the corpus count for the cap; the 2 ms and 60,000-formula measurements; T-5166's own text.
