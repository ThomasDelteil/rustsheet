# DRAFT — SPEC-0002 Design 4 and SPEC-0028 section 11 amendment: a dependency-first full pass, and per-pass indexes the document alone decides

Status: draft (s-e2-1, 2026-11-10/11). Reviewed before any code: poc-s-s1-cc (SPEC-0028) approved with 5 changes,
and poc-engine-cc (the evaluator) approved with 6. All of them are applied below. Tracks Taskei P559418140 (a total above its data declines every edit) and P559664607 (SPEC-0028 section 11's
indexes, owner E2). It becomes one task, spec first.

## 1. Problems

1. **Totals above their data.** Take a total `C1 = SUM(B1:B10000)` over `B r = A r*2`.
   - The pass loop reaches C1 in row 1, and C1's SUM starts B2..B10000 inside its own evaluation. So the full pass
     nests, and every literal edit is declined.
   - Measured, native release, 10,000 rows, 100 edits: 449-465 ms with the total in C1, the same as with incremental
     off; 36 ms with the total in C10001.
   - The flat rule stays (Design 4: nesting is excluded, not bounded), so the full pass must nest less.
2. **Section 11's indexes.** SPEC-0028 section 11 charges every input-sized loop no index bounds. A per-pass
   structure is an index when both of these hold:
   - it is built linearly once per pass;
   - whether a read uses it is a function of the document alone.

   Two caches fail the second condition today:
   - the running totals' prefix cache (`functions/math_and_trigonometry/sum_prefix.rs`);
   - the criteria aggregates' cache (`functions/statistical/ifs_cache.rs`, T-5143).

   Their use depends on two things that are not the document's:
   - `read_is_inert`: whether the range's formula cells are evaluated yet, which follows the order of the pass;
   - global capacities (`MAX_KEPT_STATES`, `MAX_CACHED_CELLS`) that other formulas can use up.

   Charged as misses, a running total filled down 100,000 rows is 2.4 to 2.9 times the pass allowance. A per-row
   SUMIFS is 2 times it.

The two problems share a cause: the pass's order, not the document, decides when a range's formulas are evaluated.
One rule fixes both.

## 2. The dependency-first full pass (SPEC-0002 Design 4)

In a full pass not in Excel's order (SPEC-0017), before the loop evaluates a root R, it evaluates the formula cells of
R's static footprint that are not evaluated yet, depth first, from its own loop, with an explicit stack:

1. Push R. While the stack holds a cell T, take the first formula cell of T's footprint, in placement order, that is
   not evaluated, is not on the stack and is a candidate.
   - If there is one, push it.
   - Else pop T and evaluate it with `evaluate_root_cell` at depth 0.
2. A formula cell is a **candidate** when the read index's walk gives its footprint (`read_index/walk.rs` `leaves`
   and `instantiate`, the one definition) and it is no dynamic anchor or spill cell. Which formulas the walk refuses
   is defined by T-5132's kinds (the always-dirty list), not restated here (poc-engine-cc, change 5): OFFSET,
   INDIRECT, an INDEX that returns a range, a volatile function, a name, LAMBDA among them. A refused formula is no
   candidate, and nor is any formula reachable only through one. If such a formula is read, it nests on demand, as
   today.
3. A cell already on the stack is a static cycle. **On finding one, R falls back**: it is evaluated as today, in
   row-major order, with nothing of its footprint evaluated first, and the cycle is left to today's on-demand
   evaluation and circular detection (poc-engine-cc, change 1). Today a static cycle is entered at the member the
   row-major loop reaches first; without the fallback the new pass could enter it at another member, and a cycle's
   values can depend on where it is entered. The wasm crate's order-independence test already lists loops as
   order-dependent (TODO(T-5288)); this pass must not widen that.

Finding a rectangle's formula cells uses a per-sheet list of formula positions by column, sorted by row. The list is
built once per pass, O(F log F), and searched in O(log F + hits) per rectangle. Footprints are walked once per
distinct parsed formula per pass, or taken from the read index when one is built.

This traversal is unmetered setup in SPEC-0028 section 11's terms, counted in T_setup (section 11's T_setup + P x
T_pass), and bounded. The position lists are measured on the 60,000-formula book before the code lands: at most 2 ms
to build, or else they are kept between passes and rebuilt only when `layout_version` changes (poc-engine-cc,
change 3).
- the position lists cost O(F log F);
- the footprint walks cost their distinct parsed formulas' nodes;
- the searches cost the footprint rectangles' hits.

The explicit stack holds at most F cells (directive 5).

What it guarantees:
- Every candidate is evaluated with its static inputs evaluated. For a document whose candidates' footprints are
  acyclic, the full pass is flat, so the total-on-top layout is restricted again.
- **Values in normal mode do not change**: a formula's value is a function of its inputs.
- **The order changes only where a footprint reaches a formula not yet evaluated.** A footprint is a superset of the
  reads, so an IF's unread branch can move a formula earlier; both orders are flat.
- RAND draws from one sequence on main, so a change of order changes which cell draws what. RAND is volatile and
  draws again on every recalculation, so no history pins its values. In the collab profile, a draw is
  `f(seed, cell)` (I-07).
