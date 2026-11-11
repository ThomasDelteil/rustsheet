# T-5145 section 4: coverage — the always-dirty list, names, unions, 3-D spans, spills (T-5132, T-5133, T-5134)

*poc-design-cc, 2026-11-11. Draft for a design read, before the reviewers. The text under "Proposed spec text" goes
into SPEC-0002 as Design 11. OFFSET and INDIRECT are left to T-5420, as ruled: here they are always dirty.*

## In short

- **Changed at 16:25Z on 2026-11-11, before review.** An independent reviewer read this section first and found
  five blocking points, three of them with a wrong value measured on today's engine (I reran the three
  experiments). All are fixed on the cautious side:
  - A formula that uses a **union** (`=SUM((A1:B5,D1))`) stays "always dirty" instead of being tracked by its
    areas: the engine charges a union by the rows of the sheet, so a cell outside its areas can change its result
    at the work limit (16,577,500 became `#VALUE!` in the experiment).
  - Spilled arrays: one more case in which the engine falls back to a full recalculation (a formula that looked
    at the area where a later array spills, without reading a spilled cell).
  - Inserting or deleting rows or columns in a workbook with a spilled array stays a full recalculation.
  - At a structural edit, formulas that the edit makes trackable are recalculated with their readers.
  One more question for the lead follows from it (7 in the CR's description: a union stays always dirty).


- **Today one unmodelled formula turns incremental recalculation off for the workbook.** On the eval corpus only
  1,479 of 5,082 workbooks have an index, and 36,357 of 50,820 edits are full passes (decision 0422's evidence).
- **11.1, the always-dirty list (T-5132).** Such a formula is re-run at every edit with its readers; the rest of
  the workbook keeps its index. This is Excel's volatile set, widened to "anything we cannot model". The rule and a
  working prototype are poc-engine-cc's. I added one obligation the prototype's base no longer covers: a formula's
  charge must not depend on what ran before it in the pass, with an inventory of the shared pass state and a test
  that fails when a cache is added without a rule.
- **11.2, names, unions, intersections, 3-D spans (T-5133).** The index walks into a name's body as if it were
  written in the formula, so a formula-bodied name works like a range name. Structural edits ask each name what its
  own rewrite did, instead of predicting it.
- **11.3, spills (T-5134).** First slice: a restricted pass is kept only when no dirty anchor's footprint changes.
  Any change of footprint is a full pass. It needs no new table: the cells already hold their footprints.
- **11.4** is a table of what each step buys on the red team's workbooks.

## Two limits to know about

1. **Lookups and conditional aggregates gain nothing from this section.** Their charges follow the order of the
   pass. A workbook that uses a conditional aggregate keeps today's cost until section 5's Design 13 lands; one
   that uses a lookup, until T-5192 (corrected 10:50Z: Design 13 does not cover the lookups' cache).
2. **A workbook whose full pass nests gains nothing either.** Design 13 removes the nesting among modelled
   formulas (a total above its data, an anchor that reads formulas). Nesting through an always-dirty formula stays
   under Design 13 as drafted (corrected 10:50Z; an open question to its authors).

## Open questions

1. Land the always-dirty list first, on poc-engine-cc's prototype, with the pass-state rule and its inventory
   test added? (yes/no; mine: yes.)
2. Names: walk into every formula-bodied name, not only names that are plain ranges? (yes/no; mine: yes; it is
   the same walk, and the task's "names resolved to their references" covers it.)
3. Spills: is "keep the pass only when no footprint changes" acceptable as the first slice? A book whose FILTER
   results change size at most edits stays on full passes until a second slice. (yes/no; mine: yes.)
4. Ctrl+Shift+Enter arrays still leave a workbook with no index after this section. Acceptable for now? (yes/no;
   mine: yes; they are the easy follow-up, since their footprint never changes.)
5. A workbook with a circular reference still recalculates fully at every edit (Taskei P559330849), and this
   section has no design for it. Leave it for later? (yes/no; mine: yes; it needs the loop rule's history
   question settled first, SPEC-0002 non-goals.)
6. A whole-column range (`SUM(A:A)`) becomes always dirty: re-run at every edit, not skipped. Is that enough for
   now, with a precise rule (the used area as its footprint) as a later task? (yes/no; mine: yes.)

## Probes this section waits for

Only V1 (the control: RAND changes after an edit nothing reads). Nothing else here is observable in Excel.

## Proposed spec text

**11. Coverage: formulas the index does not model, names, unions, 3-D spans and spills** (amendment for T-5132,
T-5133 and T-5134, 2026-11-11). OFFSET's and INDIRECT's own rules are not in this design: by the project lead's
ruling they are T-5420, a task that records the ideas. Here they are on the always-dirty list.

- **What it is for.** Today one formula the index cannot model leaves the whole workbook with no index, and every edit
  is a full pass. The red team measured it (Taskei P559330849, P559354172): beside 50,000 indexed formulas, one NOW,
  INDIRECT, OFFSET, defined name, LET, whole-column SUM or SEQUENCE spill turns an edit no formula reads from a skip
  into a full pass of 19 to 23 ms natively, linear in the workbook. On the eval corpus the nightly counts the same
  thing: of 5,082 workbooks compared, 1,479 have an index, and 36,357 of 50,820 edits are full passes (the evidence of
  decision 0422, T-5154). Real workbooks nearly always hold one such formula.
- **Excel.** "Excel reevaluates cells that contain volatile functions, together with all dependents, every time
  that it recalculates" (docs/research/recalc-modes-and-dependency-tracing-2026-11-11.md, S1). That is the
  always-dirty list. The rest of this design is not observable in Excel: it changes which cells are re-run, never
  a value, and its reference is a full recalculation (the Test plan). The probe request `t5145-recalc` has one
  control, V1: a RAND cell changes after an edit nothing reads.
- **Other engines** (the same note). Every engine read has a volatile set that is recalculated at each
  calculation (HyperFormula, LibreOffice, Formualizer, Google Sheets, Univer). None turns its dependency tracking
  off for a workbook that holds one.

**11.1 The always-dirty list** (T-5132). The rule and its first implementation are poc-engine-cc's (branch
`inc/e1/4-partial-index-e2`, read by the read-index lane). This text adds the rule on pass state, which the
conditional aggregates need now that T-5143 is not under it.

- **The rule.** A formula cell whose reads the index does not model no longer refuses the workbook. It joins
  `ReadIndex::always_dirty`, in (sheet, row, column) order, 12 bytes a cell, counted toward T-5153's cap. The index
  files no rectangle and no head for it. Such a formula is one with:
  - a function off the allowlist;
  - a whole-column or whole-row range, or a reference in the last column;
  - a defined name, a 3-D span, a union or an intersection, until 11.2 models them; LET and LAMBDA; a range
    operator; a structured or a spill reference;
  - a tree past `MAX_INDEXED_NODES`, or more operands than `MAX_INDEXED_RANGES`.
- **What still leaves the workbook with no index:** an array formula or a spill cell other than the one-cell
  scalar anchor (until 11.3), a data table, an external book, iterative calculation. A formula whose static result
  may be an array is a dynamic anchor even while it holds one cell, so it is 11.3's: a bare `=INDIRECT(…)` or
  `=LET(…)`, and an OFFSET whose height or width is not the literal 1 (`=OFFSET(Y1,0,0)` is a scalar, and a
  formula cell).
- **The closure** (Design 3) is seeded by the written cells, the moved heads and the list, and walks readers as
  before: a formula that reads an always-dirty cell is filed under that cell like any reader. Past the limit, a
  quarter of the formulas the index knows with the list counted, the recalculation is full. So a workbook with a
  list never takes the skip path (Design 5): an edit nothing reads runs a restricted pass over the list and its
  readers. The prototype's author measured 6.6 us for it beside a 10,000-row ledger (Taskei P559354172).
- **Why it stays exact.**
  1. *It is never clean.* Design 4 asks every clean cell's value and work to be a function of what the index
     tracks. An always-dirty formula is run by every restricted pass, from the new state's inputs.
  2. *Its reads are checked when they happen,* as every dirty cell's are: a read of a clean formula cell after its
     root in pass order is a read ahead, and a read of a dirty cell not yet run nests. Either rejects the pass.
     So a kept pass was flat and read nothing ahead, and Design 4's argument holds as written.
  3. *Its charge must be the one a full pass would make.* This is the new obligation. A formula's charge may
     depend on what ran before it in the pass only through state every pass builds alike. A restricted pass runs
     fewer formulas than a full one, and in it a clean formula cell is not `Evaluated` (`clean_cell_value` returns
     before the cell is entered in `evaluation.cells`), so `Model::read_is_inert` is false for it and a pass cache
     is used less than in a full pass. The state shared within a pass, and what each does:

     | State | Used by | Charge on a hit and a miss | Rule |
     |---|---|---|---|
     | search cache (`search_cache.rs`) | VLOOKUP, HLOOKUP, MATCH, LOOKUP, XMATCH, XLOOKUP, through `range_search_vector` | differs | `PassShape::searches`: a restricted pass that searches is rejected, and after a full pass that searched every restricted pass is declined (Design 4, "A search"). Unchanged |
     | IFS cache (`ifs_cache.rs`) | COUNTIF(S), SUMIF(S), AVERAGEIF(S), MINIFS, MAXIFS, SUMPRODUCT's masked blocks | differs | new with T-5132: a call sets the same flag where it enters the cached path, whatever comes back (`count_ifs_cached`, `run_ifs`'s leading-matches branch, `block_numbers`), whenever `pass_cache_safe` holds and the rectangle passes the range-value cache's size check, as `range_search_vector` sets it before any cache check. A flag set on a hit only is unsound: a restricted pass finds a range of clean formulas not ready and walks where a full pass hits. The same two consequences follow. Before T-5132 none of these ran in a restricted pass |
     | range values (`range_values.rs`), running totals (`sum_prefix.rs`) | operators over ranges; SUM of one range | the same (their tests compare the work) | none |
     | order state (`OrderState`) | WEEKNUM, AMORDEGRC | not a charge: a value | new with T-5132, and a change to the driver: a `PassShape` flag set wherever `order_state_read` is set; a restricted pass is abandoned at the first such read and rejected. Without it `evaluate_passes` re-runs the restricted roots "in Excel's order", which `pass_roots` gives in natural order, and the pass is kept with a value no full recalculation gives. The full pass that follows makes the switch, and `can_restrict` then fails. Before T-5132 no reader of it ran in a restricted pass |
     | the generator of random draws (`entropy.rs`) | RAND, RANDBETWEEN, RANDARRAY | not a charge: a value | see "Volatile draws" |
     | hidden rows, pivot indexes, dynamic links | SUBTOTAL and AGGREGATE; GETPIVOTDATA; HYPERLINK | built outside any formula's charge | none; a re-run HYPERLINK replaces its own link (`run_top` drops the cell's link first) |
     | `seen` and `seen_ranges`, what a root read as empty or occupied | every read | not a charge: a restart, when an anchor writes where an earlier root read (`spill_contradicts_a_read`) | none while no anchor spills; 11.3's fourth rejection from T-5134 on |
     | lambdas, named lambdas, captures and nested budgets; `defined_names_running`; `number_mark` | every formula | per root, per cell run, or not surviving a cell | none |
     | the pass pools (committed and thrown-away work) | every formula | a run gets the least of its own limit and the pool left | T-5172's own-limit guard (Design 4). Unchanged |
     | `evaluation.cells` | every read | a clean cell is not `Evaluated` in a restricted pass | the IFS row above, and 11.3's read of a clean anchor's spill cell |

     So a workbook that uses a conditional aggregate gets no restricted pass until Design 13's indexes make the
     IFS cache charge by the document alone, and a workbook that searches gets none until the search cache does
     (T-5192, which Design 13 does not cover). Each keeps the passes it has today. Its cost must not grow: with a
     list no closure is empty, so the edit path asks `restricted_pass_allowed` before it walks the list's
     closure, and a workbook whose list's closure passes the limit remembers that until its formulas change. Two
     paragraphs of Design 4 are read with this table from T-5132 on: "Why it then stays an upper bound" and "Why
     this is exact" say the conditional aggregates stay off the allowlist. They do, and they now run as
     always-dirty formulas under the IFS row's rule.
- **Volatile draws.** RAND, RANDBETWEEN and RANDARRAY draw anew in every pass, restricted or full, so their cells
  and readers recalculate at every edit, as Excel's volatile functions do (S1 lists RANDBETWEEN and S3 RAND;
  probe V1 asks RAND; RANDARRAY is in neither list and no case asks it, so for it the rule is ours). A bare
  `=RANDARRAY(…)` is a dynamic anchor, so it waits for 11.3. The engine draws from one unseeded generator,
  so two passes never draw the same values: the differentials compare the cells outside the closure of the
  drawing cells, which the index gives (today the harness leaves a drawing workbook out whole). The contract for
  the collaborative profile is a draw that is a function of the seed and the cell (I-07); once T-5288 makes
  the engine draw so, they compare every cell. NOW and
  TODAY do not move: `can_restrict` asks for a fixed clock.
