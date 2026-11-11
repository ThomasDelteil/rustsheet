# Recalculation modes, dependency tracing and dynamic references: what Excel documents and what other engines do (2026-11-11)

Research for T-5145, the SPEC-0002 amendments (Designs 9 to 12). Every source was fetched on 2026-11-11. Text in
double quotes is verbatim from the source named (line breaks joined, markup dropped). "Source obs." marks a reading
of fetched source code, not a quote. Nothing here is evidence of Excel's behaviour in the sense of
`docs/excel-semantics/`: documentation says what Microsoft wrote, a probe says what Excel does. The probe request
`t5145-recalc` asks Excel the questions these pages leave open.

## What we take and what we avoid

| Point | Take | Avoid |
|---|---|---|
| Marking and calculating | Excel's split (S1): cells are marked dirty at the edit in every mode, and only the calculation waits. Design 9's owed set is that mark | Apache POI's contract (8), where the caller must tell the evaluator what changed |
| Manual mode | Excel's and LibreOffice's (S1, 2, 3): F9 pays the dirty cells and the volatile ones, a second command recalculates everything | Reading the mode from the file as the document's: on the desktop it is the application's, on the web the session's (3, 4) |
| A lazy mode | Formualizer's and POI's evaluation on request (8, 9), and HyperFormula's refusal to read inside a suspended batch (1): a stale value is never handed out unnoticed | A lazy mode whose values silently differ from a full recalculation's |
| Precedents and dependents | Office.js's shape (S2): areas, by worksheet, every level; ranges, not cells, as HyperFormula returns them (1) | VBA's active-sheet limit (5); HyperFormula's one level (1) |
| Volatile functions | An always-dirty set, as every engine here has (1, 2, 9, 11, 13) | One volatile formula turning the dependency index off for the workbook, which is this engine today |
| OFFSET and INDIRECT | For later (T-5420): HyperFormula's parse-time OFFSET for literal arguments (1), and Gnumeric's dependency recorded at evaluation (12) | LibreOffice's and Formualizer's plain "always recalculate" (2, 9) as the end state |
| Ranges | Rectangles filed once, never one edge per cell, as HyperFormula's range nodes, LibreOffice's area listeners and Gnumeric's range hash do (1, 2, 12) | |

## S1. Microsoft, "Excel Recalculation"
URL fetched: learn.microsoft.com/en-us/office/client-developer/excel/excel-recalculation (page dated 2018-08-22, updated
2024-12-18).
- Triggers: "Entering new data (if Excel is in Automatic recalculation mode"; "Deleting or inserting a row or column";
  "Saving a workbook while the Recalculate before save option is set"; "Adding, editing, or deleting a defined name";
  "Renaming a worksheet"; "Hiding or unhiding rows, but not columns".
- Stages: "Construction of a dependency tree", "Construction of a calculation chain", "Recalculation of cells".
  "During recalculation, Excel revises this chain if it comes across a formula that depends on a cell that has not
  yet been calculated. In this case, the cell that is being calculated and its dependents are moved down the chain."
- Marking: "When new data or new formulas are entered, Excel marks all the cells that depend on that new data as
  needing recalculation. Cells that are marked in this way are known as dirty." "This recalculation occurs
  immediately after Excel finishes marking cells as dirty if the recalculation mode is automatic; otherwise, it
  occurs later."
- Volatile: "Excel reevaluates cells that contain volatile functions, together with all dependents, every time that
  it recalculates." The list as fetched: NOW, TODAY, RANDBETWEEN, OFFSET, INDIRECT, "INFO (depending on its
  arguments)", "CELL (depending on its arguments)", "SUMIF (depending on its arguments)".
- Modes: "Automatic", "Automatic Except Tables", "Manual". "When calculation is set to automatic, recalculation
  occurs after every data input and after certain events".
- F9, `Application.Calculate`: "Recalculates all cells that Excel has marked as dirty, that is, dependents of volatile
  or changed data, and cells programmatically marked as dirty."
- Shift+F9: "Recalculates the cells marked for calculation in the active worksheet only."
- Ctrl+Alt+F9, `Application.CalculateFull`: "Recalculates all cells in all open workbooks."
- Ctrl+Alt+Shift+F9: "Causes Excel to rebuild the dependency tree and the calculation chain for a given workbook and
  forces a recalculation of all cells that contain formulas."
- `Range.Calculate`, Manual mode: "Recalculates just the cells in the given range regardless of whether they are
  dirty or not."
- What a formula entered in Manual mode shows: not stated.

