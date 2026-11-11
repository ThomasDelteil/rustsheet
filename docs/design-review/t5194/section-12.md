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
| A union walk's charge (`MAX_AREA_WALK`) | a step per stored cell |
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
  `column_counts`, `column_rows`, `known_last_row`). `Cell::EmptyCell` is never in a row vector again, so each of
  these counts cells with content without a line of it changing. `EmptyCell` stays the value an empty position
  reads as.
- **The blank layer**: the styles of the stored blanks. Per row, a vector of (column, style) sorted by column, in a
  chunk table of the cells' shape, a chunk allocated at its first entry. A sheet with no stored blank holds an
  empty table, so the cell models of M4's storage measures do not grow.

A position is in at most one of the two, and its style is in one place: a spill member over a formatted position
keeps the style in its `SpillCell`, and the layer holds none there, so the retired-member rule of decision 0421
counts it once. Which blanks are stored does not change: decision 0421's rules decide that, and they now add to
and remove from the layer. Only where a blank is kept changes.

The layer is reached only through `SheetData`'s methods: a blank's style at a position, the stored blanks of a
range in order, and its extent. No reader holds its row vectors. So its shape can change without its readers: M4's
later sketch of columnar chunks would keep styles as runs per column for cells and blanks alike, and absorb it
(the M4 lane's condition).

**What it costs to keep.**

| Operation | Today | With the layer |
|---|---|---|
| A style on a position with no cell | an `EmptyCell` enters the row vector | a (column, style) enters the layer's row: a binary search and an insert |
| A value or formula written over a stored blank | the cell is replaced in place | the layer's entry is removed and its style goes to the new cell |
| A cell cleared, where a blank stays stored | the cell becomes `EmptyCell { s }` | the cell is removed and its style enters the layer |
| A structural edit (SPEC-0039) | the row vectors are moved or re-keyed | both tables, by the same rule; the layer's part costs its own entries |
| Undo and its records, copy and paste, the fill plan, sort, a merge's covered cells | carry the `EmptyCell` | carry the layer's entry, by position |
| Decision 0421's rules at a load and after a batch (`drop_inherited_empty_cells`, `Txn::drop_inherited_empty`) | test and remove an `EmptyCell` whose style is the inherited one | the same test, on the layer's entry |
| xlsx import of a valueless `<c>` | an `EmptyCell` | a layer entry |
| xlsx export of a row | its cells in column order | its cells and its layer entries, merged in column order |
| `Model::to_bytes` | encodes the chunks | encodes both tables. The form changes; nothing persists these bytes today and they carry no version (Taskei P559862286), so it needs no step shared with M4's cell |

Memory, by the M4 lane's figures. An entry is 8 bytes, where a blank is an `(i32, Cell)` entry of a row vector
today. Around it the layer pays what the cells' table pays: 24 bytes for a row that holds a blank, and a chunk's
1,024 slots, 24 kB, at the chunk's first entry. So one formatted blank alone in its chunk costs 24 kB, and a
formatted block ten columns wide costs about 10.4 bytes a blank. The layer's rows reserve by the cells' rule
(T-5434, one definition in `sheet_data/row.rs`), not by doubling.

**What it costs to read.**
- The content extent is read as the stored extent is today, at today's cost: `last_row` from `known_last_row`, with
  a scan of chunk slots only when the last row loses its last cell, which visits no cell; `last_column` as the last
  key of `column_counts`. T-5194 required that the content extent, like `last_row`, not visit every cell.
- There is still one definition of the last used row. `Worksheet::last_used_row` is what a tailed read's head, a
  search's cut and the read index's `Incremental::last_rows` all read today, and T-5190 changes that one function.
  It adds no second extent for evaluation to choose from, so the tail and the index cannot part; T-5149's used-area
  oracle, which checks the index against every tailed read and cut search, runs on its branch (the read-index
  lane's condition).
- A blank's style is a chunk lookup and a binary search in its row's entries, as a cell's is today.
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
| A union walk's charge | a `MAX_AREA_WALK` step per cell with content |
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
1. *No evaluation path can tell a stored blank from an absent cell.* Evaluation reads the cells table, its counts
   and its extent. No function reads a cell's style today: CELL's six style info types (color, format,
   parentheses, prefix, protect, width) return `#VALUE!` (`functions/information/mod.rs`; information.md,
   VERIFIED for the web). When they are implemented they read the style lookup, which then reads the layer: one
   position's style, inside a node that is charged as every node is, and never a walk. So the bound survives
   them. The layer's readers are these, none of them a walk a formula can repeat:
   - the style readers, the exporter and the stored extent, which hosts call;
   - decision 0421's storage rules (`canonical_storage.rs`: `cell_clear_contents`, `remove_inherited_empty`,
     `remove_inherited_empty_cells`), which run at an edit or a load;
   - pivot refresh, which reads the styles of its source cells outside evaluation: a blank's is one lookup;
   - the data-table memo (`data_table_memo.rs`), which keeps the content and the style of each cell a
     recomputation read, at most `MAX_MEMO_CELLS`, to decide whether a table must be recomputed. For a position
     with no content it keeps no style, so a format on a blank does not make a table recompute. That loses
     nothing: a table whose formula calls a function that could read a format (`CELL`, `INFO`; the memo's
     `UNMEMOIZABLE_CALLS`) is recomputed every time already. A recomputation is charged as section 4 says, each
     run of a substitution drawing on the ceiling, so a memo miss is never uncharged work. The memo's extents are
     the content extent.
2. *So a walk over positions that hold stored blanks takes the time it takes over the same positions holding
   nothing.* Whatever bounds a walk over empty positions bounds it: a sparse walk visits cells with content, and a
   dense walk at most `MIN_SPARSE_WALK_POSITIONS` positions or `DENSE_WALK_FACTOR` times its columns' cells with
   content (`stored_walk`: that plus `MIN_INDEXED_ROWS`). Those are section 11's terms. The dense walk's positions
   stay E2's open item there; this amendment neither adds to it nor needs it.
3. *The case T-5194 names.* A1 holds 1, A2:A1000001 are formatted and hold nothing, and a formula runs
   `MAX(A:A)` in the body of a LAMBDA that MAP calls 1,000 times. Today each call is a sparse walk over 1,000,001
   stored cells, uncharged: 10^9 visits for the body's node units alone (section 11 measured 200 such walks over
   200,000 cells at 3.9 s). Afterwards the content extent is row 1, the range is cut to A1, and each call visits
   one cell. The units are the same with and without the formats, and so is the time.
