# SPEC-0045 — Cell storage and memory: 64 bytes a numeric cell, 100 a text cell

- **Status:** draft
- **Milestone / lane:** M4 / engine
- **Tasks:** T-5430 (this spec), T-5431 (the memory gate), T-5433 (a 32-byte cell), T-5434 (row slack),
  T-5436 (one copy of each text), T-5437 (the exit measure met), T-5450 (a 32-byte error value). Part 2 has no
  tasks yet (see "Rollout").
- **Decision:** 0441 (how M4's storage exit is measured).

## Problem
M4's exit asks for at most 64 B per numeric cell and 100 B per text cell (repeated text shared) "on benchmark
models" (`docs/milestones.md`). Nothing says which models or how the bytes are counted, no task builds it, and
today's figures are well over it.

Measured on `main` (`527f2b8bc`, native aarch64, release) with a counting global allocator: the bytes live in the
`Model` (requested and not freed, an empty model's subtracted), divided by its stored cells, 1,000,000 cells in each
workbook.

| Workbook | As typed | After a full pass | Reloaded from `to_bytes` | Target |
|---|---|---|---|---|
| Numbers, 100,000 rows of 10 | 143.2 | 143.2 | 90.4 | 64 |
| Numbers, one column of 1,000,000 | 376.1 | 376.1 | 112.1 | 64 |
| Text, every cell distinct (11 bytes each) | 259.6 | 259.6 | 205.6 | 100 |
| Text, 100 distinct values | 143.2 | 143.2 | 90.4 | 100 |
| Formulas of one family, numeric results | 143.2 | 161.0 | 90.4 | none |
| Formulas of one family, text results (2 distinct) | 143.2 | 204.2 | 128.7 | none |
| Formulas of one family, every result `#N/A` (`=NA()`) | 143.2 | 175.4 | 102.0 | none |
| Formulas of one family, every one a lookup that misses | 143.2 | 183.5 | 110.1 | none |
| 500,000 formulas, no two alike, beside 500,000 numbers | 401.6 | 419.4 | 306.4 | none |
| One number every 1,000 rows | 24,402 | 24,402 | 24,140 | none |
| Spilled numbers, a chain of 10 spills of 100,000 | no figure | 179.2 (305.2 at the peak of a second pass) | no figure | none |

"As typed" is a workbook built one cell at a time, which is also how the collaborative client's engine loads a
document (`Workbook::load_stored`, `crates/rustsheet/src/workbook/stored.rs`: sorted writes, one `Row::insert`
each). An xlsx import and a decode size each row exactly, so they sit in the "reloaded" column.

Where the bytes are:

- **The entry.** A row is a `Vec<(i32, Cell)>` (`sheet_data/row.rs`). `Cell` is 80 bytes and the entry 88 whatever
  the cell holds, because the largest variant sets the size: `ArrayFormula { f, s, r: (i32, i32), kind, v }`, whose
  `FormulaValue` is 56 bytes for `Error { ei, o: String, m: String }`, two owned strings no number cell has. On
  wasm32, where the client runs, `Cell` is 56 bytes and the entry 64 (measured in a wasm32 build).
- **Slack.** A row built by typing grows by doubling: ten cells sit in a capacity of sixteen (143.2 against 90.4),
  and a row of one cell takes a capacity of four (376.1 against 112.1). Nothing gives slack back on a removal either.
- **The row slot.** Each row of a chunk of 1,024 costs a 24-byte slot whether it exists or not (12 on wasm32): 24
  bytes a cell in a one-column sheet, and 24 kB for a cell alone in its chunk.
- **Text.** A distinct text costs 116.4 bytes over its cell: 36.2 in the table of strings and 80.2 in the map
  beside it. It is held twice: in `Workbook::shared_strings`
  (`Vec<String>`) and as the key of `Model::shared_strings` (`HashMap<String, usize>`, `model.rs:162`).
- **Formula text.** The same shape: `Worksheet::shared_formulas` (`Vec<String>`) and the key of
  `SharedFormulaLookup::index` (`HashMap<String, i32>`, `formula_cells.rs:115`), beside the parsed tree. A formula
  of its own costs 413 bytes over its cell.
- **Formula results.** A text result is an `Arc<String>` of its own, about 42 bytes over its cell, not shared with
  an equal result in the next cell. An error result owns its two strings: the cell it came from as text
  (`Sheet1!B2`), parsed again by every read, and its message, 36 to 45 bytes of buffers in a column of `#N/A`.
- **A spilled value** is an ordinary stored cell: an 88-byte entry with its row's slack.

Three open records measure the same bytes from outside: the red team's spill chain at 150 to 400 bytes a value
(Taskei P559333484), the performance plan's 181 B per stored cell at the median with its "cheapest first" list
(`docs/amazon/plans/performance.md` 4.2 and 4.7), and the fork's assessment
(`docs/research/ironcalc-assessment.md`, "Measured").

## Goals / non-goals
Goals:
1. **A measure.** Decision 0441 names the benchmark models and the count. A test holds them in `make check`.
2. **The exit bars, however the workbook was built.** At most 64 B per numeric cell and 100 B per text cell on
   the benchmark models, typed, loaded cell by cell, imported or decoded: the worst of them is the figure.
3. **No slower.** Point reads, a row-major pass, an import and a structural edit's re-keying cost no more than
   today (`structural_edit`, `edit`, `incremental`, `aggregate` and `import` benches before and after each task).
4. **The same values.** No eval cell changes.
5. **One copy** of each distinct cell text and each distinct formula text.
6. **Nothing to walk back.** Each step survives the milestone's larger scope (columnar copy-on-write chunks, an
   O(1) `Snapshot`), which Part 2 sketches and does not build.

Non-goals:
- columnar storage, snapshots and scenarios themselves (Part 2: a sketch, a go/no-go measure and no task);
- a version check on stored bytes (Taskei P559862286): `to_bytes`, `.ic` files and peer diff lists change shape
  with `Cell`, and no product path persists them today. The check behind that sentence, for a lane that starts
  to: `rg 'from_bytes|apply_external_diffs|load_from_icalc' crates/rustsheet/src crates/rustsheet-wasm/src` finds
  nothing, the Python binding's `from_bytes` takes xlsx bytes (`bindings/python/src/facade.rs:400`), and
  `supply-chain/audits.toml` records that `rustsheet-wasm` never calls `Model::from_bytes`; the remaining
  callers are `UserModel`'s wrappers, the `.ic` converter and tests;
- the performance plan's second M4 budget, "1M x 10 ≤ 400 MB" (`performance.md` 4.7): ten million numbers at
  42.4 B are 424 MB requested, more resident. `docs/milestones.md` holds M4 to the two bars only, and this spec
  meets those; that budget needs Part 2;
- collecting unused styles (`worksheet.rs:250`) and unused shared strings (Design 5 says why not here);
- the size of a parsed formula (`Node` is 80 bytes, `expressions/parser/node.rs:268`), which sets the cost of a
  formula of its own;
- stable row and column ids inside the engine (`performance.md` 4.2 step 6): SPEC-0039 chose rewriting.

## Excel behavior to match
None: how cells are stored is not observable in a value. Two things a reader can observe must not move:
- an error's message is what `Workbook::value` returns as `XlError::detail`, and a change set compares it
  (`crates/rustsheet/src/edit/diff.rs:112`), so the stored message stays byte for byte;
- which cells are stored stays a function of the document (SPEC-0028 section 11, Taskei P559664573): nothing here
  adds or keeps a cell that an edit would have removed.

## Design
**1. The measure** (decision 0441, T-5431).

The benchmark models, 1,000,000 cells each:

| Model | Cells | Bar |
|---|---|---|
| N10 | numbers, 100,000 rows of 10 | 64 B |
| N1 | numbers, one column of 1,000,000 | 64 B, to the whole byte (Design 5) |
| T10 | text, every cell distinct, 11 bytes each, 100,000 rows of 10 | 100 B |
| R10 | text, 100 distinct values, 100,000 rows of 10 | 100 B |
| L | SPEC-0002 "Problem"'s ledger at 100,000 rows (`test_support::ledger`), after a full pass | reported |
| E10 | a number and nine lookups that miss in each of 100,000 rows, after a full pass | reported |

Each is built three ways: typed (`Model::set_user_input`), loaded as the client loads (`Workbook::load_stored`)
and decoded (`Model::from_bytes` of the typed one). The model's figure is the largest of the three.

Two sizes run the same code. The bench `memory` builds the models at 1,000,000 cells in release, and its figures
are the ones a task's outcome and this spec quote. The gate in `make check` builds them at 100,000 cells (the same
shapes, a tenth of the rows), so that the test profile holds them in seconds; its constants are counted at that
size. A bar holds at both.