## S2. Microsoft, "Work with formula precedents and dependents using the Excel JavaScript API"
URL fetched: learn.microsoft.com/en-us/office/dev/add-ins/excel/excel-add-ins-ranges-precedents-dependents (dated
2025-09-22).
- "Those input cells are the formula's "precedents". Any formula cell that depends on other cells is a "dependent" of
  those cells." "A "direct precedent" or "direct dependent" is only the first level: the cells a formula points to,
  or the cells that point straight to a formula."
- "Both APIs return a `WorkbookRangeAreas` object. It groups addresses by worksheet."
- "These APIs do not cross workbook boundaries."
- "If there are no related cells, an `ItemNotFound` error is thrown."
- "Very large sets, such as thousands of cells, can be slow."
- Names, INDIRECT, OFFSET, SUMIF's resized range, spill ranges: not stated.

## 1. HyperFormula (v3.5.0)
URL(s) fetched: raw.githubusercontent.com/handsontable/hyperformula/master/docs/guide/{volatile-functions,dependency-graph,known-limitations,batch-operations,types-of-errors}.md
(last commit 2026-10-15); .../master/src/HyperFormula.ts (doc comments behind the API reference);
handsontable.com/docs/hyperformula/guide/built-in-functions.html (its .md is a generated template, 404).
- Q: volatile set — A: RAND, RANDBETWEEN, NOW, TODAY (under "Built-in volatile functions:"). "Volatile functions are
  always treated as "dirty" and recalculated on most actions." ROW, COLUMN, ROWS, COLUMNS, FORMULATEXT "act as if
  they were volatile but only on operations on the sheet structure".
- Q: volatile actions — A: every edit method, plus `batch`, `resumeEvaluation`, `rebuildAndRecalculate` (table
  "Volatile actions").
- Q: ranges — A: "If formulas in the spreadsheet include ranges, each range is represented by a separate node."
  "it represents `B5:D20` as the composition of a range `B5:D19` and three cells in the last row".
- Q: OFFSET — A: "HyperFormula resolves the OFFSET function at parse time rather than during evaluation." "The
  row-shift, column-shift, height, and width arguments must be static integer literals known at parse time."
- Q: INDIRECT — A: not stated; the name does not occur on the function-list page.
- Q: cycles — A: #CYCLE!. "For cycle detection, all possible dependencies between cells are taken into account".
- Q: suspendEvaluation — A: "multiple CRUD operations can be done without triggering recalculation after every
  operation"; "You can't batch read operations." A batch bracket, not a manual mode.
- Q: getCellDependents / getCellPrecedents — A: "Returns all the out-neighbors in the dependency graph for a given
  cell address or range."; includes "Some of the ranges that contain the given cell address or range" and named
  expressions (sheet ID `-1`). Direct only; for all, "implement a Breadth-first search (BFS) algorithm".
- Q: dynamic arrays — A: not offered; array functions "require their output dimensions to be determinable at parse
  time".

Summary: a static graph of cell and range nodes fixed at parse time; no dependency is ever dynamic.

## 2. LibreOffice Calc (Help 26.8; core master)
URL(s) fetched: help.libreoffice.org/latest/en-US/text/scalc/01/{06070000,06080000,recalculate_hard,06030300}.html,
.../text/shared/optionen/01060900.html;
raw.githubusercontent.com/LibreOffice/core/master/{include/formula/tokenarray.hxx,formula/source/core/api/FormulaCompiler.cxx,sc/source/core/inc/bcaslot.hxx}
(FormulaCompiler.cxx last commit 2026-10-08).
- Q: AutoCalculate — A: "All cells are recalculated after a sheet cell has been modified." What a formula entered
  with it off shows: not stated.
- Q: Recalculate (F9) — A: "If AutoCalculate is disabled, the Recalculate command recalculates all (so-called dirty)
  formula cells that depend on changed cell content and their dependents, and formula cells containing volatile
  functions such as RAND() or NOW() and formula cells that depend on them."
- Q: Recalculate Hard — A: "forces recalculation of all formula cells of the spreadsheet document".
- Q: on load — A: "Never recalculate", "Always recalculate", "Prompt user".
- Q: ScRecalcMode — A: ALWAYS "exclusive, always"; ONLOAD_MUST "exclusive, always after load"; ONLOAD_ONCE
  "exclusive, once after load, import filter"; ONLOAD_LENIENT "exclusive, lenient after load (eg. macros not always,
  aliens, ...)"; NORMAL; FORCED "combined, also if cell isn't visible, for macros with side effects"; ONREFMOVE
  "combined, if reference was moved".
