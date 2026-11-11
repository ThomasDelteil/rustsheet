# Design draft: the cells on a circular reference (Taskei P559284750)

*poc-design-cc, 2026-11-11. Read by the engine API lane (poc-engine-b-cc: buildable, with the correction and the
bound under "Members and readers") and the UI lane (poc-ui-cc: the change set needs the new members, not a boolean). It would land as a short amendment to SPEC-0002, after T-5145's Designs 9 to 13, since it is
dependency introspection like Design 10. Rules marked (probe Cn) are provisional.*

## What it is for

Excel tells the user that a workbook has a circular reference, and where. The status bar shows "Circular
References" and one cell, and Formulas > Error Checking > Circular References lists cells. The collaborative sheet
shows "Ready". The values are already Excel's (recalc.md, "A circular reference without iteration keeps the cells'
values from before the recalculation"); what is missing is the list.

## Excel

- Documented (Microsoft support, "Remove or allow a circular reference in Excel", fetched 2026-11-11):
  - "The status bar can show Circular References and one cell address." "If circular references exist on another
    worksheet, you might only see Circular References without an address."
  - "Go to Formulas > Error Checking > Circular References." "Select each listed cell address and update the
    formula so it no longer points back to itself."
  - "Excel shows a warning message the first time a circular reference is created in a session." "Later circular
    references might not show the same warning dialog."
  - "When you close the warning, Excel might display either 0 or the last calculated value in the cell."
- Documented (VBA, `Worksheet.CircularReference`, fetched 2026-11-11): "Returns a Range object that represents the
  range containing the first circular reference on the sheet, or returns Nothing if there's no circular reference
  on the sheet."
- Not documented, to probe on the desktop (C1 to C6 below): which cell is "the first"; whether the menu lists
  every cell of a loop or one per loop; whether a cell that only reads a loop is listed; what both show with two
  loops, with a loop on another sheet, and with iterative calculation on.

## The engine today

After each evaluation `evaluation.circular` holds every cell on a loop **and** every cell that read one
(`circular.rs`, `mark_stack_circular`: "every cell on the stack is on the loop or reads it, however indirectly").
Nothing public reads it, and it does not tell a loop's own cells from their readers. Excel's menu is about the
formulas "that point back to themselves", so the two must be told apart.

## The design

- **Members and readers.** When a read closes a loop, the cell read is on the evaluation stack, and the stack from
  that cell to the top is the loop: those cells are **members**. The cells below it on the stack, and every cell
  marked later because it read a circular cell, are **readers**: the readers are the circular cells that are not
  members. A cell that is a member of one loop and a reader of another is a member.
  - *Where* (poc-engine-b-cc's correction). `mark_stack_circular` and `mark_readers_of` do not know where the loop
    starts. The place that does is `evaluate_formula_cell`'s `CellState::Evaluating` arm in `evaluation.rs`, the
    branch that calls `mark_stack_circular` when iteration is off: there the cell just read is the re-entered one.
  - *The bound.* Marking members must stay linear in the cells marked. `circular_below` cannot be reused for
    them: a later loop can re-enter deeper and turn earlier readers into members. So the stack keeps each
    evaluating cell's position, which finds the re-entered cell without a scan, and a high-water mark of the
    members' segment, as `circular_below` is for the marking, so that a deep stack with many closing reads does
    not cost loop length times reads.
- **The API.**
  - Engine: `Model::circular_members()` and `Model::circular_readers()`, each in (sheet, row, column) order, for
    the last recalculation.
  - Facade: `Workbook::circular_references() -> CircularReferences { members: Vec<CellRef>, readers: Vec<CellRef> }`,
    by `&self`. It is bounded by the formula cells of the workbook.
  - `ChangeSet::circular_new: Vec<CellRef>`: the members after this edit that were not members before it, in
    (sheet, row, column) order; empty when the edit made no loop. The UI warns when it is not empty, which is
    Excel's "an edit made a loop that was not there" *(probe C6: whether Excel warns again for a second loop, and
    on opening a file that holds one)*. A boolean was the first draft; poc-ui-cc showed it cannot tell a new loop
    from an old one recalculated.
  - The iterative-calculation setting is already on the facade (`Workbook::iteration`), so a UI can skip the
    warning when it is on.
- **Which cell the status bar names, and the menu's order** *(probes C1, C3)*: provisionally the members are
  returned in (sheet, row, column) order, the status bar names the first on the active sheet, and it shows no
  address when the members are all on other sheets, as the documentation says. If the probes show another order,
  `members` is returned in Excel's order with the named cell first (poc-ui-cc's ask), and the API's order rule
  changes with them.
- **Iterative calculation.** With iteration on, a loop is not an error and Excel shows no warning *(probe C5)*:
  the lists are empty, and the cycle's members are the iteration's business (`iteration.rs`).
- **Calculation modes** (SPEC-0002 Design 9). The lists are those of the last recalculation. In Manual and
  OnDemand modes they can be stale, as the values are.
- **Incremental recalculation.** `Model::can_restrict` is false while the last pass found a loop, so every
  recalculation of such a workbook is a full pass, and the lists are always a full pass's.
- **The collaborative sheet.** The lists are a function of the evaluation. Under the collaborative profile loops
  are evaluated as on a fresh open (D-ENGINE-8), so every replica has the same lists.

## What the implementation may rely on

1. `mark_stack_circular` visits exactly the cells pushed since the last marking (`circular_below`), so the split
   costs one comparison per marked cell.
2. A restricted pass never runs while a loop is known, so there is no partial list to merge.

## What it must show

- Unit tests: a 2-cell loop with one reader (two members, one reader); a self-reference; a loop that exists only
  for some inputs (`=IF(A1=0,7,B1)`), empty and then one member; two loops; a reader of two loops; a loop through
  a SUM range; a loop on another sheet; a dynamic anchor on a loop.
- The lists after each edit of the metamorphic suite's loop generator equal the lists of a fresh full
  recalculation of the same history.
- With iteration on, both lists are empty.

## Probes (desktop Excel 365; `Worksheet.CircularReference` and the status bar)

| Case | Workbook | Question |
|---|---|---|
| C1 | A1 `=B1`, B1 `=A1`, typed in that order; then the same typed B1 first | Which cell do the status bar and `CircularReference.Address` name? Does the typing order change it? |
| C2 | C1 above, and D1 `=A1+1` | Is D1, which only reads the loop, in the menu's list? |
| C3 | two loops, A1/B1 and A5/B5 | Does the menu list both loops, and every cell of each or one each? Which cell does the status bar name? |
| C4 | a loop on Sheet2, Sheet1 active | The status bar's text; `Worksheets("Sheet1").CircularReference`; the menu on Sheet1 |
| C5 | C1 with iterative calculation on | The status bar, the menu, `CircularReference` |
| C6 | C1, close the warning, then type a second loop | Is the warning shown again? After closing and reopening the workbook? |

## Open questions

1. Members and readers as two lists, with Excel's menu fed by the members (pending C2)? (yes/no; mine: yes.)
2. Settled by the UI lane's review: the change set lists the cells that newly joined a loop, not a boolean.
3. Should the engine also name, for a member, the next cell of its loop (to draw Excel's tracer arrows around the
   loop)? (yes/no; mine: no; SPEC-0002 Design 10's `precedents` gives it.)
