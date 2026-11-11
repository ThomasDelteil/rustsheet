# T-5145 section 2: calculation modes and `watch` (T-5114) — draft for the project lead

*poc-design-cc, 2026-11-11. Draft for a design read, before the reviewers. The text under "Proposed spec text" goes
into SPEC-0002 as Design 9. Rules marked (probe Mn) are provisional until the probe request answers them.*

## In short

- **One mechanism, three modes.** Every edit marks what it can change (Excel does this in every mode). The mark is
  the *owed set*. Automatic pays it at once, which is today's behaviour bit for bit. Manual pays on `calculate()`
  (F9). OnDemand pays what a read or a watched range needs.
- **One new path in the evaluator.** A deferred recalculation is a restricted pass (SPEC-0002 Design 4) whose
  edit is the union of the deferred transactions; all of Design 4's guards apply unchanged. The two commands that
  calculate less than the workbook share one new path, the *scoped run* (below).
- **Two slices of OnDemand.** First: a read pays everything owed, so N writes and one read cost one recalculation.
  Second, after the dependency-first traversal exists: a read pays only its cone.
- **`watch`** names ranges an OnDemand workbook keeps fresh at every edit (the viewport, chart ranges).
- **The collaborative sheet stays Automatic as a document** (contract I-02). Manual and OnDemand are meant to be a
  replica's own pace: it lags, it does not diverge. Corrected at about 11:00Z after the engine lane's review: that is
  true only when a settled workbook's values are a function of the document, which needs loops evaluated as on a
  fresh open, seeded draws and a fixed clock. The wasm engine has none of the three yet (T-5288), so it refuses
  the two modes until it does.
- **Corrected twice on 2026-11-11 (about 12:05Z and 12:33Z), after the two review rounds of the first change
  (CR-321684914).** The reviewer found a Microsoft page my research had missed ("Excel performance: Improving
  calculation performance"). It states what I had called undocumented:
  - a formula written in Manual mode **is evaluated at once**, from the stored values of the cells it reads, and
    stays owed until `calculate` (my first draft said: not evaluated until `calculate`). The design follows it.
  - On a **structural edit in Manual mode** the pages pull both ways: two lists name the insert of a row among the
    triggers of a recalculation without "in automatic mode", and two sentences say Manual mode recalculates only on
    request. For forty minutes this section said "paid at once"; the second round showed that this gave one side
    as the documented one. The rule is again the first draft's: it owes everything, and nothing is calculated
    until `calculate`. The probes ask Excel (M12, and M15 and M16 for a name added and a row hidden).
  A provisional rule is now what Microsoft documents where its pages agree, and says both sides where they do not.
- **Changed at 15:05Z on 2026-11-11, after round 1 of this section's own review (CR-321724568).** The reviewer
  showed four holes, each with a reproducer, and all four are fixed:
  - The entry evaluation ran only when the engine's incremental machinery was available (an index, a fixed clock,
    a small closure). On the default clock an ordinary `=A1+2` typed in Manual mode would have shown nothing until
    F9. It is now part of the mode: **every formula written in Manual mode is evaluated at once**, whatever the
    optimizations are doing, array and spilling formulas included, and the test reference does the same.
  - Calculate Sheet (Shift+F9) fell back to calculating the whole workbook when the incremental pass was refused.
    It now **never calculates another sheet**. In the first slice it runs every formula on the sheet and leaves
    the status at "Calculate" until F9; running only the dirty cells and clearing them is a later step.
  - A workbook **loaded without recalculation** was treated as calculated, so OnDemand could hand out the file's
    stale cached values. Such a load now owes a full calculation.
  - A **watch** registered over a range that was already stale was not refreshed by the next edit. A watch is now
    pending while its range is stale, however it got there.
  Both commands that calculate less than the workbook now share one path, the *scoped run*: a set of formula
  cells that evaluate each other and read every other formula cell as stored. The engine lane read it before it
  was sent and agreed.
- **Changed again at 15:31Z, after round 2.** The four holes above are closed. The reviewer found two more in
  the scoped run itself, both where I had described only the ordinary case: a circular reference inside the run
  must follow the workbook's setting (iterate when iterative calculation is on, keep the old values when it is
  off), and the few functions whose value depends on Excel's calculation order (WEEKNUM's and AMORDEGRC's
  quirks) must run in that order inside the run. The scoped run is now defined as the evaluator's own full
  recalculation with its roots confined to the chosen cells, and its test compares it with a full recalculation
  of a copy of the workbook in which every other cell is frozen at its stored value. One thing is left out by
  name: a scoped run does not refresh a data table (what-if table); F9 does.
- **Reviewed by the engine lane** (5 blockers, all folded in): a `calculate` with nothing owed still runs a full
  pass when the stored values are not a normal full pass's; Automatic keeps exactly today's passes, and no read
  or save fails there; a host gets a read of the stored value for its before-image.

## The one limit to know about

With cones, a value read before the workbook is settled is the settled value **unless the settling recalculation
leaves normal mode** (the work budget of SPEC-0028: share mode, or the ceiling). A cone pass cannot know the work of
the cells it did not run. The engine reports it (`last_recalc_outcome`), and the change set of the settling
recalculation lists the corrected cells. No other engine has this limit, because none has a work budget.

## Open questions

