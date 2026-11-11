**14. The cells on a circular reference** (amendment for Taskei P559284750, 2026-11-11; the rules marked *(probe
Cn)* are provisional until the probe list at its end is answered. Revised after an independent read, which ran
each case below on a copy of the engine).

- **What it is for.** Excel tells the user that a workbook has a circular reference, and where: the status bar
  shows "Circular References" and one cell, and Formulas > Error Checking > Circular References lists cells. The
  collaborative sheet shows "Ready". The values are already Excel's (recalc.md, "A circular reference without
  iteration keeps the cells' values from before the recalculation"); what is missing is the list.
- **Excel**, as documented (Microsoft support, "Remove or allow a circular reference in Excel", and VBA
  `Worksheet.CircularReference`; fetched 2026-11-11):
  - "The status bar can show Circular References and one cell address." "If circular references exist on another
    worksheet, you might only see Circular References without an address."
  - "Go to Formulas > Error Checking > Circular References." "Select each listed cell address and update the
    formula so it no longer points back to itself."
  - "Excel shows a warning message the first time a circular reference is created in a session." "Later circular
    references might not show the same warning dialog."
  - `Worksheet.CircularReference`: "Returns a Range object that represents the range containing the first
    circular reference on the sheet, or returns Nothing if there's no circular reference on the sheet." Its
    example "selects the first cell in the first circular reference", so the range can hold more than one cell.
  - Not documented: which cell is "the first"; whether the menu lists every cell of a loop or one a loop;
    whether a cell that only reads a loop is listed; what both show with two loops, with a loop on another
    sheet, with a loop through a spilled array, and with iterative calculation on; when the warning comes in
    Manual mode.
- **The engine today.** After each evaluation `evaluation.circular` holds every cell on a loop and every cell
  that read one (`circular.rs`, `mark_stack_circular`: "every cell on the stack is on the loop or reads it,
  however indirectly"), and the dynamic anchors the restart log marks circular. Nothing public reads it, and it
  does not tell a loop's own cells from their readers.
- **Members and readers.** A formula cell of the circular set is a **member** when a chain of reads made in the
  recalculation leads from it back to itself. Every other cell of the set is a **reader**. The lists hold
  formula cells only: a read of a member of a Ctrl+Shift+Enter array is a read of its anchor.
  - This is a function of the document and of the values the loop's cells keep, not of where the pass entered
    the loop or of the order of a formula's operands (and, for a formula that chooses its reads by a draw, of
    the draws, as its value is: 11.1, "Volatile draws"): a cell on the stack goes on evaluating after a read
    closes a loop, so every read its formula makes with the kept values is made. With B1 `=C1+D1`, C1 `=B1`,
    D1 `=C1` the members are B1, C1 and D1, whatever is typed next and whichever cell a pass reaches first.
    "The stack from the re-entered cell to the top", the first draft's rule, called D1 a reader there, and a
    member once A1 `=D1` was added or B1 was written `=D1+C1` (measured).
  - It is computed as the pass runs, by the path-based strongly-connected-components rule on the evaluation
    stack: a stack of boundaries beside the stack of cells; a read that closes on a cell of the stack pops the
    boundaries above that cell's frame; a read of an evaluated cell whose component is still open (its root is
    on the stack) does the same from that component's root; a cell that is popped as the root of a component of
    more than one cell, or that read itself, closes a component of members. The work is linear in the cells and
    the reads. One high-water mark does not do it: members need not be one segment of the stack, and
    `mark_stack_circular` visits only the cells pushed since the last marking, so a split done inside it misses
    a loop closed below that mark (with A1 `=A1`, B1 `=C1`, C1 `=A1+B1`, the read of A1 marks B1 and C1 as
    readers before the read of B1 closes their loop).
  - **An anchor the driver marks** (`RestartLog::circular`: a dynamic anchor whose spill a read contradicts at
    every order, as A1 `=SEQUENCE(A2+2)`) is a member, with iterative calculation on too. It never passes the
    evaluator's `Evaluating` arm, so the rule above does not see it.
  - The new stack state is reset wherever the stack is: a data table's substitutions, a blocked root, the replay
    after an unwind.
