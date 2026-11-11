## Amendment 3 (2026-11-11): insert and delete cells, a shift within a band
Home > Insert and Home > Delete are P0 cards of the collaborative sheet, and each has four choices: an entire
row, an entire column, and cells shifted down or right (up or left for a delete). The first two are this
spec's row and column edits. The other two are built nowhere: the editor has no dialog and no keys
(Ctrl+Shift+= and Ctrl+-), the document model has no planner, and the engine has no op (this spec's first
non-goal). The contract names the op, `DocOp::ShiftCells { sheet, area, dir }` with `dir` one of
`insert-down`, `insert-right`, `delete-up`, `delete-left` (I-03 at 1.0.0, `ops/docop.rs` on
`cr/f1/2-contracts-1.0.0-r3`, in review when this was written), and leaves its planning to the model's plan
pipeline, which no lane holds yet. This amendment is the engine's half and what that planner needs from it.

**A10. What a shift is.** An insert of the area `r1..r2` x `c1..c2` shifting down moves the cells of columns
`c1..c2` at or below row `r1` down by the area's height; shifting right moves the cells of rows `r1..r2` at or
right of column `c1` right by its width. A delete removes the area's cells and moves the cells past it back.
So a shift is a row or column edit confined to a band of the other axis: the same `AxisEdit` on the same
axis, applied only to the lines of the other axis the band names.

```rust
pub struct RefMap { pub sheet: u32, pub axis: Axis, pub edit: AxisEdit, band: Option<Band> }
/// The lines of the other axis a shift is confined to; no band is every line (a row or column edit).
pub struct Band { pub first: i32, pub last: i32 }
/// Where a thing that spans `first..=last` on the other axis lies.
pub enum Covered<'a> { Whole(LineMap<'a>), Part, Outside }
impl RefMap {
    /// A row or column edit covers every line whole.
    pub fn over(&self, first: i32, last: i32) -> Covered<'_>;
}
impl LineMap<'_> {
    pub fn line(&self, line: i32) -> Line;
    pub fn span(&self, first: i32, last: i32) -> SpanMap;
}
```

`line` and `span` are reached only through the answer to `over`, so no consumer of Design 1's table can map
a line without first saying where its thing sits on the other axis: today's `RefMap::line` and `RefMap::span`
go, and a consumer that forgot the band would not compile, where an optional field beside them would let it
apply a whole-line rule to a shift in silence (the design desk's finding). One definition of the arithmetic
stays (`AxisEdit`), and `ChangeSet.structure` still carries one type. `Outside` leaves the thing as it is,
`Whole` gives it today's rule, and `Part` takes the rule of the table below.

**A11. Excel behaviour to match, all unknown.** No probe case replays a shift today: the case format has
row and column ops only. Graph has the operation (`range/insert` and `range/delete` with a `shift`, the calls
the oracle already makes for whole lines), so each is probeable once the case format names it (T-5464).
Microsoft's page for the call says only that it "inserts a cell or a range of cells into the worksheet in
place of this range, and shifts the other cells to make space" (learn.microsoft.com/graph/api/range-insert,
fetched 2026-11-11): nothing on references, areas or refusals. The provisional rules below are from memory of
Excel and of how LibreOffice Calc updates references for a block shift (a reference follows only when its
extent on the other axis lies inside the block); neither is evidence, and T-5465 replaces them. Where a shape
is doubtful the provisional rule is a refusal, not a rewrite: a refused shift loses nothing.

