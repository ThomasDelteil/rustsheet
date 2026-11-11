# T-5145 section 3: `precedents` and `dependents` (T-5115) — draft for the project lead

*poc-design-cc, 2026-11-11. Draft for a design read, before the reviewers. The text under "Proposed spec text" goes
into SPEC-0002 as Design 10. Rules marked (probe) are provisional until the P cases of the probe request answer them.*

## In short

- **Changed again at 18:53Z on 2026-11-11, after round 1 of this section's review (CR-321762904).** The reviewer
  found the relation coherent and four things to fix, all small:
  - the "may be out of date" mark on an answer (`stale`) is now defined on the answer itself, so it is the same
    whichever way the engine computes it, and it covers a cell that a spilled array would grow onto;
  - the list of functions that read cells their arguments do not name (INDIRECT, OFFSET, the database
    functions, SUMIF, AVERAGEIF, LOOKUP) is one list in the code, with a test that fails when a function is
    missing from it (LOOKUP was missing from the draft);
  - the `stale` mark is built after Manual mode exists; the rest of the feature does not wait for it;
  - "the answer is too large" (65,536 areas) is the same everywhere; "too much work" can depend on whether the
    engine's index is available.
  One limit is now stated plainly: tracing the dependents of a parameter cell that feeds a column of running
  totals is refused for work from about 23,000 rows on, until a later step walks such columns as ranges.

- **Changed at 16:05Z on 2026-11-11, before review.** After the formal reviewer found holes in the section on
  calculation modes, I had this section read by an independent reviewer first. It found seven blocking points, all
  fixed in the text below. What it changes for a user of the feature:
  - Tracing **level by level now works across a spilled array**: the dependents of a cell include the whole
    spill of a formula that reads it, so the next level starts from the spilled cells. Before, a formula that
    read a spilled cell was reachable only by asking for all levels at once. The UI lane confirmed this is what
    its arrows need.
  - The answer is **the same with the engine's index and without it**, including the list of formulas it cannot
    speak for.
  - "All levels" answers are **joined into rectangles** for precedents too (a running balance over 40,000 rows
    is two areas, not 79,998, which would have been refused as too large).
  - A trace says when it used a spill's extent from **before the last edits** (Manual mode): `stale`.
  - Five rules about Excel that had neither a documentation quote nor a probe case now have one or the other:
    three new probe cases, and two rules marked as ours and unverified.
  Two more questions for the lead follow from it (6 and 7 in the CR's description: the spill among the
  dependents; a trace answers from the last calculation and says so instead of failing).


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
  either without reading every formula. It also goes most of the way to "which formulas can change if this
  cell's value changes": the answer names the formulas whose reads their text does not give (`unresolved`,
  below). It does not say what a cell's occupancy changes: a write into an anchor's spill area blocks the
  anchor, and clearing a cell can free a blocked one, and neither is in the relation. A data table's results
  and a pivot's cells hold no formula and are in no list.