- Q: always recalculated — A: `IsOpCodeVolatile`: ocRandom, ocGetActDate, ocGetActTime, ocFormula, ocInfo,
  ocIndirect, ocOffset, ocDebugVar, ocRandArray, ocCall, ocByCol, ocByRow, ocMakeArray, ocMap, ocReduce, ocScan.
  "ocIndirect otherwise would have to do StopListening and StartListening on a reference for every interpreted
  value." "ocOffset results in indirect references." ocColumn, ocRow, ocCell recalculate on reference move.
- Q: Detective — A: "Draws tracer arrows to the active cell from formulas that depend on values in the active cell."
  "This function works per level." An API: not stated.

Summary: one toggle, F9 (dirty and volatile) and hard recalculation. INDIRECT and OFFSET are always-recalculate to
avoid re-registering listeners; ranges are listened to by area (bcaslot.hxx).

## 3. Microsoft: calculation options
URL(s) fetched: support.microsoft.com/en-us/office/change-formula-recalculation-iteration-or-precision-in-excel-73fc7dac-91cf-4d36-86e8-67124f6bcce4
(redirects to /en-us/excel/...); learn.microsoft.com/en-us/troubleshoot/microsoft-365-apps/excel/current-mode-of-calculation
(2026-03-25); raw.githubusercontent.com/MicrosoftDocs/VBA-Docs/main/api/Excel.{Application.Calculation,XlCalculation,Application.CalculateBeforeSave,Workbook.ForceFullCalculation,Range.Dirty,Range.Calculate,Application.CalculationState,XlCalculationState}.md
(source of learn.microsoft.com/office/vba/api; 2017-2019).
- Q: Manual — A: "To turn off automatic recalculation and update formulas only when you manually recalculate (for
  example, by pressing F9)". xlCalculationManual "Calculation is done when the user requests it.";
  xlCalculationSemiautomatic "Excel controls recalculation but ignores changes in tables." What a formula entered in
  Manual mode shows: not stated.
- Q: before save — A: "When you select Manual, Excel automatically selects the Recalculate workbook before saving
  check box." CalculateBeforeSave "is preserved even if you change the Calculation property".
- Q: keys — A: F9 "Recalculate formulas that have changed since the last calculation, and formulas dependent on them,
  in all open workbooks."; Shift+F9 the same "in the active worksheet"; Ctrl+Alt+F9 "Recalculate all formulas in all
  open workbooks, regardless of whether they have changed since the last recalculation."; Ctrl+Shift+Alt+F9 first
  "Check dependent formulas".
- Q: scope — A: "In Excel desktop apps, changing these options affects all open workbooks." Web: "will affect the
  current workbook only". "The first document that is opened uses the calculation mode with which it was last saved.
  Documents that are opened later use the same mode." "If the mode of calculation in a workbook changed and the file
  is saved, the current mode of calculation is saved."
- Q: ForceFullCalculation — A: "dependencies are ignored and all worksheets are marked to calculate fully every time
  a calculation is triggered".
- Q: Range.Dirty — A: "Designates a range to be recalculated when the next recalculation occurs."; in automatic mode
  it "instructs Excel to perform a recalculation". Range.Calculate: per-mode behaviour not stated.
- Q: CalculationState — A: xlPending "Changes that trigger calculation have been made, but a recalculation has not
  yet been performed."

Summary: on the desktop the mode is application-wide, taken from the first workbook opened and saved into every
workbook saved afterwards; on the web it is per workbook.

## 4. Office JavaScript API and Microsoft Graph
URL(s) fetched: raw.githubusercontent.com/OfficeDev/office-js-docs-reference/main/docs/docs-ref-autogen/excel/excel/excel.{application,calculationmode,calculationtype,calculationstate}.yml
(source of learn.microsoft.com/javascript/api/excel; 2026-10-15);
raw.githubusercontent.com/microsoftgraph/microsoft-graph-docs-contrib/main/api-reference/v1.0/{resources/workbookapplication.md,api/workbookapplication-calculate.md}
(2024).
- calculationMode: "This is a runtime property. The `calculationMode` setting is not persisted in the workbook."
- CalculationMode: automatic "The default recalculation behavior where Excel calculates new formula results every
  time the relevant data is changed."; automaticExceptTables "Calculates new formula results every time the relevant
  data is changed, unless the formula is in a data table."; manual "Calculations only occur when the user or add-in
  requests them."
