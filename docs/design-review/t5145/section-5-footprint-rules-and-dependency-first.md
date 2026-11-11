# T-5145 section 5: footprint rules (T-5143), with the dependency-first pass folded in

*poc-design-cc, 2026-11-11. Draft for a design read, before the reviewers. The text under "Proposed spec text" goes
into SPEC-0002 as Designs 12 and 13. Design 13 is poc-s-e2-1-cc's reviewed draft, folded in at their request.*

## In short

- **Design 12, footprint rules.** SUMIF and AVERAGEIF read a resized sum range; LOOKUP reads on past a short result
  vector. Each joins the index with a rule that names those cells. The rules are T-5143's first change, which was
  withdrawn when its review found two holes. This design adds the two preconditions that close them: LOOKUP's
  searches are heads, and a conditional aggregate's charge must be a function of what the index files.
- **Design 13, the dependency-first full pass and document-decided indexes.** Before a full pass evaluates a
  formula, it evaluates the formulas of its static footprint, from its own loop. A total above its data then no
  longer nests, so its edits are restricted again (measured: 449 to 465 ms against 36 ms for 100 edits on 10,000
  rows). The running-total and criteria caches become indexes whose use the document alone decides, which is what
  lets conditional aggregates into restricted passes (lookups wait for T-5192), and what SPEC-0028 section 11 asks
  before range walks can be charged. Corrected 10:50Z after a fact-check: a flat full pass is not enough for a
  total above its data, so Design 13 now also says how a restricted pass orders its cells and what "reading
  ahead" means in that order. That part is mine, for the draft's authors to confirm.
- **The order of work that follows:** T-5132 (always-dirty list), then Design 13, then T-5143, T-5192 and the
  second slices of sections 2 and 4.

## Open questions

1. T-5143 is implemented after Design 13, not before. Until then SUMIF, LOOKUP and the conditional aggregates are
   always dirty and keep today's cost. Agree with that order? (yes/no; mine: yes; the withdrawn change showed the
   other order is unsound.)
2. Design 13 goes into this CR as its own section, though it also amends SPEC-0028 section 11 (a pointer line
   there). Agree? (yes/no; mine: yes; its author asked for it and both owning lanes reviewed the draft.)
3. Design 13 changes the order of a full pass where a footprint reaches ahead. Values in normal mode do not
   change; which cell draws which RAND value does, and the depth-limit figures of some tests change. Accept?
   (yes/no; mine: yes.)
4. D-functions, and criteria that hold a range, stay always dirty with no rule. Acceptable? (yes/no; mine: yes.)

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
    row or one cell, as far as the lookup vector is long. The rule files the result range extended that far. A
    two-dimensional result vector is `#N/A` and reads nothing more.
  - A resized or read-on argument must be a written reference or range. One computed by a function has no rule,
    and its formula stays always dirty.
  - **COUNTIF, COUNTIFS, SUMIFS and AVERAGEIFS** read exactly their ranges and criteria, and need no footprint
    rule; the strict shape of the *IFS functions makes a mismatched range `#VALUE!`. MINIFS and MAXIFS join with
    a test of their own, as any function does (Design 1).
  - **A criterion that holds a range** (`COUNTIF(A1:A9,A1:A3)`, `">"&B1:B2`) lifts the call, which charges by the
    positions its walk visits, and those follow the used area: its formula stays always dirty.
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
- **Maintenance.** A refile applies the rule as the build does.
- **`precedents` and `dependents`** (Design 10) answer the written relation: the rule's rectangle is not a
  precedent, and Design 10's check drops it.
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
  - Mutations: a rule's rectangle not filed; LOOKUP's head left out; a criteria walk the hook does not record.