- **Excel.**
  - Documented (docs/research/recalc-modes-and-dependency-tracing-2026-11-11.md; S2 is Microsoft's Office.js
    how-to, the numbers are that note's sections). "Precedent cells — cells that are referred to by a formula in
    another cell. Dependent cells — these cells contain formulas that refer to other cells" (6). "A 'direct
    precedent' or 'direct dependent' is only the first level" (S2). Office.js has four calls,
    `getDirectPrecedents`, `getPrecedents`, `getDirectDependents` and `getDependents` (the probe script calls
    them). S2 says of them: "Both APIs return a `WorkbookRangeAreas` object. It groups addresses by worksheet.";
    "These APIs do not cross workbook boundaries."; "If there are no related cells, an `ItemNotFound` error is
    thrown." The older VBA properties are narrower: "The DirectPrecedents property only works on the active sheet
    and cannot trace remote references" (5). Among what tracer arrows cannot trace: "References to named
    constants." (6).
  - Microsoft lists OFFSET, INDIRECT and SUMIF "depending on its arguments" among the volatile functions (S1),
    and says of SUMIF that with a `sum_range` of another shape "the formula will sum a range of cells that starts
    with the first cell in sum_range but has the same dimensions as range" (addendum). Microsoft does not say why
    they are volatile. Our reading, an inference, is that Excel's dependency tree does not hold what such a
    function reads. So this design takes Excel's relation to be the **written** one: what the formula's text
    refers to, not what its evaluation reads *(probe: P7 to P9 ask it of INDIRECT, OFFSET and SUMIF's resized
    range)*.
  - Not documented, and requested for probing (`PREC`, `DEP`): whether adjacent areas are merged (P1); a whole
    column (P3); names (P5, P6); the branch an IF does not take (P10); 3-D spans (P12); unions and
    intersections (P18, P19); spill cells and spill references, a blocked anchor's among them (P11), and a
    spill reference to a cell that is no anchor (P11h); a table's structured reference (P13), and `[#This Row]`
    on and off the table's rows (P13b); a named LAMBDA's body (P22); a range operator (P23); a sum range longer
    than its criteria range (P9b); a range target (P15) and one that holds one of its own precedents (P15b); an
    implicit intersection (P20, P20b); a LET-bound reference (P21); a conditional format's rule and a data
    validation's formula (P24).
  - Not asked, because the script cannot build them: a Ctrl+Shift+Enter array and a data table. The two rules
    below that speak of them are ours and stay UNVERIFIED until a case built from a file asks.
- **Other engines** (the same note). HyperFormula's `getCellDependents` and `getCellPrecedents` return "all the
  out-neighbors in the dependency graph", ranges among them, one level only: for more, "implement a Breadth-first
  search (BFS) algorithm" (1). LibreOffice's Detective "works per level" and has no API we found (2). Apache POI
  tracks dependents inside its cache and exposes none (8). We take HyperFormula's ranges-not-cells answer and
  avoid its one level: Excel's API gives every level, and so does this one.
- **The relation.** For a formula cell F, its **direct precedents** are the areas its formula refers to as written:
  - every cell reference and range, on any sheet, a whole column or row as written *(probe P3)*;
  - each area of a union, and both operands of an intersection *(probes P18, P19)*;
  - a range operator between two written references on one sheet: their bounding rectangle *(probe P23)*. A
    written reference is a cell or range reference in the formula's own text; a name is not one;
  - the area on every sheet of a 3-D span;
  - the areas a defined name stands for, resolved as the evaluation resolves the name: a reference or range
    name placed at F (`NameReference::at`), a formula body walked, through names that use names *(probes P5,
    P6)*; and the areas the body of a named LAMBDA that F calls, or passes as a value (`=MAP(A1:A3,MyFn)`),
    refers to, resolved as the call resolves it, the using cell's sheet first *(probe P22)*;
  - a structured reference's area, as lowered (SPEC-0016), `[#This Row]` at F's row; off the table's data rows
    it has no area, as its evaluation is `#VALUE!` *(probe P13b)*;
  - a spill reference `A1#`: the anchor's spill area as it is stored; the anchor cell alone when the anchor is
    blocked *(probe P11)*; the cell A1 when it holds a formula that is no anchor, and no area when it holds
    none, as the evaluation gives (`get_cell_spill`) *(probe P11h)*;
  - the whole operand of an implicit intersection, not the cell it picks *(probes P20, P20b)*;
  - the references in every branch and argument, read or not *(probe P10)*.

  Not precedents: the cell INDIRECT's text names; OFFSET's result (its reference argument is one); the cells a
  function reads beyond its arguments, such as SUMIF's resized range *(probes P7 to P9)*; an area off the grid
  (Excel's `#REF!`), which is dropped. A spill cell's precedents are its anchor's *(probe P11)*. The members of
  a Ctrl+Shift+Enter array are spill cells of its anchor in this engine, and every rule here for a spill cell
  holds for them (ours, UNVERIFIED). A cell that holds no formula has none. A conditional format's rule, a data
  validation's formula and a pivot are not formula cells and are not in the relation *(probe P24 asks whether
  Excel traces a rule that reads the cell)*, and a data table's result cells hold no formula (ours,
  UNVERIFIED).

  The **direct dependents** of a cell C are the formula cells and anchors one of whose direct precedent areas
  holds C, and with each such anchor the cells of its stored footprint: a spill cell's precedents are its
  anchor's, so the two relations are inverse at every depth *(probe P11: whether Excel lists the anchor or the
  whole spill)*. `Depth::All` is the transitive closure of `Depth::Direct`, in both directions: a caller that
  asks level by level reaches exactly what `All` returns. So with J1 `=SEQUENCE(3)+A1` and K2 `=J2*2`,
  `dependents(A1, Direct)` is J1:J3 and `dependents(J2, Direct)` is K2; `precedents(K2, Direct)` is J2, and
  `precedents(J2, Direct)` is A1. A spill cell leads to its anchor (`SpillCell::a`) only when that cell holds
  an array formula whose stored footprint holds the spill cell; an orphan has no precedents. For a range target,
  a cell is a precedent or dependent of the range when it is one of any of its cells; a target cell is listed
  only when another target cell reaches it, and a cycle through the target lists it *(probe P15b)*.

  A formula **reads beyond its text** in one of two ways.
  - It is **unresolved** when its evaluation can read cells that nothing names: it calls a function that reads
    beyond its arguments and has no footprint rule, itself or through a name or a named LAMBDA it uses; it has
    a range operator or a `#` whose operand is not a written reference; or its names pass the walk's limit
    (below). The functions that read beyond their arguments are one list in the code, beside
    `reads_only_its_arguments` (`walk.rs`): today INDIRECT, OFFSET, the D-functions, SUMIF and AVERAGEIF (the
    resized sum range, criteria.md) and LOOKUP (past a short result vector, lookup.md), the ones "Excel behavior
    to match" names. A function on neither list is taken to read its arguments only, and "What it must show"
    holds that against what the evaluation reads.
  - It has a **rule area** when it calls a function of that list that has a footprint rule (Design 12): the
    rule gives, from the formula's text and place, the rectangle the function reads. Until a function's rule
    exists, a formula that calls it is unresolved.

  The relation is exact for what is written and silent about the rest, so the answer carries the formulas that
  read beyond their text, as far as they can matter to it (`unresolved`, below).
