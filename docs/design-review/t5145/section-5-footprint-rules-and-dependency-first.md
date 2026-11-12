# T-5145 section 5: footprint rules (T-5143), with the dependency-first pass folded in

*poc-design-cc, 2026-11-11. Draft for a design read, before the reviewers. The text under "Proposed spec text" goes
into SPEC-0002 as Designs 12 and 13. Design 13 is poc-s-e2-1-cc's reviewed draft, folded in at their request.*

## In short

- **Text refreshed on 2026-11-12**, after two more independent reads and a check of the last fold (31 blocking
  points over all reads, each folded in). What changed for a user of the product:
  - **A dynamic array that reads formula cells is left as it is today** (a FILTER or SORT over a computed column
    still recalculates the workbook fully at every edit). Both of the draft's reviewers ruled so; it is the
    next amendment.
  - **A workbook whose totals sit above their data** gets quick recalculation for value edits. After typing a
    formula or inserting a row in such a workbook, the next recalculation is a full one.
  - **No workbook gets slower for it.** A layout whose formulas each read a wide and tall block of formulas
    keeps today's pass (it is refused by a count made from the formulas, not charged).
  - **Every collaborator computes the same way**: whether a lookup goes through one of the new per-pass indexes
    is decided by the document alone, as the work budget's owner requires.
- **Design 12, footprint rules.** SUMIF and AVERAGEIF read a resized sum range; LOOKUP reads on past a short result
  vector. Each joins the index with a rule that names those cells, and two preconditions close the holes the
  first attempt's review found.
- **Design 13, the dependency-first full pass and document-decided indexes.** Before a full pass evaluates a
  formula, it evaluates the formulas of its static footprint, from its own loop. A total above its data then no
  longer nests (measured: 449 to 465 ms against 36 ms for 100 edits on 10,000 rows). The running-total and
  criteria caches become indexes whose use the document alone decides.
- **The order of work that follows:** T-5132 (always-dirty list), then Design 13, then T-5143, T-5192 and the
  second slices of sections 2 and 4.

## Open questions, each with the recommended answer
An answer that differs from the recommendation becomes a round of this CR.
1. T-5143 is implemented after Design 13, and until then SUMIF, LOOKUP and the conditional aggregates are always
   dirty, at today's cost. Recommended: yes; the withdrawn change showed the other order is unsound.
2. Design 13 lands here although it also amends SPEC-0028 (one sentence in section 11, one in section 3).
   Recommended: yes; both owning lanes reviewed the draft, and section 11's owner ruled on this text.
3. Design 13 changes the order of a full pass where a footprint reaches ahead. Values in normal mode do not
   change; which cell draws which RAND value does, the depth-limit figures of some tests change, and a document
   that reaches share mode today only through the work a deep chain throws away computes normally. Recommended:
   accept.
4. D-functions, and criteria that may be arrays, stay always dirty with no rule. Recommended: yes.
5. Anchors keep today's evaluation in this slice, so a workbook whose dynamic array reads formula cells (a
   FILTER or SORT over a computed column) still recalculates fully at every edit. Recommended: accept for now, and
   count such workbooks on the corpus before the follow-up amendment (Taskei P559254376).
6. A workbook whose wide count passes the cap keeps today's pass. The draft's two reviewers ruled differently:
   one for this refusal, or a per-rectangle structure; the other for charging the probes to the pass's work.
   Recommended: the refusal. A charge changes what every document spends and can send a layout that computes
   today to share mode. The cap's two figures (16 lines a formula cell, and 2^20) are this change's and are
   refitted on the 60,000-formula book.
7. After a full pass that took a cell early, typing a formula or inserting a row recalculates fully, where main
   (T-5166) restricts a formula write today. A workbook whose totals sit above their data therefore gains
   restricted passes for value edits and loses them for formula edits. Recommended: accept for this slice; the
   rule that keeps them needs the turns to move with the edit, and is named as later work.
8. A SUMIFS per row with a key per row leaves the criteria sets after 319 keys and is answered by the range's
   value index, which this design does not make an index. Recommended: accept, with the value index as the next
   amendment; its author is asked whether it was meant to join.

9. No evaluation begins in Excel's order. A workbook whose order of calls shows (SPEC-0017) then throws one pass
   away at every recalculation, where today only its first evaluation does. Recommended: accept. Such a workbook
   has no restricted pass, and the rule is what makes index use a function of the document, which SPEC-0028's
   owner requires. The evaluator's owner is asked to confirm it.

## Probes this section waits for

None. The footprints are already evidenced (criteria.md, lookup.md), and the pass order is not observable in
Excel outside what SPEC-0017 already covers.

## Proposed spec text

**12. Footprint rules** (amendment for T-5143, 2026-11-11). The rules are those of T-5143's first change
(CR-321578012, poc-s-e2-1-cc), which was withdrawn when its review found two holes. This design keeps the rules and
adds the two preconditions that close the holes. T-5143 is implemented after Design 13.

- **What it is for.** SUMIF, AVERAGEIF, LOOKUP and the conditional aggregates are among the most used functions.
  Until they are indexed, a workbook that holds one runs them at every edit (11.1), and gets no restricted pass
  while their charges follow the order of the pass.
- **Excel.** A function that reads beyond its arguments is the reason Excel calls SUMIF volatile "depending on its
  arguments" (research note, S1). The footprints are evidenced in docs/excel-semantics: "SUMIF resizes sum_range
  from its top-left cell and never errors" (criteria.md), and "LOOKUP past a short result vector" (lookup.md). Microsoft
  documents the first: "the formula will sum a range of cells that starts with the first cell in sum_range but has
  the same dimensions as range" (research note, addendum).
