# Proposal: `Applied` carries the subscribed tiles' changed cells; the rest is named stale (I-06)

From E2 (poc-s-e2-1-cc), 2026-11-10, after the coordinator's ruling on the E2 gate: "edit to the changed cells of the
subscribed tiles being delivered, p95 at most 16 ms at 100k cells in wasm". The coordinator wrote "I-07"; the reply
shape is I-06 (engine protocol v2 and tiles v2, `INTERFACES.md` L534; `pkg/f1`
`crates/rustsheet-contracts/src/engine/doc.rs` `Applied`), so this is an I-06 proposal. I-07 (determinism, the value
digest) is unchanged by it.

## What is there today (read on disk)

- `pkg/f1` `engine/doc.rs` `Applied`: `changed: Vec<ChangedCell>` is "Computed cells whose value changed", each with its
  value and display text; `tiles: TileSet` is "Subscribed tiles to refetch". `engine/view.rs` `Subscribe`: the tiles
  the page shows (I-08's `tilesOf(viewport)` plus a prefetch ring), replaced whole.
- `crates/rustsheet-wasm/src/engine.rs` `Engine::recalc` (main) builds a `CellOut` for every cell in the facade's value
  diff (`cell_out_at`: value, display text, formula flags) and returns them all.

## Why it matters (measured, node, wasm, the 100,000-cell ledger, main + the E2 stack)

- An edit of A5000 changes 5,000 balance cells; its 41 ms are 44% recalculation, 45% building those 5,000 cells' reply
  (display formatting 25%, formula printing 13%) and 6% JSON. Random edits of A..E: p95 70-77 ms.
- A page shows about 40 rows: the subscription is a handful of tiles (each 64 x 16), so at most a few thousand cells
  and, on this workload, a few dozen changed ones. The other ~5,000 changed cells are formatted, serialised and posted
  for nobody.

## The change

1. `Applied.changed` lists the changed computed cells **inside the subscribed tiles**, and the cells of the ranges a
   `query.watch` holds (charts, I-13), with value and display as today.
2. A new field `Applied.unseen: Vec<TileKey>`: every tile outside the subscription that holds a changed computed cell,
   each once, sorted. Its size is at most the changed cells / 1, usually changed cells / 64 (a column's run).
3. **Consistency rule** (the one a client needs): a tile the page cached is *current* only while no reply since its
   fetch has named it in `unseen` (or in `tiles`). A page that holds a cached copy of a tile named in `unseen` drops it
   (or marks it stale and never paints it as current); on subscribe or scroll it fetches the tile with `view.tiles`,
   whose values are the engine's current ones at the reply's `rev`/`gen`. So a stale value is never shown as current.
4. Unchanged: the engine holds every value (the value digest, I-07, is over all of them); `query.cell`,
   `query.readRange` and `view.tiles` read current values; `calc.recalc` has the same shape.
5. Versioning: a feature `doc.applyCells@2` (I-06 features are versioned `family.op@n`); `@1` (every changed cell) stays
   for tests and the standalone build.

## What it buys, and what it does not

- The reply's cost follows the visible changes, not the closure: on the ledger the 45% + 6% go to a few dozen cells.
  Expected A5000 edit about 20 ms (the 44% recalculation, 18 ms), random A..E p95 about 30 ms (the recalculation of
  the closure near the top). To be measured with the change, on both workloads the ruling names.
- It does not make the recalculation of an off-screen closure faster. Reaching 16 ms at p95 on the ledger also needs
  either incremental aggregates (the 5 column totals re-sum 10,000 cells each per edit, 19% of the edit; filed) or a
  visible-first pass (evaluate the subscribed tiles' dirty cells and their inputs first, reply, finish the closure
  before the next op). The second is a SPEC-0002 amendment: a restricted pass is exact only as a whole (its work bound
  decides share mode, SPEC-0028), so values sent early would need a provisional mark until the pass completes. Not
  part of this proposal.

## Who does what

- I-06 (F1 steward poc-s-f1-cc, with the engine lane): the field, the rule, the feature flag, fixtures.
- rustsheet-wasm (E2 slice b, poc-s-e2-2-cc): filter by the subscription in `Engine::recalc` / `apply_structure`.
- The page (I-05/I-08 owners): drop or mark cached tiles named in `unseen`.
- E2 (poc-s-e2-1-cc): the gate's measurement on both workloads with the change; reported "not met" until it lands.
- Review: I-06's owner and poc-i-m2-cc (per the coordinator).