- **The API** (facade; after docs/research/api-landscape.md 4.9, which has `Depth` with a `Max(n)` and returns a
  list of references; the differences are what the consumers asked for).
  - `Workbook::precedents(&self, target, depth: Depth) -> Result<Trace, TraceError>` and
    `Workbook::dependents(&self, target, depth: Depth) -> Result<Trace, TraceError>`, with `Depth::{Direct, All}`.
    Both take `&self`, as every read does. `dependents` uses the read index when the workbook holds a built one
    (`ReadIndexCache::Built`; one that an edit made stale is not) and scans otherwise, with the same answer; it
    never builds one, since a build can cost more than the scan it would save and a read must not mutate.
  - A batch form for the arrows, `Workbook::precedents_each(cells, depth)` and `dependents_each(cells, depth)`,
    answers `Traces { each: Vec<Trace>, unresolved: Vec<RangeRef> }`: one `Trace` per cell, in the order given,
    and, once for the batch, the part of `unresolved` that is the workbook's (below). Excel draws an arrow from
    each precedent area to the formula cell that reads it, so a press of Trace Precedents needs the answer per
    formula cell of its frontier, in one call (poc-ui-c-cc: the UI does the levels itself, with
    `Depth::Direct`). A batch takes at most 65,536 cells, and one of more is refused before any work
    (`TraceError::TooManyCells`); it returns at most 65,536 areas in all, `unresolved` included.
  - `Trace { areas: Vec<RangeRef>, unresolved: Vec<RangeRef>, external: u32, stale: bool }`. `Trace`, `Traces`
    and `TraceError` are `#[non_exhaustive]`, as Design 9 makes `ChangeSet`: a list of rule areas or a
    truncation flag may follow.
    - `areas` are rectangles, never exploded cells, in (sheet by tab position, top, left, bottom, right) order
      with no duplicates. For `precedents` with `Depth::Direct` they are the written areas, as written *(probe
      P1)*. For `Depth::All`, and for `dependents` at either depth, the cells of the answer are joined into
      rectangles by one rule: runs down each column first, then equal runs of adjacent columns joined. So the
      running balance `B(i) = B(i-1)+A(i)` over 40,000 rows has two precedent areas, not 79,998. The join works
      on rectangles and never enumerates the cells of an area.
    - `unresolved` holds, joined the same way, the formulas that read beyond their text and can matter to the
      answer. For `precedents`: those among the target and the formula cells it walked.
      For `dependents`: (1) every unresolved formula of the workbook that is not in `areas`, since any of them
      may read the target; and (2) every formula with a rule area that holds a cell of the target or of `areas`
      while none of its written areas does. In a batch, `Traces::unresolved` holds every unresolved formula of
      the workbook, once, and each `Trace::unresolved` holds part (2) alone: a cell's own answer is
      `Traces::unresolved` less the cells of its `areas`, with its part (2). For `precedents_each`,
      `Traces::unresolved` is empty and each `Trace` carries its own.
    - `external` is for `precedents`: the written references into other workbooks among the target and the
      formula cells walked (the UI lane asked for it). They give no area. For `dependents` it is 0.
    - `stale` is set when `is_stale` (Design 9) holds for a cell of the target or of `areas`: a recalculation can
      then change the answer, which is that of the last recalculation's footprints ("Calculation modes", below).
      An owed anchor makes every cell of its sheet stale (Design 9, "What is owed"), so a target the anchor would
      spill onto, or that an `A1#` would grow to hold, is marked although no stored footprint holds it. It can be
      set for an answer a recalculation leaves as it is: the owed set is a superset.
  - `areas`, `unresolved` and `external` are a function of the document, the stored footprints, the target and
    the depth, and `stale` of those and of what the workbook owes. All four are the same with the index and by
    the scan, and the first three on every replica that holds the same document and footprints.
  - An empty `Trace` is the answer where Excel fails with `ItemNotFound`: an Excel error is a value
    (docs/architecture/api-design.md, principle 1). `Err` is for a target that names no cell
    (`TraceError::Ref(RefError)`), and for the two bounds.
  - Two bounds (directive 5), each a `DEVIATION` entry, since Excel documents neither ("Very large sets, such as
    thousands of cells, can be slow.", S2).
    - `TraceError::TooLarge`: the finished answer holds more than 65,536 areas, `unresolved` included. It is
      decided on the joined answer, so it depends neither on the order in which the walk found the cells nor on
      the path: it is the same with the index and by the scan, and on every replica.
    - `TraceError::TooMuchWork`: the call passes `MAX_TRACE_WORK`, 2^28 steps, or its own structures (its
      frontier, the cells it reached, the areas it files for the check) pass the index's cap (`MAX_INDEX_BYTES`,
      512 MiB), counted as the index counts its memory. A step is one node of a formula or of a name's body
      walked, one area placed or tested, one candidate enumerated, one rectangle visited or one cell
      enumerated, so that nothing a call does is free. Both depend on the path: the scan files every formula's
      areas and the index does not. The bytes depend on the target too, as the index's do (Design 1, "The
      cap"). So a call can answer with a built index and fail with `TooMuchWork` without one, or answer on a
      server and fail in a browser. When both answer, they answer the same. The call's bytes are beside the
      index's, not summed with them; who sums the model's structures toward one cap is T-5167's question, as
      for Design 9's owed set.
  - The API reads no value. Trace Error is the caller's walk over `Depth::Direct` and the cells' values.