- Anchors still run first, in `anchor_order`. Excel's order is untouched.
- The depth-limit unwinds of a chain read from the top go away: its thrown-away work becomes 0. The tests that pin
  today's figures change to the new ones and keep their checks (T-5152's sweeps, SPEC-0028's thrown-away tests).

## 3. Per-pass indexes the document alone decides (SPEC-0028 section 11)

### 3.1 The running totals and the criteria aggregates become indexes

1. **Use decided by the document.** With the pass of section 2, a range whose formula cells are all candidates has
   them evaluated before any reader of the range runs. So `read_is_inert` holds for it by construction, decided by
   the formulas, not by the order.
   - Such a range is read through the index whenever the index has room for it, and the room is per range (rule 2).
   - A range holding a non-candidate, a dynamic reference (OFFSET, INDIRECT, an INDEX that returns a range) or a
     cycle's member is not read through the index: its reads charge their walk, as section 11 says.
   - Which reads use the index is then a function of the document: its formulas, its values and its stored cells.
2. **Capacity per range, from the document.**
   - Each index's room is the range's own size, never a share of a global capacity other formulas can use up:
     `MAX_KEPT_STATES` and `MAX_CACHED_CELLS` go.
     - A running total keeps one state per stored cell of its start's prefix.
     - A criteria index keeps one bit set per distinct criterion of a range, and its read values once.
   - What bounds the total is the charge, not the stored cells alone: a thousand running totals with different
     starts over a million-row column would be 10^9 states. Each state held is charged when built, against the
     pass's ceiling, outside any formula's run (section 11's "counted once per pass"). So the indexes hold at most the
     pass allowance's worth of states, each within a stated byte bound (directive 5).
   - The build walks the ranges in an order the document alone fixes: sorted by (sheet, top row, left column, bottom
     row, right column). A range whose index would pass the ceiling, and every range after it in that order, gets
     none, and their reads charge their walk.
   - Which reads use an index is then a function of the document, and of its stored cells. The stored cells are not
     yet the same on every replica (Taskei P559664573, canonical storage). P559664573 is the engine lane's, with its
     CR on 2026-11-11 (poc-engine-cc, change 6). So the index rule depends on it, and the live-against-loaded half of
     its test lands with it.
   - At the ceiling, values can change: a range past the cut-off charges its walk, and a pass that runs out of units
     stops where the units say. The cut-off follows the document's order, so two replicas of one document cut at the
     same range. The tests build documents at the ceiling's edge and check that replicas and both entry orders give
     the same values and spend the same units (poc-engine-cc, change 2).
3. **A read through the index charges a constant.** Any other read charges its walk, a cached one included: Design
   4's "the same on a hit and a miss".

### 3.2 The dense walk: option (a)

The dense walk goes for walks that charge. A charged walk visits stored cells only, so its trips are the stored-cell
count section 11 charges. The used area does not become an input of every range reader in the read index, which
would dirty every range reader on every last-row move. T-5149 keeps that input to tailed array arguments.

### 3.3 What this gives the read index

The conditional aggregates and running totals stop being "searches" whose charge follows the order (Design 4, "A
search"; T-5151, T-5143). A full pass that ran one need no longer decline restricted passes, and T-5192 (restricted
passes for workbooks with lookups) gets its precondition.

## 4. Tests

The rows marked (spec) are stated by the spec.

| Test | What it checks |
|---|---|
| **Total on top** | Restricted, with values equal to a full recalculation; declined without the pass. |
| **Same units in two orders** (s-s1) | A running total and a SUMIFS column, in two orders. Entry order alone may not change the pass, since placement order does not follow it. So the second order comes from (a) the same document built by edits (with clears) and loaded through `Engine::load`, after P559664573; and (b) the same document with its sheets' formulas stored in different internal orders, built row by row and column by column. Both give the same values and spend the same units in a full pass. |
| **Flat documents unchanged** | Where no footprint reaches ahead, the order is unchanged, and so are the values, bit for bit, on the metamorphic flat histories. |
| Metamorphic | Gains a totals-above-data generator: incremental = full = fresh over 10,000 sequences. |
| Depth and stack sweeps | T-5152's sweeps at depth 1 and stack 0, with their new figures. |
| RAND | Under the collab profile's seeded draw, the same values in either order. |
| Cycles (poc-engine-cc, change 1) | A 2-cycle, a 3-cycle, a cycle with a reader above and below it, and a cycle through a SUM range, each entered from every member by moving the root. Its values and its readers' values equal today's. |
| Ceiling edge (change 2) | Documents whose ranges cross the indexes' ceiling: replicas, and both entry orders, give the same values and spend the same units. |
| Replays (change 4) | T-5298's replay tests pass unchanged, and an OFFSET chain is kept among them: a chain the walk refuses still nests on demand and gives today's values. |
| Index bound (spec) | Each state held is charged at build, and stays within the byte bound. A document past the ceiling gets no index for the ranges after the cut-off in the stated order, and their reads charge their walk. |

## 5. Effort and order

- Spec: about 1 day.
- Code: the pass about 2 to 3 days; the indexes about 2 days.
- Tests: about 1 day.
- Reviews: 2 to 3 rounds.
- About 2 engineer-weeks in all.

It goes after the default flip (T-5154), and after T-5132 (the always-dirty list), whose candidates' rule it uses.