- **The used area.** An always-dirty formula whose work follows the used area files no head and needs none: it is
  in every closure.
- **Structural edits** (Design 8; this rule lands with T-5136, which builds Design 8: today a structural edit is a
  full pass). The lists of both indexes are in D0 of every structural edit Design 8 restricts: the old index's,
  each cell at its place after the edit, and the new index's, built for the formulas after the edit. The new one
  covers a volatile cell, a name the edit shrinks, a moved cell that reads a name's relative part, and `ROW()`.
  The old one covers a formula the edit takes off the list. With A3 `=SUM(B1:XFD2)` and A4 `=A3*2`, a delete
  of column C rewrites A3 to `=SUM(B1:XFC2)`, which the index models: A3 is on the old list alone, it filed no
  rectangle before the edit, and Design 8 runs a rewritten formula without its readers. Without the old list
  A4 keeps its value where a full pass changes it.
- **Maintenance** (Design 2). A written formula the index does not model joins the list, and one that leaves it
  is taken out. Neither makes the index stale. This replaces "a written cell that holds a formula the index does
  not model" in the list of what still makes the index stale, in T-5166's text of Design 2's "Maintenance"
  (Design 7).
- **What it does not give.** A workbook whose full pass nests keeps no restricted pass: an always-dirty cell read
  by a formula placed before it, or reading one placed after it. Design 13 removes the first case for a **leaf**,
  an always-dirty formula that reads no cell (`=TODAY()`, `=ROW()`): its pass runs a leaf before its reader.
  TODAY is in 72 and ROW in 72 of the 1,136 corpus workbooks with formulas (11.4), so that is the common case.
  An always-dirty formula with an unknown or dynamic footprint (INDIRECT, OFFSET, a whole column) is no
  candidate there, and a workbook where one nests still declines. And until per-cell spend is on main
  (T-5167), every restricted pass adds the list's committed work to the summing bound again, so a costly
  always-dirty formula has its pass rejected every floor((L - F) / r) + 1 edits (Design 4, "Per-cell spend").