- **Where the answer comes from.** One walk of a parsed formula gives its written areas (`read_index/walk.rs`).
  Today the walk's leaves are reference and range nodes, found once per parsed formula and placed at each cell.
  The written reading needs areas that are not such nodes: a name placed by `NameReference::at`, `[#This Row]`
  at a row, a spill's stored footprint. So the walk's leaf becomes one type of area source, and placing takes
  the model. The index's reading refuses what it does not model, as today, and is the written one cut down,
  so the two cannot drift apart; the index build's measured costs (Design 1's tables) must not move.
  `tailed_leaves` stays: it answers another question (which arguments are read tailed). The eval harness's
  text-based `precedents` (`rustsheet-eval`, `volatile.rs`) stays too, as an oracle that shares no code; it
  strips the arguments of FORMULATEXT and ISFORMULA, so on those two it is no oracle.
  - **Names are walked once.** Each (name, scope) body is walked at most once per formula: every name in one
    formula is placed at the same cell, so the first walk's areas are the name's. The walk has one limit of its
    own, the evaluation's number: at most `MAX_DEFINED_NAME_EVALUATIONS` (10,000) distinct bodies per formula. It
    has no depth limit and needs none, since a body met again is not entered: a recursive named LAMBDA stops at
    its second visit. The evaluation counts every use and stops at 64 levels, so a formula it gives `#NAME?`
    (a doubling chain of 14 names) still has its written areas here. With `na_1 = na_2+na_2` down to `na_63`, a
    walk that entered every use would take 2^63 steps. A formula whose names pass the limit is unresolved, and
    its areas are those of its own text and of the bodies walked, in the walk's order, before the limit.
  - **`precedents`** reads the cell's formula, places its areas at the cell, and for `Depth::All` goes on through
    the formula cells, anchors and spill cells inside them. It needs no index. Finding them in a large area uses
    a per-sheet list of positions by column, sorted by row, of the kind Design 13 builds for its pass, here with
    spill cells listed too; outside a pass it is built per call, within the budget. Each column's rows are
    enumerated once per call: a later area is cut by what was enumerated already, so `C1 = SUM(B1:Bn)` over
    `B(i) = SUM($A$1:A(i))` costs n, not n²/2.
  - **`dependents`** takes candidates and checks each against its written areas and its rule areas:
    - with an index: its readers of the cell; for a spill cell or an anchor, the readers of the anchor too
      (Design 11.3 files `A1#` under the anchor cell, and its written area is the whole spill); and the
      formulas the index does not model, the always-dirty list (Design 11.1), which holds every unresolved
      formula;
    - without one: every formula cell.

    A candidate one of whose written areas holds the cell is a dependent, and an anchor among them brings its
    stored footprint. A candidate that only a rule area ties to the cell goes to `unresolved`, and so does an
    unresolved candidate that no written area ties to it. So the answer is the same whatever else the index
    files for a formula, and the same by the scan, which applies the same rules.
  - **One pass over the candidates, not one per cell.** Three shapes would otherwise be quadratic with a tiny
    answer. So for `Depth::All`, for a target of more than 1,024 cells and for a batch, the call first files
    the written areas of the formulas it will have to check more than once, in a structure of its own that
    lives for the call (the index's rectangle store; its bytes count toward the call's cap, above): the
    always-dirty list when there is an index, every formula when there is none. Then it runs its walk against
    that structure. A batch is checked through its cells, sorted, and a range target as its rectangle: a target
    is never held cell by cell (`A:XFD` is 2^34 cells). The index's queries are per cell, so a target of more
    than 1,024 cells takes its candidates by the scan, with an index too: for a target that large the scan is as
    cheap as anything. The index's own large rectangles are the remaining cost: with `C(i) = SUM($B$1:B(i))`, a
    walk from each B(j) visits the n-j+1 rectangles that hold it, so `dependents(X1, All)` over
    `B(i) = A(i)*$X$1` is n(n+1)/2 visits for two areas. The budget stops it from n = 23,170 rows on
    (23,170 x 23,171 / 2 = 268,436,035, past 2^28): the call ends in `TooMuchWork`, and so does the second level
    of a level-by-level walk, `dependents_each(B1..Bn, Direct)`. A parameter cell over a column that feeds
    running totals is an ordinary ledger, so this is a limit of the first steps, not of the design: step (d)
    walks the frontier rectangle by rectangle, as it is joined for the answer anyway, with a rectangle query on
    the index, and visits each filed rectangle once per frontier rectangle.
