# T-5145 section 3: `precedents` and `dependents` (T-5115) — draft for the project lead

*poc-design-cc, 2026-11-11. Draft for a design read, before the reviewers. The text under "Proposed spec text" goes
into SPEC-0002 as Design 10. Rules marked (probe) are provisional until the P cases of the probe request answer them.*

## In short

- **The relation is the written one**: what a formula's text refers to, through names, on any sheet. It is not what
  the evaluation reads. That is what Excel's own tracing can be, since Excel calls OFFSET, INDIRECT and some SUMIFs
  volatile exactly because its dependency tree does not hold what they read.
- **A complete answer for what is written, and it says when that is not all.** `Trace::unresolved` lists the
  formulas whose evaluation reads cells their text does not name (INDIRECT, OFFSET, SUMIF's resized range), so a
  caller asking "what moves if I change this" knows when the answer is a lower bound (the engine lane's review).
  Two bounds keep a call from running away: 65,536 areas, and a work budget.
- `precedents` reads the formula; it needs no index. `dependents` takes candidates
  from the read index and from the list of formulas the index does not model, and checks each candidate
  against its written areas. With no index it scans every formula: same answer, slower. So the API never says "I
  cannot tell", which T-5115's task text allowed.
- **Areas, not cells**, grouped and ordered by one rule, so every replica gives the same list.
- **One walk.** The read index's formula walk gains a second reading that accepts every node. The index's reading
  is that walk cut down, so the two cannot drift apart.

## Open questions

1. `precedents` and `dependents` answer the **written** relation (Excel's Trace arrows), not "what would be
   recalculated". The second question is the dirty closure, which Design 9 exposes as `is_stale`. Agree? (yes/no;
   mine: yes, pending probes P7 to P9.)
2. An **empty result** where Excel throws `ItemNotFound`. Agree? (yes/no; mine: yes; an Excel error is a value in
   this API, and `Err` is misuse.)
3. `dependents` **always answers**, by a scan when there is no index (O(formulas)), instead of failing, and never
   builds an index to answer one call. Agree? (yes/no; mine: yes; the UI and engine lanes answered yes.)
4. A cap of **65,536 areas** per answer, then `TraceError::TooLarge`. Acceptable? (yes/no; mine: yes. A million
   dependents in one column are one area.)
5. Should the agent API's "explain cell" (CONTRACT.md, interface 4) be built on this in the same task? (yes/no;
   mine: no, it is its lane's task; this gives it the walk.)

## Probes this section waits for

`/mail/oracle/t5145-recalc-probe/`: `PREC` and `DEP`, 25 formulas and 16 cells. The three that can change the
design, not only a detail, are P7 (INDIRECT), P8 (OFFSET) and P9 (SUMIF's resized range).

## Proposed spec text

**10. `precedents` and `dependents`** (amendment for T-5115, 2026-11-11; the rules marked *(probe)* are provisional
until the P cases of the probe request `t5145-recalc` answer them).

- **What it is for.** Excel's Trace Precedents and Trace Dependents in the collaborative sheet, and an agent's
  questions: "what does this formula refer to" and "which formulas refer to this cell". Today nothing answers
  either without reading every formula. It also answers "what moves if I change this input", but only as far
  as formulas say in their text what they read: the answer names the formulas that do not (`unresolved`, below),
  and with none of them it is the whole answer.
- **Excel.**
  - Documented (docs/research/recalc-modes-and-dependency-tracing-2026-11-11.md; S2 is Microsoft's Office.js how-to, the
    numbers are that note's sections). "Precedent cells — cells that are referred to by a formula in another
    cell. Dependent cells — these cells contain formulas that refer to other cells" (6). "A 'direct precedent' or
    'direct dependent' is only the first level" (S2). `getDirectPrecedents`, `getPrecedents`,
    `getDirectDependents` and `getDependents` return areas grouped by worksheet; they "do not cross workbook
    boundaries"; with nothing to return they fail with `ItemNotFound` (S2). The older VBA properties are
    narrower: "The DirectPrecedents property only works on the active sheet and cannot trace remote references"
    (5). Among what tracer arrows cannot trace: "References to named constants." (6).
  - Microsoft lists OFFSET, INDIRECT and SUMIF "depending on its arguments" among the volatile functions (S1), and
    says of SUMIF that with a `sum_range` of another shape "the formula will sum a range of cells that starts with
    the first cell in sum_range but has the same dimensions as range" (addendum). Excel makes such a function
    volatile because its dependency tree does not hold what it reads. So this design takes Excel's relation to
    be the **written** one: what the formula's text refers to, not what its evaluation reads *(probe: P7 to P9
    ask it of INDIRECT, OFFSET and SUMIF's resized range)*.
  - Not documented, and probed (`PREC`, `DEP`): whether adjacent areas are merged (P1); a whole column (P3); names
    (P5, P6); the branch an IF does not take (P10); 3-D spans (P12); unions and intersections (P18, P19); spill
    cells and spill references, a blocked anchor's among them (P11); a table's structured reference (P13); a
    named LAMBDA's body (P22); a range operator (P23); a sum range longer than its criteria range (P9b); a
    conditional format's rule and a data validation's formula (P24).
- **Other engines** (the same note). HyperFormula's `getCellDependents` and `getCellPrecedents` return "all the
  out-neighbors in the dependency graph", ranges among them, one level only: for more, "implement a Breadth-first
  search (BFS) algorithm" (1). LibreOffice's Detective "works per level" and has no API we found (2). Apache POI
  tracks dependents inside its cache and exposes none (8). We take HyperFormula's ranges-not-cells answer and
  avoid its one level: Excel's API gives every level, and so does this one.
- **The relation.** For a formula cell F, its **direct precedents** are the areas its formula refers to as written:
  - every cell reference and range, on any sheet, a whole column or row as written *(probe P3)*;
  - each area of a union, and both operands of an intersection *(probes P18, P19)*;
  - a range operator between two written references: their bounding rectangle *(probe P23)*;
  - the area on every sheet of a 3-D span;
  - the areas a defined name stands for, resolved as the evaluation resolves the name: a reference or range
    name placed at F (`NameReference::at`), a formula body walked, through names that use names *(probes P5,
    P6)*; and the areas the body of a named LAMBDA that F calls refers to, resolved as the call resolves it,
    the using cell's sheet first *(probe P22)*;
  - a structured reference's area, as lowered (SPEC-0016), `[#This Row]` at F's row;
  - a spill reference `A1#`: the anchor's spill area as it is stored; the anchor cell alone when the anchor is
    blocked *(probe P11)*;
  - the references in every branch and argument, read or not *(probe P10)*.

  Not precedents: the cell INDIRECT's text names; OFFSET's result (its reference argument is one); the cells a
  function reads beyond its arguments, such as SUMIF's resized range *(probes P7 to P9)*; an area off the grid
  (Excel's `#REF!`), which is dropped. A spill cell's precedents are its anchor's *(probe P11)*. A cell that holds
  no formula has none. A conditional format's rule, a data validation's formula and a pivot are not formula
  cells and are not in the relation, and a data table's result cells hold no formula *(probe P24 asks whether
  Excel traces a rule that reads the cell)*.

  The **direct dependents** of a cell C are the formula cells and anchors one of whose direct precedent areas holds
  C. A spill cell is not listed as a dependent: its anchor is *(probe P11: whether Excel lists the anchor or
  the whole spill)*. `Depth::All` is the transitive closure. It passes through spill cells: an anchor that is
  reached puts the cells of its footprint on the frontier, and a spill cell met in a precedent area leads to
  its anchor (`SpillCell::a`). So with J1 `=SEQUENCE(3)+A1` and K2 `=J2*2`, K2 is in `dependents(A1, All)` and
  A1 in `precedents(K2, All)`. For a range target, a cell is a precedent or dependent of the range when it is
  one of any of its cells; a target cell is listed only when another target cell reaches it, and a cycle through
  the target lists it.

  A formula is **unresolved** when its evaluation can read cells its text does not name: it calls INDIRECT or
  OFFSET, a function with a footprint rule (Design 12) or a D-function, or it has a range operator or a `#` whose
  operand is not a written reference. The relation is exact for what is written and silent about the rest, so
  the answer carries the unresolved formulas it met.
- **The API** (facade; after docs/research/api-landscape.md 4.9, which has `Depth` with a `Max(n)` and returns a
  list of references; the differences are what the consumers asked for).
  - `Workbook::precedents(&self, target, depth: Depth) -> Result<Trace, TraceError>` and
    `Workbook::dependents(&self, target, depth: Depth) -> Result<Trace, TraceError>`, with `Depth::{Direct, All}`.
    Both take `&self`, as every read does. `dependents` uses the read index when the workbook has built one and
    scans otherwise; it never builds one, since a build can cost more than the scan it would save and a read
    must not mutate.
  - A batch form for the arrows, `Workbook::precedents_each(cells, depth)` and `dependents_each(cells, depth)`,
    answers one `Trace` per cell, in the order given. Excel draws an arrow from each precedent area to the
    formula cell that reads it, so a press of Trace Precedents needs the answer per formula cell of its frontier,
    in one call (poc-ui-c-cc: the UI does the levels itself, with `Depth::Direct`). A batch takes at most
    65,536 cells and returns at most 65,536 areas in all.
  - `Trace { areas: Vec<RangeRef>, unresolved: Vec<RangeRef>, external: u32 }`. `areas` are rectangles, never
    exploded cells, in (sheet by tab position, top, left, bottom, right) order with no duplicates. Precedent
    areas are the written ones; for `Depth::All` an area that lies inside another is dropped, so a running
    total over 65,537 rows is one area, not 65,537. Dependent cells are joined into rectangles by one rule, so
    the answer is the same on every replica: runs down each column first, then equal runs of adjacent columns
    joined. `unresolved` holds, joined the same way, the unresolved formulas the call met: for `precedents`,
    those among the target and the formula cells it walked; for `dependents`, every unresolved formula of the
    workbook that is not in `areas`, since any of them may read the target. `external` counts the written
    references into other workbooks among the formulas walked (the UI lane asked for it); they give no area.
  - An empty `Trace` is the answer where Excel fails with `ItemNotFound`: an Excel error is a value, and `Err` is
    API misuse (SPEC-0001).
  - Two bounds (directive 5), each a `DEVIATION` entry, since Excel documents neither ("very large sets ... can
    be slow", S2): `TraceError::TooLarge` when the answer would hold more than 65,536 areas, counted while the
    walk runs; `TraceError::TooMuchWork` when the call passes `MAX_TRACE_WORK`, 2^28 steps, where a step is one
    candidate enumerated, one rectangle visited, one cell enumerated or one check.
  - The API reads no value. Trace Error is the caller's walk over `Depth::Direct` and the cells' values.
- **Where the answer comes from.** One walk of a parsed formula gives its written areas (`read_index/walk.rs`).
  Today the walk's leaves are reference and range nodes, found once per parsed formula and placed at each cell.
  The written reading needs areas that are not such nodes: a name placed by `NameReference::at`, `[#This Row]`
  at a row, a spill's stored footprint. So the walk's leaf becomes one type of area source, and placing takes
  the model. The index's reading refuses what it does not model, as today, and is the written one cut down,
  so the two cannot drift apart; the index build's measured costs (Design 1's tables) must not move.
  `tailed_leaves` stays: it answers another question (which arguments are read tailed). The eval harness's
  text-based `precedents` (`rustsheet-eval`, `volatile.rs`) stays too, as an oracle that shares no code.
  - **Names are walked once.** Each (name, scope) body is walked at most once per formula: every name in one
    formula is placed at the same cell, so the first walk's areas are the name's. Bodies count against
    `MAX_DEFINED_NAME_EVALUATIONS`, as the evaluation's do, and a recursive named LAMBDA stops at its second
    visit. Depth alone does not bound it: with `n1 = n2+n2` down to `n63`, a walk that enters every use is 2^63
    steps.
  - **`precedents`** reads the cell's formula, places its areas at the cell, and for `Depth::All` goes on through
    the formula cells, anchors and spill cells inside them. It needs no index. Finding them in a large area uses
    a per-sheet list of positions by column, as Design 13 builds for its pass but holding anchors and spill
    cells too; outside a pass it is built per call, within the budget. Each column's rows are enumerated once
    per call: a later area is cut by what was enumerated already, so `C1 = SUM(B1:Bn)` over
    `B(i) = SUM($A$1:A(i))` costs n, not n²/2.
  - **`dependents`** takes candidates and checks each against its written areas:
    - with an index: its readers of the cell; for a spill cell or an anchor, the readers of the anchor too
      (Design 11.3 files `A1#` under the anchor cell, and its written area is the whole spill); and the
      formulas the index does not model, the always-dirty list (Design 11.1), which holds every formula with an
      OFFSET or an INDIRECT;
    - without one: every formula cell.

    The check makes the answer the written relation whatever else the index files for a formula: a footprint
    rule's rectangle (Design 12) is a read, not a written area, so it yields candidates the check drops.
  - **One pass over the candidates, not one per cell.** Three shapes would otherwise be quadratic with a tiny
    answer. So for `Depth::All`, for a target of more than 1,024 cells and for a batch, the call first files
    the written areas of the formulas it will have to check more than once, in a structure of its own that
    lives for the call (the index's rectangle store, under its cap): the always-dirty list when there is an
    index, every formula when there is none. Then it runs its walk against that structure, and a batch or a
    large target is checked through a per-call set of the target cells. The index's own large rectangles are
    the remaining cost: with `C(i) = SUM($B$1:B(i))`, a walk from each B(j) visits the n-j+1 rectangles that hold
    it, so `dependents(X1, All)` over `B(i) = A(i)*$X$1` is n²/2 visits for two areas. The budget stops it, and
    the closure's own answer (Design 3 has a limit for the same reason) is the bench that says whether a
    rectangle-by-rectangle walk is needed.
- **Why the candidates are enough.** An indexed formula files every reference and range of its tree (`leaves`
  takes every `ReferenceKind` and `RangeKind`, in every branch), with Design 11.2 the rectangles its names
  resolve to, and with Design 12 a footprint rule's rectangle beside the written range. So every written area
  of an indexed formula is filed, with one exception, `A1#`, which is filed at its anchor, whose readers the
  candidates include. A formula with a written area the index does not file is on the always-dirty list. The
  candidates are a superset, and the check makes them exact.
- **Structural edits and maintenance.** Design 10 keeps nothing between calls, so nothing is maintained. A call
  after any edit reads the formulas as they are.
- **Calculation modes.** The relation is about formulas, not values, so it is the same in every mode, with cells
  owed or not. A spill reference's area is the stored one: in Manual mode it can be stale, as the cells are.
- **What the implementation may rely on.**
  1. Within the index, `leaves` and `instantiate` are the one definition of a formula's areas at a cell
     (`walk.rs`). The written reading extends them with the area sources above; it does not copy them.
  2. A `DefinedNameKind` node resolves through `Model::parsed_defined_name` with the scope the parser bound. A
     named LAMBDA call resolves elsewhere (`name_key::get_name`, the cell's sheet and then the workbook), and
     the walk uses that resolver for it.
  3. The index, when built, files every written area of every formula it models, `A1#` at its anchor.
  4. The always-dirty list is a list the index keeps (Design 11.1); before T-5132 lands, a workbook with such a
     formula has no index, and `dependents` scans.
  5. `get_cell_spill` gives an anchor's stored footprint, and a blocked anchor stores one cell.
- **What it must show.**
  - Two oracles that share no code with the walk. The generator writes each formula from a list of areas it
    draws (cells, ranges, whole columns, names, 3-D spans, unions, spill references, an unread IF branch), so
    the list is the expected answer: `precedents(F, Direct)` must equal it, and `dependents(C, Direct)` must be
    exactly the formulas whose list holds C. And the harness's text-based `precedents` agrees on the formulas
    it can read.
  - `dependents` with the index equals `dependents` by the scan, on the metamorphic generators' documents and on
    every eval suite workbook, spill references among them. Mutation: the anchor's readers dropped from the
    candidates.
  - `precedents(All)` and `dependents(All)` are inverse over formula cells and anchors: for such an F, C is in
    `precedents(F, All)` exactly when F is in `dependents(C, All)`, through a spill cell too.
  - Each P case of the probe request as a test, once answered.
  - Bounds: a formula over 65,536 distinct areas gives `TooLarge`; a million dependents in one column give one
    area; the name chain `n1 = n2+n2` to `n63` answers within the budget, timed.
  - Benches, each with its step count: `dependents` of an input read by 100,000 formulas, with the index and by
    the scan; the running-total shape above for `dependents(All)` and for `precedents(All)`; a 1M chain in a
    workbook with no index; 100,000 always-dirty SUMIFS beside a 100,000-cell chain; a batch of 10,000 cells.
- **Steps**, each S: (a) the written reading of the walk, `precedents`; (b) `dependents` from the index and by the
  scan, the batch form and the budget; (c) the Python and wasm bindings and the engine worker's `explain`, by
  their lanes.
