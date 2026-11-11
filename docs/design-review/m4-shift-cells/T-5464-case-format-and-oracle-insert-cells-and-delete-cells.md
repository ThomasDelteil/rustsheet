---
id: T-5464
title: "Case format and oracle: insert_cells and delete_cells ops replayed through Graph"
milestone: M4
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-5463]
spec: docs/specs/SPEC-0039-structural-edits.md
---

## Goal
A probe case can ask Excel what a shift of cells does: the case format has `insert_cells` and `delete_cells`, the oracle replays them, and the eval harness reads them.

## Context
SPEC-0039 Amendment 3, A11 and A14 step 2. A case's `sheet_ops` name whole rows and columns only (`tools/src/rustsheet_tools/sheetop.py`, `crates/rustsheet-eval/src/case.rs`). The Graph backend already replays those with `range(address=...)/insert` and `/delete` and a `shift` (`oracle/backends.py`, `_RANGE_EDITS`): a shift of cells is the same call with a range's address.

## Acceptance criteria
- [ ] `{"op": "insert_cells", "sheet": "S", "range": "B2:C3", "shift": "down" | "right"}` and `{"op": "delete_cells", ..., "shift": "up" | "left"}` are validated by `sheetop.py` (a range on the grid, a shift that fits the op), recorded by `spec2xlsx`, replayed by the Graph backend with the range's address, and parsed by `SheetOp` in the eval harness; tests in both languages cover a good op and each refusal.
- [ ] The engine adapter answers a typed "not built" for both until T-5466, and a case holding one is reported as such, never as a pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The probe cases (T-5465) and the engine op (T-5466).