- **Why the candidates are enough.** An indexed formula files every reference and range of its tree (`leaves`
  takes every `ReferenceKind` and `RangeKind`, in every branch), with Design 11.2 the rectangles its names
  resolve to, and with Design 12 a footprint rule's rectangle beside the written range. So every written area
  of an indexed formula is filed, with one exception, `A1#`, which is filed at its anchor, whose readers the
  candidates include. A formula with a written area the index does not file is on the always-dirty list. The
  candidates are a superset, and the check makes them exact. The same holds for `unresolved`: the index models
  no unresolved formula, so each is on the always-dirty list; and a formula with a rule area is among the
  index's readers of every cell of that area. The scan finds both by the same walk and the same rules.
- **Structural edits and maintenance.** Design 10 keeps nothing between calls, so nothing is maintained. A call
  after any edit reads the formulas as they are.
- **Calculation modes.** The written areas are about formulas, not values, and are the same in every mode. Three
  rules read what a recalculation writes: the area of `A1#`, a spill cell's anchor, and an anchor's footprint
  among the dependents. They use the stored footprints, which are the last recalculation's. With J1
  `=SEQUENCE(Z1)` and K1 `=SUM(J1#)` in Manual mode, after Z1 goes from 3 to 5, `precedents(K1)` is J1:J3 until
  `calculate`. The footprints can be stale in Manual and OnDemand modes, after a load without recalculation,
  and after a structural edit there, which leaves the dynamic anchors of its sheet one cell until `calculate`
  (`reset_dynamic_array_spills`). `Trace::stale` says so. A trace does not fail for it in OnDemand mode: it
  reads no value and its answer is marked, for every target that `in_spill` would refuse there.