| | Question | Provisional rule (UNVERIFIED) |
|---|---|---|
| U7 | A cell reference into the band, at or after the shift, before it, and outside the band | follows as under a line edit; stays; stays |
| U8 | A range the band covers whole: inside, across the shift's first line, wholly deleted | as under a line edit (shifts, grows or shrinks, `#REF!`) |
| U9 | A range across the band's edge (`A1:D9` under a shift of `B2:C3`), and one that holds the band, where cells under it move | the shift is refused until the probe says whether the range stays (the expected answer), follows or splits; a range of that shape under which nothing moves stays |
| U10 | Whole-row and whole-column references, 3-D spans, and a reference from another sheet | a whole line of the edited axis follows (the band covers it whole); a whole line of the other axis is U9's unless the band is every line; a span never shifts (E4) |
| U11 | Names: an absolute name into the band, one across its edge, a relative name | as formulas, U9 included; relative parts stay (E5) |
| U12 | Refusals: a CSE array or a spill the band cuts or the area splits; a merge across the band's edge or across the shift's first line; a table the moved cells touch but do not hold whole; a data table; a pivot; a nonblank cell pushed off the grid | each refused, with its `StructureRefusal` kind (`Merge` stops being reserved) |
| U13 | What moves with a cell besides its content: its style (a stored blank's too), a conditional format's area, a data validation's, a link, a note or a comment thread, a merge the band covers whole; and what an autofilter's range, a sparkline and a chart's series range do | each of the first group moves with the cells, and an area across the band's edge is U9's; the second group is refused where the moved cells touch one |
| U14 | The inserted cells' format, for a block and for a whole line alike (Excel's Insert Options offer "Format Same As Above" and its siblings for both; this spec has no rule for an inserted line's format either), and a table that the band holds whole | no style of their own, one rule for a line and a block; a table held whole moves as cells do |

The rules live in one function, a table from the consumer's kind and its `Covered` answer to what happens,
each row marked UNVERIFIED with its probe, in code and in `references.md`, as U1 and U2 are; the shift's
cases stay pending and out of the gate until T-5465 fills them, so a wrong provisional rule shows as a
failing cell the day its probe lands.

**A12. The engine op.**
- **Cells.** `SheetData::shift_band(axis, edit, band)` moves the band's cells as they are (Design 5): for a
  shift down or up, the run of entries of columns `c1..c2` of each row at or after the shift goes to the row
  `count` lines further or nearer; for a shift right or left, the entries at or after the shift in each row
  of the band are re-keyed in place. A stored blank's style (SPEC-0028 section 12's layer, T-5190) moves by
  the same rule, as section 12 says of every structural edit. Rows and columns keep their own records
  (heights, widths, hidden, line styles): a shift moves cells, not lines.
- **Cost.** Not the band's cells alone: a run taken out of one row vector and put into another moves each
  row's later entries too, so a shift down costs the sum of the lengths of the rows from the shift to the last
  row the band reaches (a one-column shift over 100,000 rows of a sheet 1,000 columns wide moves about 10^8
  entries). T-5466 has a bench for that wide case beside the narrow one.
- **A bound.** A shift moves at most `MAX_SHIFT_CELLS` cells and rewrites at most as many formulas (the model's
  operations are capped at 10,000 cells, `MAX_CELLS_PER_OP` in `rustsheet-sheet-ops`; the engine's bound is
  set with the model lane, no lower than its). Past it the shift is refused before anything changes, with its
  own `StructureRefusal` kind (`TooLarge`), so a user who inserts a cell above a long column is told, and never
  left with part of a shift. `plan_shift`'s reply is bounded by the same number.
- **Formulas.** The tree rewrite of Design 2 with two more tests. A cell moves when the band covers it and
  it is at or after the shift. A part's target follows the edit when it reads the edited sheet, is neither a
  span nor a name's relative part, and the band covers its extent on the other axis whole. The per-formula
  memo of Design 2 needs each cell's and each part's place on the other axis as well, so a shift rewrites its
  candidates one by one in the first cut; they are the formulas that read the band, not the workbook's.
- **Refusals** are checked before anything changes, as today, and are U9's, U12's, U13's second group and the
  bound.
- **Recalculation.** A shift through the facade makes the read index stale and recalculates fully, as a row
  or column edit did before T-5136: SPEC-0002 Design 8's dirty set is built for a whole-line map and does not
  cover a banded one. In a document a shift is cell writes (A13), so it takes the path a large paste takes,
  and the index's maintenance files each written formula again (T-5166).