- **What the implementation may rely on.**
  1. Three places refuse today. `leaves` or `instantiate` gives `None` for a formula the walk does not model.
     `Model::read_index` returns `None` for a tree past the node bound. `indexed_formula` gives `Err` for a cell
     kind (an array formula, a spill cell). The first two become "always dirty"; the third still refuses the
     workbook, and so does the memory cap (`index.add` and `push_counted` failing).
  2. `evaluate_cells` needs no new guard for reads: `clean_cell_value` and the nesting flag see every read.
  3. The closure's limit counts the list, so a workbook that is mostly unmodelled formulas recalculates fully, as
     today.
- **What it must show.**
  - One formula of each kind beside indexed ones: every edit is a restricted pass and equals a full recalculation,
    values and change sets (the prototype's `edit/tests/partial_index.rs`).
  - The metamorphic suite and the eval-corpus differential with these functions in their generators. The prototype's
    commit (3d69da29e) reports restricted passes kept going from 53 to 202 on any histories and from 695 to 1,597 on
    flat ones, every edit equal to a full recalculation.
  - **The pass-state inventory as a test.** It reads from the source every field of the evaluation state
    (`Evaluation`, evaluation.rs), every field of `Model`, and every `thread_local!` and static of the engine
    crate outside its tests, and fails when one has no line in a list beside it that names its row of the table
    above or says why it is no pass state. So a cache added later, in the model or beside it
    (`groupby_filter_spent`, the collation thread-locals, a sheet's lazy column index), cannot join a restricted
    pass unnoticed.
  - The IFS row: `B1:B200` hold `=A1&""`, D1 is 50, and C201 and C202 both hold
    `=COUNTIF(B1:INDEX(B1:B200,D1),"x*")`. The full pass reads 50 cells, under the cache's floor. Write D1 = 100:
    the restricted pass must be rejected, and at C202's own limit the values equal a full recalculation's.
    Mutation: the flag set on a hit only.
  - The order-state row, on a fixed clock: A1 `=WEEKNUM(45000,1)`, A2 `=WEEKNUM(45000,B2)`, B2 = 1. Write
    B2 = 22: A2 must be what a full recalculation gives (`#NUM!`: the chain runs A2 first, on an empty state),
    and the edit a rejected pass. The AMORDEGRC twin: its period cell going from 1 to 0.5. Mutation: the flag
    dropped. A reader written as a formula does not test it: a formula write was a full pass before T-5166.
  - A re-run HYPERLINK keeps one link.
  - The structural fixture above (A3 `=SUM(B1:XFD2)`, A4 `=A3*2`, column C deleted): A4 equals a full pass's.
  - Mutations that must fail the suites: the list left out of the closure; a refused formula filed with a
    partial footprint; the list left out of D0; the old index's list left out of D0.
  - The red team's `rt-index-whole-column`, `-volatile`, `-offset`, `-offset-in-sum`, `-indirect-in-sum` and
    `-let-in-sum`: an unread edit is a restricted pass, and their budgets tightened. `rt-index-name` joins them
    until 11.2 makes it a skip. The bare `rt-index-indirect` and `-let` hold a formula whose static result may
    be an array, so a dynamic anchor: they wait for 11.3.

**11.2 Names, unions, intersections and 3-D spans** (T-5133).

- **The rule.** The walk models four more nodes, and a formula that uses only them is indexed, not always dirty.
  A union stays unmodelled:
  - **A defined name.** The resolver is the evaluation's (`Model::parsed_defined_name`, with the scope the parser
    bound to the node), and what it gives decides:
    - a name that is a reference or a range (`ParsedDefinedName::CellReference`, `RangeReference`) is its
      rectangle where the name is used, by the evaluation's own placing (`NameReference::at`: an absolute part
      as written, a relative part counted from the cell that uses the name);
    - a name whose body is any other formula (`ParsedDefinedName::Formula`) is walked: the walk takes the body's
      references and calls as the reader's own, placed at the reader's cell, through names that use names, to
      `MAX_DEFINED_NAME_DEPTH`. A name past the depth makes its readers always dirty;
    - a body the walk refuses (a function off the allowlist, a whole column) and a LAMBDA
      (`ParsedDefinedName::LambdaDefinition`) make the reader always dirty;
    - a name that is an error literal, or does not parse, reads nothing; a name the resolver does not find
      makes the reader always dirty, since defining it later is a name op, which builds the index again;
    - a name that is a range to the last row or the last column is refused as a written one is
      (`instantiate`), and its reader is always dirty.
  - **An intersection**: both operands' references, a superset of the cells it reads, when each operand is one
    area.
  - **A union stays unmodelled**, and its reader always dirty. A union's areas are walked over the sheet's
    existing rows with charged steps, a step for each existing row even with no cell in the area's columns
    (`existing_walk.rs`), and past `MAX_AREA_WALK` the argument is `#VALUE!`. So a write in no rectangle of the
    union can change its reader's value: with B1:R1950 filled and AA1 `=SUM((B1:R2400,T1), …)` 500 times over,
    one more literal in column Z turns 16,577,500 into `#VALUE!` (measured on main). The same holds for an
    intersection with a union as an operand and for a name whose body holds a union. A union joins when its
    walk's steps are a function of its areas' own cells.
  - **A 3-D span**: its rectangle on every sheet from the first to the last, in tab order.
  - A range operator between two written references on one sheet: their bounding rectangle. With any other
    operand it stays unmodelled.