1. In OnDemand mode a plain read of a stale cell **fails** (`RefError::NotCalculated`) instead of returning the
   stale value or computing behind `&self`. The caller runs `compute` first; a binding does both. Agree? (yes/no;
   mine: yes. HyperFormula refuses reads inside a suspended batch the same way. The model, UI and engine lanes
   all answered yes; the engine lane's condition, a separate read of the stored value for hosts, is in.)
2. The facade **does not adopt the file's** `calcMode="manual"`; it keeps it for the round trip and exposes it
   (`file_calc_mode()`), and a UI may apply it. Agree? (yes/no; mine: yes. On the desktop the mode is the
   application's, on the web the session's, and contract I-02 makes it a per-user option.)
3. A **save while cells are owed fails** (`SaveError::NotCalculated`) rather than writing stale cached values,
   in Manual and OnDemand modes only. Agree? (yes/no; mine: yes for now. Excel recalculates before a save by
   default in Manual mode; the option that keeps stale values waits for probe M9. In Automatic a save never
   fails for this: the engine lane showed it would break saves that work today.)
4. A **structural edit in Manual or OnDemand mode owes a full recalculation**, and nothing is calculated until
   `calculate`. For Manual mode Microsoft's pages pull both ways (above), and probe M12 asks Excel. Acceptable?
   (yes/no; mine: yes.)
5. `AutomaticExceptTables` is **not** in this task: data tables have no index yet, and the file's `autoNoTable`
   round-trips as text. Agree? (yes/no; mine: yes.)
6. Is the cone slice (c) worth building before the collaborative sheet asks for it, given the limit above? (yes/no;
   mine: build (a) and (b) now, (c) when the dependency-first pass has landed and a viewport measurement asks for
   it.)

7. A formula written in Manual mode is **evaluated at once**, as Microsoft documents, whatever the engine's
   optimizations are doing. Formulas written together evaluate each other; every other formula cell is read as
   stored (probe M2 asks Excel whether it calculates a dirty precedent first). Acceptable? (yes/no; mine: yes.)
8. **Calculate Sheet** runs every formula on the sheet and releases nothing in the first slice, so after it the
   status still says "Calculate" where Excel's may not (Microsoft: Shift+F9 "resets all previously uncalculated
   (dirty) cells as calculated"; probe M14). The values on the sheet are the same, and nothing off the sheet is
   calculated. Acceptable for the first slice? (yes/no; mine: yes.)
9. A workbook **loaded without recalculation owes a full calculation**: in Manual mode the status says
   "Calculate" and a save needs a calculation first; in OnDemand mode the first read computes; Automatic is as
   today. Agree? (yes/no; mine: yes.)

## Probes this section waits for

`/mail/oracle/t5145-recalc-probe/` (README.md, cases.js, three workbooks): M1 to M4, M6, M8 to M16 and V1. None blocks
slice (a): each provisional rule is one hook.

## Proposed spec text

**9. Calculation modes and `watch`** (amendment for T-5114, 2026-11-11; the rules marked *(probe Mn)* are
provisional until the probe request `t5145-recalc` answers them. A provisional rule is what Microsoft documents
where it documents anything; where this design departs from that, it gives the reason).

- **What it is for.**
  - A host that batches. A what-if run writes inputs many times and reads a few outputs. It should pay one
    recalculation per read, not one per write.
  - The collaborative sheet. A client paints a viewport and a few chart ranges. When a remote edit's closure is
    large, what is on screen should be computed first and the rest after (the engine protocol's `Priority`, I-06, and
    the value service's `pending` areas, I-17, in the collaborative sheet's interface plan; this repository's
    `crates/rustsheet-contracts` holds I-01 and I-07 so far).
  - Excel's Manual mode. A user of a slow workbook edits freely and presses F9. The file keeps the mode
    (`calcPr@calcMode`).
- **Excel.**
  - Documented (docs/research/recalc-modes-and-dependency-tracing-2026-11-11.md; S1 is Microsoft's "Excel
    Recalculation", S3 its "Excel performance: Improving calculation performance", the numbers are that note's
    sections). There are three modes: Automatic, Automatic Except Tables and Manual.
    Cells are marked dirty when data or formulas are entered, in every mode: "This recalculation occurs
    immediately after Excel finishes marking cells as dirty if the recalculation mode is automatic; otherwise, it
    occurs later" (S1). F9 "recalculates all cells that Excel has marked as dirty, that is, dependents of volatile
    or changed data"; Ctrl+Alt+F9 "recalculates all cells in all open workbooks" (S1). The state between an edit
    and its recalculation has a name, `xlPending`: "Changes that trigger calculation have been made, but a
    recalculation has not yet been performed" (3), and in Manual mode the status bar shows "Calculate" (S3).
  - Two things happen in Manual mode without F9 (S3):
    - the formula that is entered or edited: "A formula or part of a formula is immediately evaluated
      (calculated), even in manual calculation mode, when you do one of the following:" "Enter or edit the
      formula.";
    - a save: Manual mode "recalculates all open workbooks only when you request it by pressing F9 or
      Ctrl+Alt+F9, or when you save a workbook", the last while "Recalculate workbook before saving" is on,
      which choosing Manual selects (3).

    Shift+F9 "resolves dependencies for the worksheet and resets all previously uncalculated (dirty) cells as
    calculated" (S3).
  - On a structural edit the pages pull both ways. S1 and S3 both list the insert or delete of a row or column
    among the actions that trigger a recalculation, without the words "in automatic mode" that other items of
    the same lists carry. But S1 says of its list "When calculation is set to automatic, recalculation occurs
    after every data input and after certain events such as the examples given in the previous section", S3
    says Manual mode recalculates "only when you request it", and the two lists disagree on which items carry
    the words. This design takes the two sentences over the missing words *(probe M12; M15 and M16 ask the
    same of a defined name added and a row hidden)*.
  - The mode is not the document's. On the desktop it is the application's: "The first document that is opened uses
    the calculation mode with which it was last saved. Documents that are opened later use the same mode", and a save
    writes the current mode into the file (3). On the web it "will affect the current workbook only" (3), and
    Office.js says the setting "is not persisted in the workbook" (4). In the file, `calcPr@calcMode` is `auto`,
    `autoNoTable` or `manual`, and ECMA-376 says of `fullCalcOnLoad`: "If manual <calcMode> is true, then a full
    recalculation will not be performed on load, even when this attribute is set" (7). "When you select Manual, Excel
    automatically selects the Recalculate workbook before saving check box" (3).
  - Documentation is not evidence, so the probe request `t5145-recalc` asks Excel each of these (the M cases).
    Documented and requested for probing: an entered or edited formula is evaluated at once (M3, M4); F9 runs
    a volatile cell with nothing dirty (M6); what a save and a load of a Manual workbook do (M9, M10); what
    Shift+F9 leaves dirty (M14). Not settled by the pages, and requested for probing: whether a row inserted, a
    name added or a row hidden in Manual mode recalculates (M12, M15, M16). Not documented, and requested for
    probing: what a read of a dirty cell returns (M1); whether a formula entered over a dirty precedent reads
    its stored value or calculates it first (M2); whether a pasted formula counts as entered (M13); a switch
    back to Automatic (M11).
  - OnDemand and `watch` are not Excel's. They are this engine's, and their reference is a full recalculation, as
    for every other path of this spec.
- **Other engines** (the same note).
  - HyperFormula has a bracket, not a mode: between `suspendEvaluation` and `resumeEvaluation` "multiple CRUD
    operations can be done without triggering recalculation after every operation", but "You can't batch read
    operations." (1). We take the second half: a lazy mode must not hand out a stale value unnoticed.
  - LibreOffice has one toggle, AutoCalculate. With it off, F9 "recalculates all (so-called dirty) formula cells
    that depend on changed cell content and their dependents, and formula cells containing volatile functions"
    (2), and Recalculate Hard recalculates everything. This is Excel's Manual, F9 and Ctrl+Alt+F9.
  - Apache POI and Formualizer evaluate on request: POI "keeps a cache of all previously calculated intermediate
    cell values" that the caller must invalidate (8); Formualizer has a "demand-driven `evaluate_cell()`" (9).
    That is OnDemand. We avoid POI's contract: here the engine marks what is owed, not the caller.
  - Google Sheets documents an Automatic and a Manual mode (11).
  - None of them has a work budget, so none meets the limit this design states under "Why it is exact", item 3.
- **What is owed.** Beside the work bound, the model keeps what the stored values still owe (`Incremental::owed`):
  nothing; a set of formula cells, closed under the index's readers; or everything. The workbook is **settled**
  when nothing is owed. The engine already has one such fact, the full evaluation a reparsing op leaves owed while
  evaluation is held (`Hold::Held { owed }`, `evaluation_hold.rs`): the owed set takes it over, as "everything", so
  that there is one concept.
  - A transaction adds what Design 3 computes for it: the closure of its written cells and of the last used rows
    moved since the last pass, with its written formula cells (Design 2, "Maintenance"). The marking happens when
    the transaction is applied, in every mode, as Excel marks. Only the evaluation waits.
  - It becomes everything when the closure passes Design 3's limit, when the union of the owed cells passes that
    same limit (a quarter of the formulas: a full pass costs no more there), when the workbook has no index,
    when `can_restrict` does not hold, when incremental recalculation is off, when the transaction is empty
    (callers use one to force a recalculation, Design 5), and when the transaction holds an op that is not a
    cell write. Until T-5166's maintenance is on main, a transaction that writes a formula makes the index
    stale, so it owes everything too, and no index is built at a transaction.
  - A structural edit is the one op that differs by mode. In Automatic mode a transaction Design 8 restricts is
    paid at once by Design 8's pass (once T-5136 has built it; today it is a full pass). In Manual and OnDemand
    modes it owes everything: the owed set and the recorded last used rows would have to be mapped through the
    edit's `RefMap`, and this design does not do that. Nothing is calculated until `calculate` *(probe M12: if
    Excel recalculates at an insert in Manual mode, the transaction ends by paying everything it owes, as an
    Automatic one does, and the rule changes in this one place)*.
  - A load that does not recalculate leaves its cached formula values unverified and owes a full calculation.
    Automatic and Manual retain their specified stored-value read behavior, but OnDemand treats those caches as
    stale: `compute` must run a full pass before exposing them as calculated. A load that recalculates is settled
    only if that calculation was not cancelled. Imported cached values alone do not establish a work bound or
    freshness. In Automatic mode nothing reads the owed set between edits, so today's behaviour stands: a read
    returns the file's cached value, a save succeeds, and the first edit recalculates fully.
  - A setter that forgets the work bound (`set_iteration`, a lowered ceiling, the depth and stack limits,
    `set_clock`) and `load_stored` make everything owed: the stored values are no longer a recalculation's under
    the new setting or cells. In Automatic mode nothing reads the owed set between edits, so today's behaviour
    stands: the setting applies from the next recalculation, and a save in between succeeds.
  - "Everything" means every cell, not every formula cell: with `=SEQUENCE(B1)` owed, an empty cell below it can
    be stale. While a dynamic anchor of more than one possible cell is owed (Design 11.3), every cell of its
    sheet is stale for a read, since its spill area is not known until it runs.
  - The set lives in the model and is bounded by its own rule, the quarter above, at 12 bytes a cell. Who sums
    the model's incremental structures with the index's toward T-5153's cap is still T-5167's question (Design
    1, "The cap"); this design does not settle it.