- **The rules.** A function that reads cells beyond its references joins the allowlist with a rule that names
  those cells, filed as reads of the formula.
  - **SUMIF and AVERAGEIF** with a sum range read the range of the criteria range's shape that starts at the sum
    range's top-left cell, whatever the sum range's own shape, clipped to the grid (`resize_to`). The rule files
    that rectangle on the sum range's sheet **beside** the written sum range. T-5143's first change filed it in
    place of the written range, which the functions do not read as written. Keeping both costs a needless re-run
    when a cell of the written range outside the rectangle is written, and it keeps Design 10's premise that the
    index files every written area of a formula it models: with `=SUMIF(A1:A3,">0",B1:B10)` the index files B1:B3
    and B1:B10, so `dependents` of B7 finds the formula.
  - **LOOKUP**'s vector form reads a short result range on past its end, down for a column and to the right for a
    row or one cell, as far as the lookup vector is long. A two-dimensional lookup vector is searched along its
    first row when it is wider than tall and along its first column otherwise (`lookup.rs`), so its length is
    the larger of its rows and its columns. The rule files the result range extended to that many cells. A
    two-dimensional result vector is `#N/A` and reads nothing more. LOOKUP's array form reads within its
    array, so it needs no rule, only the first precondition below.
  - A rule is a function of the formula's text and place. So both the argument that gives the shape (SUMIF's
    and AVERAGEIF's range, LOOKUP's lookup vector) and the one resized or read on must be written references or
    ranges, bare: a name, even one 11.2 instantiates, a parenthesized range and an `@` give no rule in this
    slice. With `=SUMIF(CHOOSE(D1,A1:A3,A1:A9),">0",B1)` the rectangle is B1:B3 or B1:B9 by D1's value, and
    `=LOOKUP(2,1/(A1:A100<>""),B1)` has a computed lookup vector: such a formula has no rule and stays always
    dirty. So does one whose rectangle, after the clip to the grid, reaches the last row or the last column,
    as a written range that does (`instantiate`).
  - **COUNTIF, COUNTIFS, SUMIFS and AVERAGEIFS** read exactly their ranges and criteria, and need no footprint
    rule; the strict shape of the *IFS functions makes a mismatched range `#VALUE!`. MINIFS and MAXIFS join with
    a test of their own, as any function does (Design 1).
  - **A criterion whose static result may be an array** (the static analysis 11.1 uses for a formula's result:
    `COUNTIF(A1:A9,A1:A3)`, `">"&B1:B2`, `CHOOSE(D1,B1,B1:B3)`) can lift the call, which then charges by the
    positions its walk visits, and those follow the used area: its formula stays always dirty. Whether a call
    lifts is a fact of its values (`if_ifs.rs`), so the rule asks what the text allows.
  - **D-functions** evaluate a criterion formula at each record's row. They stay always dirty.
- **The two preconditions** (review of CR-321578012, round 1; Taskei P559979639).
  1. **A search is a head.** LOOKUP searches through the search cache, as VLOOKUP and MATCH do, so every range of
     its lookup vector is a head over its whole span (Design 1, "A search's cut"), and so is its extended result
     range. Without it, a write no formula reads that moves the last used row changes LOOKUP's work, and the edit
     is skipped.
  2. **The charge is a function of what the index files.** The conditional aggregates' work follows state the
     index does not file: the block of the range inside the used area, the stored-cell counts of its columns, and
     the formula's budget of rectangles read empty. A write the formula does not read changes their charge, so
     even the skip path is unsound for them. They join only when each of these holds:
     - the criteria indexes are Design 13's indexes, built once per pass from the document, so a read through
       one charges a constant;
     - a read that is not through an index charges per stored cell of its ranges (SPEC-0028 section 11), and
       those cells are inside the rectangles the formula files;
     - whatever of the charge still follows the used area is registered as a head, and T-5184's hook test
       records it: the hook is extended to the criteria walks, and the test fails for a walk the index does not
       register.
- **Why it is then exact.** With both preconditions an indexed call's value and charge are functions of the cells
  in its filed rectangles and of its heads, which is what Design 4 asks of a clean cell. Design 13 removes the
  order from the charge, so a full pass that ran one need not decline restricted passes.
- **Structural edits.** A rule's rectangle is derived from two written ranges, so the rewrite does not predict it.
  A formula with a footprint rule is in D0 when either written range is changed by Design 8's classification, or
  when the rectangle the rule gives after the edit is not the old one moved by `RefMap::line` with as many lines.
  So a shifted index (Design 8, "Re-keying") files such a formula again with the rest of D0, and keeps by the
  shift only a rule's rectangle that the edit moved whole, as it requires of every entry it keeps.
- **Maintenance.** A refile applies the rule as the build does.
- **`precedents` and `dependents`** (Design 10) answer the written relation: the rule's rectangle is not a
  precedent. It is the formula's rule area: a formula that only its rule area ties to a cell is listed in that
  cell's `unresolved`, with the index and by the scan alike.
- **What the implementation may rely on.**
  1. The footprints above are the implementations' (`resize_to`; LOOKUP's read-on), each with a test against it.
  2. Design 11.1's flag keeps these functions out of restricted passes until this lands, so main is safe at every
     step.
- **What it must show.**
  - The flat generator's third quarter (Test plan): allowlisting SUMIF, AVERAGEIF or LOOKUP alone without its
    rule fails it, at the seeds it already names.
  - The two reproducers of CR-321578012's review as tests: LOOKUP after an unread write that moves the last used
    row, at the work limit; a conditional aggregate after an unread write that changes its range's used block,
    its columns' stored cells, and its empty-rectangle budget.
  - The used-area oracle (`work_that_follows_the_used_area_is_always_in_the_closure`) over the conditional
    aggregates' shapes and LOOKUP's, at the last used rows and columns it sweeps.
  - The rule is the text's: the CHOOSE and computed-vector formulas above stay always dirty, and so does a
    criterion that CHOOSE can make an array; a 2-by-4 and a 4-by-2 lookup vector file a result range extended
    to 4 cells (`=LOOKUP(99,A1:B4,H1)` reads as far as K1, measured).
  - Mutations: a rule's rectangle not filed; LOOKUP's head left out; a criteria walk the hook does not record;
    a rule applied to a shape-giving argument that is computed.

