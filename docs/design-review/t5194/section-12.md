### 12. Cells with content: a styled blank changes no work (amendment for T-5194, 2026-11-11)

Implementation pending: T-5190. Until it lands, sections 1 to 11 describe the code.

**The gap.** Formatting a cell that holds nothing stores a cell (`Worksheet::set_cell_style` stores
`Cell::EmptyCell { s }`), and so does clearing a cell (`Worksheet::cell_clear_contents`). Every count and extent the
work accounting uses is a count or an extent of stored cells:

| Mechanism | What it reads today |
|---|---|
| The used area's end (`Worksheet::used_area_end`, `used_area_corner`, `clip_to_used_area`, `clip_block_to_used_area`) | `SheetData::last_row` and `last_column`: the last row and column holding a stored cell |
| Whether a walk is dense or sparse (`walks_sparsely`, `stored_walk`) | `SheetData::cells_in_columns`: stored cells of the range's columns |
| A sparse walk (`for_each_existing_cell`, `for_each_existing_position`) | every stored cell of the range |
| A union walk's charge (`MAX_AREA_WALK`, `area_budget.rs`) | a step per area walked, per search of the column index, per row found (a row with no cell included), per unallocated chunk passed over, per 16 row slots tested, and per stored cell handed on |
| A lifted call's visits (`criteria_walk_cost`, `range_walk_cost`, `subtotal_walk_cost`, against `MAX_LIFTED_CELL_VISITS`) | the clipped block, or the stored cells |
| The seen-range budget (`MAX_SEEN_RANGES`) | the strips a cut at the used area leaves out |
| A search's cut (`clip_search_rows`) | the last used row |
| The read index's used-area input (`Incremental::last_rows`; SPEC-0002 Design 1, "The used area") | the last used row |
| The per-pass list of positions (`all_positions`, in `T_pass`) | every stored cell |

So a format on a blank cell can change a value at the work limits. T-5190's Context has three reproducers at the
default limits: a format on blank XFD1 takes a `SUM(IFERROR(AGGREGATE(…)))` from 5 to 0; a format on blank A2 takes
a lifted `COUNTIF` from 2000 to `#ERROR!`, by turning its walk dense; formats on B1:B10000 take a sum over a union
from 10,000,000 to `#VALUE!`. A fourth, a row record past the content after a save and a load, is closed already:
since decision 0421 the last row is the last row holding a cell, and a row with no cell does not count.

Charging the walks for the blanks they visit, as section 11's stored-cell charge would, does not close the gap:
the format then changes the work by design. Leaving the blanks uncharged and letting the walks skip them does not
either: a walk's time would then follow cells its charge does not count, against rule 1. T-5194 asked for a
design in which both hold.

**What other programs do** (fetched 2026-11-11).
- Excel has both extents. Office.js: "The used range is the smallest range that encompasses any cells that have a
  value or formatting assigned to them", and `getUsedRange(valuesOnly)`: "If `true`, considers only cells with
  values as used cells (ignoring formatting)" (learn.microsoft.com/en-us/javascript/api/excel/excel.worksheet).
  Which one Excel's own calculation clips at is not documented, and is not observable through values.
- LibreOffice Calc keeps a column's formats apart from its cells: `ScColumnData` holds `pAttrArray`, an array of
  entries "where the pattern applies to all rows up till nEndRow", and `ScColumn` holds `maCells`, "Cell values"
  (`sc/inc/attarray.hxx`, `sc/inc/column.hxx`, MPL-2.0; read for the idea only). Formatting a blank stores no cell
  there.
- We take LibreOffice's split, for blanks only: a cell with content keeps its style where it is.

**The rule.** Evaluation and its accounting see the cells with content, and nothing else. A cell has content when it
holds a value, a formula or a spill member: every `Cell` variant but `EmptyCell`. A stored blank, which a style or
a clear leaves, has none.

