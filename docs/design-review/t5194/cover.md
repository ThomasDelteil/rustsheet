# T-5194: a styled blank cell changes no work (SPEC-0028 section 12) — draft for a design read

*poc-design-cc, 2026-11-11. The text is `section-12.md` beside this file (branch `agent/design/T-5194-styled-blank-cells`,
568deadfb, 7 files, +240/-5). T-5190 implements it. It goes to the cr-broker once the owning lanes have read it.*

## In one page

- **The gap.** Formatting a cell that holds nothing stores a cell. The engine's work accounting counts stored cells
  (the used area's end, whether a walk is dense or sparse, the union and lifted charges, the read index's used-area
  input), so a format on a blank can change a formula's value at the work limits. T-5190 has three reproducers.
- **Why neither obvious fix works.** Charge the walks for the blanks they visit, and a format changes the work by
  design. Leave the blanks uncharged and skip them, and a walk's time follows cells its charge does not count: a
  LAMBDA that repeats `MAX(A:A)` over a million styled blanks does 10^9 visits for nothing.
- **The design.** A stored blank's style leaves the cell table for a small layer of its own (row, column, style).
  The cell table then holds cells with content only, so every count and extent follows content with no new rule,
  and a walk cannot tell a styled blank from an empty position. The time bound follows from that: a walk over
  styled blanks costs what it costs over nothing. LibreOffice keeps formats apart from cells in the same way.
- **What does not change.** Which blanks are stored (decision 0421), what a host sees as the used range, what a file
  holds. A stored blank goes from a full cell entry to 8 bytes.
- **Found on the way.** One of T-5190's four cases (a row record past the content, after a save and a load) is
  already closed by decision 0421. Excel documents two used ranges, one with formatting and one with values only;
  both are in Graph, so a probe is cheap.

## Open questions for the project lead

1. A stored blank's style moves out of the cell table, which changes the engine's saved byte form once more
   (M4's storage change does too). (yes/no; mine: yes, and the two land as one format step if their timing allows.)
2. `Workbook::used_range` keeps counting formatted blanks, as Excel's used range is documented to; a values-only
   extent for hosts is a later, separate change. (yes/no; mine: yes.)

## Questions for the owning lanes

- **Engine** (T-5190; the stored-cell walk's charge, Taskei P559664616): is "no evaluation path can tell a stored
  blank from an absent cell" true of every reader you know? I found three readers of the layer beside styles and
  export: pivot refresh, the data-table memo (which I propose keeps no style for a position with no content) and
  decision 0421's functions. Is `EmptyCell` out of the row vectors acceptable, or is there a reader that needs it
  in place?
- **M4 storage** (SPEC-0045): the layer mirrors the chunk table's shape and is empty for a sheet with no blank, so
  your cell models do not move. Does anything in your Part 2 sketch want it shaped otherwise? Can the byte form
  change once for both?
- **SPEC-0028's owner** (section 11): the sentence "a trip is a stored cell" gains one sentence that points to
  section 12; rule 1 is otherwise untouched. Is section 12 where you want this, and is the bound argument sound?
- **Read index** (SPEC-0002 "The used area"): the last used row becomes the last row with content; a format on a
  blank is no longer a move. One sentence is added there.