The count is the allocator's: the bytes requested and not yet freed, after the build and (for L) one full pass,
less the bytes of an empty workbook, divided by the stored cells. It is exact and repeats to the byte on one
target, so the test compares with `<=` and no tolerance. A count kept by hand (`heap_bytes()` summing capacities)
would drift from the structures it describes; the allocator cannot.

The allocator is `Counting` in a new workspace crate, `crates/rustsheet-heap` (a dev-dependency only): the system
allocator with a live and a peak counter. `fuzz/src/heap.rs` holds that code today for one fuzz target; it moves
there, and the fuzz crate depends on it, so there is one definition. Its `unsafe impl GlobalAlloc` forwards every
call unchanged, under an `#[allow(unsafe_code)]` with the reason, as `rustsheet-wasm`'s `GrowAhead` has (decision
0423); decision 0441 records it. The gate is an integration test of the facade crate with its own
`#[global_allocator]`, run single-threaded by its harness setting (`harness = false`), so no other test's
allocations are counted.

The test holds two things:
- the bars above, each as a constant that may only fall; until T-5437 they stand at today's figures rounded up,
  so that every task before it shows its gain as a lowered constant and cannot regress one;
- `size_of::<Cell>()` and the entry's size, asserted at compile time for 64-bit and for wasm32
  (`cfg(target_pointer_width)`), in the engine itself.