- calculate(calculationType): "Recalculate all currently opened workbooks in Excel."
- CalculationType: recalculate "Recalculates all cells that Excel has marked as dirty" (S1's F9 sentence); full
  "This will mark all cells as dirty and then recalculate them."; fullRebuild "This will rebuild the full dependency
  chain, mark all cells as dirty and then recalculate them."
- CalculationState: "Represents the state of calculation across the entire Excel application."; calculating, done,
  pending (the xlPending sentence in 3).
- suspendApiCalculationUntilNextSync: "Once set, it is the developer's responsibility to re-calc the workbook, to
  ensure that any dependencies are propagated."
- Graph: calculate, the Office.js sentence; calculationType `Recalculate`, `Full`, `FullRebuild`; calculationMode
  `Automatic`, `AutomaticExceptTables`, `Manual`.

Summary: three modes, three depths of recalculation, and a status with an explicit pending state.

## 5. VBA precedents and dependents
URL(s) fetched: raw.githubusercontent.com/MicrosoftDocs/VBA-Docs/main/api/Excel.Range.{Precedents,DirectPrecedents,Dependents,DirectDependents,ShowPrecedents,NavigateArrow}.md
(2019).
- Precedents: "Returns a Range object that represents all the precedents of a cell. This can be a multiple selection
  (a union of Range objects) if there's more than one precedent." Active-sheet limit: not stated on this page.
- DirectPrecedents: "The DirectPrecedents property only works on the active sheet and cannot trace remote
  references." The Dependents and DirectDependents pages say the same of themselves.
- ShowPrecedents: "Draws tracer arrows to the direct precedents of the range."
- NavigateArrow: "This method causes an error if it's applied to a cell without visible tracer arrows."

Summary: four properties, each returning one Range union; three of the four pages state the active-sheet limit.

## 6. Microsoft support: tracing
URL(s) fetched: support.microsoft.com/en-us/office/display-the-relationships-between-formulas-and-cells-a59bef2b-3701-46bf-8ff1-d3518771d507
(redirects to /en-us/excel/...).
- precedent: "Precedent cells — cells that are referred to by a formula in another cell."
- dependent: "Dependent cells — these cells contain formulas that refer to other cells."
- untraceable: "References to text boxes, embedded charts, or pictures on worksheets." "PivotTable reports."
  "References to named constants." "Formulas located in another workbook that refer to the active cell if the other
  workbook is closed."
- other sheets: "a black arrow points from the selected cell to a worksheet icon".

Summary: the relation is reference by a formula; named constants and closed workbooks cannot be traced.

## 7. ECMA-376 `calcPr`
URL(s) fetched: c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_calcPr_topic_ID0EWJ63.html and
.../OOXML_P4_DOCX_ST_CalcMode_topic_ID0EI1CFB.html (ECMA-376 1st edition, Part 4);
learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.spreadsheet.calculationproperties (element paragraph
only); defaults also in raw.githubusercontent.com/python-openxml/python-docx/master/ref/xsd/sml.xsd.
- calcMode (default auto): "Specifies when the application should calculate formulas in the workbook." auto "The
  application recalculates those cells that are dependent on other cells that contain changed values."; autoNoTable
  "Indicates tables be excluded during automatic calculation."; manual "Indicates that calculations in the workbook
  be triggered manually by the user."
- fullCalcOnLoad (false): "After load and successful calculation, the application should set this value to false."
  "Note: If manual <calcMode> is true, then a full recalculation will not be performed on load, even when this
  attribute is set."
- calcOnSave (true): "indicates whether the application will recalculate values when the workbook is saved".
- calcCompleted (true): "determines whether workbook data was recalculated before the workbook was saved".
- forceFullCalc (none): "indicates whether the application will perform a full recalculation when one was not
  indicated by other calculation properties".
- iterate (false), iterateCount (100), iterateDelta (0.001): "The application stops calculating after
  <iterateCount> iterations or after all values in the circular reference change by less than <iterateDelta> between
  iterations, whichever comes first."
- fullPrecision (schema default true): false "indicates the application uses the display values of the referenced
  cells when performing calculations".
- calcId (none): "Specifies the version of the calculation engine used to calculate values in the workbook."
- concurrentCalc (true): "indicates whether concurrent calculation processes are enabled for this workbook".
- refMode (A1): "Specifies the reference style for this workbook."

Summary: manual mode overrides fullCalcOnLoad.

## 8. Apache POI
URL(s) fetched: poi.apache.org/components/spreadsheet/eval.html (2026);
raw.githubusercontent.com/apache/poi/trunk/poi/src/main/java/org/apache/poi/ss/{usermodel/FormulaEvaluator.java,formula/FormulaCellCacheEntry.java,formula/CellCacheEntry.java}.
- cached results: "this class keeps a cache of all previously calculated intermediate cell values".
- invalidation: "If you do end up making changes to cells part way through evaluation, you should call
  notifySetFormula or notifyUpdateCell to trigger suitable cache clearance."
- dependents tracked: internally — "Stores the cached result of a formula evaluation, along with the set of
  sensitive input cells"; "Calls formulaCell.setFormulaResult(null, null) recursively all the way up the tree of
  dependencies." A public API: not stated.
- setForceFormulaRecalculation: "Delegate re-calculation to Excel. The application will perform a full
  recalculation when the workbook is opened:"
- volatile, OFFSET/INDIRECT: not stated.

Summary: evaluation on request; each cached result remembers the cells it read, and the caller must report changes.

## 9. Formualizer (0.13)
URL(s) fetched: raw.githubusercontent.com/psu3d0/formualizer/main/README.md (2026-11-09) and
.../main/docs-site/content/docs/{core-concepts/dependency-graph-and-recalc,core-concepts/spill-behavior,guides/workbook-edits-and-batching,guides/deterministic-testing,reference/functions/reference-fns/indirect,reference/functions/reference-fns/offset}.mdx
(source of formualizer.dev/docs).
- graph: "Dependencies are tracked by region, so editing one input touches only the cells that read it."
  "Recalculation visits dirty subgraphs instead of the entire workbook."
- on-demand: "After staging formulas, run `evaluate_cell`, `evaluate_cells`, or `evaluate_all` explicitly where your
  flow needs computed values."; "demand-driven `evaluate_cell()`". Automatic/Manual modes: not stated.
- INDIRECT: "The function is volatile because target references can change without direct dependency links."
  OFFSET: "`OFFSET` is volatile and returns a reference".
- volatile list, precedents/dependents API: not stated.
- spill: "`A1#` refers to the whole current spill of the anchor in `A1`, and follows it as it grows or shrinks."
  "A blocked or erroring anchor makes its `A1#` readers return `#REF!`."
- cycles: "When a circular dependency is found, the anchor cell receives a `#CIRC!` error."; runtime detection and
  iteration are opt-in.

Summary: a region-based graph with dirty propagation and explicit, optionally per-cell, evaluation; INDIRECT and
OFFSET are plain volatile.

## 10. IronCalc
URL(s) fetched: raw.githubusercontent.com/ironcalc/IronCalc/main/{README.md,base/src/evaluation.rs,docs/docs.ironcalc.com/src/features/unsupported-features.md}
(2026-11-04); www.ironcalc.com/roadmap.html.
- strategy: "Evaluates every formula in the workbook." "to evaluate a cell, evaluate the cells it reads as it meets
  them. A cell met while it is already being evaluated closes a cycle."
- spill: "when an anchor is about to contradict a record, the pass is abandoned and started again with that anchor
  before the cells that read it".
- cycles: "A marked anchor stores `#CIRC!` and keeps no spill cells".
- dependency graph, incremental recalculation, volatile functions, modes: not stated.

Summary: full recalculation by memoized recursion on each `evaluate()`; no persistent graph is documented.

## 11. Google Sheets
URL(s) fetched: support.google.com/docs/answer/58515?hl=en (2026); support.google.com/docs/answer/3092981?hl=en (NOW).
- settings: "Automatic mode instructs Sheets to look for and apply changes as soon as you make a spreadsheet
  action." "Manual mode only calculates the document when you select Calculate at the top of the screen." The
  on-change, every-minute and every-hour options: not on the page as fetched.
- functions: "volatile functions such as TODAY, NOW, RAND, and RANDBETWEEN, because those values inherently change
  all the time".
- NOW: "Note that NOW is a volatile function, updating on every edit made to the spreadsheet".

Summary: the page now documents Automatic and Manual modes and names four volatile functions.

## 12. Gnumeric
URL(s) fetched: gitlab.gnome.org/GNOME/gnumeric/-/raw/master/src/{dependent.c,dependent.h,expr.c,func.h,workbook.c}
(dependent.c at 1e1b638d2f55, 2026-10-21) and .../master/plugins/fn-lookup/functions.c.
- dynamic dependencies: `dependent_add_dynamic_dep (GnmDependent *dep, GnmRangeRef const *rr)`; "When a dynamic
  dependent changes, we mark its container." Source obs.: expr.c calls it whenever a function call or a range
  operator yields a cell range; the set is cleared at the start of each `dependent_eval` (also on unlink and in
  `dependent_set_expr`) and rebuilt by the evaluation. 3-D targets link as ranges: "A single-cell 3D reference
  (Sheet1:Sheet3!A1) is not a singleton".
- volatile: `GNM_FUNC_VOLATILE`, "eg now(), today()". Source obs.: `indirect` and `offset` are registered
  `GNM_FUNC_SIMPLE`.
- ranges: "Large ranges hashed on 'range' to accelerate duplicate culling."
- mode: "Sets the recalculation mode for @wb." (per-workbook `recalc_auto`).

Summary: the only engine here that records, at evaluation time, the range a function returned as a dependency of
the calling cell; INDIRECT and OFFSET need not be volatile.

## 13. Univer (optional)
URL(s) fetched: raw.githubusercontent.com/dream-num/univer/dev/packages/engine-formula/src/engine/dependency/formula-dependency.ts
(2026-11-10).
- "Calculate the range required for collection in advance, including references and location functions (such as
  OFFSET, INDIRECT, INDEX, etc.)."
