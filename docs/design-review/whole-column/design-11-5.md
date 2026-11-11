**11.5 Whole-column ranges** (amendment for Taskei P559330849, first half, 2026-11-11; it goes after T-5132, whose
list holds these formulas until this rule models them, and after 11.3's rule on a full pass that moves a sheet's
last used row. The cycle half of P559330849 is Design 14. Revised after an independent read, which measured each
point below on main: `/out/drafts/whole-column/experiments/`).

- **What it is for.** After 11.1 a formula with a whole-column range (`SUM(A:A)`, `VLOOKUP(x,A:C,2,0)`) is always
  dirty: it is re-run at every edit. That is far better than a full pass, but it is not what the workbook needs:
  the formula can change only when its columns are written or the sheet grows or shrinks. The corpus census counts
  a whole-column or whole-row reference in 96 of 1,013 workbooks with formulas
  (docs/research/corpus-ontology-census-2026-11-09.md), and it is the usual argument of the conditional aggregates
  (`SUMIF(A:A,x,B:B)`), which Designs 12 and 13 bring into the index.
- **Excel.** Nothing to match: what the index files is not observable. The values are a full recalculation's.
- **Why the index refuses it today.** `instantiate` (`read_index/walk.rs`) refuses a range whose last row is the
  grid's: "A whole column or row is evaluated over the sheet's used area by some functions, so a write anywhere on
  the sheet can change what it holds". What a whole-column read depends on, beside the cells of its columns:
  - **The sheet's last used row**, and nothing else of the sheet. Every cut reads `SheetData::last_row`
    (`sheet_data.rs`): the aggregates' walk (`clip_block_to_used_area`), the value reads (`clip_to_used_area`),
    the tailed reads (`tailed_array.rs`), the running totals (`sum_prefix.rs`), the lookups' search
    (`lookup_table.rs`, `clip_search_rows`), the existing-cell walk (`existing_walk.rs`), and the range arms of a
    lifted IF or CHOOSE (`read_range_cells`, `cast.rs`: `=SUM(IF(A1:A3>0,B:B,0))` costs 3 units more for each
    used row). Measured over 48 whole-column shapes (aggregates, tailed, lifted, INDEX, MATCH in its three modes,
    VLOOKUP, HLOOKUP, picks through IF and CHOOSE, `@`, COUNTBLANK): value and units are the same whatever the
    last used column, the rows of other columns, hidden rows and row heights.
  - Not every reader follows it. An operator over the range builds and charges all the range's rows
    (`evaluate_range`): `=A:A*2` costs 3,145,740 units at last used rows 2,000, 3,000 and 70,000. The head below
    is filed all the same: it is the range's, not the reader's.
  - A whole-**row** range is cut at the last used **column**, which the index does not follow.
  - This holds because a single range's sparse walk is free (`WalkBudget::Free`). SPEC-0028 section 11's planned
    charge for range walks must count the cells handed on, never the rows searched, or it adds the rows of other
    columns as an input, as a union's walk has (11.2).
- **The rule.**
  - A written range that reaches the last row (`A:A`, `A2:A1048576`) and whose columns stop before the last column
    is filed as its rectangle (a large rectangle, `read_index/large.rs`: one interval a column up to 64 columns,
    and an entry of the list every query scans beyond that) **and** as a head over the whole row span, (0, last
    row), on the range's own sheet: each spanned sheet for a 3-D span. So its reader is dirty when a cell of its
    columns is written, and when that sheet's last used row moves at all.
  - The head is filed for every such range, whatever function reads it. It is not found by the head walk, which
    finds tailed reads and searches only.
  - A whole-row range, and a range that reaches the last column, stay unmodelled: their formulas are always dirty
    (11.1).
  - A whole-column range inside a union follows 11.2: the union stays unmodelled. One given through a name is
    filed by this rule in place of 11.2's refusal.
  - A footprint rule's rectangle that reaches the last row (Design 12, `SUMIF(A:A,x,B:B)`) is filed by this rule
    too, in place of Design 12's "stays always dirty".
  - Nothing else changes: the formula's functions must be on the allowlist, and a search still counts as a search
    (Design 4, "A search").