- **The lists' life.** The engine keeps the two lists apart from `evaluation.circular`, sorted, with a flag that
  says whether they are **known**.
  - A full pass sets them. A restricted pass can find a loop too, a formula that comes to read itself (B1
    `=IF(A1=0,7,B1)` after A1 goes from 0 to 1: a self-reference does not nest, so the pass is kept), and then
    it sets them: every member and reader of the new loop is in its closure. After that `can_restrict` is false
    (`found_circular`), and every pass is full until the loop is gone.
  - A scoped run (Design 9) leaves the lists and the loop record as they were, with one addition: the members of
    a loop it finds among its own cells are added, so a formula typed in Manual mode that refers to itself is
    reported when it is entered *(probe C9)*. A loop that an entry closes through a cell its transaction did not
    write is found at `calculate`, since the entry reads that cell as stored *(probe C7)*.
  - A cell that an edit overwrites or clears leaves both lists at that edit.
  - They are **not known** after a load without recalculation (the file says nothing of its loops), after a
    recalculation that left normal mode or was cancelled (in share mode a loop's cells hold `#VALUE!` and the
    set is empty: measured), and after `set_iteration`, until the next full pass.
  - They are not known either after a structural edit, or the removal of a sheet, that no full pass has
    followed, which is the case in Manual and OnDemand modes (Design 9: such an edit owes everything). The
    lists name cells by their place, and its lines moved: a list kept as it was would name other cells.
  - Otherwise, in Manual and OnDemand modes they can be stale, as the values are.