- "RAND, RANDBETWEEN, NOW, TODAY are volatile functions that are marked dirty and recalculated every time a
  calculation occurs."

Summary: address functions are pre-calculated at dependency-build time to obtain their ranges.

## Addendum for point (d), beyond the list
URL(s) fetched: support.microsoft.com/en-us/office/sumif-function-169b8c99-c05c-4483-a712-1697a653039b and
.../lookup-function-446d94af-663b-451d-8251-369d5e3864cb.
- SUMIF: "Sum_range should be the same size and shape as range. If it isn't, performance may suffer, and the formula
  will sum a range of cells that starts with the first cell in sum_range but has the same dimensions as range."
- LOOKUP: "The result_vector argument must be the same size as lookup_vector."; a resizing rule: not stated.

## Cross-engine table
Numbers are sections; S1 and S2 are the two sections at the top; the LibreOffice names are our mapping of the opcodes in 2.

| Design point | Excel (documented) | HyperFormula | LibreOffice | POI | Formualizer | IronCalc | Google Sheets | Gnumeric |
|---|---|---|---|---|---|---|---|---|
| Volatile set | NOW, TODAY, RANDBETWEEN, OFFSET, INDIRECT; INFO, CELL, SUMIF by arguments (S1) | RAND, RANDBETWEEN, NOW, TODAY (1) | RAND, TODAY, NOW, INFO, INDIRECT, OFFSET, lambda callers (2) | ? | NOW, TODAY, INDIRECT, OFFSET named (9) | ? | TODAY, NOW, RAND, RANDBETWEEN (11) | per-function flag (12) |
| OFFSET / INDIRECT | volatile (S1) | OFFSET rewritten at parse time, literals only; no INDIRECT (1) | always recalculated (2) | ? | volatile (9) | ? | ? | dynamic dependency recorded at evaluation; not volatile (12) |
| Ranges in the graph | ? | range nodes, built from shorter range plus last row (1) | area listeners in slots (2) | cells read, per cached result (8) | by region (9) | no graph documented (10) | ? | range hash plus single-cell hash (12) |
| Manual / suspended mode | three modes; application-wide, from first workbook (3, 4, 7) | suspend/resume bracket; reads throw (1) | AutoCalculate toggle; F9 (2) | never automatic (8) | explicit evaluate calls (9) | explicit `evaluate()` (10) | Automatic, Manual (11) | per-workbook flag (12) |
| Lazy / on-demand | ? (Range.Calculate, Range.Dirty, 3) | no (1) | ? (FORCED flag hints at it, 2) | yes, cached (8) | `evaluate_cell` (9) | ? | ? | ? |
| Precedents / dependents API | Range.Precedents etc., active sheet (5); Office.js (S2) | direct: cells, ranges, names (1) | Detective arrows, per level (2) | ? (internal, 8) | ? | ? | ? | ? |
| Spill as dependency | ? | dynamic arrays not offered (1) | ? | ? | `A1#` follows the anchor (9) | reads recorded; restart with anchor first (10) | ? | ? |
| Cycles | iterate, count 100, delta 0.001 (7) | #CYCLE!, parse-time dependencies (1) | ? | ? | #CIRC!; runtime and iterative opt-in (9) | #CIRC!, met in recursion (10) | ? | recursion flag, iteration settings (12) |
