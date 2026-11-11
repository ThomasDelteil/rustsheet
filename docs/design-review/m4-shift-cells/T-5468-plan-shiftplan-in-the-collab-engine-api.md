---
id: T-5468
title: "plan.shiftPlan in the collab engine API"
milestone: M4
lane: engine
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-5467]
spec: docs/specs/SPEC-0039-structural-edits.md
---

## Goal
The collaborative sheet's engine API answers `plan.shiftPlan`: the dry run of a shift of cells that the model's planner turns into cell writes.

## Context
SPEC-0039 Amendment 3, A13 and A14 step 6. The call is an I-06 query the contracts lane adds in its second wave (its owner's mail of 2026-11-11: an engine-side dry run like `plan.structurePlan`, no new event). `crates/rustsheet-wasm` holds `plan.structurePlan` once T-5128 lands; this call sits beside it.

## Acceptance criteria
- [ ] `plan.shiftPlan` takes the contract's shift (`sheet`, `area`, `dir`) and returns I-06's reply: the refusal with its kind, or the rewritten formulas as stored inputs at their places after the shift; it changes nothing and leaves staged deltas staged; a test covers each refusal kind and a plan a host applies as cell writes to a second engine, which then equals the first after `Txn::insert_cells`.
- [ ] `make check` green; reviewers approve.

## Blocked because
I-06 has no such query yet: it is the contracts lane's wave-2 addition. Unblock when `grep -rq shiftPlan crates/rustsheet-contracts/src` succeeds on `main`.