**13. The dependency-first full pass, and per-pass indexes the document decides** (amendment to Design 4 and to
SPEC-0028 section 11, 2026-11-11). The design is poc-s-e2-1-cc's draft of 2026-11-10, folded in at their
request. It was reviewed before any code by poc-s-s1-cc for SPEC-0028 (5 changes) and by poc-engine-cc for the
evaluator (6 changes), all applied. It answers Taskei P559418140, which decision 0422 (T-5154) names as the
owner of declined passes, and Taskei P559664607. Taskei P560151028 (a refused build is tried again only when
what refused it changes) is beside it and not part of it. Before its change went to review it was read
independently three times (2026-11-11 and 12): six blocking points, each ruled by poc-engine-cc; ten more, with
two rulings by poc-engine-cc and poc-s-e2-1-cc; and ten more on the text those gave, with poc-s-s1-cc's reading
of SPEC-0028 section 11. A check of that last fold found five more. Anchors keep today's evaluation in this
slice; a layout whose traversal is not linear in its formulas is refused, not paid for; index use is a function
of the document alone; and a formula write or a structural edit ends the record a full pass leaves. All are
folded in below.

- **The two problems.**
  1. *Totals above their data.* With `C1 = SUM(B1:B10000)` over `B r = A r*2`, the pass loop reaches C1 in row
     1, and C1's SUM starts B2 to B10000 inside its own evaluation. The full pass nests, so every literal edit
     is declined. Measured natively, 10,000 rows and 100 edits: 449 to 465 ms with the total in C1, the same as
     with incremental off, against 36 ms with the total in C10001. Design 4 excludes nesting, it does not bound
     it, so the full pass must nest less.
  2. *SPEC-0028 section 11's indexes.* A per-pass structure is an index when it is built linearly once per pass
     and whether a read uses it is a function of the document alone. The running totals' prefix cache
     (`sum_prefix.rs`) and the criteria cache (`ifs_cache.rs`) fail the second condition: their use follows
     `read_is_inert`, which follows the order of the pass, and capacities other formulas can use up
     (`MAX_KEPT_STATES`, `MAX_CACHED_CELLS`). Charged as misses, a running total filled down 100,000 rows is 2.4
     to 2.9 times the pass allowance, and a SUMIFS per row is 2 times it.

  Both have one cause: the order of the pass, not the document, decides when a range's formulas are evaluated.
