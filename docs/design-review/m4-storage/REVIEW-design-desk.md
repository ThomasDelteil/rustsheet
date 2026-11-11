# Design read of SPEC-0045 (cell storage and memory) and decision 0441 — poc-design-cc, 2026-11-11

Read: `/mail/design-review/m4-storage/SPEC-0045-cell-storage-and-memory.md`, `0441-m4-s-storage-exit-measures.md`,
`measured-mem.txt`; `docs/milestones.md` M4; `docs/amazon/plans/performance.md` 4.2 and 4.7.

**Verdict: go, after F1 to F4.** The shape is right: a measure first, then four small steps that each lower a
constant, and a Part 2 that keeps them. The spec opens with measurements, which is what I asked for. Nothing below
changes the design; F1 and F2 change what T-5434 and T-5437 can promise.

## Your three questions

- **(a) "The largest of three builds":** yes. It is the widest reading, and it is the one the collaborative client
  lives under, since it loads cell by cell.
- **(b) N1 at 64.0 exactly:** acceptable for the exit only if the gate's *count* is at most 64, and only if the
  decision says plainly that N1 has no margin. See F1: I do not think the count will be exactly 64.0.
- **(c) Part 2:** keep it as the sketch in this spec, because it is what shows that nothing is walked back. Give it
  its own spec number when someone is told to start it (CLAUDE.md directive 4), not now.

## Findings

**F1. N1's 64.0 is arithmetic, not a count, and the gate has no tolerance.** Design 6 gets 64.0 as 40 (entry) + 24
(row slot). A one-column sheet of 1,000,000 rows also pays whatever a chunk holds beyond its 1,024 slots and the
table of chunks: about 977 chunks. Any byte there puts the count above 64, and Design 1 says the test compares with
`<=`. Please measure it before the spec promises it: you already build the two types alone for both targets, so
build N1 with a 40-byte entry and exact rows under the counting allocator and put the count in Design 6. If it is
above 64, say how the bar is met (a smaller slot, or a decision that reads the bar with the chunk table left out,
stated as a narrowing). Either way decision 0441 should say that N1 has no margin, and that anything added to a
row slot or an entry breaks the exit until Part 2.

**F2. `load_stored` should set rows whole, and it needs a bench.** Decision 0441 calls the cell-by-cell load "what a
user's browser pays". Under T-5434's rule a ten-cell row loaded cell by cell reallocates nine times where doubling
did five. `load_stored` receives sorted cells, so it can group them by row and call `set_row`, which sizes exactly:
no slack and no reallocation on the client's path, and T-5434's eighth then covers only typing. Add
`Workbook::load_stored` at 1,000,000 cells (N10) to the benches that guard each task. Today's list has `import`
(rows set whole, so it will not move) and a new line for one row typed, and neither is the client's load.

**F3. An error cell is not in the expected figures, and it gets a third allocation.** `Error(Box<ErrorValue>)` is
one pointer in the cell, a 56-byte box, and the two strings' buffers. Your measurement has "every result #N/A" at
175.4 B after a pass today. Design 6 should give its expected figure after T-5433, and the gate should report that
model beside the ledger. Lookup misses make columns of `#N/A` common, and this is the one shape where the box costs
an allocation per evaluation. If the figure disappoints, the typed origin you list under Alternatives stops being
optional for it.

**F4. The performance plan has a second M4 budget, and this design does not meet it.** `performance.md` 4.7 reads
"≤ 64/100 B and 1M x 10 ≤ 400 MB (M4)". N10 at 42.4 B is 424 MB requested for ten million cells, more resident.
`docs/milestones.md` M4 has only the 64 and 100 B bars, so the exit is not affected. Say so in the spec in one
line, with what would meet it (Part 2), so the lead does not read "M4's bars met" as "the plan's M4 budget met".

**F5. Requested bytes are not resident bytes.** Decision 0441 item 3 leaves the allocator's overhead out, and says
so. For N1 that is a million 40-byte allocations, which most allocators round up. Keep the gate as it is, and add
to T-5437's outcome one resident figure per model natively, and the wasm module's memory after a load, so the bar
is not taken for what a tab pays.

**F6. `ArraySize`'s byte order.** The six bytes go into `to_bytes`. Say they are little-endian, whatever the host.

**F7. "No product path persists these bytes."** State the check behind it in the spec (the callers of
`Model::from_bytes` and of the diff lists outside tests), since the cell's stored shape changes with no version
check. A lane that starts persisting them later needs to find this sentence.

**F8. Sparse sheets (optional).** A lone cell still costs its chunk's 24 kB. If it is cheap, let a chunk's slots
grow only to the highest row it holds. Not needed for the exit.

**F9. "No slower" should name the full pass.** Add the full-recalculation throughput bench of T-5165 to the list.
A cell that is 2.5 times smaller should help it, and that is worth showing.

## What I checked and found right

- The three builds and the allocator count are a sound, exact measure, and "constants that may only fall" makes
  every task show its gain.
- The 32-byte cell: the argument for one `Box` and for an align-1 `ArraySize` is the compiler's layout rule, and you
  checked it by building the types for both targets.
- Row slack at an eighth: amortized about nine copies an entry, as stated.
- `TextIndex` as an index-only table hashed through the `Vec` it indexes: one copy of each text, and one definition
  for both lookups.
- What stays (unused shared strings, text results not shared, the row slot) is argued, with the measure that would
  reopen each.
- Stored cells stay a function of the document (SPEC-0028 section 11), which the read index and the work budget
  rely on.