- **Why it is exact.** Design 4 asks a clean cell's value and work to be functions of what the index files. The
  value is a function of the columns' cells, which the rectangle files. The work is a function of those cells and
  of the sheet's last used row, which the head files: any move of that row dirties the reader, and a full pass
  that moves the row inside itself records it as unknown (11.3), so no clean reader was computed with another
  row. This is Design 1's "The used area" rule with the widest window, so it over-dirties and never
  under-dirties.
- **What it costs.**
  - A sheet that grows by appended rows dirties every whole-column reader on it at each append. For a few totals
    that is nothing. For ten thousand `SUMIF(A:A,…)` it is every edit, and a little worse than under 11.1: there
    a list whose closure passes the limit is remembered, and a closure with moved rows is not (Design 3), so each
    append walks to the limit before it gives up. Inside the used rows it is precise.
  - Memory: one large rectangle a distinct range, shared by its readers, and 20 bytes a (formula, sheet) head.
- **Structural edits** (Design 8; with T-5136). A row insert or delete leaves a whole-column range's text as it
  is but not what it names: rows shift inside it and a delete takes cells out. With A1:A10 = 1 to 10, an insert
  at row 3 turns `INDEX(A:A,5)` from 5 into 4 and `MATCH(7,A:A,0)` from 7 into 8, and a delete of row 3 turns
  `SUM(A:A)` from 55 into 52; the same through a name `Sales = Sheet1!$A:$A`, whose text and rectangle do not
  change (measured). The head does not take this: Design 8 maps the recorded row through the edit. So every
  reader of a whole-column rectangle on the edited sheet, written or given through a name, is in D0 of every row
  insert and delete of that sheet, whatever T-5123 settles for whole-line spans; for such a rectangle this
  replaces 11.2's name test. A column edit shifts, grows or shrinks it as any range.
- **Maintenance.** A refile applies the rule as the build does.
- **`precedents` and `dependents`** (Design 10). Unchanged: the written area is the whole column, as written.
- **What the implementation may rely on.**
  1. Every cut of a whole-column read reads `SheetData::last_row`, which already notes the read
     (`self.reads.extent()`). The head is filed for every whole-column range, whatever reads it.
  2. `Model::moved_last_rows` gives the sheets whose last used row moved since the last pass (Design 3), and a
     head of (0, last row) is met by every move (`read_index.rs`).
- **What it must show.**
  - T-5149's oracle (`work_that_follows_the_used_area_is_always_in_the_closure`) and its lookup twins over
    whole-column shapes of every allowlisted reader: plain aggregates, operators over the range, tailed arguments,
    INDEX, VLOOKUP and MATCH tables, and the lifted shapes `SUM(IF(A1:A3>0,B:B,0))`, `COUNT(IF({TRUE},A:A,C:C))`
    and `@IF(A:A>3,A:A,C:C)`, at the last used rows it sweeps, above row 1,044,481 and at a first write to an
    empty sheet among them.
  - The value and work oracle, with writes that do **not** move the last used row: a cell of another column
    inside the used rows; a cell that moves the last used column only; other columns' rows at 65,536 positions
    or more with over 16 used columns; a second copy of the formula. Each asserts that the reader is not in the
    closure and that its value and units equal a full recalculation's.
  - Precision on the reader's own sheet: an unread edit of the sheet that holds the range, inside the used rows,
    is skipped (the red team's `rt-index-whole-column` edits another sheet).
  - A range on another sheet: `=SUM(Sheet1!A:A*2)` on Sheet2 follows Sheet1's last used row and not Sheet2's.
  - Structural edits: `INDEX`, `MATCH` and `SUM` over `A:A` and over a name, with a row insert and a row delete
    inside the used rows, equal a full recalculation.
  - A full pass that moves the row inside itself (11.3's fixture, with `=INDEX(A:A*2,5)` in K1), then an unread
    edit: K1 is in the closure.
  - The metamorphic generators gain whole-column ranges.
  - Mutations: the head left out; the head filed only for tailed or searched arguments; the head on the reader's
    sheet; the head with the tailed window in place of (0, last row); the rectangle filed without its last rows;
    a whole-column rectangle classified unchanged by a row edit.