wasm32 has no allocator count in `make check`; its figure is the entry's size and the row slot's, which the
compile-time assertions hold, and the nightly's `load-check.mjs` reports the module's memory after a load.

Requested bytes are not resident bytes: N1 is a million 40-byte allocations, which most allocators round up.
The gate stays on the exact count; T-5437's outcome adds one resident figure per model natively and the wasm
module's memory after a load, so that the bar is not read as what a tab pays.

**2. A 32-byte cell** (T-5433).

Two changes to `types.rs`, each checked by building the types alone for both targets before this spec was written
(16-byte `FormulaValue`, 32-byte `Cell`, 40-byte entry on aarch64 and on wasm32):

```rust
pub enum FormulaValue {
    Unevaluated,
    Boolean(bool),
    Number(f64),
    Text(Text),
    Error(Box<ErrorValue>),       // was Error { ei, o: String, m: String }
}
pub struct ErrorValue { pub ei: Error, pub o: String, pub m: String }

ArrayFormula { f: i32, s: i32, r: ArraySize, kind: ArrayKind, v: FormulaValue },
                                  // was r: (i32, i32)
/// An array's width and height, six bytes with an alignment of one.
pub struct ArraySize { width: [u8; 2], height: [u8; 4] }
impl ArraySize {
    pub fn new(width: i32, height: i32) -> Option<ArraySize>;   // None off the grid
    pub fn width(self) -> i32;
    pub fn height(self) -> i32;
    pub fn is_one_cell(self) -> bool;
}
```

- The error must be one pointer. With `ei` beside a boxed pair the variant is 16 bytes with an alignment of 8, the
  compiler cannot place it beside `Text`'s tag, and `FormulaValue` stays at 24 bytes (`Cell` at 48).
- What the box costs an error cell: one more allocation of 56 bytes, so an error cell goes from 88 bytes and its
  buffers to 96 and its buffers, while every other cell loses 48. A column of lookup misses is common, so E10 is
  in the gate's report, and Design 2b takes the error value itself down.