- **The walk into names is bounded** (directive 5). Depth alone does not bound it: with `n1 = n0+n0` up to `n40`
  and a cell `=n40`, a walk that enters a body at every use enters one 2^40 times, where the evaluation gives
  `#NAME?` after `MAX_DEFINED_NAME_EVALUATIONS` bodies. So each name's leaves are resolved once per build and
  kept by its key: its absolute parts as rectangles, its relative parts as nodes to place at each reader, with
  the body's calls and whether an `@` in it picks by the reader's row (`positional`). The reference and range
  nodes of a formula and of its names count together toward `MAX_INDEXED_RANGES`, and the bodies' nodes toward
  `MAX_INDEXED_NODES`, which counts only the cell's own today. Kept leaves count toward T-5153's cap as they are
  kept, at most `MAX_INDEXED_RANGES` a name (past it the name is kept as refused, in constant space); past the cap
  the workbook has no index.
- **Why it is exact.** A name's body is evaluated at the cell that uses it, under bounds that are functions of the
  formula and of the names (`MAX_DEFINED_NAME_DEPTH`, `MAX_DEFINED_NAME_EVALUATIONS`) and, at depth 0 where a flat
  pass runs every formula, of the stack limit, whose setter already forgets the work bound (Design 4). So the
  reader's value and work are functions of the cells its own references and its names' references name: the
  rectangles the walk files. For the other three nodes the rectangles filed hold every cell the node can read,
  and none of them is walked by the sheet's rows as a union is.