**13. The dependency-first full pass, and per-pass indexes the document decides** (amendment to Design 4 and to
SPEC-0028 section 11, 2026-11-11). The design is poc-s-e2-1-cc's draft of 2026-11-10, folded in at their request. It
was reviewed before any code by poc-s-s1-cc for SPEC-0028 (5 changes) and by poc-engine-cc for the evaluator (6
changes), all applied. It answers Taskei P559418140, which decision 0422 (T-5154's change) names as the owner of
declined passes, and Taskei P559664607. Taskei P560151028 (a refused build is tried again only when what refused it
changes) is beside it and not part of it.

- **The two problems.**
  1. *Totals above their data.* With `C1 = SUM(B1:B10000)` over `B r = A r*2`, the pass loop reaches C1 in row 1,
     and C1's SUM starts B2 to B10000 inside its own evaluation. The full pass nests, so every literal edit is
     declined. Measured natively, 10,000 rows and 100 edits: 449 to 465 ms with the total in C1, the same as with
     incremental off, against 36 ms with the total in C10001. Design 4 excludes nesting, it does not bound it, so
     the full pass must nest less.
  2. *SPEC-0028 section 11's indexes.* A per-pass structure is an index when it is built linearly once per pass
     and whether a read uses it is a function of the document alone. The running totals' prefix cache
     (`sum_prefix.rs`) and the criteria cache (`ifs_cache.rs`) fail the second condition: their use follows
     `read_is_inert`, which follows the order of the pass, and capacities other formulas can use up
     (`MAX_KEPT_STATES`, `MAX_CACHED_CELLS`). Charged as misses, a running total filled down 100,000 rows is 2.4 to
     2.9 times the pass allowance, and a SUMIFS per row is 2 times it.

  Both have one cause: the order of the pass, not the document, decides when a range's formulas are evaluated.
- **The pass.** In a full pass that is not in Excel's order (SPEC-0017), before the loop evaluates a root R, an
  anchor or a cell, it evaluates the formula cells of R's static footprint that are not evaluated yet, depth first,
  from its own loop, with an explicit stack:
  1. Push R. While the stack holds a cell T, take the first formula cell of T's footprint, in placement order,
     that is not evaluated, is not on the stack and is a candidate. If there is one, push it. Otherwise pop T and
     evaluate it at depth 0 (`evaluate_root_cell`).
  2. A formula cell is a **candidate** when the read index's walk gives its footprint (`read_index/walk.rs`, the
     one definition) and it is no dynamic anchor and no spill cell. A formula the walk does not model (Design
     11.1's list) is no candidate, and neither is a formula reached only through one. If it is read, it nests on
     demand, as today. One kind of unmodelled formula is a candidate all the same: a **leaf**, a formula with
     no reference, no range and no name anywhere in its tree (`=TODAY()`, `=NOW()`, `=RAND()`, and `=ROW()` or
     `=COLUMN()` with no argument). It reads no cell, so running it first cannot read ahead or nest
     (ruled by the draft's authors when it was folded in). An always-dirty formula whose footprint is unknown or
     dynamic (INDIRECT, OFFSET, a whole column) stays a non-candidate.
  3. A cell already on the stack is a static cycle. On finding one, R falls back: it is evaluated as today, in
     row-major order, with nothing of its footprint evaluated first, and the cycle is left to today's detection.
     A cycle's values can depend on where it is entered, and this pass must not move that.

  Finding a rectangle's formula cells uses a per-sheet list of formula positions by column, sorted by row: built
  once per pass, O(F log F), searched in O(log F + hits). It is measured on the 60,000-formula book before the
  code lands; above 2 ms to build, it is kept between passes and rebuilt only when the layout changes. The
  traversal is unmetered work in SPEC-0028 section 11's terms: once per pass, so `T_pass` (the position lists
  are `T_setup` if they are kept between passes). It is bounded by the formulas, their footprints and the
  stack's F cells (directive 5). Design 9's cones and Design 10's `precedents` use the same
  lists and the same traversal.
- **What it guarantees.**
  - Every candidate is evaluated with its static inputs evaluated. For a document whose formulas are all
    candidates with acyclic footprints the full pass is flat: a total above its data no longer nests, and neither
    does an anchor that reads formula cells (Design 11.3; T-5297's reloaded twins), nor a leaf read by a formula
    placed before it. A formula that is no candidate still nests when it is read before the loop reaches it
    (Design 11.1, "What it does not give").
  - Values in normal mode do not change: a formula's value is a function of its inputs.
  - The order changes only where a footprint reaches a formula not yet evaluated. A footprint is a superset of
    the reads, so an IF's unread branch can move a formula earlier; both orders are flat.
  - RAND draws from one sequence, so a change of order changes which cell draws what. No history pins a draw,
    and under the collaborative profile a draw is a function of the seed and the cell (I-07).
  - Anchors still run first, in `anchor_order`. Excel's order is untouched.
  - The depth-limit unwinds of a chain read from the top go away: its thrown-away work becomes 0. The tests that
    pin today's figures change to the new ones and keep their checks.
- **The restricted pass follows the same order** (added when the draft was folded in: a flat full pass is not
  enough, since Design 4's restricted pass runs its roots in (sheet, row, column) order and calls a clean cell
  placed after its root "ahead"; to be confirmed by the draft's authors). With `C1 = SUM(B1:B10000)` and an edit
  of A5, the closure is B5 and C1. In natural order C1 runs first, starts B5 inside itself and reads B2 onward
  ahead, so the pass would be rejected at every edit, whatever the full pass did. So:
  1. A restricted pass runs its roots through the traversal above, kept to dirty cells: before a dirty root
     runs, the dirty formula cells of its static footprint run, from the pass's own loop. A clean cell of the
     footprint is read as stored.
  2. Design 4's read-ahead rule is restated for this order. A dirty cell reads **ahead** when it reads a clean
     formula cell that a full pass would evaluate after it. Under this pass that is a cell placed after its
     root in natural order that the root's static footprint does not hold. A read inside the footprint is never
     ahead: a full pass evaluates the footprint's formula cells before the root, from its own loop, as the last
     full pass did. Every clean formula cell is a candidate or an anchor, since a formula the index does not
     model is never clean (11.1), and an anchor runs before every cell that is none. A root with no footprint
     (a formula the index does not model) keeps today's rule.
  3. `Incremental::last_full_flat` keeps its meaning, and a static cycle met in a restricted pass rejects it, as
     nesting does today.
- **The indexes** (SPEC-0028 section 11 rule 1).
  1. *Use decided by the document.* A range whose formula cells are all candidates has them evaluated before any
     reader of the range runs, so `read_is_inert` holds for it by construction. Such a range is read through the
     index whenever the index has room for it. A range that holds a non-candidate, a dynamic reference or a
     cycle's member is not: its reads charge their walk.
  2. *Room per range, from the document.* `MAX_KEPT_STATES` and `MAX_CACHED_CELLS` go. A running total keeps one
     state per stored cell of its start's prefix; a criteria index keeps one bit set per distinct criterion of a
     range, and its read values once. Each state held is charged when built, against the pass's ceiling, outside
     any formula's run, so the indexes hold at most the allowance's worth of states. The build walks the ranges
     in (sheet, top, left, bottom, right) order; a range whose index would pass the ceiling, and every range
     after it, gets none and charges its walk. So two replicas of one document cut at the same range.
  3. *A read through an index charges a constant.* Any other read charges its walk, a cached one included:
     Design 4's "the same on a hit and a miss".
  4. *The dense walk goes for walks that charge.* A charged walk visits stored cells only. The used area does not
     become an input of every range reader, which would dirty every range reader at every last-row move.
  5. *Canonical storage first* (Taskei P559664573): which reads use an index depends on the stored cells, which
     are not yet the same on every replica.
- **What this gives the index.** The conditional aggregates and the running totals stop charging by the order of
  the pass. A full pass that ran a conditional aggregate need no longer decline restricted passes (11.1's table,
  the IFS cache's row; the running totals charge alike already), and T-5143 has its second precondition. The
  search cache's row stays: restricted passes for a workbook that searches are T-5192, which gets its premise
  from this pass.
- **What it must show.**

  | Test | What it checks |
  |---|---|
  | Total on top | restricted, with a full recalculation's values; declined without the pass |
  | Same units in two orders | a running total and a SUMIFS column give the same values and spend the same units when the document is built by edits and when it is loaded, and with its formulas stored row by row and column by column |
  | Flat documents unchanged | where no footprint reaches ahead, the order and the values are unchanged bit for bit on the metamorphic flat histories |
  | Metamorphic | a totals-above-data generator: incremental = full = fresh over 10,000 sequences |
  | Depth and stack sweeps | T-5152's sweeps at depth 1 and stack 0, with their new figures |
  | RAND | under the seeded draw, the same values in either order |
  | Cycles | a 2-cycle, a 3-cycle, a cycle with a reader above and below it, a cycle through a SUM range, each entered from every member: today's values |
  | The ceiling's edge | documents whose ranges cross the indexes' ceiling: replicas and both entry orders give the same values and spend the same units |
  | Replays | the replay tests T-5298 asks for (it is `todo`), once they exist, pass unchanged, with an OFFSET chain among them |
  | A leaf | a `TODAY()` cell placed below the formula that reads it: the full pass is flat and an unread edit is a restricted pass of the leaf and its reader; the same with `=SUM(INDIRECT("A1"))` in its place still declines |
  | The restricted pass | the total-on-top edit is kept, not rejected; a dirty root that reads a clean cell outside its footprint and after it is still rejected; the metamorphic totals-above-data generator counts restricted passes kept |
  | The index bound | each state is charged at its build and stays within its byte bound |
- **Order and effort** (its authors' estimate): about 2 engineer-weeks. It goes after T-5132, whose list is its
  candidates' rule.