- **The API.**
  - Engine: `Model::circular_members()` and `Model::circular_readers()`, each a sorted slice in (sheet, row,
    column) order, and `Model::circular_known()`.
  - Facade: `Workbook::circular_references() -> CircularReferences { members: Vec<CellRef>, readers:
    Vec<CellRef>, known: bool }`, by `&self`, bounded by the formula cells of the workbook.
  - `ChangeSet::circular_new: Vec<CellRef>`: the members after this call that were not members at the last
    moment the lists were known, in (sheet, row, column) order; empty when the lists are not known now. `edit`,
    `calculate`, `calculate_sheet`, `compute` and `set_calc_mode` carry it: a loop's cells keep their values, so
    a loop found at `calculate` reaches a host through no value change. A UI warns when it is not empty *(probe
    C6)*. A boolean cannot tell a new loop from an old one recalculated (the UI lane's read).
  - The engine worker. The collaborative UI reads the engine only through the worker's protocol, never the
    facade, so the lists are there too (the UI lane's need): `RecalcResult` carries `circularNew: CellRef[]` on
    every op that evaluates (a load, a cell write, a name write, rows hidden, a recalculation), empty when the
    lists are not known; and one query, `circularReferences`, returns `{ members, readers, known }`. A UI asks
    it when the menu opens and after a `circularNew` that is not empty. When `known` is false it says so ("not
    known until the sheet recalculates") instead of showing an empty list.
  - The iterative-calculation setting is on the facade already (`Workbook::iteration`), so a UI can skip the
    warning when it is on.
- **Which cell the status bar names, and the menu's order** *(probes C1, C3)*. Provisionally the members are in
  (sheet, row, column) order, the status bar names the first on the active sheet, and it shows no address when
  the members are all on other sheets, as the documentation says. If the probes show another order, `members` is
  returned in Excel's order with the named cell first.
- **Iterative calculation.** With iteration on, a loop is not an error and Excel shows no warning *(probe C5)*.
  The cycle's cells are the iteration's business (`iteration.rs`) and are in neither list. The lists then hold
  only the anchors the driver marks, and their readers *(probe C8)*.
- **The collaborative sheet.** The lists are a function of the evaluation, and so of the values a loop's cells
  keep, which today depend on a replica's history (with X1 `=Y1`, Y1 7 evaluated, then Y1 `=X1`, D1
  `=IF(X1=7,E1,0)` and E1 `=D1`, a replica with that history has four members and one that loads the document
  has two: measured). So the lists are per replica until T-5288 gives the wasm engine loops evaluated as on a
  fresh open; after it, replicas that hold the same document have the same lists.
- **Designs 9 and 13.** A scoped run's rule is above. Design 13 enters a cycle through a non-candidate at
  another member than today's pass; the members are the same, since they do not depend on the entry.
- **Who builds what.** The lists, the facade call and `ChangeSet::circular_new` are the engine lane's. The
  worker's two fields are for the lane that owns the engine worker, which is not on main yet (the engine-js port
  of the collaborative UI). The warning, the status bar and the menu are the UI's (Taskei P559284750).
- **What the implementation may rely on.**
  1. A cell on the stack goes on evaluating after a read closes a loop, and no closing read happens while the
     recursion unwinds or the pass is abandoned (`evaluation.rs`), so the reads made are those of the formulas
     with the kept values.
  2. `found_circular` is recomputed from the circular set after every evaluation (`recalc_budget.rs`); a scoped
     run must not let that clear the record of a loop outside its cells (Design 9).
- **What it must show.**
  - Unit tests, each with its members and readers: a two-cell loop with one reader; a self-reference; a loop
    that exists only for some inputs (`=IF(A1=0,7,B1)`), found by a restricted pass; two loops; a reader of two
    loops; a loop through a SUM range (members A3 and A5, reader B1); a loop on another sheet; the three-cell
    case above in its three writings, with the same members; the A1, B1, C1 case above (members B1 and C1 beside
    A1); a marked anchor and its reader, with iteration off and on; a 30,000-cell loop found through an unwind
    and a replay.
  - The lists after each edit of the metamorphic suite's loop generator equal the lists of a fresh full
    recalculation of the same history, and do not change when the pass enters the loop elsewhere (the anchors
    reversed, Excel's order switched on by an order-state reader that is then removed).
  - `known`: false after a load without recalculation, after a share-mode recalculation and after
    `set_iteration`; and the loop that was there before is not reported as new by the next normal pass.
  - `circular_new` at `calculate` in Manual mode for a loop closed by an entry through an unwritten cell; empty
    for an edit that only adds a reader of a loop.
  - A member overwritten with a literal leaves the lists at the edit, in Manual mode too.
  - The cost: `c_N = c0 + r1 + c0 + r2 + …`, where each `r_i` reads a fresh two-cell loop, marks in time linear
    in the cells, by a counter of the test build.
  - Mutations: the boundary stack replaced by the stack segment; a marked anchor left out; a scoped run that
    clears the lists; `circular_new` computed against lists that were not known.
- **Probes** (the request, with each case's steps, is `docs/research/circular-references-probe/`). No API reads
  Excel's list on the web (Office.js has none; `Worksheet.CircularReference` is VBA),
  so each case is a person's reading of the status bar and of the menu, in Excel for the web first, which is the
  set of record (decisions 0266 and 0039); a desktop reading, with `CircularReference.Address`, is a second set
  and counts only signed (decision 0427). Until then each rule above stays UNVERIFIED in
  docs/excel-semantics/recalc.md.

  | Case | Workbook | Question |
  |---|---|---|
  | C1 | A1 `=B1`, B1 `=A1`, typed in that order; then the same typed B1 first | Which cell does the status bar name? Does the typing order change it? |
  | C2 | C1, and D1 `=A1+1` | Is D1, which only reads the loop, in the menu's list? |
  | C3 | two loops, A1/B1 and A5/B5 | Does the menu list both loops, and every cell of each or one each? Which cell does the status bar name? |
  | C4 | a loop on Sheet2, Sheet1 active | The status bar's text; the menu on Sheet1 |
  | C5 | C1 with iterative calculation on | The status bar and the menu |
  | C6 | C1, close the warning, then type a second loop | Is the warning shown again? After closing and reopening the workbook? After an undo that brings a loop back? |
  | C7 | Manual mode; A1 `=B1+1` settled, then type B1 `=A1` | Is the warning shown when B1 is entered, or at F9? What do A1 and B1 show in between? |
  | C8 | A1 `=SEQUENCE(A2+2)`, C1 `=A1+1`; then the same with iterative calculation on | What do A1 and C1 show? Is A1 in the menu's list, and C1? The status bar's text |
  | C9 | Manual mode; type A1 `=A1+1` | Is the warning shown at the entry? Is A1 in the menu's list before F9? |
  | C10 | B1 `=C1+D1`, C1 `=B1`, D1 `=C1` | Does the menu list D1, which is on the loop through B1's second operand? |