- **The used area and the searches.** The head walk (Design 1, "The used area") and `searched_spans` run over the
  references and calls the walk found, names entered. A tailed argument or a lookup's table given through a name
  is a head as a written one is. T-5184's hook test is what shows it: its generator gains these nodes.
- **Lifetime.** A name op is an undo entry that is not a cell write, and a sheet op changes a span: both make the
  index stale already (Design 2), so names are resolved again and spans placed again at the next build.
- **Maintenance.** A refile resolves names as the build does, and keeps the lists of each name's readers and of
  each span's readers in step: a refiled formula leaves the lists it was on and joins those of the names and
  spans it now uses, as it does for the position readers (Design 2).
- **Structural edits** (Design 8; with T-5136, as 11.1's rule). SPEC-0039's rules: a name shifts and grows as a
  reference does (E1), its
  relative parts stay where they are (E5), and it is rewritten by its text, not on the tree; a 3-D span never
  shifts (E4).
  - *An intersection's operands* are written ranges. Design 8's classification applies to them as it stands.
  - *A name.* The index lists each name's readers (12 bytes a reader, with the name's key; counted toward the cap)
    and keeps the rectangles its absolute parts resolve to. After the edit each name with readers is resolved
    again. A reader through a name, directly or through another name, is in D0 when:
    - one of the name's absolute rectangles is not the old one moved by `RefMap::line` with as many lines (the
      name shrank, grew, lost a line or became `#REF!`); or
    - the name has a relative part, and the edit is on the reader's own sheet or on the sheet that part reads:
      the part stays where it is (E5), so a reader that moved, or one whose target moved, reads another cell.

    This asks the name's own rewrite what it did, so it does not depend on the text rewrite following
    `RefMap::span`.
  - *A 3-D span.* The index lists each span's readers by spanned sheet (12 bytes per reader and sheet). Every
    reader of a span that holds the edited sheet is in D0: the span keeps its text, so after the edit it names
    other cells.