- **What the implementation may rely on.**
  1. Within the index, `leaves` and `instantiate` are the one definition of a formula's areas at a cell
     (`walk.rs`). The written reading extends them with the area sources above; it does not copy them.
  2. A `DefinedNameKind` node resolves through `Model::parsed_defined_name` with the scope the parser bound. A
     named LAMBDA call resolves elsewhere (`name_key::get_name`, the cell's sheet and then the workbook), and
     the walk uses that resolver for it.
  3. The index, when built, files every written area of every formula it models, `A1#` at its anchor.
  4. The always-dirty list is a list the index keeps (Design 11.1); it holds every unresolved formula, since
     the index models none of them. Before T-5132 lands, a workbook with such a formula has no index, and
     `dependents` scans.
  5. `get_cell_spill` gives an anchor's stored footprint; a blocked anchor stores one cell; a formula cell that
     is no anchor gives itself; and a cell with no formula gives an error, which the evaluation turns into
     `#REF!`.
  6. A structural edit resets the dynamic anchors of its sheet to one cell
     (`Model::reset_dynamic_array_spills`; a Ctrl+Shift+Enter array keeps its range), and the next recalculation
     spills them again.
- **What it must show.**
  - Two oracles that share no code with the walk. The generator writes each formula from a list of areas it
    draws (cells, ranges, whole columns, names, 3-D spans, unions, spill references, an unread IF branch), so
    the list is the expected answer: `precedents(F, Direct)` must equal it, and `dependents(C, Direct)` must be
    exactly the formulas whose list holds C. And the harness's text-based `precedents` agrees on the formulas
    it can read.
  - `dependents` with the index equals `dependents` by the scan, `areas`, `unresolved` and `stale` alike, on the
    metamorphic generators' documents and on every eval suite workbook, spill references among them, and with a
    footprint rule in the document once T-5143 lands (H1 `=SUMIF(A1:A3,">0",B1)`: `dependents(B3)` lists H1 in
    `unresolved` on both paths, `dependents(Z9)` lists it on neither). Mutations: the anchor's readers dropped
    from the candidates; a rule area treated as a written area.
  - `unresolved` is complete. On the metamorphic generators' documents, on every eval suite workbook and on one
    formula per catalog function that takes a reference, a formula F that is not unresolved reads no cell
    outside `precedents(F, All)` and the rule areas of the formulas walked: checked against the engine's
    recorded reads (`sheet_reads.rs`, the candidate oracle Design 1 names), which share no code with the walk.
    Mutation: LOOKUP taken off the list.
  - `Depth::All` is the closure of `Depth::Direct`: asking level by level, in either direction, reaches exactly
    the cells of `All`, through a spill, through a Ctrl+Shift+Enter array and through a blocked anchor.
  - `precedents` and `dependents` are inverse at each depth: C is in `precedents(F)` exactly when F is in
    `dependents(C)`, for F a formula cell, an anchor or a spill cell.
  - `stale`: the Manual-mode example above answers J1:J3 with `stale` set, and J1:J5 without it after
    `calculate`; in the same state `precedents(J5)` and `dependents(J5)` are empty with `stale` set, with the
    index and by the scan, and are Z1 and K1 after `calculate`; a load without recalculation and a structural
    edit in Manual mode set it.
  - Each P case of the probe request as a test, once answered.
  - Bounds. A formula over 65,537 distinct areas gives `TooLarge`; a million dependents in one column give one
    area; the running balance over 40,000 rows gives two precedent areas for `Depth::All`; `TooLarge` is the same
    with the index and by the scan. The name chain `na_1 = na_2+na_2` to `na_63` (names that are no cell
    address), over `na_63 = S!$A$1`, answers S!A1 with an empty `unresolved` in 63 body walks, timed. `dependents`
    by the scan over 100,000 cells `=top`, where `top` sums 1,000 names of 1,000 references each, ends in
    `TooMuchWork`, timed: the bench records what a refusal after 2^28 steps costs, natively and in wasm. A
    target of `A:XFD` answers within the memory cap. A batch of 65,536 cells in a workbook with 10,000
    unresolved formulas returns them once.
  - Benches, each with its step count: `dependents` of an input read by 100,000 formulas, with the index and by
    the scan; the running-total shape above for `dependents(All)` and for `precedents(All)`; a 1M chain in a
    workbook with no index; 100,000 always-dirty SUMIFS beside a 100,000-cell chain; a batch of 10,000 cells.
- **Steps**, each S: (a) the written reading of the walk, `precedents`; (b) `dependents` from the index and by the
  scan, the batch form and the two bounds; (b2) `Trace::stale` and its tests, after Design 9's step (a)
  (T-5114): until then no call sets it; (c) the Python and wasm bindings and the engine worker's `explain`, by
  their lanes; (d) the frontier walked rectangle by rectangle, for the running-total shape.