- **The pass.** In a full pass that is not in Excel's order (SPEC-0017), with iterative calculation off, before
  the loop evaluates a root R that is a candidate (step 2), it evaluates the formula cells of R's static
  footprint that are not evaluated yet, depth first, from its own loop, with an explicit stack:
  1. Push R. While the stack holds a cell T, take the next formula cell of T's footprint that is not evaluated,
     in the frame's order: the references of T's formula in the order the formula holds them (the walk's leaves,
     before `instantiate` sorts them), each placed when the cursor reaches it, and each rectangle by its lines
     (below). The cursor is that place in the frame's order and only moves forward: a cell evaluated while T
     waited is passed, not taken again, and a frame that resumes places one reference, not its whole footprint.
     A reference written twice is walked twice. This is not placement order, and nothing rests on which of a
     footprint's cells is taken first. If the cell is on the stack, that is a static cycle (step 3). If it is a
     candidate that no traversal of this pass has left, push it. Otherwise it is a formula the traversal cannot
     take, a non-candidate or a candidate left before, and T is **left**. When T's footprint has no unevaluated
     formula cell, pop T and evaluate it at depth 0 (`evaluate_root_cell`). A cell that is left is popped
     unevaluated and keeps today's place: the cell that waited for it is left in turn, and no other cell's
     traversal of this pass takes it. When the loop reaches it, it is a root like any other and its own
     traversal runs, so a cell is a frame twice at most. When the root R is left, it is evaluated at once, as
     today, its footprint read on demand. A cell is **complete** when the pass took it early, or when it is a
     root that its own traversal did not leave: it ran from the pass's own loop with every formula cell of its
     footprint evaluated. A spill cell is evaluated when its anchor is.
  2. A formula cell is a **candidate** when the read index's walk gives its footprint (`read_index/walk.rs`, the
     one definition) and it is no array anchor and no spill cell. A dynamic anchor runs in the anchors' phase,
     T-5297's one-cell scalar anchor included, and `indexed_formula` refuses a Ctrl+Shift+Enter one. Candidacy
     is a property of the cell, not of the path that reached it. A formula the walk does not model (Design
     11.1's list) is no candidate. If it is read, it nests on demand, as today, and as a root it has no
     traversal and is not complete. One kind of unmodelled formula is a candidate all the same: a **leaf**, a
     formula that is no dynamic anchor, whose tree holds no reference, range, name, structured reference or
     spill reference, and whose every function is TODAY, NOW, RAND, RANDBETWEEN, ROW, COLUMN or on the
     allowlist. It reads no cell, so running it first cannot read ahead or nest (ruled by the draft's authors
     when it was folded in). The test is on the tree, not on the functions alone: TODAY and NOW read a time-zone
     argument here (`now_in_timezone_arg`) and RANDBETWEEN its bounds, so `=TODAY(C5)` and `=RANDBETWEEN(C6,C6)`
     are no leaves (measured: each nests into the formula cell it names), and neither is `=TODAY()+A1`.
     `=SUM(INDIRECT("A1"))` is no leaf either: its tree holds no reference, but INDIRECT is on neither list. An
     always-dirty formula whose footprint is unknown or dynamic (INDIRECT, OFFSET, a whole column) stays a
     non-candidate.
  3. A cell already on the stack is a static cycle. On finding one, every cell on the stack is left: the
     traversal from R stops, the cells it already evaluated stay evaluated, and R and the rest of its footprint
     are evaluated as today, on demand in row-major order, with the cycle left to today's detection. A cycle's
     values can depend on where it is entered, and this pass must not move that. A cycle through a non-candidate
     is not a static cycle, and the traversal does not enter it. With `A1: =C5+B5`, `B5: =C5+1`, `C5: =N(B5)` (N
     is off the allowlist), A1 meets C5, which is no candidate, and is left; it is evaluated as today, and the
     cycle is entered at C5, as today.

  Candidacy is the walk's, computed in every full pass whether or not the workbook has an index. With no index
  (incremental recalculation off, a build the cap refused) the pass is still dependency-first, so draws and
  thrown-away work do not depend on an optimization's availability. With iterative calculation on, the pass is
  today's, in natural order.

  *No evaluation begins in Excel's order.* Today an evaluation's first pass is in Excel's order when the
  evaluation before it ended so (`Evaluation::excel_order`), whatever the document holds now, and that pass
  counts whether or not the order shows in it (`OrderWatch::order_shows`). So the pass that counts follows the
  history. With `A1: =WEEKNUM(45000,1)`, `A2: =WEEKNUM(45000,B2)` and A3 to A40 `=A(r-1)+1`, the document built
  with B2 = 1 spends 720 units; the same document reached through B2 = 22 spends 1,176 at its first evaluation
  and 720 at the next. And it can follow it for good, since a run that stops in a pass in Excel's order leaves
  its marks on the order watch: with `A1: =20+2`, `A5: =WEEKNUM(45000,23-A1)` and `C1: =WEEKNUM(45000,D1)`, the
  document built with D1 = 1 spends 66 units and restricts; reached through D1 = 22 it spends 96 at every
  evaluation and never restricts (both measured). With this design every evaluation begins with a
  dependency-first pass, and goes to Excel's order only when the order shows in that pass, as an engine that
  just loaded the document does. The order of the pass that counts is then a function of the document, and with
  it the traversal, the turns and the index reads below. The price is one thrown-away pass at every evaluation
  of a workbook whose order shows, where today its first evaluation pays it. Such a workbook has no restricted
  pass (`can_restrict`), and the thrown-away pass counts toward its allowance at every evaluation as it does
  today at the first.

  *Anchors keep today's evaluation* (ruled by poc-engine-cc and poc-s-e2-1-cc, 2026-11-11). An anchor runs in
  the anchors' phase, in `anchor_order`, its footprint read on demand, and no cell is run by a traversal on an
  anchor's behalf. A traversal there would run plain cells before the anchors ranked after theirs have laid
  their spills. With Z1 = FALSE, E1 = 10, E2 = 20, `A1: =IF(Z1,C3,0)+E1:E2`, `C3: =G2+1` and
  `G1: =A2+SEQUENCE(2)`, today's engine gives A1:A2 = 10, 20, G1:G2 = 21, 22 and C3 = 23 with no restart (measured:
  72 units at the first evaluation and at the second). A traversal before A1 would run C3, which would read G2
  before G1 laid it, and the restart that follows has no reader the restart log can place: C3 is in no anchor
  order, and A1's own formula never read C3. Learned for C3, the fact moves nothing and the same restart
  repeats; learned for A1, it is false, and with the true one (G1 reads A2) the log finds a loop and marks both
  anchors circular. So an anchor that reads a formula cell still evaluates it inside itself, and such a
  workbook's restricted passes stay declined, as on main
  (`an_anchor_that_reads_a_formula_cell_declines_restricted_passes`). Running an anchor's precedents first is a
  later amendment with its own proof and its own rule for the restart log (Taskei P559254376), backed by a
  corpus count of anchors that read formulas. The second read's text is where it starts: a traversal's reads are
  recorded on the anchor's behalf, and a restart asked by a cell the traversal ran teaches the log nothing and
  puts that anchor back on today's evaluation.

  Finding a rectangle's formula cells uses two per-sheet lists of the formula cells and the spill cells, with
  their kinds: by column, each column sorted by row, and by row, each row sorted by column. They are built once
  per pass after the anchors' phase, when every anchor in `anchor_order` has laid its spill: O(F log F) for F
  such cells. Each line of a list, a column or a row, also keeps the next cell not yet evaluated at or after a
  place (a union-find with path compression, or a successor set), so an evaluated cell is passed once, not once
  per probe. A rectangle is walked by its list columns, from left to right and each from top to bottom, or by
  its list rows, from top to bottom and each from left to right, whichever are fewer (the columns on a tie). A
  **probe** is one such lookup in one line. A frame makes one probe for each line of each of its rectangles and
  one more for each cell it takes, and a cell is a frame twice at most. So the traversal makes at most 2 (R + E)
  + F probes: R is the candidates' references, each as often as it is written, and E, the **wide count**, is the
  lines their rectangles are walked by beyond the first of each (four binary searches a rectangle, counted when
  the lists are built). R is the walk's own size. E is 0 where every range is one row or one column, whatever
  its length. W columns by 20 rows of formulas that each read the whole row above make one probe a rectangle,
  15,200 at W = 800, and a table of literals with a total under each column and beside each row makes one probe
  a formula (counted on the structure). With lists by column alone the first layout makes 12,160,000 probes at W
  = 800, four times as many for twice the formulas, where today's pass makes none (290 ms, 18 us a formula,
  measured on the structure), and the second, a common one, makes a probe for every column of every row's total.
  E still grows where many formulas each read a block of formulas that is both wide and tall: 10,600 formulas
  that each read a 100 by 100 block of formulas have E = 1,049,400. And a footprint is a superset: an unread
  branch costs its probes all the same. So:
  - *The design's bound is per rectangle.* A per-sheet structure that answers "the next formula cell not
    evaluated in this rectangle" in O(log² F), whatever lines the rectangle meets (a tree over the list's
    columns whose nodes keep their cells by row, each with the same successor), makes the traversal O(F log F +
    (F + R) log² F). It is a follow-up, built when the corpus shows the refusal below.
  - *Until then the pass refuses, it does not pay.* When E is more than 16 for each formula cell and more than
    2^20 (E > 16 F and E > 1,048,576), no root is traversed: the pass is today's, in natural order, no cell is
    complete and no read uses an index. E and F are functions of the formulas and of which rows and columns hold
    a formula or a spill cell, so every replica refuses alike, with an index or without. Under the cap the
    traversal makes at most 2 (R + max(16 F, 2^20)) + F probes. Measured on the lists by column, a probe costs 9
    to 24 ns among a line's neighbours and 170 to 360 ns at random places of a million-cell list. Both figures
    of the cap are this change's. They are refitted when the lists are measured on the 60,000-formula book, with
    what the count of E costs a workbook that has no index, and the corpus is counted for the workbooks the cap
    refuses. A refused workbook's conditional aggregates read by today's path and keep 11.1's flag (rule 3 of
    the indexes).
  - *No unit is charged for it.* Under the cap the probes are bounded by the formulas and their rectangles, as
    the walk that files them is, so they are `T_pass` in SPEC-0028 section 11's terms (the lists are `T_setup`
    if they are kept between passes). A charge would bound a refused layout's cost at the allowance instead, but
    it would change what every document spends, and send to share mode a layout that computes today.

  The lists are measured on the 60,000-formula book before the code lands; above 2 ms to build, they are kept
  between passes. Kept lists hold the formula cells and the spill cells with their kinds, so they live only
  while those are the same cells. A write to a cell that was or became a formula, an array formula or a spill
  cell drops them, a literal typed over one included: typed over a spill cell of `=SEQUENCE(5)`, a literal
  removes the other spill cells at the write, before any pass (measured). So do a structural edit, a name op and
  a sheet op, and a pass, or an exhausted recalculation, that leaves another set of anchors or spill cells than
  the lists hold. The test is on the set, not on the writes, since every pass writes each spill cell again. A
  literal write is enough to move a spill: with `A1: =SEQUENCE(Z1)`, Z1 = 2 and then 5 turns A3 to A5 into spill
  cells (measured). When the anchors' phase is what changed the set, the pass builds the lists again after that
  phase, before any traversal. A cell can also become an anchor when the loop runs it: a name over one cell,
  redefined over three, turns the formula cell that reads it into an anchor with two spill cells, in the pass
  the name op starts (measured). From that point such a pass reads through no index and is recorded as having
  made a read that is not through one (rule 3 of the indexes below), and the next pass builds the lists again.
  Such a cell can also cost one restart that today's pass does not make, when a cell taken early read where it
  then spills; the values are the same, and the cell joins `anchor_order` at that restart. An index read records
  its range as read, as today's caches do (`range_values.rs`), so a spill laid in it later restarts the pass. A
  pass with the lists kept and one with them rebuilt are the same pass: the test compares them. The traversal is
  bounded by the formulas, their footprints and the stack's F cells (directive 5); a frame holds its cell and a
  cursor, not its rectangles, since a filled column shares one parsed formula. Design 9's cones and Design 10's
  `precedents` use the same lists. Their walk is not this traversal: it enters every formula cell a footprint
  holds, an anchor, a spill cell's anchor and a left cell included, the wide count does not refuse it, and its
  bound is its caller's (Design 10's budget; past it a cone is everything owed).