- **What the implementation may rely on.**
  1. `ParsedDefinedName::Formula` holds a formula body's tree, which the walk reads as it reads a cell's, and
     `NameReference::at` places a reference name at a cell: the red team's `rt-index-name` uses one (`Corner`).
  2. Design 8 builds the index for the formulas after the edit before it runs its pass; the names are resolved
     again in that build.
  3. E1, E4 and E5 are evidenced (references.md; t4702-names-*, t4747-*, t4760-*, t4272-3d-span-*).
- **What it must show.**
  - The conservativeness oracle by perturbation, with absolute, relative, formula-bodied and nested names, unions,
    intersections and 3-D spans.
  - The metamorphic generators gain them, and the structural suites gain names and spans under inserts and deletes
    on each sheet a span holds.
  - Mutations: a changed name's readers left out of D0; the readers of a name with a relative part left out; a
    span's readers left out; a head through a name not registered.
  - The exponential family above builds within the bound, timed, and a name past the depth leaves its reader
    always dirty.
  - The red team's `rt-index-name`: an unread edit is skipped.

**11.3 Spills** (T-5134). The first slice keeps a restricted pass only when no dirty anchor's footprint changes.

- **The rule.**
  - **A dynamic anchor** with a modelled formula is filed as a reader of its formula's reads, as a formula cell
    is. With an unmodelled formula it is on the always-dirty list. FILTER, SORT, UNIQUE, SEQUENCE and their kin are
    off the allowlist today, so their anchors are always dirty; each joins the allowlist with a test of its
    footprint, as any function does (Design 1), outside this task.
  - **A spill cell** is an output of its anchor. It files nothing. The anchor's **footprint** is what the last
    pass left in the anchor cell: its extent `r`, whether its value is `#SPILL!` (a blocked anchor stores one
    cell and that error), and its circular mark. Each spill cell's `a` names its anchor.
  - **A spill reference `A1#`** is a read of the anchor cell.
  - **A blocked anchor** (`#SPILL!`) is seeded into every closure, as an always-dirty cell is. The model records
    the blocked anchors at the end of every full pass, beside the anchors' ranks; a kept restricted pass changes
    no footprint, so the record holds.
  - **The closure.** When an anchor is in the closure, the other cells of its footprint are taken as written, and
    their readers join it. The footprint's cells count toward the closure's limit as written cells do.
  - **A write over an anchor or a spill cell** is not a write of literals (`writes_only_literals` already asks for
    cells outside any spill), so the edit recalculates fully and the index is built again (Design 2).
