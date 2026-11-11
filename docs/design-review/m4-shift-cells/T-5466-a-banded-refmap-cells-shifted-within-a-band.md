---
id: T-5466
title: "A banded RefMap: cells shifted within a band, formulas rewritten, refusals, undo"
milestone: M4
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-5463, T-5464]
spec: docs/specs/SPEC-0039-structural-edits.md
---

## Goal
The engine applies an insert or a delete of cells with a shift, and undoes it exactly: the band's cells move as they are, the formulas that read them are rewritten on their trees, and what Excel refuses is refused before anything changes.

## Context
SPEC-0039 Amendment 3, A10 and A12. A shift is the `AxisEdit` of a row or column edit confined to a band of the other axis, so it reuses Design 1's rule set, Design 2's rewrite, Design 5's cell move and Design 6's record. T-5465's probes may not be filled when this starts: each rule then follows Amendment 3's provisional one and is marked UNVERIFIED in code and in `references.md`.

## Acceptance criteria
- [ ] `RefMap::band` and `covers`; every consumer of Design 1's table asks it (`None` unchanged, `Whole` today's rule, `Part` Amendment 3's), and a row or column edit (no band) behaves as before, shown by the existing structural suites unchanged.
- [ ] `SheetData::shift_band` moves the band's cells with their values and styles, never through text; rows and columns keep their own records.
- [ ] The rewrite on the tree: a cell moves when the band covers it at or after the shift; a part follows when the band covers its extent on the other axis whole; a differential compares the op with the same shift done as cell moves and formulas typed at their new places, over random workbooks, and an undo oracle gives the workbook back (`Workbook: PartialEq`, as T-5124's).
- [ ] Every refusal of U12 is typed (`StructureRefusal`, `Merge` included), changes nothing, and has a test.
- [ ] Limits: the edit over formulas at `MAX_VALUE_OPERATORS`, `MAX_SIGNS` and `MAX_FORMULA_DEPTH` completes under `on_small_stack`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The facade ops and the plan (T-5467).
