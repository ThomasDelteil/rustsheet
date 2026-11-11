---
id: T-5465
title: "Probe a shift of cells: references, ranges across the band's edge, refusals"
milestone: M4
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-5464]
spec: docs/specs/SPEC-0039-structural-edits.md
---

## Goal
Excel's behaviour for SPEC-0039 Amendment 3's unknowns U7 to U14 has oracle evidence.

## Context
SPEC-0039 Amendment 3, A11. Every rule there is from memory and marked UNVERIFIED; the engine op (T-5466) and the model's planner follow what these cases find.

## Acceptance criteria
- [ ] Probe cases (excel-probe skill), each with FORMULATEXT beside its formula, for the four shifts: U7 (a cell reference into the band at, after and before the shift, and beside the band), U8 (a range the band covers: inside, across the shift's first line, deleted whole), U9 (a range across the band's edge, and one that holds the band), U10 (whole rows and columns, a 3-D span, another sheet's reference), U11 (names), U13 (a style, a conditional format's area, a link, a merge the band covers) and U14 (the inserted cells; a table the band holds whole).
- [ ] U12's refusals each as a case whose wanted answer is the refusal and its error code: a CSE array and a spill the band cuts, a merge across the band's edge, a table the moved cells touch in part, a data table, a pivot, a nonblank cell pushed off the grid.
- [ ] The cases are sent as a probe request with the engine's provisional answers frozen beside them; `references.md` records each finding when the fill lands, and the provisional rules of Amendment 3 are replaced by the evidenced ones.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine.
