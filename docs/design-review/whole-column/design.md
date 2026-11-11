# Design draft: whole-column ranges in the read index (Taskei P559330849, first half)

*poc-design-cc, 2026-11-11. A draft for the read-index lane (poc-s-e2-1-cc) to read. It would land as SPEC-0002
Design 11.5, after T-5145's Designs 9 to 13 and after T-5132. The cycle half of P559330849 is not designed here.*

## What it is for

After Design 11.1 a formula with a whole-column range (`SUM(A:A)`, `VLOOKUP(x,A:C,2,0)`) is always dirty: it is
re-run at every edit. That is far better than today's full pass, but it is not what the workbook needs: the formula
can only change when its columns are written. The corpus census counts a whole-column or whole-row reference in 96
of 1,013 workbooks with formulas (docs/research/corpus-ontology-census-2026-11-09.md), and it is the usual argument
of the conditional aggregates (`SUMIF(A:A,x,B:B)`), which Designs 12 and 13 bring into the index.

## Why the index refuses it today

`read_index/walk.rs` `instantiate` refuses a range whose last row is the grid's: "a whole column or row is evaluated
over the sheet's used area by some functions, so a write anywhere on the sheet can change what it holds". What
follows the used area (read in `used_area.rs`, `cast.rs`, `tailed_array.rs`, `lookup_table.rs`):

- `clip_to_used_area` cuts a whole-column range at the sheet's last used row, whatever column set that row. The
  cells cut off are blank, so for a walker that skips blanks the **value** is a function of the cells of the
  range's columns alone.
- The **work** can follow the sheet's last used row: an operator over the range (`A:A*2`) makes an array as long
  as the cut, and a tailed argument (SPEC-0020) has a head that ends at that row. A lookup's search is cut there
  too (Design 1, "A search's cut").
- A whole-**row** range is cut at the last used **column**, which the index does not follow.

## The rule (proposed)

- A range whose rows are the whole column, and whose columns stop before the last column, is filed as its
  rectangle (a large rectangle, `read_index/large.rs`: one interval per column) **and** as a head over its whole
  row span on its sheet. So its reader is dirty when a cell of its columns is written, and when the sheet's last
  used row moves at all.
- A whole-row range, and a range that reaches the last column, stay unmodelled: their formulas are always dirty
  (Design 11.1).
- Nothing else changes: the formula's functions must be on the allowlist, and a search still counts as a search.

## Why it would be exact

Design 4 asks a clean cell's value and work to be functions of what the index tracks. The value is a function of
the columns' cells, which the rectangle files. The work is a function of those cells and of the sheet's last used
row, which the head files: any move of that row dirties the reader, so no clean reader was computed with another
row. This is Design 1's "The used area" rule with the widest window, so it over-dirties and never under-dirties.

## What it costs

- A sheet that grows by appended rows dirties every whole-column reader on it at each append. For a few totals
  that is nothing. For ten thousand `SUMIF(A:A,…)` it is every edit, as under 11.1: no worse, and precise for
  edits inside the used rows.
- Memory: one large rectangle per distinct range, shared by its readers, and 20 bytes per (formula, sheet) head.

## What it must show

- T-5149's oracle (`work_that_follows_the_used_area_is_always_in_the_closure`) and its lookup twins over
  whole-column shapes of every allowlisted reader: plain aggregates, operators over the range, tailed arguments,
  INDEX, VLOOKUP and MATCH tables, at the last used rows it sweeps and with writes in other columns.
- T-5184's hook test: every tailed read and every search cut of a whole-column argument is registered.
- The metamorphic generators gain whole-column ranges; the red team's `rt-index-whole-column`: an unread edit is
  skipped.
- Mutations: the head left out; the rectangle filed without its last rows.

## Open questions

1. Is "any move of the last used row dirties the reader" acceptable as the first rule, with a tighter window later
   if a measurement asks for it? (mine: yes.)
2. Is there a reader on the allowlist whose **value** for a whole-column range follows the used area? I found
   none by reading the clip's callers, but the oracle above is what decides.