- **The pass.** Dirty anchors run first, in the order the last full pass left (`Incremental::anchor_rank`), and
  write their spill cells in place. A clean anchor's spill cells are read as stored values. This is new code in
  the driver: today a read of a spill cell goes to `evaluate_spill_cell`, never to `clean_cell_value`, and it
  raises a restart for any anchor that has not run in the pass, which every clean anchor has not. So in a
  restricted pass, for an anchor outside it, `evaluate_spill_cell` returns the stored value, with what
  `clean_cell_value` does for the anchor's own cell: `read_evaluated`, `mark_readers_of` and the read-ahead
  test by the anchor's rank. A restricted recalculation also makes one attempt only: where a full one would
  run its pass again (share mode, Excel's order, a restart), it ends as rejected, since nothing is restored
  between attempts when the spill scans are skipped. Beyond Design 4's cases the pass is rejected, and a full
  pass follows, when:
  - a dirty anchor would end with another footprint than it began with: another extent, a `#SPILL!` gained or
    lost, a circular mark. The test runs where the spill is written (`spill_dynamic_array`, and the scalar
    path's `retire_own_spill_cells`), **before** anything is written: the pass is flagged and abandoned there;
  - the pass asks for a restart. Only a full recalculation's log may learn from one, so a restricted pass ends
    as rejected before `RestartLog::record` reorders `anchor_order` (today `evaluate_passes` handles a restart
    in its own loop, whatever the pass);
  - a spill cell of a dirty anchor is read before that anchor has run;
  - a dirty anchor records as read empty (`record_seen_empty_range`) a rectangle that meets the footprint of an
    anchor ranked after it, clean or dirty: a full pass restarts there (`spill_contradicts_a_read` tests every
    such record against the cells the later anchor writes), and a clean anchor does not run here to ask for it.
    The first slice may reject on any such record made for a dirty anchor that is not last in rank.

  On rejection `evaluate_cells` puts back the closure's cells and the cells of the dirty anchors' footprints,
  saved before the pass. Because the footprint test runs before a spill is written, a rejected pass has written
  nothing outside those cells: with `A1 = SEQUENCE(Z1)` and Z1 going from 2 to 3, A3 is never written. The facade
  captures the same cells with the closure (Design 6), so the full pass that follows starts from the values
  before the edit, and the anchors' order is the one the last full recalculation left.
- **Why it is exact.** In a kept pass no footprint changed, so every position holds a cell of the same kind as
  before, and the used area did not move. A reader of a spill cell is clean only if the cell's anchor is clean,
  whose value is unchanged. A full pass of the new state runs the anchors first in the same order. It meets no
  restart: a restart needs an anchor to write where a formula read before it. The footprints are those of the
  last full recalculation's final pass; a dirty anchor that reads a spill cell of an anchor ranked after it has
  been rejected by the second or the third test above; and one that recorded as empty a rectangle over a later
  anchor's footprint, with no spill cell read, by the fourth. With A1 `=SUMIFS(INDEX(E1:F10,0,Z1),D1:D10,G1:G2)`
  ranked before E1 `=H1:H10*2`, a write of Z1 from 2 to 1 makes A1's walk record E1:E10 while no criterion
  matches, so no spill cell is read: a full pass restarts twice there and throws 394 units away (measured on
  main), which the fourth test catches. So it is one pass, flat as the restricted one was, with the same values,
  and Design 4's bound holds.
- **What it does not give.** A full pass runs anchors first, so an anchor that reads a formula cell evaluates it
  inside itself: the pass is not flat, and restricted passes are declined (Design 1; T-5297). Until Design 13
  evaluates an anchor's precedents before it, this slice helps the workbooks whose anchors read literals and
  other anchors' spills. A footprint that changes is a full pass.
- **Structural edits** (Design 8; with T-5136). Unchanged: Design 8's "the workbook holds no dynamic anchor"
  stands, and now also covers spilled and blocked anchors. An insert or a delete resets the dynamic anchors of
  the edited sheet (`reset_dynamic_array_spills`) and a full pass re-spills them; the ranks are keyed by place,
  and `sync_anchor_order` ranks a moved anchor last; and an anchor on another sheet that the edit rewrites has
  its spill cleared. So a structural edit of a workbook with a dynamic anchor runs a full pass.
- **Maintenance.** Unchanged: a written cell that was or became an array formula or a spill cell makes the index
  stale, and the edit recalculates fully.
- **Out of this slice:** Ctrl+Shift+Enter arrays, which still leave the workbook with no index (their footprint
  never changes, so they can follow as anchors that need no footprint test); a closure that grows with a changed
  footprint instead of rejecting.
- **What the implementation may rely on.**
  1. `Cell::ArrayFormula::r` and `Cell::SpillCell::a` are the footprint; no table is added for spilled anchors.
  2. `Model::pass_anchors` already orders a restricted pass's dirty anchors by rank, and `clean_cell_value`
     already compares ranks for a read of an anchor's own cell. A read of a spill cell does not reach it today
     ("The pass", above).
  3. The spill scans a full pass makes (`dynamic_spills`, `sync_anchor_order`) stay skipped in a restricted pass:
     a kept pass made and removed no anchor and changed no footprint. `sync_anchor_order`'s debug assertion that
     every anchor is one cell goes.
- **What it must show.**
  - The metamorphic generators gain anchors with modelled formulas (`=A1:A3*2`, `=IF(A1:A3>0,B1:B3,0)`), anchors
    with unmodelled ones, readers of spill cells and of `A1#`, blockers written and cleared.
  - Kept: a value change with the same footprint. Rejected and equal to a full recalculation: a footprint that
    grows, shrinks, becomes `#SPILL!`, stops being `#SPILL!`; a dirty anchor that reads a later anchor's spill;
    and the SUMIFS-over-INDEX fixture above, at a pass allowance of 352 to 393 units, where a full
    recalculation ends in share mode with `#VALUE!` and a kept restricted pass would hold 0.
  - `E1 =A1:A3*2` and `G2 =E2+B1`: a write of B1 is kept (G2 reads a clean anchor's spill cell).
  - Mutations: the footprint test dropped, or run after the write; a footprint's readers left out of the closure;
    the blocked anchors not seeded; the put-back without the footprints; the capture without them; a restart
    recorded in a restricted pass; a clean anchor's spill cell not read as stored; the rectangle test (the
    fourth rejection) dropped.
  - The red team's `rt-index-spill`, `rt-index-indirect` and `rt-index-let`: an unread edit is a restricted pass.

**11.4 What each step buys.** The red team's workbooks (50,000 indexed formulas and one more formula), an edit no
formula reads, by what is on main:

| The extra formula | Today | After 11.1 | After 11.2 | After 11.3 |
|---|---|---|---|---|
| none (control) | skipped | skipped | skipped | skipped |
| NOW, RAND; `=OFFSET(Y1,0,0)`; `SUM(OFFSET(…))`, `SUM(INDIRECT(…))`, `SUM(LET(…))`; a whole-column SUM | full pass | restricted: that cell and its readers | the same | the same |
| a defined name, a 3-D span | full pass | restricted: that cell and its readers | skipped | skipped |
| a union | full pass | restricted: that cell and its readers | the same | the same, until its walk charges by its areas' cells |
| a SEQUENCE spill | full pass | full pass | full pass | restricted: the anchor and its spill's readers |
| a SUMIFS | full pass | full pass | the same | the same, until Design 13's indexes and Design 12 |
| a VLOOKUP (indexed today) | skipped, and no restricted pass | the same | the same | the same, until T-5192 |
| a bare `=INDIRECT(…)` or `=LET(…)`, whose result may be an array | full pass | full pass | full pass | restricted: a dynamic anchor, always dirty |
| a two-cell cycle | full pass | full pass | full pass | full pass (`can_restrict`; Taskei P559330849) |

A workbook whose full pass nests gets none of the restricted cells of this table. Design 13 removes the nesting
among modelled formulas (a total above its data, an anchor that reads formulas) and through a leaf (a `TODAY()`
cell read from above). Nesting through any other always-dirty formula stays (11.1, "What it does not give").

**How common each form is,** so the steps are ordered by the workbooks they reach.
- Of the 1,136 SpreadsheetBench workbooks with formulas (eval/catalog/usage.tsv), INDEX is in 149, MATCH in 125,
  VLOOKUP in 116, COUNTIF in 95, SUMPRODUCT in 81, TODAY in 72, ROW in 72, SUMIFS in 63, COUNTIFS in 53, SUMIF in
  47, UNIQUE in 25, INDIRECT in 11, SORT in 10, FILTER in 8, RAND in 6, OFFSET in 5, LET in 5 and NOW in 3.
- Of the corpus census's 1,013 workbooks with formulas (docs/research/corpus-ontology-census-2026-11-09.md), 178
  hold a dynamic array formula, 96 a whole-column or whole-row reference, 45 a Ctrl+Shift+Enter array, 37 a
  structured reference, 30 an external reference, 23 a data table, 12 a spill reference and 4 a 3-D reference;
  275 workbooks define a name, and 79 turn iterative calculation on.

So the lookups (T-5192) and the conditional aggregates (Designs 12 and 13) reach the most workbooks, then the
always-dirty list for TODAY and ROW (11.1), dynamic arrays (11.3), whole-column references and names (11.2). OFFSET
reaches the fewest, which is the project lead's ruling on its priority.