- **What it guarantees.**
  - Every cell the traversal takes early is evaluated at depth 0 with every formula cell of its footprint
    evaluated. For a document with no anchor, whose formulas are all candidates with acyclic footprints and
    whose wide count is under the cap, the full pass is flat: a total above its data no longer nests, nor does a
    leaf read by a formula placed before it. A formula that is no candidate still nests when it is read before
    the loop reaches it (Design 11.1, "What it does not give"), and so does a formula cell an anchor reads
    (above).
  - A full pass that is flat today stays flat. A cell taken early reads only evaluated cells. A root at its
    place reads what today's pass read there, and every formula cell placed before it is evaluated by then, as
    today. A footprint is a superset of the reads, so an IF's unread branch names formulas its formula never
    reads, and the traversal takes such a formula early only when every formula it could read is evaluated
    first. With Z1 = FALSE, `A1: =IF(Z1,C3,0)`, `B2: =YEAR(Y2)` and `C3: =B2+1`, today's pass is flat
    (measured). YEAR is off the allowlist, so B2 is no candidate: C3 is left, then A1, and the order is today's.
    Taken early, C3 would start B2 inside itself, and a workbook that restricts at every edit after 11.1 would
    have every restricted pass declined.
  - Values in normal mode do not change: a formula's value is a function of its inputs.
  - The order changes only where a footprint reaches a candidate that is not evaluated and is not left. RAND
    draws from one sequence, so a change of order changes which cell draws what. No history pins a draw, and
    under the collaborative profile a draw is a function of the seed and the cell (I-07).
  - A static cycle no value closes keeps today's values and today's entry: the read that would close it is made
    on demand, by a cell that was left, as today.
  - Anchors still run first, in `anchor_order`, as today. The order of a pass in Excel's order is untouched.
  - The depth-limit unwinds of a chain read from the top go away: its thrown-away work becomes 0. The tests that
    pin today's figures change to the new ones and keep their checks. A document that reaches share mode today
    through that thrown-away work alone stays in normal mode, so its formulas compute where share mode gave
    `#VALUE!`; SPEC-0028 section 3 ("Where it is not") says so.