- **Paying.** To pay is to recalculate owed cells, by the paths an Automatic edit takes today:
  - nothing owed, and `can_restrict` holds: the always-dirty list and its readers run (Design 11.1), as F9 runs
    the volatile cells, or nothing when the list is empty;
  - nothing owed, and `can_restrict` does not hold: a full pass. This is today's gate (`edit.rs`: a skip needs
    `can_restrict`). It is what recovers a workbook whose last recalculation shared its work out or ran on the
    system clock, or whose limits were changed since;
  - a set of cells owed: `Model::edit_closure` is asked once more with no written cell, for a last used row
    moved since the last transaction; then `Model::evaluate_cells` runs the cells, sorted by (sheet, row,
    column) as a closure is, under Design 4, and a declined or rejected pass is followed by a full pass;
  - everything owed: a full pass.

  Afterwards nothing is owed, with one exception: a recalculation a host cancelled (`RecalcOutcome::cancelled`,
  SPEC-0028 section 9) leaves everything owed, since its values are no recalculation's.
- **The modes** (`CalcMode`, on the facade and in `LoadOptions`; Automatic by default).
  - **Automatic.** `Workbook::edit` ends by paying everything it owes. The same passes run as today, with the
    same values, outcomes and edit paths: nothing is owed when `edit` returns, an empty transaction still runs a
    full pass, and no read and no save ever fails for being owed (the two errors below are Manual's and
    OnDemand's). Between edits something can be owed, after a setter or a `calculate_sheet`: it is paid by the
    next edit, as today after a setter.
  - **Manual.** `edit` marks. It pays nothing, with one exception: it evaluates the formulas it wrote (the
    entry evaluation, below). `Workbook::calculate` pays everything (F9). `Workbook::recalculate` stays a full
    pass (Ctrl+Alt+F9), and ends settled: the edits it makes itself (it refreshes `refreshOnLoad` pivots
    through `edit`) pay, whatever the mode. `Workbook::set_calc_mode(Automatic)` pays everything *(probe M11)*.
  - **The scoped run.** Two commands calculate less than the workbook: the entry evaluation and Calculate Sheet
    (the next two items). Each runs a set R of formula cells. The run is the evaluator's own driver
    (`Model::evaluate`: its passes and their restarts, the pass in Excel's order, the iteration sweeps, the
    limits) with its roots confined to R, by these rules.
    - The run uses the evaluator's calculation-order rules with its roots confined to R. Ordinary evaluation
      runs the dynamic anchors of R first and the remaining roots in (sheet, row, column) order, with the usual
      nested evaluation. When order-state readers require SPEC-0017's Excel-order driver, the scoped run uses
      that driver's ordering, dependency-driven rescheduling and order-state restoration within R instead. Any
      order-mode restart remains within R; a stored read outside R neither evaluates nor schedules that cell.
    - A cycle inside R uses the workbook's configured circular-reference semantics. With iterative calculation
      disabled, the cycle and its in-scope readers retain their pre-run stored values as described in recalc.md.
      With iterative calculation enabled, the run discovers and sweeps the in-scope cycle members and their
      in-scope readers using the configured iteration count, convergence threshold and SPEC-0028 limits. Reads
      outside R remain stored reads, and no sweep evaluates or writes a formula cell outside R.
    - A read of a formula cell outside R takes the value stored at that moment and evaluates nothing, whether
      that cell is owed or not and wherever it is placed. So does a read of a member of a spill or of an array
      formula whose anchor is outside R: it runs no anchor and restarts nothing. A cycle that closes only
      through such a read is no cycle for the run.
    - The run writes the cells of R and what their evaluation writes, the members of a spill or of an array
      formula, and no other cell. It runs no data table: a table's result cells hold no formula, and they keep
      their values until `calculate`. A workbook with a data table has no index, so everything is owed there.
    - It starts as a pass starts, with fresh per-pass caches and a fresh order state. It is a recalculation for
      SPEC-0028, confined to R: each formula has its own limit; the run has the pass allowance and the ceiling,
      and does at each what a recalculation does (share mode, exhaustion); a host can cancel it; and
      `last_recalc_outcome` reports it.
    - It depends on nothing an optimization decides. It runs the same cells, from the same stored values to the
      same values, with incremental recalculation on or off, with an index or without one, whatever is owed
      and whatever `can_restrict` says. It asks none of Design 4's conditions, and it is never rejected or
      followed by another pass.
    - It is no pass for Design 4. What Design 4 keeps from the last full pass stays as it was (the work bound,
      the flat, search and own-stop flags, the anchors' ranks, whether that pass ran in Excel's order), and so
      does the record of the last used rows: a row the run moves is a move since the last pass, and the pay
      takes it. Where a run cannot leave one of these as it was, everything is owed after it. Two such runs are
      named (the engine lane): one in which a dynamic anchor of R restarts, since a restart reorders the
      anchors; and one in which the footprint of a spill changes, since it writes or clears cells outside R,
      possibly over stored blanks.
    - It makes no claim of a recalculation's values, and it takes no cell out of the owed set. Every formula
      cell whose value the run changed is owed after it, with its readers; where no index gives the readers,
      everything is. A spill's members are their anchor's for this.
  - **A formula written in Manual mode is evaluated when it is written**, as Microsoft documents (S3, above)
    *(probes M3 and M4)*. A pasted or filled formula is a written formula *(probe M13)*. The transaction ends
    with the **entry evaluation**, a scoped run over its written formula cells. Entry evaluation is part of
    Manual-mode semantics, not an incremental optimization. A formula write uses the same entry-evaluation rules
    with incremental calculation enabled or disabled, whether the debt is a set or everything, and whether an
    index is available. The closure threshold and `can_restrict` do not determine whether an entered formula is
    evaluated. The incremental-off reference performs the same entry evaluations. Their cached results are part
    of the subsequent calculation history: a later circular reference can retain them. Design 4 restores a
    rejected attempt to the state before that attempt; it does not remove earlier committed entry evaluations.
    - No formula is excepted. A dynamic array formula spills at entry, an array formula fills its range, a
      formula that draws (RAND) draws, an order-state reader runs in Excel's order among the written cells, and
      a cycle among them is swept when iteration is on, in the reference as here.
    - A written formula reads the stored value of every formula cell its transaction did not write, stale or
      not *(probe M2: whether Excel calculates a dirty precedent first is not documented)*. Formulas written
      together evaluate each other. So an entry that closes a loop through a cell it did not write finds no
      loop: `calculate` does.
    - The written cells are owed, with their readers, by the transaction's own marking. So `is_stale` reports
      them, and `calculate` runs them again.
    - A transaction that owes everything (a structural edit beside the formula, no index) still evaluates the
      formulas it wrote, after all its ops are applied.

    In OnDemand mode, which is ours, a written formula is not evaluated at entry: it is owed, and a read of it
    fails until it is computed.
  - **Calculate Sheet.** `Workbook::calculate_sheet(sheet)` is Shift+F9, which "recalculates the cells marked for
    calculation in the active worksheet only" (S1). `calculate_sheet(sheet)` calculates the requested worksheet
    only and reads stored formula values on other worksheets. It never substitutes a workbook-wide `calculate`.
    A cell it runs remains owed while an external precedent remains owed. This command needs a worksheet-scoped
    evaluation path and its own budget handling; failure of Design 4's full-workbook-equivalence guards does not
    authorize evaluating another sheet. Cancellation or exhaustion preserves outstanding debt and does not
    silently expand the calculation scope.
    - It is a scoped run over every formula cell of the sheet, owed or not, as Excel's `Range.Calculate`
      "recalculates just the cells in the given range regardless of whether they are dirty or not" (S1). So a
      volatile cell of the sheet draws with nothing dirty.
      Running the sheet's owed cells alone would make the cells that run, and so what draws on the allowance,
      depend on the index. That is a later step with an argument of its own.
    - In the first slice it releases nothing: every cell that was owed stays owed, the ones with no precedent
      outside the sheet too, and `calculate` settles them. To release a cell is to claim a full calculation's
      value for it, which needs Design 4's conditions for the sheet's run; that is the same later step.
      Microsoft documents that Shift+F9 "resets all previously uncalculated (dirty) cells as calculated" (S3),
      so until then `is_stale` and `calc_pending` report cells Excel's status bar would not *(probe M14)*.
    - It makes the same run in every mode, and pays nothing else. In Automatic mode what it leaves owed (the
      readers, on other sheets, of a volatile cell that drew) is paid by the next edit, as after a setter; in
      OnDemand mode so are the watches it makes pending.
    - The UI lane asked for it (Formulas > Calculate Sheet).
  - **OnDemand.** `edit` marks, then pays what the watched ranges need. `Workbook::compute(targets)` pays what the
    targets need. `calculate` pays everything.
    - First slice: what a target needs is everything owed, when the target is stale (`is_stale`, below), and
      nothing otherwise. A batch of N writes and one read then costs one recalculation.
    - Second slice, after the dependency-first traversal exists (Design 13): what a target needs is its
      **cone**, the owed cells among the targets and their precedents. The precedents are read from what the
      index's build files for a formula (`leaves` and `instantiate`, a footprint rule's rectangle included,
      Design 12), not from a walk of its own: the cone must be the exact inverse of the readers the closure
      followed, or it misses an owed precedent. The traversal is Design 13's, started at the targets and kept
      to owed cells. A cone that holds a formula whose reads the index does not model (Design 11.1's list) is
      everything owed, since that formula may read any owed cell. The cone runs as a restricted pass under
      Design 4. What stays owed is still closed under readers: a reader of a cell left owed is owed, and it
      cannot be in the cone, which holds every owed precedent of its cells.
- **`watch`.** `Workbook::watch(range) -> WatchId` and `unwatch(id)` name ranges an OnDemand workbook keeps fresh:
  every `edit` pays what they need before it returns, and its change set lists the values that changed. In
  Automatic and Manual modes a watch changes nothing. At most 4,096 ranges are watched; one more is refused
  (directive 5). Watch scheduling uses the same stale predicate as `is_stale`, including everything-owed and
  spill-induced sheet-wide staleness. A new watch and a switch into OnDemand must account for existing debt.
  Watches left stale by cancellation remain pending. Every OnDemand edit pays all pending watches before
  returning unless calculation is cancelled. Scanning only newly added debt is an optimization for updating this
  pending set, not the test for whether a watch needs calculation.
  - The pending set: a watch is pending while its range is stale. `watch` and `set_calc_mode(OnDemand)` decide
    it by `is_stale` of the range. An edit or a scoped run adds the watches that its added cells touch, in one
    walk of those cells against the watches' rectangles, sorted, O(added cells x log watches); every watch of
    a sheet on which it owes a dynamic anchor; and every watch when it owes everything. A pay takes out the
    watches whose range is no longer stale, and a cancelled one takes out none.
  - `watch` itself computes nothing: its range may be stale until the next edit, and a read of it then fails as
    any read of a stale cell does. A caller that needs it at once calls `compute`.
  - A watch is a rectangle of positions. It does not follow an insert or a delete of rows or columns: after a
    structural edit the caller unwatches and watches again, as the engine worker does for the viewport's
    rectangles (at most four a sheet, with frozen panes) and the charts' ranges (poc-model-cc). A deleted
    sheet's watches go with it.
- **Reads.** The facade's reads take `&self` and never evaluate.
  - In Manual mode they return the stored value, stale or not, as Excel shows it *(probe M1)*.
    `Workbook::calc_pending()` is Excel's status-bar "Calculate" (`xlPending`): something is owed.
  - In OnDemand mode a read that touches a stale cell fails with `RefError::NotCalculated`, so a stale value is
    never returned unnoticed. That holds for every read a recalculation can change: `value`, `values`, `cell`,
    `formatted`, `displayed`, `text_utf16`, `in_spill`, `array_kind`, `used_range` and `stored_cells`. The caller
    runs `compute` first; a binding does both in one call.
  - A host needs the stored value as it is in two places: the before-image it takes of a cell it is about to
    overwrite (the wasm engine reads a staged cell before it writes it), and what it shows while `compute`
    runs. `Workbook::stored(target)` gives the same reads with no check, in every mode, and says nothing about
    freshness. The engine worker answers its UI with the values it last sent and the stale areas (I-17's
    `pending`, which is its staged deltas and these owed cells together).
  - `Workbook::is_stale(range)` says whether the range holds a stale cell. It may say so of a cell whose value
    would not change: the owed set is a superset, as the closure is.
  - Readers outside a pass (a conditional format's rule, a pivot's source by name, `outside_cell.rs`) read stored
    values while anything is owed. They do not evaluate an owed formula cell.
- **Change sets.** `edit` lists the value changes of the cells it paid and of the cells it wrote. `calculate`,
  `calculate_sheet`, `compute` and `set_calc_mode` return a change set that holds values only. Each equals the
  difference of the stored values across its call, the oracle Design 6's test already uses. `ChangeSet::owed`
  says whether something is still owed; `ChangeSet` becomes `#[non_exhaustive]` with it, since a new field breaks
  a struct literal outside the crate. A host that applies area formats to spill members from a change set (the
  wasm engine's `format_spills`) does so for these change sets too.
- **The file.** The importer keeps `calcPr@calcMode` (today it is dropped, and the regenerating export writes an
  empty `calcPr`; the patch tier already copies `workbook.xml`), and the regenerating export writes it back.
  `Workbook::file_calc_mode()` gives it, `autoNoTable` included, so the round trip loses nothing. The facade
  does not adopt it, as Excel does not adopt the mode of a workbook opened second: a workbook starts in
  `LoadOptions::calc_mode`, Automatic by default, and `LoadOptions::recalc` keeps its meaning. A UI that wants
  the desktop's behaviour reads `file_calc_mode()` and sets the mode. In Manual and OnDemand modes a save while
  something is owed fails with `SaveError::NotCalculated`: Excel recalculates before a save by default in
  Manual mode (S3 and 3, above), `fullCalcOnLoad` would not make it recalculate a Manual file on open, and a
  file must not carry stale cached values unnoticed. The file is the one Excel writes, by another route: a
  save takes `&self` and cannot calculate, so the caller calls `calculate` and saves *(probes M9 and M10 say
  what Excel writes and shows; a save option that keeps stale values, as Excel does with the check box off,
  waits for them)*. In Automatic mode a save never fails for this.
- **The collaborative sheet.** The document's mode is Automatic (I-02, `CalcIdentity.calculation_mode`: "manual is
  a per-user session option"). The aim is that a replica in Manual or OnDemand mode lags and never diverges:
  that once settled its values are those of an Automatic replica, bit for bit. That holds only when a settled
  workbook's values are a function of the document, and on this build they are not:
  - main's loop rule keeps a loop's values from before the recalculation, so a replica that recalculates less
    often keeps other values. With A1 `=IF(C1,A2,D1)`, A2 `=A1`, C1 FALSE and D1 1, the edits D1 = 2 and then
    C1 = TRUE leave an Automatic replica at 2 and a Manual one, settled once, at 1 (the engine lane's example);
  - draws are unseeded, and without a `clock_ms` the wasm engine runs on the system clock.

  So the wasm engine refuses Manual and OnDemand until T-5288 gives it loops evaluated as on a fresh open
  (D-ENGINE-8) and seeded draws (I-07), and then only with a fixed clock. Its order-independence test gains
  the modes when it stops leaving loops and RAND out. The engine worker maps the viewport and the charts' ranges
  (I-06 `subscribe` and `watch`) to watches.
- **Why it is exact.**
  1. *Automatic* runs the passes it runs today.
  2. *Manual.* The reference is the same workbook with incremental recalculation off. A scoped run is the same in
     both, by its rules: the same cells, from the same stored values, to the same values (a draw apart: the
     engine draws from one unseeded generator, in a scoped run as in a pass, so the differentials leave a
     workbook that draws out, as they do today). So the reference makes the same entry evaluations and the same
     sheet runs, and before every `calculate` the two hold the same stored values: the last recalculation's, and
     over them what the same scoped runs wrote. The reference owes everything, and its `calculate` is
     `Model::evaluate`. Here, a formula cell that is not owed holds the value the last recalculation gave it, and
     so does every cell it reads:
     - each transaction adds its closure in the index as it then is, and a formula written later is owed
       itself, with its readers. A formula that stopped reading a written cell stays owed, which is a superset;
     - a scoped run adds every formula cell whose value it changed, with its readers;
     - the last used rows' moves are taken since the last pass at every transaction and once more at the pay;
       a row a single step crosses lies between the recorded row and the row after that step or the step
       before it, so a head any step crosses is owed.

     So `calculate` is a restricted pass whose edit is the union of the deferred transactions and of the scoped
     runs' changes, started from the stored values the reference's full pass starts from, with Design 4's
     records as the last full pass left them. Designs 3 to 6 apply to it as written: an accepted pass has the
     full pass's values, and a rejected one is followed by the full pass itself. Where a full pass keeps a
     stored value, it keeps the same one in both, a scoped run's included: a loop that a later edit closes
     keeps the values from before the recalculation (recalc.md), and those are the same in both. When nothing
     is owed and `can_restrict` fails, the pay is the reference's own full pass. A history with a loop, an
     Excel-order reader, a volatile draw or a work limit passed gives Manual other values than Automatic,
     because it recalculates less often. That is Excel's behaviour too, and it is why the reference is Manual
     with incremental off, not Automatic.
  3. *OnDemand.* After `calculate` it is Manual's argument, without entry evaluations, which OnDemand does not
     make. The first slice only moves the `calculate` to the first read, so it is exact at every read; a load
     that did not recalculate owes a full calculation, so the first `compute` after it runs one. With cones, a
     value `compute` returned before the workbook is settled is the settled value **unless the recalculation that
     settles it leaves normal mode** (SPEC-0028: share mode, or the ceiling spent), which
     `Workbook::last_recalc_outcome` reports and whose change set lists the corrected cells. In normal mode a
     formula's value is a function of its inputs, the cone holds every owed input of its cells, and the guards of
     Design 4 reject a cone pass that nests or reads ahead. What a cone pass cannot know is the work of the cells
     it left owed, so it cannot tell whether a full pass of the new state would share its work out. A host that
     needs the settled value calls `calculate`.
- **What must be on main first.** T-5166's maintenance, or a formula write owes everything (above). Design
  11.1's list, or a workbook with a volatile cell has no index and every pay is a full pass. Design 13's
  traversal, for cones. With incremental recalculation off (`set_incremental(false)`; it is on by default since
  decision 0422), every edit owes everything. A scoped run needs none of these.
- **What the implementation may rely on.**
  1. Design 3's closure may be computed at the transaction and kept: the readers of a cell depend on the formulas
     alone (`closure.rs`).
  2. Design 4's guards, the summing work bound and `evaluate_cells`' put-back need no change. A deferred pass is
     a restricted pass with more roots, sorted as `edit_closure` returns them: `pass_roots` runs the roots in the
     order given, and the read-ahead test compares natural order.
  3. `Model::moved_last_rows` gives the net move since the last pass's end, which is what the argument above
     needs. It is crate-private: the facade reaches it through `edit_closure`.
  4. The owed set belongs to the model, beside the work bound (`Incremental`).
  5. A cone pass leaves the owed set closed under readers, by the argument above, provided its precedents are
     the inverse of what the index files; the implementation asserts it in debug builds.
  6. The scoped run is the one new path, for the entry evaluation and for `calculate_sheet`. It is
     `Model::evaluate` with its roots confined to R and without its data-table phase. Its reader is the
     restricted pass's: `clean_cell_value` (`read_index/pass.rs`) already lets a member of the set it is given
     run and gives the stored value of any other formula cell, through `mark_readers_of` and `read_evaluated`,
     so an index that exists records the read (the engine lane's point). What differs from `evaluate_cells`:
     - the read-ahead and nesting flags decide nothing; `restricted_pass_allowed` is not asked; nothing is put
       back and no full pass follows; `Incremental`'s records of the last full pass are not written;
     - the dynamic anchors of R and their spills are handled as a full pass handles them (a restricted pass
       runs today only where nothing spills);
     - the pass in Excel's order (`run_excel_order`) and the iteration sweeps (`iterate_cycles`) run too, each
       over the cells of R alone. A restricted pass reaches neither today: `can_restrict` asks for no
       iteration and no Excel order;
     - one more reader needs the switch: `evaluate_spill_cell` (`evaluation.rs`) counts a member as a value
       only if its anchor wrote it in the current pass, and otherwise runs the anchor, reads empty or restarts
       the pass. In a scoped run a member whose anchor is outside R is read as stored there (the engine lane).
- **What it must show.**
  - The metamorphic suite runs each generator in Manual mode with `calculate` at random points: incremental on
    against off after every `calculate`, values and change sets; on flat histories also against an Automatic
    twin and a fresh build.
  - In OnDemand mode: `compute` of random targets at random points equals the Automatic twin's values for those
    cells on flat histories; `calculate` then equals it everywhere; every read on the list fails for a stale
    cell and `stored` does not; every cell whose stored value differs from the twin's is reported by `is_stale`,
    an empty cell under an owed anchor among them.
  - Entry evaluation, in Manual mode: a formula written over settled literals holds the Automatic twin's value
    when `edit` returns; written over an owed precedent, or over a formula cell placed after it, it holds the
    value the stored precedent gives, and `is_stale` reports it; two formulas written together, the first
    reading the second, hold the pair's values; a dynamic array formula spills and an array formula fills its
    range, and everything is owed after a spill whose footprint changed; a formula that reads a member of a
    spill it did not write holds the member's stored value, and the spill's anchor did not run; and no formula
    cell the transaction did not write was evaluated. Each of these on the system clock,
    with incremental recalculation off, in a workbook with no index, and with a closure past the quarter limit:
    `=A1+2` over literals holds the same value in all four.
  - The twin, an oracle for a scoped run that shares none of its confinement. Clone the workbook. Replace every
    formula cell outside R by the value a stored read of it gives; an array formula or a dynamic anchor outside
    R by a constant array formula that holds its stored values over the same footprint, so that a spill
    reference to it still resolves and its members stay its members; and every data table's result cell by its
    stored value. Drop the data tables (`TABLE()`; an Excel table and its range stay, so structured references
    resolve). Run `Model::evaluate`. While the run stays in normal mode (SPEC-0028), the cells of R and their
    members hold the clone's values, errors included. The generators check it for entries of one to many
    formulas and for every sheet, on the metamorphic documents, on the loop generator with iteration off and
    on, and on SPEC-0017's order shapes (the two replacements that keep references resolving are the engine
    lane's).
  - Cycles in a scoped run, in each caller. Iteration off: a self-reference and a two-cell cycle keep their
    pre-run values, a nonzero one included, and so does a reader of the cycle in R. Iteration on, three
    iterations, from stored zeros: `=A1+1` holds 3; B1 `=C1+1` with C1 `=B1+1` hold 5 and 6 (the sweep is in
    sheet order and in place, recalc.md). A cycle that crosses the boundary of R is not swept: the cell outside
    R is read as stored and is not written. A run cancelled during a sweep takes no cell out of the owed set
    and writes no cell outside R.
  - Order state in a scoped run, in each caller. With A1 `=WEEKNUM(45292,22)` and B1 `=WEEKNUM(45292,2)` both in
    R, both hold 1, as a full recalculation gives, not the `#NUM!` that forward order from a fresh state gives
    A1 (the reproducer of CR-321724568's round 2). With the setter B1 outside R, A1 finds a fresh state and
    holds `#NUM!`. With a reader in R that reads a formula cell of R the chain has not reached, the reader is
    moved behind it and both hold the twin's values.
  - The history: with A1 `=IF(C1,B1,D1)`, B1 `=A1`, C1 FALSE and D1 1 settled, A1 is rewritten as
    `=IF(C1,B1,D1)+1`, then C1 is set to TRUE, then `calculate`. Incremental on and the reference agree on A1
    and B1, both having made the entry evaluation (the reproducer of CR-321724568's review, where a reference
    with no entry evaluation gives 0 against 2).
  - `calculate_sheet`: with a formula on one sheet reading a formula on another whose input changed, the other
    sheet's cell keeps its stored value and stays owed, in both sheet orders, after a full pass that was not
    flat, and on the system clock; a run cut by the ceiling or cancelled leaves what was owed owed and
    evaluates nothing off its sheet; a volatile cell of the sheet draws, and its readers on another sheet are
    owed; incremental on and off hold the same stored values after it and after the `calculate` that follows.
  - A load: in OnDemand mode a workbook loaded with `recalc` false (the probe request's `m10-manual-stale.xlsx`,
    whose cached B1 is 999) reports B1 stale, and a `compute` of it gives 2; a load whose recalculation was
    cancelled owes everything.
  - A watch's life: a watch registered over an owed range is paid by the next edit, an edit of an unrelated
    literal included; so is one that was stale at a switch into OnDemand, one left stale by a cancelled pay, and
    one on the sheet of an owed spill anchor.
  - A transaction with a structural edit owes everything.
  - Automatic is unchanged: `Workbook::edit_paths` and `last_recalc_outcome` after every edit of the suites equal
    main's, an empty transaction runs a full pass, and a save after `set_iteration` succeeds.
  - The recoveries: a `calculate` after a pass that shared its work out and a raised ceiling computes the cells;
    a cancelled `compute` leaves every cell stale and the next `calculate` recovers.
  - Mutations that must fail them: an owed cell dropped after a cone pass; a watched range not paid; a formula
    written while cells are owed not added; a scoped run taking a cell out of the owed set, evaluating a formula
    cell outside its set, writing the work bound, or leaving unowed a formula cell it changed; one that keeps a
    cycle's stored values with iteration on, or runs an order-state reader in forward order; an entry
    evaluation skipped when `can_restrict` is false, when there is no index, when incremental recalculation is
    off or when everything is owed; `calculate_sheet` followed by a pass over another sheet; a load without
    recalculation leaving nothing owed; a watch's pending state decided by the cells an edit added alone; a
    non-cell op while cells are owed not owing everything; `set_calc_mode(Automatic)` not paying; `is_stale`
    false for an owed cell; nothing owed and `can_restrict` false not running a full pass; the owed cells
    passed to `evaluate_cells` unsorted.
  - An xlsx with `calcMode="manual"` or `autoNoTable` round-trips the attribute, in the regenerating and the
    source-preserving export.
  - A bench: 1,000 writes and one read on the 1M-formula chain, OnDemand against Automatic.
- **Steps**, each S or M, in order: (a) the owed set, Manual with the scoped run (the entry evaluation and
  `calculate_sheet`), `calculate`, `calc_pending`, `is_stale`, `stored`, the file's mode; (b) OnDemand's first
  slice, `compute`, `watch`, `NotCalculated`; (c) cones, after Design 13's traversal; (d) the Python and wasm
  bindings, by their lanes, the wasm one after T-5288; (e) `calculate_sheet` over the sheet's owed cells alone,
  and what it releases, as an amendment that carries their argument.