- The array's size must pack behind the tag with `kind`. With `r: (i32, i32)` kept, `Cell` is 40 bytes and the
  entry 48, which misses the bar on N1 (48 + 24). A width fits 16 bits (16,384 columns) and a height 32, but a
  struct of a `u16` and a `u32` is aligned to four and does not pack either (checked: 40 bytes). `ArraySize` holds
  the two as bytes, so its alignment is one and the compiler places it in the tag's padding. The field keeps its
  name: of the 71 places that name `r` (34 files, 14 of them tests), those that only copy it are untouched, and
  those that read the pair call `width()` and `height()`. The seven that match a one-cell anchor by the literal
  `r: (1, 1)` (`read_index.rs`, `circular.rs`, `positions.rs`, `structural_edit.rs`, `formula_cells.rs`, `model.rs`
  and the facade's `value.rs`) take `is_one_cell()`, one definition. The bytes are little-endian whatever the
  host, so `to_bytes` does not depend on it. A size off the grid cannot be built: an import or a
  decode that carries one is refused, as an index out of range is. Two fields of the variant (`width: u16,
  height: u32`) pack as well and read more plainly, and change all 71 places, several in files other lanes are
  changing now (`read_index.rs`, `structural_edit.rs`); the task takes `ArraySize`.
- `SpillCell { s, a: (i32, i32), v: SpillValue }` is 32 bytes as it stands, and sets the floor: a smaller cell
  needs spilled values out of the cell store (Part 2).
- 48 lines in 14 files name `FormulaValue::Error`'s fields, 9 of them constructions outside tests. The rest of
  `Cell`'s 475 uses are untouched.
- `docs/security/bitcode-derives.txt` gains `ErrorValue`'s line; the golden bytes of
  `test_to_from_bytes.rs` stay an `Err`, never a panic.

**2b. A 32-byte error value** (T-5450, after T-5433; not needed for the bars).

```rust
pub struct ErrorValue {
    pub ei: Error,
    pub origin: Option<ErrorOrigin>,   // was o: String, "Sheet1!B2", parsed by every read
    pub m: Arc<str>,                   // was m: String, one buffer a cell
}
pub struct ErrorOrigin { pub sheet_id: u32, pub row: i32, pub column: i32 }
```

- The origin is the cell the error came from. As text it costs a buffer a cell, a parse on every read of the
  cell (`model.rs:1240`), a rewrite when a structural edit moves it (`structural_edit.rs:186`), and it is lost
  when its sheet is renamed (the parse fails and the read falls back to the cell itself). Typed by the sheet's id
  it costs 12 bytes, needs no parse, moves by arithmetic and follows a rename. A deleted sheet's origin reads as
  none, which is today's fallback.
- A column of one error holds one message. The model keeps the last few messages it stored (a small table by
  text, bounded) and hands out the shared one; a message is compared once per stored error.
- An import writes the origin and message it writes today (`import/worksheets.rs:534`), typed.
- What a reader sees is unchanged: `XlError::detail` is the same text, and the origin the evaluator gets is the
  same cell wherever today's text parses.

An error cell then costs 40 + 32 = 72 bytes with no buffer of its own (140.8 after T-5433 for a lookup miss: 40,
the 56-byte box and 44.8 of buffers).

**3. Row slack** (T-5434). A row's capacity is at most its length plus an eighth of it, so a row of fewer than
eight cells has none:
- `Row::insert` into a full row reserves exactly `max(1, len / 8)` more, instead of doubling;
- a removal (`remove_cell`, a column delete's `retain_mut`) gives back capacity past that bound;
- `set_row` already sizes exactly, and `Workbook::load_stored` uses it: its cells arrive sorted, so it groups
  them by row and sets each row whole. The client's load then has no slack and no reallocation, and the eighth
  covers typing alone.

A row of n cells built left to right reallocates about 8 + 8.5 ln(n / 8) times (73 for 16,384 cells) and copies
about nine entries per cell in all, against two for doubling: 5.9 MB of copies for a row of 16,384 cells, once. The benches that guard it
are `import` (rows set whole, unchanged), a new `load_stored` of N10 at 1,000,000 cells (the client's load), and a
new line in `edit` (10,000 cells typed into one row).

**4. One copy of each text** (T-5436). `TextIndex`, one type for both lookups (a shared string by its text, a
shared formula by its R1C1 text): a `hashbrown::HashTable<u32>` holding indices only, hashed and compared through
the `Vec<String>` it indexes.

```rust
pub(crate) struct TextIndex { table: HashTable<u32> }
impl TextIndex {
    pub(crate) fn of(texts: &[String]) -> TextIndex;            // the last of equal texts wins, as today
    pub(crate) fn get(&self, texts: &[String], text: &str) -> Option<u32>;
    pub(crate) fn insert(&mut self, texts: &[String], index: u32);
}
```

It costs 5.7 to 11.4 bytes a text (four for the index and one control byte, at a load between 7/8 and 7/16)
against the second `String`'s 24 bytes, its buffer and the map's 33-byte slot: 117 bytes a distinct text today,
about 46 after. `shared_formula_index` stops cloning the text it interns.
`hashbrown` becomes a direct dependency of the engine at the version the lock already holds through others
(0.17.1); decision 0441 records why (the standard map has no index-only form, and a second map keyed by a hash
with a chain array is that table written by hand).

**5. What stays, and why.**
- **Unused shared strings stay.** The table only grows, and the source-preserving export relies on it
  (`export/patch.rs:85`, `live.shared_strings.starts_with(&base.shared_strings)`). Collecting them needs that
  check to learn an epoch first, as the shared formulas' compaction has one; it is Part 2's, with its own measure
  (a long session that types and deletes distinct text).
- **Text results are not shared across cells.** Sharing equal results needs a hash per stored text result, on the
  pass's hot path; the gain (about 42 bytes a text result) is outside the exit bars and is measured in Part 2.
- **A read of a text cell still copies it** (`model.rs:1188`, `s.clone().into()`): `shared_strings` is a
  `Vec<String>` and a value is a `Text` (`Arc<String>`). Holding `Text` there would share it and cost 16 bytes
  more a distinct text (T10 at about 99 B, on the bar). It is recorded here as a cost of evaluation, not of
  storage, and filed with the engine lane.
- **The row slot stays**, so N1 has no margin. Counted on `main` and reduced by the entry's 48 bytes a cell, a
  decoded N1 is 64,051,959 bytes: 40 for the entry, 24 for the slot and 0.05 for its 977 chunks and their table.
  That is 0.05 B over the bar, which the criterion's whole bytes do not resolve; decision 0441 reads the bar to
  the whole byte for it and says so, and the gate holds N1 at its count. Anything added to an entry or a row slot
  breaks that until Part 2, the only store with no per-row record (wasm32 pays 40 + 12). A slot that holds a
  lone cell inline was checked and not taken ("Alternatives"). A sparse sheet still pays 24 kB a chunk.

**6. Expected figures** (arithmetic from the sizes above; T-5437 replaces them with the gate's counts):

| Model | Today (worst build) | After T-5433 | After T-5434 | After T-5436 | Bar |
|---|---|---|---|---|---|
| N10 | 143.2 | 66.4 | 42.4 | the same | 64 |
| N1 | 376.1 | 184.1 | 64.05 | the same | 64 |
| T10 | 259.6 | 182.8 | 158.8 | 89.1 | 100 |
| R10 | 143.2 | 66.4 | 42.4 | the same | 100 |
| E10, after a pass | 183.5 | 157.1 | 133.1 | the same | none: 71.2 after T-5450 |

The figures are today's counts less what each step removes: 48 bytes an entry (T-5433), the slack of a typed row
(T-5434: 16 entries for 10, 4 for 1), the map's 80.2 bytes a text less the table's 10.5 (T-5436). E10's are
from its count after a pass (183,540,634 bytes typed) with a 56-byte box for each of its 900,000 errors.

A spilled value falls with N10: from 179 bytes to about 43 after a pass (Taskei P559333484, re-measured by T-5437
with the red team's `rt-spill-chain`).

**Part 2 (a sketch, not built here): columnar copy-on-write chunks.**
The milestone's scope names them, with an O(1) `Snapshot`; `docs/architecture/overview.md` and
`docs/research/api-landscape.md` (line 285) give the shape: per column, chunks of 1,024 rows behind an `Arc`, each
a presence bitmap, a tag per present cell and typed blocks (numbers, text indices, formula indices with their
values), styles as runs. The performance plan measured 8.1 B a numeric cell and a `SUM` 4.4 times faster on a
prototype. What it takes:
1. Readers stop taking `&Cell`: `SheetData::cell` and its kin (about 86 engine call sites of `.cell(`, 27 of
   `.cells()` and 10 of `cell_mut`, tests aside) return a view by value. This step is independent of the layout and can land
   first, reader by reader.
2. The chunks, behind the same `SheetData` API, with the row store kept until the differential (both stores, every
   eval case and every structural history) is empty.
3. `Snapshot` as a clone of the chunk table; a `Txn`'s rollback and a structural edit's undo record become a
   snapshot instead of a capture.

It is 8 to 10 engineer-weeks by the plan's estimate, and it removes the row slot, the entry's 32-byte floor and
the capture a transaction makes. This spec's steps all survive it: the 32-byte `Cell` stays the value every reader
and every undo record holds, and `TextIndex`, `ErrorValue` and the gate carry over; only the row-slack rule goes
with the rows. Whether and when Part 2 starts is the lead's call (it is past the exit bars); the go/no-go is the
gate's figures and the `aggregate` and `structural_edit` benches on a prototype behind the API of step 1.

## Alternatives considered
- **Columnar first.** It reaches about 9 to 13 B a cell and makes this spec's row rule moot. It also rewrites
  every reader of `&Cell` before any byte is saved, on the engine's hottest paths, while the collaborative client
  pays 143 B a cell today. The exit bars are reachable in four small steps that Part 2 keeps.
- **One sorted run of entries per chunk** (row offsets instead of row vectors): about 40.4 B a cell at any row
  density, inside `sheet_data` alone. A new cell then moves every entry after it in its chunk (400 kB for ten
  columns, far more for a wide sheet), and Part 2 would discard it.
- **A row slot that holds a lone cell inline** (`None`, `One(column, cell)`, `Many(Vec)`: 40 bytes on both
  targets, checked). N1 falls to 40.05 and every row length is within 60 B natively. Every row of two or more
  cells pays 16 bytes more (N10 44.0), wasm32 pays 28 more a row (its slot is 12 bytes today), and a cell alone in
  its chunk 41 kB instead of 24: typical sheets and the client would pay for the one-column benchmark.
- **`ei` kept inline beside a boxed detail.** Checked: `FormulaValue` stays 24 bytes (Design 2).
- **`Box` the array anchor's extra fields.** All 125 uses of `ArrayFormula` change shape.
- **Interning text results by index into the shared strings.** Every reader of a `FormulaValue::Text` would need
  the workbook, and transient results would grow a table that is never collected.
- **A byte count kept by the structures themselves.** See Design 1.

## Test plan
- The gate (Design 1): four bars and the reports of L and E10, three builds each; compile-time sizes on both
  targets.
- No value moves: `make check` (every eval gate, the bitcode round trips, the structural differential and its undo
  oracle, the fuzz seeds of `bitcode_workbook`).
- A decode and an import refuse an array size off the grid and never panic (T-5433).
- `TextIndex` against the map it replaces: a property test over random inserts, duplicates included (the last of
  equal texts wins), and the rebuild after a compaction and after an undo that restores a workbook
  (`structure_undo.rs:426`).
- Benches before and after each task, in its outcome: `structural_edit`, `edit`, `incremental` (its full
  recalculation line, T-5165's, which a cell 2.5 times smaller should help, and its one-cell edit), `aggregate`,
  `import` and the new `load_stored`; the red team's `rt-spill-chain` peak with T-5437.

## Rollout
1. T-5430: this spec and decision 0441.
2. T-5431: `crates/rustsheet-heap`, the gate at today's figures, the compile-time sizes at today's.
3. T-5433: the 32-byte cell; the gate's constants fall to its counts.
4. T-5434: the row slack; N10, N1 and R10 pass their bars.
5. T-5436: `TextIndex`; T10 passes its bar.
6. T-5437: the gate's constants are the bars; `docs/milestones.md` and `performance.md` 4.7 carry the measured
   figures; P559333484 is re-measured and closed or re-scoped to Part 2.
7. T-5450: the 32-byte error value, after T-5433 and in any order with the others; E10's constant falls.

Tasks 3 to 5 touch different files (`types.rs` and its readers; `sheet_data/row.rs`; `model.rs` and
`formula_cells.rs`) and may land in any order after task 2.