4. *Nothing became free that costs time.* A style read of a blank is a lookup, and no formula can cause one.
5. *A consequence for a shared document.* Today a peer's format operations on blank cells can raise the work every
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
1. The counts and indexes of `SheetData` need no new rule: they count what the row vectors hold.
2. A reader that matched `Some(Cell::EmptyCell { .. })` from storage now gets `None`, which every evaluation path
   already treats alike (`evaluate_cell` returns `EmptyCell` for both).
3. The layer's entries are positions, and move with the positions under every edit that moves cells.

**What it must show** (T-5190's tests, sharpened).
- T-5190's three reproducers give the same value before and after their blanks are formatted, each at its limit.
- A property: on the metamorphic generators' workbooks, formatting any set of blank positions, inside the used
  area, past its last row and past its last column, changes no value, no error, no `RecalcOutcome` and no work
  unit of a full recalculation, and moves no head of the read index.
- The LAMBDA case of item 3 runs in under 10 ms with the formats and visits no blank
  (`SheetData::cells_visited`).
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
- Mutations that must fail these: an `EmptyCell` left in a row vector; a layer entry counted in `column_counts`;
  the layer consulted by `clip_to_used_area`; a blank's style dropped by a row insert.
- The existing cap tests keep their limits, and no bench of `benches/` regresses.

The layer and two changes in flight edit the same files (`sheet_data.rs`, `sheet_data/row.rs`): M4's row
reserve rule (T-5434) and the structural-edit trims (T-5425). Whichever lands second rebases.

**Steps**, each S or M: (a) the layer, its three writers (a style on no cell, a clear, the import) and its
readers (styles, export, the stored extent, decision 0421's functions), with `EmptyCell` out of the row vectors;
(b) the tests above, the dead branches for stored blanks removed from the walks, and the texts of section 11 and
SPEC-0002 brought to the present tense; (c) if T-5185 has landed, its skip extended to formats on blanks (T-5190's
last criterion).