- **Undo** is the tree record of Design 6 over the band: the deleted cells, the rewritten formulas, and the
  merges, links, validations, notes and conditional formats the band covers.
- **Limits.** The walks are Design 10's; the band adds none.

**A13. The facade and a document.** `Txn::insert_cells(sheet, area, Shift::Down | Shift::Right)` and
`Txn::delete_cells(sheet, area, Shift::Up | Shift::Left)` are ops like the four line ops, refused with
`EditError::Structure`, with the shift in `ChangeSet.structure` (its `RefMap` carries the band; the charts
layer, which reads that field for its series ranges, sees a shift there in a single-user workbook and cell
writes in a document).

A collaborative document cannot take a shift as a structure event: no row or column identity is inserted
or deleted, so there is nothing for `doc.applyStructure` to replay, and A6's rule that an engine's rewrite
is never written to the document does not hold for a shift. A shift reaches a document as ordinary cell
writes, which is what I-03 intends for `shiftCells` (agreed with the contract's owner, 2026-11-11):
- the planner derives the moves from the area and the direction (the band's tail at its new places, the
  vacated cells cleared);
- the engine gives what the planner cannot derive, as a dry run that changes nothing, like
  `plan.structurePlan`: the refusal, if any, and each formula the shift rewrites with its stored input after
  it, at its place after it (`Workbook::plan_shift`, the engine API's `plan.shiftPlan`, an I-06 query);
- the author's host writes the plan in one transaction, and every engine, the author's included, receives
  cell writes. No engine applies a shift op in a collaborative document, so no reconciliation is needed and
  no engine can be left stale by one: `rustsheet-wasm` has `plan.shiftPlan` and no call that applies a shift,
  which T-5468's test of the engine API's surface holds.

A shift as cell writes has two divergences that a row or column edit, where identities move and writes follow
them, cannot have. Both belong to the planner, and neither is closed by this amendment:
- **A cell's content can be lost.** The author's shift writes A5 as what A4 held, A6 as what A5 held, and so
  on, and clears A4. A second author, at the same time, types into A5. If the typed value wins at A5, what A4
  held is nowhere: the shift cleared it and its copy was overwritten. Nobody deleted it.
- **A formula can read a cell that moved away.** A formula written concurrently with the shift is resolved as
  any concurrent cell write is, so the document can hold a formula whose reference the shift would have moved.
The contract's owner describes the plan as one with a fence and a move map, chunked under 1 MiB as one undo
group. Whether that fence refuses, or re-plans, a shift whose band another author wrote into is the model's
plan pipeline's to state before `shiftCells` ships (package M4, which no lane holds yet); until it does, the
first divergence is an open hazard with that owner. T-5467's property test covers one author only, and says so.

**A14. Rollout.**
1. T-5463: this amendment.
2. T-5464: the case format's `insert_cells` and `delete_cells` ops (`{op, sheet, range, shift}`), recorded by
   `spec2xlsx`, replayed by the Graph backend, parsed by the eval harness; the engine adapter answers "not
   built" until step 4, so its cases stay out of the gate suite until then, as T-5123's refused ones do.
3. T-5465: probe cases for U7 to U14, sent as a probe request; `references.md` gets the findings.
4. T-5466: `Band`, `covers`, `shift_band`, the rewrite, the refusals and the undo record, against the
   probes where they are filled and the provisional rules where not (each one marked UNVERIFIED in code and
   in `references.md`); a differential against the same shift done as cell moves and typed formulas.
5. T-5467: the `Txn` ops, `plan_shift`, `ChangeSet.structure`, and the plan's purpose as a property test (the
   op on one copy equals, in the canonical live state, the plan's writes on another).
6. T-5468: `plan.shiftPlan` in `rustsheet-wasm`, when I-06 has the query (the contracts lane's wave-2
   addition).
The model's planner, the editor's dialog and keys, and the engine-js call are their lanes'; each can start
from step 1's text and the engine's plan reply of step 5.