**The representation.** `SheetData` holds two tables:
- **The cells**, in the chunks and row vectors it has today, with the counts and indexes it has today (`row_count`,
  `column_counts`, `column_rows`, `known_last_row`). Two invariants make each of these a count of content:
  - `Cell::EmptyCell` is never in a row vector. It stays the value an empty position reads as.
  - The content table contains no cell-less row. A row slot exists exactly when the row contains a value, formula
    or spill member, and a chunk exists only while it contains such a row. Importing a row whose cells all move to
    the blank layer must not retain an empty content row. Row metadata and any row-presence information required
    by export are stored outside the evaluation-visible table. This invariant applies to imports, clears,
    structural edits and restores. (Today `SheetData::set_row` keeps a row with no cell, the importer calls it for
    every `<row>`, and a union walk finds and pays for such a row: with the same two cells, 9,999 cell-less rows
    take a sum over a union from 5,000 to `#VALUE!`, the reviewer's probe of CR-321706251.)
- **The blank layer**: what is stored and is not content. An ordered map from (row, column) to the style of a
  stored blank, and a sorted set of the rows a file stored with no cell. Its lookup, insert and remove are point
  operations: a number of comparisons that grows with the logarithm of the entries, and a bounded number of
  entries moved, whatever else its row holds (a B-tree, `BTreeMap<(i32, i32), i32>` in the first cut; its
  entries in order give a row's blanks and a range's). A sheet with no stored blank and no cell-less row holds
  an empty map, so the cell models of M4's storage measures do not grow.

A position is in at most one of the two tables: a stored blank has no cell, and a cell has no layer entry.

**Transitions inside a recalculation.** Edits are not the only thing that turns a blank into a cell and back.
Recalculation does it too, some of it thousands of times: `spill_dynamic_array` writes a spill member over a
formatted blank and clears retired members, `set_cells_with_result` writes a Ctrl+Shift+Enter array's cells,
`restore_dynamic_spills` puts an abandoned pass's spills back, and `evaluate_substituted` saves, overwrites
and restores each input of a data table at every substitution. Each of these is the transition it is today, with
the layer in the place of the `EmptyCell`:
- A cell written over a stored blank takes the blank's style, and the layer's entry is removed.
- Spill retirement preserves decision 0421 item 2. Compare the retiring member's style with the current row/column
  inherited style: if they match, no stored blank remains; otherwise a stored blank remains with the member's
  style. This result is not determined solely by whether a blank preceded the spill. Style preservation and
  canonical retirement, including during rollback, use deterministic bounded point operations whose accounting is
  independent of unrelated formatted blanks. No such operation scans, compacts or shifts unrelated blank entries
  during recalculation. Data-table restoration restores the saved input's content and blank state rather than
  applying the spill-retirement rule to it.
- So a saved data-table input is one of three things, a cell, a stored blank with its style, or nothing, and
  the restore puts back exactly that.

Which blanks are stored does not change: decision 0421's rules decide that, and they now add to and remove from
the layer. Only where a blank is kept changes.

The layer is reached only through `SheetData`'s methods: a blank's style at a position, the stored blanks of a
range in order, and its extent. No reader holds its row vectors. So its shape can change without its readers: M4's
later sketch of columnar chunks would keep styles as runs per column for cells and blanks alike, and absorb it
(the M4 lane's condition).

**What it costs to keep.**

| Operation | Today | With the layer |
|---|---|---|
| A style on a position with no cell | an `EmptyCell` enters the row vector | an entry enters the layer: one point insert |
| A value or formula written over a stored blank, by an edit or by evaluation | the cell is replaced in place | the layer's entry is removed, one point remove, and its style goes to the new cell |
| A spill member retired, in a recalculation or a rollback | decision 0421's comparison: the cell is removed, or becomes `EmptyCell { s }` in place | the same comparison: the cell is removed, and where a blank remains, one point insert |
| A data table's input restored after a substitution | the saved cell is written back | the saved state is put back: a cell, or one point insert of the saved blank, or nothing |
| A cell cleared, where a blank stays stored | the cell becomes `EmptyCell { s }` | the cell is removed and its style enters the layer |
| A structural edit (SPEC-0039) | the row vectors are moved or re-keyed | both tables, by the same rule; the layer's part costs its own entries |
| Undo and its records, copy and paste, the fill plan, sort, a merge's covered cells | carry the `EmptyCell` | carry the layer's entry, by position |
| Decision 0421's rules at a load and after a batch (`drop_inherited_empty_cells`, `Txn::drop_inherited_empty`) | test and remove an `EmptyCell` whose style is the inherited one | the same test, on the layer's entry |
| xlsx import of a valueless `<c>` | an `EmptyCell` | a layer entry |
| xlsx import of a `<row>` that holds no cell with content | `set_row` keeps the row, with no cell, in the cell table | the row's presence goes to the layer; the cell table gets no row |
| xlsx export of a row | its cells in column order | its cells and its layer entries, merged in column order |
| `Model::to_bytes` | encodes the chunks | encodes both tables. The form changes; nothing persists these bytes today and they carry no version (Taskei P559862286), so it needs no step shared with M4's cell |

Memory. An entry is a key of two `i32` and a style, 12 bytes, in a B-tree whose leaves hold up to 11 entries:
about 20 bytes a blank at the fill a B-tree keeps, against the `(i32, Cell)` entry of a row vector a blank is
today. T-5190 measures it with M4's counting allocator and states the figure in its outcome. A sorted vector per
row would be 8 bytes an entry, and was this section's first shape; it is under Alternatives, because every
transition above would then move the row's later entries.

**What it costs to read.**
- The content extent is read as the stored extent is today, at today's cost: `last_row` from `known_last_row`, with
  a scan of chunk slots only when the last row loses its last cell, which visits no cell; `last_column` as the last
  key of `column_counts`. T-5194 required that the content extent, like `last_row`, not visit every cell.
- There is still one definition of the last used row. `Worksheet::last_used_row` is what a tailed read's head, a
  search's cut and the read index's `Incremental::last_rows` all read today, and T-5190 changes that one function.
  It adds no second extent for evaluation to choose from, so the tail and the index cannot part; T-5149's used-area
  oracle, which checks the index against every tailed read and cut search, runs on its branch (the read-index
  lane's condition).
- A blank's style is one point lookup in the layer, a search of the order of a cell's today.
- The stored extent, which the host-facing readers use (below), is the union of the content extent and the
  layer's. The layer keeps its last row and its columns as the cells do. `Worksheet::dimension` visits every
  stored cell today; it must not cost more afterwards.

**What each mechanism counts afterwards.**

| Mechanism | Afterwards |
|---|---|
| The used area's end and every clip | the content extent: the last row and the last column holding a cell with content |
| `cells_in_columns`, and so `walks_sparsely` and `stored_walk` | cells with content |
| A sparse walk | visits the cells with content of its range, and no blank |
| A dense walk | visits every position of its block, cut at the content extent where a cut applies; a blank's position reads as an empty one, by the same lookup |
| A union walk's charge | `MAX_AREA_WALK` retains its area, index-search, merge, row/chunk-search and cell charges; every occupancy/index input to those charges is derived from content, not from blank-layer entries or row records |
| A lifted call's visits | follow the two lines above |
| The seen-range budget | the strips a cut at the content extent leaves out |
| A search's cut | the last row with content |
| The read index's used-area input | the last row with content: a format on a blank moves no head. SPEC-0002's "The used area" says so |
| The column index (`build_column_index`), `all_positions` | cells with content |
| A spill's footprint test | as today: a stored blank does not block a spill |
| Section 11's stored-cell charge, when it lands | a trip per cell with content |
| `Worksheet::dimension`, `Workbook::used_range`, the export's `<dimension>`, the clipboard, the sheet markup | unchanged: the stored extent, blanks included |

`Workbook::used_range` keeps counting formatted blanks, which is what Excel's used range is documented to do
(above; UNVERIFIED in docs/excel-semantics/formatting.md). A values-only extent for hosts, Excel's `valuesOnly`, is
not part of this amendment.

**Why the charge still bounds the time.**
1. *Reads.* Formula value reads, range walks, counts and clipping consult only the content table. No function
   reads a cell's style today: CELL's six style info types (color, format, parentheses, prefix, protect, width)
   return `#VALUE!` (`functions/information/mod.rs`; information.md, VERIFIED for the web). When they are
   implemented they read one position's style, inside a node that is charged as every node is.
2. *Transitions.* Recalculation also accesses styles while materializing, retiring or restoring spills and while
   substituting and restoring data-table inputs. These paths must preserve each position's style and stored-blank
   state, including on rollback, and they keep decision 0421's retirement comparison ("Transitions inside a
   recalculation"). Their point transfers must not scan or compact unrelated blank entries: each is a
   deterministic bounded point operation, and its bound is in the transition's accounting independently of
   whether the position is formatted. Every cell a transition writes or removes makes one lookup in the layer,
   formatted or not, and at most one insert or remove. In a B-tree that is bounded by the tree's depth times the
   entries of a node: with 11 entries a node, about 150 comparisons at the most a sheet can hold and under 100
   for a million blanks, with the entries of one path's nodes moved at an insert or a remove. It is the same
   for a row with one blank and a row with 16,383. So the cost of a
   transition is a constant a cell, which the charge that cell already carries covers (a spill's cells are
   charged when its array is built, a substitution's run draws on the ceiling), and it has no factor of the
   unrelated blanks of a row. The representation and tests must demonstrate that repeated substitutions and
   spill transitions do not introduce a factor proportional to unrelated formatted blanks. The reviewer of
   CR-321706251 measured what a sorted vector per row costs instead: 100,000 remove-and-reinsert pairs take 633
   microseconds with one blank in the row and 484 ms with 16,383.
3. *Walks.* A walk over positions that hold stored blanks, or over rows that hold nothing but blanks, takes the
   time it takes over the same positions holding nothing. Whatever bounds a walk over empty positions bounds it:
   a sparse walk visits cells with content, a union walk pays `MAX_AREA_WALK`'s classes over content rows and
   chunks only, and a dense walk visits at most `MIN_SPARSE_WALK_POSITIONS` positions or `DENSE_WALK_FACTOR` times
   its columns' cells with content (`stored_walk`: that plus `MIN_INDEXED_ROWS`). Those are section 11's terms.
   The dense walk's positions stay E2's open item there; this amendment neither adds to it nor needs it. The
   range-walk argument alone does not establish the bound of item 2, which is why item 2 is its own.
4. *The case T-5194 names.* A1 holds 1, A2:A1000001 are formatted and hold nothing, and a formula runs
   `MAX(A:A)` in the body of a LAMBDA that MAP calls 1,000 times. Today each call is a sparse walk over 1,000,001
   stored cells, uncharged: 10^9 visits for the body's node units alone (section 11 measured 200 such walks over
   200,000 cells at 3.9 s). Afterwards the content extent is row 1, the range is cut to A1, and each call visits
   one cell. The units are the same with and without the formats, and so is the time.
5. *Who else reads the layer.* None of them is a walk a formula can repeat:
   - the style readers, the exporter and the stored extent, which hosts call;
   - decision 0421's storage rules (`canonical_storage.rs`). `remove_inherited_empty` and
     `remove_inherited_empty_cells` run at an edit or a load. `cell_clear_contents` also runs inside a
     recalculation, for a retired member, and is item 2's point operation;
   - pivot refresh, which reads the styles of its source cells outside evaluation: a blank's is one lookup;
   - the data-table memo (`data_table_memo.rs`), which keeps the content and the style of each cell a
     recomputation read, at most `MAX_MEMO_CELLS`, to decide whether a table must be recomputed. For a position
     with no content it keeps no style, so a format on a blank does not make a table recompute. That loses
     nothing: a table whose formula calls a function that could read a format (`CELL`, `INFO`; the memo's
     `UNMEMOIZABLE_CALLS`) is recomputed every time already. A recomputation is charged as section 4 says, each
     run of a substitution drawing on the ceiling, so a memo miss is never uncharged work. The memo's extents are
     the content extent.
6. *What an edit pays.* An edit that adds or removes a layer entry pays the same point operation as a
   transition does.
7. *A consequence for a shared document.* Today a peer's format operations on blank cells can raise the work every
   replica's evaluation does, up to changing values at the limits. Afterwards a format on a blank adds its own
   operation and its 8 bytes, and no evaluation work.

**The bounds that count stored cells still count the layer.** Moving a blank out of the cell table moves it out of
no bound on memory or admission:
- `Workbook::stored_cells`, and the collaborative engine's `stored_cell_count` over it (what replicas must agree
  on, `rustsheet-wasm`), list a stored blank as before.
- sheet-ops' cap on the cells one operation names (`MAX_CELLS_PER_OP`) counts positions, whatever they come to
  hold.
- An xlsx import is bounded by its inflated input and by the importer's allocation allowances (decision 0433),
  which count the XML and not the stored cells. A valueless `<c>` costs the input it cost, and stores a smaller
  entry.
- A bound added later on a document's stored cells counts both tables. Section 12 gives no reader a reason to
  count the cells table alone, outside evaluation.

**What changes elsewhere.**
- Section 11, rule 1, "Stored-cell walks of a single range": a trip is a cell with content. The walks over cleared
  and styled cells that section measured are no longer walks. Canonical storage (decision 0421) is still what makes
  two replicas save the same file; for the work accounting, the cells with content are a function of the document
  by themselves, since a position holds one exactly when the document holds a value, a formula or a spill there.
- Decision 0421 item 4 ("the last row is the last row holding a cell") reads "holding a cell with content" once
  T-5190 lands; its directory holds the amendment.
- SPEC-0002 Design 1, "The used area": a style on an empty cell is no longer a move of the last used row.
- SPEC-0045 (M4's cell storage, in review): its measures are of cells with content and are not touched. `EmptyCell`
  leaves the row vectors it sizes.

**Not in this amendment.**
- Runs. A layer entry is one position. A block of blanks with one style could be one entry per row, or per
  column as in LibreOffice; that is a change behind the same interface, for when a corpus measurement asks for it.
- The styles of cells with content, which stay in the cell.
- Excel's own behaviour for a format on a blank (T-5187), and whether `Workbook::used_range` should offer both
  extents.
- Bounding the host-facing readers. `Worksheet::dimension`, the clipboard and the sheet markup keep the stored
  extent, so a styled blank at XFD1048576 still stretches it. "Unchanged" there is not "bounded": each must cost
  what the stored entries cost, cells and layer entries, never the positions of the extent. T-5190 checks the
  three and files what it finds; `dimension` visits every stored cell today and must not cost more.

**What the implementation may rely on.**
1. The counts and indexes of `SheetData` need no new rule once the two invariants hold (no `EmptyCell` in a row
   vector, no cell-less row): they count what the row vectors hold, and `next_row_between` finds content rows.
2. A reader that matched `Some(Cell::EmptyCell { .. })` from storage now gets `None`, which every evaluation path
   already treats alike (`evaluate_cell` returns `EmptyCell` for both). The writers that took a style from the
   cell they found (`spill_dynamic_array`, `set_cells_with_result`, `write_constant`) take it from the position's
   style lookup, and `evaluate_substituted` saves a blank's state with the cell it saves today.
3. The layer's entries are positions, and move with the positions under every edit that moves cells.

**What it must show** (T-5190's tests, sharpened).
- T-5190's three reproducers give the same value before and after their blanks are formatted, each at its limit.
- A property: on the metamorphic generators' workbooks, formatting any set of blank positions, inside the used
  area, past its last row and past its last column, changes no value, no error, no `RecalcOutcome` and no work
  unit of a full recalculation, and moves no head of the read index.
- The LAMBDA case of item 4 visits no blank (`SheetData::cells_visited`). Measure its evaluation time separately
  from formatting/setup in an optimized build, record the profile, hardware and workload, and target under 10 ms
  on that benchmark host. Ordinary unit tests assert visit counts and results, not wall-clock timing.
- Rows: a round-trip regression compares a repeated union before and after formatting blank-only rows, including
  at the existing area-walk limit; and the probe above (A1 = 5 and a LET-bound union summed 1,000 times, with and
  without 9,999 rows that hold no cell, built by edits and by a save and a load) gives one value.
- Transitions keep decision 0421's results, in a shrink during evaluation and in a rollback, not only when an
  edit clears or replaces the anchor: a stored blank whose style equals the inherited one, General and a
  nondefault style, under a spill that retires (no blank remains); a member explicitly formatted to the
  inherited style; the inherited style changed in either direction while a member exists (a blank remains with
  the member's style where they now differ, and none where they now match); a data table whose input is a
  formatted blank, after each substitution and after a rollback (the saved blank is back, whatever its style).
- The bound: each of those with one blank in the row and with 16,383 unrelated formatted blanks, asserting that
  the layer's comparisons and moved entries per cell written or removed stay under one constant in both, and that
  no layer operation inside a recalculation visits an entry of another position's row range.
- A blank's style survives what moved it today: a structural edit on each side of it (`structural_edit.rs`'s
  cell moves and their undo records), undo and redo, copy, paste and fill, sort, a merge, a save and a load in
  xlsx and in bytes.
- Decision 0421's tests run unchanged over the layer: the differential of `test_canonical_storage.rs` (its
  40-edit histories) and the replicas' late-joiner tests (`replica_storage.rs`, `replica_files.rs`), whose
  stored-cell counts do not move, since `Workbook::stored_cells` still lists a blank.
- `Workbook::used_range`, `Worksheet::dimension`, `Workbook::stored_cells` and the exported `<dimension>` are
  what they were.
- The bounds: a workbook of valueless styled `<c>` elements at an import bound is refused or admitted exactly as
  before; an operation that formats `MAX_CELLS_PER_OP` blank cells is admitted and one more cell is refused, as
  before; `stored_cell_count` of a live replica and a late joiner agree and count the blanks.
- Mutations that must fail these: an `EmptyCell` left in a row vector; a cell-less row left in the content
  table by an import; a layer entry counted in `column_counts`; the layer consulted by `clip_to_used_area`; a
  retired member's blank decided by what preceded the spill and not by decision 0421's comparison; a restored
  data-table input given the retirement rule; a layer operation that walks its row's entries; a blank's style
  dropped by a row insert.
- The existing cap tests keep their limits, and no bench of `benches/` regresses.

The layer and two changes in flight edit the same files (`sheet_data.rs`, `sheet_data/row.rs`): M4's row
reserve rule (T-5434) and the structural-edit trims (T-5425). Whichever lands second rebases.

**Steps**, each S or M: (a) the layer, its three writers (a style on no cell, a clear, the import) and its
readers (styles, export, the stored extent, decision 0421's functions), with `EmptyCell` out of the row vectors;
(b) the tests above, the dead branches for stored blanks removed from the walks, and the texts of section 11 and
SPEC-0002 brought to the present tense; (c) if T-5185 has landed, its skip extended to formats on blanks (T-5190's
last criterion).

