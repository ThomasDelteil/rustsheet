---
id: T-5463
title: "SPEC-0039 Amendment 3: insert and delete cells, a shift within a band"
milestone: M4
lane: engine
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0039-structural-edits.md
---

## Goal
The engine's half of Insert Cells and Delete Cells (shift down, right, up, left) is specified: what a shift is in the rule set, what is unknown of Excel, the op, and how a collaborative document takes it.

## Context
Home > Insert and Home > Delete are P0 cards of the collaborative sheet. Their row and column halves are SPEC-0039's; the cell shifts are built nowhere (no dialog or keys in the editor, no planner in the model, no op in the engine: SPEC-0039's first non-goal), as the UI lane confirmed on 2026-11-11. The contract names the op (`DocOp::ShiftCells`, I-03 at 1.0.0, in review), and its owner confirmed that a shift reaches a document as planned cell writes, not as a structure event.

Held by the M4 lane (session poc-m4-cc) since 2026-11-11.

## Acceptance criteria
- [ ] SPEC-0039 Amendment 3 defines a shift as a row or column edit confined to a band (`RefMap::band`, `covers`), lists Excel's unknowns U7 to U14 with a provisional rule each, marked UNVERIFIED, and designs the cells' move, the formula rewrite, the refusals and the undo record on Designs 1, 2, 5 and 6.
- [ ] It says how a document takes a shift (cell writes planned by the model, with the engine's dry run giving the refusal and the rewritten formulas; no `doc.applyStructure`), agreed with the contract's owner, and the design desk has read it.
- [ ] The tasks T-5464 to T-5468 exist with their dependencies.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any code.
- Cut and paste.
