# DRAFT — SPEC-0002 Design 4, amendment: a dependency-first full pass (Taskei P559418140)

Status: draft for the engine lane's review (s-e2-1, 2026-11-10). It becomes its own task: spec first, then code.

## Problem (measured)

A total above the formulas it reads is the layout of a report. Example: `C1 = SUM(B1:B10000)` over `B r = A r*2`. In
it, every full pass nests: the pass loop reaches C1 in row 1, and C1's SUM starts B2..B10000 inside its own
evaluation. A restricted pass runs only after a flat full pass (`restricted_pass_allowed` needs `last_full_flat`), so
every literal edit is declined.

Native release, 10,000 rows, 100 edits of column A:

| total at | path | on | off |
|---|---|---|---|
| C1 | declined 100 | 449-465 ms | 444-464 ms |
| C10001 | restricted 100 | 36 ms | 461-477 ms |

That is a factor of 12.8 lost for the layout people read reports in.

## Why the flat rule stays

"Nesting has to be excluded rather than bounded" (Design 4) still holds:
- A nested run can be cut short by the depth limit or the stack limit (256 KiB), and its work is then thrown away.
- A dirty IF that changes branch changes the route and the depth by which a full pass reaches a clean cell.
- The stack a nested start sees depends on how deep in its reader's tree the reference sits, and on the build.

So this amendment does not weaken the rule. It changes the full pass so that it nests less.

## The rule

In a full pass that is not in Excel's order (SPEC-0017), before the pass loop evaluates a root R, it evaluates, from
its own loop, the formula cells R's static footprint holds that are not evaluated yet. The walk is depth first, with an
explicit stack, never recursion:

1. Push R. While the stack is not empty, look at its top T.
2. Take the first formula cell in T's footprint, in placement order, that is not evaluated, is not on the stack and
   is a candidate (below). If there is one, push it. If there is none, pop T and evaluate it with the ordinary
   `evaluate_root_cell`, at depth 0.
3. A formula cell is a candidate only when its own footprint is known:
   - the read index's walk gives its references (`read_index/walk.rs` `leaves` and `instantiate`, the one definition
     of a footprint);
   - it holds no function the walk refuses (the always-dirty kind of T-5132: volatile, INDIRECT, OFFSET, names,
     LAMBDA and the rest);
   - it is no dynamic anchor and no spill cell (anchors run first, in `anchor_order`).

   A non-candidate is left where it is. If T reaches it, it nests as it does today.
4. A cell already on the stack is a static cycle. It is left to today's on-demand evaluation and circular detection.

Every formula the walk reaches is then evaluated from the loop with its static inputs already evaluated. If the static
footprints are acyclic and every formula has one, the pass is flat. The total-on-top layout is flat, and its edits are
restricted again.

## What it changes, and what it does not

- **Values in normal mode do not change.** A formula's value is a function of its inputs, whatever the order.
- **The order changes only where a footprint reaches a formula not evaluated yet.** A footprint is a superset of the
  reads: both branches of an IF. So even a workbook whose pass is flat today can have a formula evaluated earlier
  than its natural place, when an IF's unread branch names a later formula. Both orders are flat, and a flat
  formula's work depends on its inputs alone (Design 4, "Why this is exact"). So the work bound and the share-mode
  outcome do not change.
- **What can see the order, and how each is held:**
  - The search cache's and the IFS cache's charges follow the order of the searches. A full pass that searched
    already declines restricted passes (Design 4, "A search"). Its values in normal mode do not change.
  - RAND draws from one sequence on main, so a change of order changes which cell gets which draw. RAND is volatile:
    every recalculation draws again, so no history pins its values. In the collaborative sheet's profile the draw is
    `f(seed, cell)` (I-07), which no order changes.
  - Spill order: anchors run first, in `anchor_order`, and the walk never moves one.
  - Depth and stack limits (T-5152): the walk evaluates at depth 0, so a chain that unwinds today, such as
    `A r = A(r+1) + 1` read from the top, no longer unwinds. Its thrown-away work goes to 0. A test that pins today's
    unwinding through such a chain changes: it must state the new figure, not lose its check.
- **Excel order (SPEC-0017)** is unchanged. It already never nests.
- **Fresh = full:** the walk is a function of the formulas and the pass state. A reload walks as the live model does.

## Cost

- Footprints: the walk per distinct parsed formula, as the index build already does once per sheet (T-5284). It is
  kept per pass, or shared with the index when one is built.
- A rectangle's formula cells: a per-sheet list of formula positions by column and row, built once per pass in
  O(F log F), and searched in O(log F + hits) per rectangle. Running totals over a literal column find no hits. A total
  over a column of formulas finds them, as its SUM reads them anyway.
- To be measured before and after on the bench matrix (`incremental_patterns`): the full pass's own time at 100k cells
  for the ledger, the running totals, the total on top and the chain from the top.
- Budget: at most 10% on a full pass whose order does not change.

## Tests

- The total on top: restricted, with values equal to a full recalculation. It fails without the walk (declined).
- The metamorphic suite gains a generator shape with totals above their data. Incremental = full = fresh, 10,000
  sequences.
- Flat workbooks: the order and the values are unchanged where no footprint reaches ahead, compared bit for bit with
  the previous pass order on the flat histories.
- The depth and stack sweeps of T-5152 at depth 1 and stack 0, with their new figures.
- RAND under the collab profile's seeded draw: the same values in either order.

## Effort

Spec amendment 1 day; code 2 to 3 days; tests 1 day; 2 to 3 review rounds. 1 to 1.5 engineer-weeks.

## Open questions for the engine lane

1. Is the full pass's order an E1 contract (the determinism fingerprint, I-07)? If so, the order change needs E1's
   decision record.
2. Should the walk reuse the read index when one is built (no walk cost), and build footprints only when none is?
3. Do any of the engine's tests pin today's nesting? For example, the unwinding sweeps, or a SPEC-0028 thrown-away
   figure.
