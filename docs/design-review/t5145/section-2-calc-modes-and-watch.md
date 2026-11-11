# T-5145 section 2: calculation modes and `watch` (T-5114) — draft for the project lead

*poc-design-cc, 2026-11-11. Draft for a design read, before the reviewers. The text under "Proposed spec text" goes
into SPEC-0002 as Design 9. Rules marked (probe Mn) are provisional until the probe request answers them.*

## In short

- **One mechanism, three modes.** Every edit marks what it can change (Excel does this in every mode). The mark is
  the *owed set*. Automatic pays it at once, which is today's behaviour bit for bit. Manual pays on `calculate()`
  (F9). OnDemand pays what a read or a watched range needs.
- **Nothing new in the evaluator.** A deferred recalculation is a restricted pass (SPEC-0002 Design 4) whose edit is
  the union of the deferred transactions. All of Design 4's guards apply unchanged.
- **Two slices of OnDemand.** First: a read pays everything owed, so N writes and one read cost one recalculation.
  Second, after the dependency-first traversal exists: a read pays only its cone.
- **`watch`** names ranges an OnDemand workbook keeps fresh at every edit (the viewport, chart ranges).
- **The collaborative sheet stays Automatic as a document** (contract I-02). Manual and OnDemand are meant to be a
  replica's own pace: it lags, it does not diverge. Corrected at about 11:00Z after the engine lane's review: that is
  true only when a settled workbook's values are a function of the document, which needs loops evaluated as on a
  fresh open, seeded draws and a fixed clock. The wasm engine has none of the three yet (T-5288), so it refuses
  the two modes until it does.
- **Corrected twice on 2026-11-11 (about 12:05Z and 12:45Z), after the two review rounds of the first change
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

