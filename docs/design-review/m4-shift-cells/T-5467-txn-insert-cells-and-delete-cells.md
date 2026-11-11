---
id: T-5467
title: "Txn::insert_cells and delete_cells, with a plan of what a document writes"
milestone: M4
lane: api
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-5466]
spec: docs/specs/SPEC-0039-structural-edits.md
---

## Goal
The facade has `Txn::insert_cells` and `Txn::delete_cells`, and `Workbook::plan_shift` gives a collaborative document's planner what it cannot derive: the refusal, and each formula the shift rewrites with its stored input after it.

## Context
SPEC-0039 Amendment 3, A13. A document takes a shift as cell writes (I-03's `shiftCells`): the planner derives the moves from the area and the direction, and asks the engine for the rest in a dry run. No engine applies a shift op in a collaborative document.

## Acceptance criteria
- [ ] `Txn::insert_cells(sheet, area, Shift::Down | Shift::Right)` and `Txn::delete_cells(sheet, area, Shift::Up | Shift::Left)`: validated before anything changes, refused with `EditError::Structure`, rolled back exactly, with the shift in `ChangeSet.structure` and the changed values in `ChangeSet.values`.
- [ ] `Workbook::plan_shift` is a dry run that changes nothing and returns the refusal or the rewritten formulas at their places after the shift, each as a stored input; the plan's purpose is a property test: the op on one copy equals, in the canonical live state, the moves (derived from the area and direction alone) and the plan's formulas written as stored inputs on another.
- [ ] The Python bindings mirror the ops; SPEC-0001 is amended for the new ops and the field.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine API's call (T-5468); the model's planner and the editor's dialog (their lanes).
