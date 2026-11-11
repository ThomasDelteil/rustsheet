# T-5145 section 1: index maintenance (T-5166) — design read for the project lead

*poc-design-cc, 2026-11-11. Status: draft for a design read. Nothing here is normative until the CR lands.*

## What is proposed

The amendment for index maintenance already exists, and it is not mine. poc-s-e2-1-cc wrote it with the
implementation: SPEC-0002 Design 2, "Maintenance", on branch `e2/t5166-main` (CR-321604031, round 1 answered in
commit 33b7c8d87). I read the text and the code (`crates/rustsheet-engine/src/read_index/maintain.rs`,
`Model::refile_formula`). I propose that the T-5145 CR does **not** restate it. T-5145 records where it lives
(Design 7 becomes a table of amendments), and each later section says what maintenance must do for the forms that
section adds. One definition, in one place.

## The design in ten lines

1. Today a formula write makes the index stale, and the next edit of a literal rebuilds it: O(formulas), 0.1 to
   0.8 s per million formulas natively (SPEC-0002 Design 1's table: 107 ms for one shared range, 829 ms for ten
   own reads a formula).
2. With maintenance, a transaction that writes only cells keeps the index current. Each written cell whose formula
   changed is taken out (`ReadIndex::unfile`) and filed again by the build's own path (`Model::file_formula`).
3. The edit then recalculates as a write of literals does (Designs 3 to 6), with the written formula cells added to
   the closure. A new formula that reads a later cell, or closes a loop, makes the pass nest or read ahead, so the
   pass is rejected and a full pass follows. The values are a full recalculation's either way.
4. The index still goes stale, and the edit recalculates fully, when a written cell was or became an array formula
   or a spill cell, when it holds a formula the index does not model, on any op that is not a cell write (sheet,
   name, table, structural), and when maintenance is worn.
5. Worn: maintenance counts the bytes of every list it scans. Past four times what the index holds, or past as many
   dead shared rectangles as live ones plus 32, it gives up and the index is built afresh. So a transaction never
   costs more than about two-thirds of a build before its full pass.
6. Measured by its author (release, 100,000 cells): a formula written in the middle of the ledger costs 8.25 ms
   against 34.37 ms with incremental off.

## What I checked

- The exactness argument holds under Design 4's rules, with no new proof needed. A reader of the written cell is
  in the closure. A reader placed before the written cell starts its evaluation inside its own, which is the
  nesting Design 4 rejects. The summing bound keeps the old formula's work and adds the new one's, so it stays an
  upper bound.
- The old formula is taken out from the same parsed tree it was filed from: the shared-formula table is compacted
  only after the refile (`compact_if_orphaned`).
- What the index remembers about closures past the limit is dropped at every refile, as it must be.

## What the later sections add to it (they are in sections 4 and 5)

- **Always-dirty list** (T-5132): a written formula the index does not model joins the list. It no longer makes
  the index stale. Item 4 above changes with T-5132, not before.
- **Names** (T-5133): a refile resolves names as the build does. A name op stays a rebuild.
- **OFFSET and INDIRECT**: nothing. By the project lead's ruling they are always dirty (section 4, 11.1), and
  their own rules are a task that records ideas (T-5420).
- **Spills** (T-5134): a write over an anchor or a spill cell stays a full pass in the first slice.
- **Position readers** (Design 8, T-5136): a write that adds or removes one rebuilds the index. Design 8 already
  says so; T-5136 implements it.
- **Calculation modes** (section 2): maintenance runs at each transaction, in every mode. Only the evaluation is
  deferred. Until this maintenance is on main, a formula write in Manual or OnDemand mode owes a full pass.

## Open questions

1. Do you agree that T-5166's amendment stays in its own CR (CR-321604031), and T-5145 only points to it?
   (yes/no; my answer: yes)
2. Sheet, name and table operations still rebuild the index: 0.1 to 0.8 s per million formulas natively (the same table), more in
   wasm. A structural edit rebuilds it too, until SPEC-0039 Design 9 re-keys it in place (T-5279 measures first).
   Is that acceptable for the first release of incremental recalculation? (yes/no; my answer: yes, with the
   measurement T-5279 already owns)
3. None about Excel. Maintenance is not observable in values, so it needs no probe.