- **The restricted pass follows the same order** (added when the draft was folded in: a flat full pass is not
  enough, since Design 4's restricted pass runs its roots in (sheet, row, column) order and calls a clean cell
  placed after its root "ahead"; confirmed by the draft's author). With `C1 = SUM(B1:B10000)` and an edit of A5,
  the closure is B5 and C1. In natural order C1 runs first, starts B5 inside itself and reads B2 onward ahead,
  so the pass would be rejected at every edit, whatever the full pass did. So:
  1. A full pass records each formula cell's **turn**, its place in the order the pass ran them, as it records
     the anchors' ranks today (`Incremental::anchor_rank`): the anchors' turns are their ranks, and the other
     formula cells follow in the order the loop and its traversals ran them. With the turn it records whether
     the cell was complete. A spill cell has its anchor's turn. A restricted pass runs its dirty cells in the
     order of their turns, each from the pass's own loop. It needs no list and no traversal: the turns hold what
     the traversal decided. The turns are kept only while a restricted pass could follow, under the conditions
     Design 4 gives for per-cell spend ("When it is kept": incremental recalculation on, and a fixed clock), in
     a dense store by formula ordinal that counts toward T-5153's cap; past the cap none are kept and restricted
     passes are declined. A full pass that took no cell early keeps none: a turn is then the anchor's rank and,
     after the anchors, the cell's place, as today. The update is measured on T-5165's bench, with the lists,
     before the code lands.
  2. Design 4's read-ahead rule is restated for this order. A dirty cell reads **ahead** when it reads a clean
     formula cell whose turn is later than its own: a full pass would evaluate that cell after it. That is
     today's rule with the turns in place of the order it names (the anchors by rank, then every position in
     natural order), and it is one rule for every dirty cell: a complete cell, whose footprint's formula cells
     all have earlier turns; a root that was left; a formula the index does not model; an anchor. Three
     documents show what it tells apart. With Z1 = Y1 = FALSE, `A1: =IF(Z1,B5,0)`, `B5: =IF(Y1,C5,1)` and
     `C5: =IF(Y1,2,B5)`, B5 and C5 are a static cycle no value closes, so A1 is left and runs before B5. After Z1 =
     TRUE its read of B5 is ahead: a full pass starts B5 inside A1, and at depth limit 1 it spends 72 units
     where it spends 54 with no limit, the run of A1 that unwinds, which a restricted pass does not make
     (measured on today's engine, whose pass is the one a left root keeps). With `A1: =IF(Z1=1,C1:C2,E1:E2)` and
     `C1: =E1:E2*2`, both anchors and A1 ranked first, Z1 = 1 makes A1 read an anchor ranked after it: ahead, as
     today (measured). With Z1 = FALSE, `A1: =B5+IF(Z1,B9,0)`, `B5: =E5*2` and `B9: =YEAR(Y9)`, A1's traversal
     takes B5 early and then meets B9, which is no candidate, so A1 is left and runs at once: B5's turn is
     before A1's and the full pass is flat. B9 is always dirty, so A1 is in every closure, and its read of B5 is
     not ahead. A rule by place would call it ahead and reject every restricted pass of a workbook whose full
     pass is flat, where today's engine nests, so that every edit is declined (measured).
  3. `Incremental::last_full_flat` keeps its meaning: a full pass in which a cell started another's evaluation
     declines restricted passes, and a restricted pass in which a dirty cell starts a dirty cell with a later
     turn is rejected, as nesting is today.

  *Why it is then exact.* Take a kept restricted pass of literal writes, and the full pass of the new state. The
  formulas, their places and the cells' kinds are those of the last full pass: a write that changes one is under
  "Lifetime" below, and a spill that moves rejects the pass (11.3). By induction along its order, that full pass
  gives every cell the turn it had. A clean cell has the inputs it had, so it reads what it read then, and those
  cells had earlier turns. A dirty cell's reads were checked in the restricted pass: each was of a cell with an
  earlier turn, clean or already run. So no cell is started on demand, each traversal finds the same cells
  evaluated and takes and leaves the same ones, and the turns and the complete cells stand. That is what Design
  4 asks of a kept pass, and why one full pass's record serves every restricted pass of literal writes until the
  next.
- **Lifetime of the record.** The turns, the complete cells and the flatness they stand for are functions of the
  formulas, their places and the cells' kinds: under this pass a cell's turn follows other cells' footprints,
  not its place alone. A literal write changes none of them. A formula write and a structural edit can. Take
  `A1: =B9*1`, a chain `B9: =B10+1` filled down to B2008, and `A2: =IF(Z21,B3,0)+B9+K2` over a static cycle no
  value closes (`B3: =IF(Y21,C3,1)`, `C3: =IF(Y21,2,B3)`). A1 takes the chain early, A2 is left and reads B9
  evaluated, and the full pass is flat. Delete row 1, or write `=1` into A1: no formula that remains reads
  anything that changed, yet the full pass of the new state starts the chain inside the old A2 (today's engine
  on the document the delete leaves is not flat, and at depth limit 1 it spends 288 units more: measured). So
  when the last full pass took a cell early, a write to a cell that was or became a formula, an array formula or
  a spill cell, and a structural edit, each drop the record and clear `last_full_flat`. The recalculation that
  follows is a full pass, also where Design 2's maintenance (T-5166) or Design 8 would run a restricted one;
  after an edit that Design 8 skips, the next recalculation is. When the last full pass took no cell early, its
  order was natural order: every read it made was of a cell placed before its reader or run in the anchors'
  phase, the checks of a restricted pass keep that true, and a cell that a later pass takes early reads only
  what its own traversal evaluates first. Design 2's and Design 8's arguments then stand as written. A turn is
  then the anchor's rank and, after the anchors, the cell's place (rule 1 above), so a cell that became a
  formula, or that a structural edit moved, has one. The complete cells do not carry over: such an edit clears
  them, and until the next full pass no read uses an index. With `A1: =COUNTIF(B1:B200,">0")+IF(Z1,C5,0)` over
  literals, A1 is complete and reads through an index; write `=YEAR(Y5)` into C5, and a full pass of the new
  state leaves A1 at C5, so A1 must not go on reading through one. Keeping such an edit off a full pass after a
  cell was taken early needs a rule for how the turns move with it, which is not in this slice.
- **The indexes** (SPEC-0028 section 11 rule 1).
  1. *Use decided by the document.* A read uses an index only when all four hold: the pass is not in Excel's
     order; the read is made by the own formula of a cell that is complete (in a restricted pass, one the record
     holds as complete), that cell alone on the stack; the range lies in a rectangle of that cell's footprint
     and its formula cells are all candidates; and no cell of the range is circular or a cycle's member
     (`read_is_inert` gives `None`), since reading such a cell marks the reader. All four are functions of the
     document (ruled by poc-s-s1-cc for SPEC-0028 section 11, which admits no new exception). No evaluation
     begins in Excel's order (above), so whether the pass that counts is in it follows the document. An anchor
     makes no index read and no build, and the loop runs after every anchor has, so `anchor_order` is no input:
     which plain cells the anchors evaluated on demand follows the values of the pass that counts, not the order
     that reached them. Whether a range's formula cells are all candidates is asked of the cells when the index
     over it is built, which walks them: an array anchor or a spill cell is none, and a formula cell is one when
     it is a candidate of step 2: the walk models it, or it is a leaf. The test reads the cell's kind and its
     formula, not 11.1's list, so a full pass and a restricted pass decide alike, over a range that holds a
     clean anchor's spill and over a column of `=ROW()` cells too. The range's formula cells were then evaluated
     before the reader ran. Two documents show why the second and the fourth condition are needed (measured on
     today's engine). With `A1: =Z1+1` and `Z1: =COUNTIF(Q1:Q100,">0")+YEAR(Y1)` over `Q r = P r*2`, Z1 is no
     candidate (YEAR), so it runs inside A1 and reads Q1:Q100 before any Q cell has run. With `A1: =B1`,
     `B1: =A1`, C1 to C3 `=A1+1`, `=A1+2`, `=A1+3` and `D5: =SUM(C1:C3)+E5`, C1 to C3 are candidates that read a
     cycle; today D5 keeps 41 when E5 becomes 7, and a read through an index, which marks nothing, would give
     43. An anchor has no traversal, so its reads use no index. Neither does a read made while a run unwinds or
     its pass is abandoned (`pass_cache_safe`), and an index built in such a run is not kept. This rule only
     adds the index path. A range that holds a non-candidate (a spill cell, or a formula off the allowlist such
     as a `=MONTH(A2)` helper column), a dynamic reference or a circular cell is read and cached as today, once
     its cells are evaluated (`read_is_inert`), and its reads are not charged a walk each. Until Design 12 makes
     the conditional aggregates candidates, a root that calls one has no footprint, so none of its reads uses an
     index.
  2. *A fixed room.* The indexes' room is a constant of the pass and does not depend on what the recalculation
     has spent. It is as large as today's `MAX_KEPT_STATES`, 4,000,000 states of 24 bytes (`SumParts`), 96 MB of
     states, and as today's `MAX_CACHED_BITS`, 32,000,000 bits, 4 MB of bits, for the criteria sets; keys, table
     slots and vector headers are outside those figures, as they are today. The room is apart from today's
     caches' rooms, so no read that is not through an index uses up an index's room; the two together are
     bounded by twice 96 MB and twice 4 MB. A running total keeps one state per stored cell of its start's
     prefix, so three prefixes of a full column fit. A criteria set over 100,000 cells costs 100,096 bits, so
     319 of them fit: a SUMIFS per row with a key per row leaves the index at row 320 and reads on by today's
     path, which answers such rows from the range's value index (`ifs_cache.rs`). That index is not one of this
     design's, so that shape keeps 11.1's flag; making it one, with a build per range and a room of its own, is
     the amendment after this one. A build that would pass the room is not made, and its reads fall back to
     today's paths. A criteria set is built when its criterion is first met. Its build is charged once to the
     pass, outside any formula's run (SPEC-0028 section 11), so a read through it charges the constant whichever
     formula met the criterion first, in a full pass and in a restricted one. Charged to the formula that met it
     first, the build would follow the order again: with `C1: =COUNTIF(A1:A100000,"x")` and
     `C2: =COUNTIF(A1:A100000,"x")+B2`, a full pass would charge it to C1 and, after a write of B2, the restricted
     pass to C2. A full pass records which builds it made and what each took of its room (their keys, not their
     sets). A kept restricted pass adds the builds it made that the record does not hold, since the formulas it
     ran are clean in the next one, and for a build the record holds, what it took beyond the record's figure: a
     running total's prefix grows with the longest range read, and a restricted pass, whose room starts empty,
     can read further than the full pass built it. A restricted pass that would take the record past a room is
     rejected: the record holds every build a full pass of the new state makes, so within the room that pass
     makes them all. The record only grows between full passes, so a workbook near the room runs a full pass
     every so many edits of a criterion cell. A build's charge is committed work outside any cell's run, the
     first of its kind. Under Design 4's per-cell spend a restricted pass adds to the bound the charge of each
     build it made that the record does not hold, and what a build grew by; the summing bound adds every build
     it made (Design 4, "Why it then stays an upper bound"). Which builds fit follows the order of the pass,
     which is a function of the document (rule 1). The `MAX_CACHED_CELLS` of `range_values.rs` stays, with
     today's path (rule 1) and its other readers (operator range operands, SUMPRODUCT's masked blocks); the one
     of `search_cache.rs` stays, with the lookups (T-5192).
  3. *A read through an index charges a constant.* A read that is not through an index is today's read, with
     today's charges, and it sets 11.1's flag as today: a full pass that made one still declines restricted
     passes. An anchor's reads are such reads (rule 1), so a workbook whose anchor calls a conditional aggregate
     keeps the flag.
  4. *The dense walk goes for walks that charge.* A charged walk visits stored cells only. The used area does
     not become an input of every range reader, which would dirty every range reader at every last-row move.
  5. *Canonical storage first* (Taskei P559664573): which reads use an index depends on the stored cells, which
     are not yet the same on every replica.
- **What this gives the index.** The conditional aggregates and the running totals that read through an index
  stop charging by the order of the pass. A full pass whose conditional aggregates all read through indexes need
  no longer decline restricted passes (11.1's table, the IFS cache's row; the running totals charge alike
  already), and T-5143 has its second precondition. The search cache's row stays: restricted passes for a
  workbook that searches are T-5192, which gets its premise from this pass.
- **What it must show.**

  | Test | What it checks |
  |---|---|
  | Total on top | restricted, with a full recalculation's values; declined without the pass |
  | Same units in two orders | a running total and a SUMIFS column give the same values and spend the same units when the document is built by edits and when it is loaded, and with its formulas entered row by row and column by column; on documents with no anchor (with anchors, every unit comparison waits for SPEC-0028 section 11's exception 3) |
  | Excel's order | the two WEEKNUM documents of "No evaluation begins in Excel's order", each built directly and reached through the call that shows the order: the pass that counts commits the same units, makes the same index reads and restricts alike |
  | Flat documents unchanged | where no footprint reaches ahead, the order and the values are unchanged bit for bit on the metamorphic flat histories; the unread branch over a formula the walk does not model ("What it guarantees") keeps today's order and stays flat |
  | Metamorphic | a totals-above-data generator: incremental = full = fresh over 10,000 sequences |
  | Depth and stack sweeps | T-5152's sweeps at depth 1 and stack 0, with their new figures |
  | RAND | under the seeded draw (T-5288), the same values in either order |
  | Cycles | a 2-cycle, a 3-cycle, a cycle with a reader above and below it, a cycle through a SUM range, each entered from every member: today's values. A cycle through a non-candidate (the N example above): the traversal leaves it and the entry is today's. A SUM of one range and a SUMIFS over readers of a cycle: today's values (D5 keeps 41). Iteration on: today's order, so today's values |
  | Anchors as today | the document of "Anchors keep today's evaluation": 10, 20, 21, 22, 23 with no restart; and `A1: =C3+E1:E2`, `C3: =G2+1`, `G1: =SEQUENCE(2)`: one restart, whose reader is A1. Today's values, restarts and facts in the log. A formula cell that a name op turns into an anchor: today's values, and at most one restart more than today |
  | No index | with incremental recalculation off, and with a build the cap refused, the pass is the same: the order, the values, the thrown-away work and, under the seeded draw, the draws; the wide count refuses alike |
  | The traversal's cost | asserted by a counter of the test build, not by time. `=SUM($B$1:Br)` for r = 1 to 100,000 over a formula column: the probe count is linear in r. W columns by 20 rows of formulas that each read the row above: one probe a rectangle at W = 100, 400 and 800. One formula over n columns that each hold one formula cell, on a diagonal: 2n probes at n = 500, 1,000 and 2,000. A 100 by 100 block of formulas read whole by m formulas: at m = 10,500 the counter stays within 2 (R + E) + F; at m = 10,600 no probe is made and the pass is today's, bit for bit. One formula of 2,000 references and a range of n formula cells: the references placed are linear in n |
  | Kept lists | the pass is the same with the lists kept and with them rebuilt, over histories that move a spill by a literal write, that type a literal over an anchor, over a spill cell and over a formula cell, and that turn a formula cell into an anchor by a name op; an index read made before a spill is laid in its range restarts the pass; a restricted pass and a full pass make the same reads through an index over a range that holds a clean anchor's spill |
  | The room's edge | documents whose ranges cross the indexes' room: a loaded and an edited replica give the same values and spend the same units; a restricted pass whose builds would take the record past the room is rejected; running totals in enough columns to pass the states' room: a restricted pass that reads past where the full pass's prefix stopped is rejected; a SUMIFS per row with a key per row over 100,000 cells leaves the index at row 320 |
  | Criteria builds | the two COUNTIF cells of rule 2: the full pass, and the restricted pass after a write of B2, charge each formula the same. With `C1: =COUNTIF(A1:A100000,G1)`, `C2: =COUNTIF(A1:A100000,H1)` and H1 changed from "x" to "y", the per-cell bound holds both builds |
  | Today's cache kept | a SUMIFS column over a spilled column, and one over a column of formulas off the allowlist, at 100,000 rows: normal mode, and a spend no higher than today's; operator range operands and SUMPRODUCT's blocks stay within `MAX_CACHED_CELLS` |
  | The eval suites | `make eval` changes no cell |
  | Replays | the replay tests T-5298 asks for (it is `todo`), once they exist, pass unchanged, with an OFFSET chain among them |
  | A leaf | a `TODAY()` cell placed below the formula that reads it: the full pass is flat and an unread edit is a restricted pass of the leaf and its reader; the same with `=SUM(INDIRECT("A1"))`, `=TODAY(C5)`, `=RANDBETWEEN(C6,C6)` or `=TODAY()+A1` in its place still declines |
  | The restricted pass | the total-on-top edit is kept, not rejected; an always-dirty cell that reads a clean cell with a later turn is still rejected; of the three documents of rule 2 the first two are rejected, at depth limit 1 too, and the third is kept at every edit that leaves Z1 as it is; a static cycle no value closes restricts where it does today; the metamorphic totals-above-data generator counts restricted passes kept |
  | Lifetime | the two edits of "Lifetime of the record" (delete row 1; `=1` written into A1) each run a full pass. So does the write of `=IF(Z1,C7,0)` into an A1 that held `=1`, over `B5: =YEAR(Y5)`, `C7: =B5+D9` and `D9: =E9*2`, when the pass before it took D9 early. Where the last full pass took no cell early, a formula write and a structural edit restrict as they do without this design, and then: with `A1: =IF(Z1,C9,0)`, `=YEAR(Y9)` written into C9 and Z1 set to TRUE, the edit of Z1 is rejected (C9's turn is its place, after A1's); with the COUNTIF document of "Lifetime of the record", A1 reads through no index after the write into C5 |
  | The index bound | the indexes hold at most 96 MB of states and 4 MB of criteria bits, whatever the document, beside today's caches' own rooms |
  | Mutations | a traversal run before an anchor; a candidate taken early over an unevaluated formula the walk does not model; a left cell taken by another cell's traversal; a turn taken from the cell's place after a cell was taken early; no turn for a cell that became a formula; the record kept across a formula write or a structural edit after a cell was taken early; the complete cells kept across a formula write; an evaluation that begins in Excel's order; a resumed frame that places its whole footprint; a leaf judged by 11.1's list; an index read made inside another cell's run; an index read over a reader of a cycle; candidacy asked of a restricted pass's dirty cells only; a criteria build charged to a formula's run; a build, or a prefix's growth, not added to the record; a build's charge left out of the per-cell bound; lists kept across a moved spill or a literal typed over a spill cell; a rectangle walked by its columns when its rows are fewer; the wide count not applied when there is no index |
- **Order and effort** (its authors' estimate): about 2 engineer-weeks. It goes after T-5132, whose list is its
  candidates' rule. Its RAND test needs T-5288's seeded draw. The per-rectangle structure, anchors' precedents,
  the value index and restricted passes after a formula write that follows an early take are not in it.