7. A formula written in Manual mode is **evaluated at once**, as Microsoft documents, from the stored values of
   the cells it reads; no other formula cell is evaluated (the engine lane's rule). In the first slice three
   kinds show their value only at `calculate`: an array formula or spill anchor, a formula that draws (RAND), and
   any formula of a workbook with no index. A stale or absent value is reported (`is_stale`), never a wrong one.
   Acceptable? (yes/no; mine: yes.)

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
    Documented and probed: an entered or edited formula is evaluated at once (M3, M4); F9 runs a volatile cell
    with nothing dirty (M6); what a save and a load of a Manual workbook do (M9, M10); what Shift+F9 leaves
    dirty (M14). Not settled by the pages, and probed: whether a row inserted, a name added or a row hidden in
    Manual mode recalculates (M12, M15, M16). Not documented, and probed: what a read of a dirty cell returns
    (M1); whether a formula entered over a dirty precedent reads its stored value or calculates it first (M2);
    whether a pasted formula counts as entered (M13); a switch back to Automatic (M11).
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
  - A load leaves nothing owed, whether or not it recalculated: the stored values are the file's or the load's,
    and the first pay after it is a full pass, since no work bound is known ("Paying").
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
    OnDemand's). Between edits something can be owed, after a setter: as today, it is paid by the next edit.
  - **Manual.** `edit` marks. It pays nothing, with one exception: it evaluates the formulas it wrote (the next
    item). `Workbook::calculate` pays everything (F9).
    `Workbook::recalculate` stays a full pass (Ctrl+Alt+F9), and ends settled: the edits it makes itself (it
    refreshes `refreshOnLoad` pivots through `edit`) pay, whatever the mode. `Workbook::set_calc_mode(Automatic)`
    pays everything *(probe M11)*. `Workbook::calculate_sheet(sheet)` is Shift+F9, which "recalculates the cells
    marked for calculation in the active worksheet only" (S1): it runs the owed cells of that sheet as a
    restricted pass over those cells alone, which read stored values on the other sheets, and a cell it ran
    stays owed while one of its precedents is, so `calculate` pays it again; a sheet pass that Design 4 rejects
    is followed by `calculate` *(probe M14)*. Microsoft also documents that Shift+F9 "resets all previously
    uncalculated (dirty) cells as calculated" (S3). This design keeps such a cell owed because its value is
    stale, and F9 must run it again in Excel too, as a dependent of a dirty cell on another sheet. The two
    differ only in what `is_stale` and `calc_pending` say in between, and the F9 after the probe's Shift+F9
    tests which. The UI lane asked for it (Formulas > Calculate Sheet).
  - **A formula written in Manual mode is evaluated when it is written**, as Microsoft documents (S3, above)
    *(probes M3 and M4)*. A pasted or filled formula is a written formula *(probe M13)*. The transaction ends
    with a pass over its written formula cells alone, the **entry evaluation**. The written cells stay owed, with
    their readers, so `calculate` runs them again under Design 4; the value shown in between makes no claim of
    being a recalculation's, and `is_stale` says so. Its rules:
    - it never evaluates another formula cell. A read of any formula cell, placed before or after the written
      one, owed or not, takes its stored value *(probe M2: whether Excel calculates a dirty precedent first is
      not documented)*. So Design 4's nesting and read-ahead rules do not apply to it, and it is not rejected
      for them: the pay recomputes the written cells in the exact order, and nothing the entry evaluation left
      is read as clean (the engine lane's rule);
    - it runs only when the transaction owes a set of cells. A transaction that owes everything (a written
      array formula or spill anchor, a formula the index does not model, an op that is not a cell write)
      evaluates nothing at entry, and its formulas show their values at `calculate`;
    - a written formula that draws (RAND, RANDBETWEEN, RANDARRAY) is left unevaluated. This guards Design 11.1:
      today a workbook that holds one has no index, so the rule before this one already covers it; once such
      a formula is always dirty instead, a draw at entry would move the sequence every later pass reads. A
      written formula that reads the order state ends the pass as Design 11.1 ends any restricted pass, and
      its cells are left unevaluated. Under a fixed clock TODAY and NOW leave nothing behind, and are
      evaluated;
    - one that leaves normal mode makes everything owed, since the work bound is lost;
    - it records the last used rows as every pass does.

    In OnDemand mode, which is ours, a written formula is not evaluated at entry: it is owed, and a read of it
    fails until it is computed.
  - **OnDemand.** `edit` marks, then pays what the watched ranges need. `Workbook::compute(targets)` pays what the
    targets need. `calculate` pays everything.
    - First slice: what a target needs is everything owed, when the target holds an owed cell, and nothing
      otherwise. A batch of N writes and one read then costs one recalculation.
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
  (directive 5). Whether a watch holds an owed cell is found in one walk of the cells the edit added to the owed
  set against the watches' rectangles, sorted, so an edit costs O(added cells x log watches), not O(watches x
  area). A watch is a rectangle of positions. It does not follow an insert or a delete of rows or columns:
  after a structural edit the caller unwatches and watches again, as the engine worker does for the viewport's
  rectangles (at most four a sheet, with frozen panes) and the charts' ranges (poc-model-cc). A deleted sheet's
  watches go with it.
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
  2. *Manual.* The reference is the same workbook with incremental recalculation off. There every transaction
     owes everything, so `edit` evaluates nothing, and `calculate` runs `Model::evaluate`. With incremental
     recalculation on, the only cells a pass changes between two `calculate` calls are the written formula
     cells of an entry evaluation, and they are owed; so the stored value of every cell that is not owed is the
     last recalculation's, as in the reference. The two differ between `calculate` calls only in owed cells,
     which `is_stale` reports in both. The owed set holds every formula cell one of whose inputs changed since
     then: each transaction adds its closure in the index as it then is, and a formula
     written later is owed itself, with its readers. A formula that stopped reading a written cell stays owed,
     which is a superset. The last used rows' moves are taken since the last pass at every transaction and once
     more at the pay; a row a single step crosses lies between the recorded row and the row after that step
     or the step before it, so a head any step crosses is owed. So `calculate` is a restricted pass whose edit
     is the union of the deferred transactions, and Designs 3 to 6 apply to it as written. An entry
     evaluation evaluates no cell but the written ones, neither draws nor reads the order state (its rules
     above), so it leaves nothing behind that a later pass reads, and `calculate` computes what the reference's
     does. When nothing is
     owed and `can_restrict` fails, the pay is the reference's own full pass. A history with a loop, an
     Excel-order reader, a volatile draw or a work limit passed gives Manual other values than Automatic,
     because it recalculates less often. That is Excel's behaviour too, and it is why the reference is Manual
     with incremental off, not Automatic.
  3. *OnDemand.* After `calculate` it is Manual's argument. The first slice only moves the `calculate` to the
     first read, so it is exact at every read. With cones, a value `compute` returned before the workbook is
     settled is the settled value **unless the recalculation that settles it leaves normal mode** (SPEC-0028:
     share mode, or the ceiling spent), which `Workbook::last_recalc_outcome` reports and whose change set
     lists the corrected cells. In normal mode a formula's value is a function of its inputs, the cone holds
     every owed input of its cells, and the guards of Design 4 reject a cone pass that nests or reads ahead. What
     a cone pass cannot know is the work of the cells it left owed, so it cannot tell whether a full pass of the
     new state would share its work out. A host that needs the settled value calls `calculate`.
- **What must be on main first.** T-5166's maintenance, or a formula write owes everything (above). Design
  11.1's list, or a workbook with a volatile cell has no index and every pay is a full pass. Design 13's
  traversal, for cones. With incremental recalculation off (`set_incremental(false)`; it is on by default since
  decision 0422), every edit owes everything.
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
    value the stored precedent gives, and `is_stale` reports it; written as an array formula, or with a draw,
    it is left unevaluated; in each case `calculate` then gives the twin's values, and no formula cell other
    than the written ones was evaluated by the entry. A transaction with a structural edit owes everything.
  - Automatic is unchanged: `Workbook::edit_paths` and `last_recalc_outcome` after every edit of the suites equal
    main's, an empty transaction runs a full pass, and a save after `set_iteration` succeeds.
  - The recoveries: a `calculate` after a pass that shared its work out and a raised ceiling computes the cells;
    a cancelled `compute` leaves every cell stale and the next `calculate` recovers.
  - Mutations that must fail them: an owed cell dropped after a cone pass; a watched range not paid; a formula
    written while cells are owed not added; an entry evaluation taking its cells out of the owed set, or
    evaluating a formula cell it did not write; a non-cell op while cells are owed not owing everything;
    `set_calc_mode(Automatic)` not paying; `is_stale` false for an owed cell; nothing owed and `can_restrict`
    false not running a full pass; the owed cells passed to `evaluate_cells` unsorted.
  - An xlsx with `calcMode="manual"` or `autoNoTable` round-trips the attribute, in the regenerating and the
    source-preserving export.
  - A bench: 1,000 writes and one read on the 1M-formula chain, OnDemand against Automatic.
- **Steps**, each S or M, in order: (a) the owed set, Manual with its entry evaluation, `calculate`,
  `calculate_sheet`, `calc_pending`, `is_stale`, `stored`, the file's mode; (b) OnDemand's first slice, `compute`, `watch`, `NotCalculated`; (c)
  cones, after Design 13's traversal; (d) the Python and wasm bindings, by their lanes, the wasm one after
  T-5288.
