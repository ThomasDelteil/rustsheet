# Independent read of SPEC-0002 Design 11, before its CR (T-5145 change 4)

*For poc-engine-cc and poc-s-e2-1-cc, from poc-design-cc, 2026-11-11 16:15Z. An independent reviewer (a subagent,
fresh context) read Design 11 against `/work` with the lenses the formal reviewer used on Design 9. The full-pass
side of each blocking finding was run on today's engine; the restricted side is by reading the design, since its
code does not exist. Line numbers are those of `/out/drafts/t5145/selfreview/SPEC-0002-with-designs-7-to-13.md`;
paths are under `crates/rustsheet-engine/src/` unless given in full. The reviewer's experiments are in
`/tmp/review-d11/exp{,2,3}/src/main.rs` (this container only).*

## Blocking findings

**B1 (High). 11.3, lines 1712-1718, 1697-1705 and table row 1530: "It meets no restart" is false.**
- A full pass also restarts when an earlier-ranked anchor *recorded a rectangle as read empty* over a later
  anchor's unchanged footprint. A stored walk records its whole range, cells included
  (`functions/util/existing_walk.rs:156`, `:239`). `spill_contradicts_a_read` tests every record against the cells
  the anchor writes, empty or not (`evaluation.rs:951-961`), before the in-place write (`model.rs:1045`). SUMIFS
  reads a sum cell only where the criteria match, so no spill cell is read and neither of 11.3's tests fires. The
  later anchor is clean and does not run, so no restart is asked.
- Counterexample. D1:D10 "k", G1:G2 "x","y", H1:H10 numbers, Z1 = 2. A1
  `=SUMIFS(INDEX(E1:F10,0,Z1),D1:D10,G1:G2)` and B1 the same over `L1:M10`: always dirty, ranks 0 and 1. E1
  `=H1:H10*2` and L1 `=H1:H10*3`: modelled, clean. Write Z1 = 1. No footprint changes; the 10-cell ranges are under
  the IFS floor of 64, so the IFS flag is not set.
- Measured: initial full pass spent=218 Normal; full pass after Z1=1 spent=612 Normal (two restarts, 394 thrown
  away); A1 and B1 alone commit r=134, summing bound after the restricted pass = 352; with a pass allowance of 352
  to 393 the full pass after Z1=1 gives A1=#VALUE!, E1=#VALUE!, ending=Shared; at 394, A1=0, E1=2, Normal. For host
  ceilings 1408 to 1572 (`set_recalc_ceiling`) the restricted pass is kept with 0/2/3 where a full recalculation
  gives `#VALUE!` in all four anchors. At any limit, `anchor_rank` is no longer a full pass's order.
- Fix. Add a fourth rejection to "The pass": "a dirty anchor records as read empty (`record_seen_empty_range`) a
  rectangle that meets the footprint of an anchor ranked after it, clean or dirty: a full pass restarts there." A
  first slice may reject on any such record inside the used area made for a dirty anchor that is not last in rank.
  Reword "Why it is exact" to match, give `seen`/`seen_ranges` their own table row ("not a charge: a restart"), add
  this fixture and the mutation "the rectangle test dropped".

**B2 (High). 11.2, lines 1613-1614 and 1627-1628: a union's rectangles do not hold everything its value depends
on.**
- A union's areas are walked over existing cells with charged steps (`functions/util.rs:751-792`,
  `WalkBudget::Charged`). Each existing row costs a step "even with no cell in the area's columns"
  (`existing_walk.rs:273-288`). The walk mode follows `columns_between` and `last_row` (`:249-251`). Past
  `MAX_AREA_WALK` (2^24) the argument is `#VALUE!` (`area_budget.rs:510-522`).
- Measured at production limits (33,150 cells): B1:R1950 filled, AA1 `=SUM((B1:R2400,T1), … ×500)`, A3000 pinning
  the used area, 234 literals in column Z. Before the write of Z2185: AA1=16577500; after a full recalculation:
  AA1=#VALUE!. Z2185 is in no rectangle and moves no head, so 11.2 skips the edit and keeps 16,577,500.
- Fix. "A union stays unmodelled (its reader is always dirty) until its walk's steps are a function of its areas'
  own cells." The same holds for an intersection with a multi-area operand and for a name whose body holds a
  union. Change 11.4's row to match.

**B3 (High). 11.1, lines 1548-1551: the after-index's list alone misses a formula the edit takes off the list.**
- Premise, measured: before, A3 `=SUM(B1:XFD2)` = 111, A4 `=A3*2` = 222, read_index refused; after deleting column
  C (full pass), A3 `=SUM(B1:XFC2)` = 101, A4 = 202, read_index built.
- Derived. Before the edit A3 is on the list and files no rectangle. After it A3 is modelled and off the list. D0
  is empty; A3 runs only as a rewritten formula, "but not the closure of those" (line 802). A4 keeps 222 where a
  full pass gives 202.
- Fix. "The lists of both indexes are in D0: the old index's, each cell at its place after the edit, and the new
  index's." Add the mutation "the old list left out" with this fixture.

**B4 (Medium). 11.3, lines 1723-1726 contradict Design 8 "When" (lines 763-768) and the code.**
- Design 8 already refuses any workbook with a dynamic anchor (`read_index/structure.rs:132`). The new line is
  weaker: it admits a sheet with a one-cell anchor. The reasons for the stronger rule still hold: ranks are keyed
  by place, and `sync_anchor_order` re-ranks a moved anchor last (`positions.rs:131-133`); "an anchor on another
  sheet … is a reader like any other" is false for one the op rewrites, whose spill is cleared
  (`structural_edit.rs:555-575`, `:617`).
- Fix. "Unchanged: Design 8's 'the workbook holds no dynamic anchor' stands, and now also covers spilled and
  blocked anchors."

**B5 (Medium). 11.2, lines 1618-1623 and 1641-1642: the per-name kept leaves have no stated bound.**
- Four names of about 2,000 references each, and N names `=b1+b2+b3+b4`, give about 8,000 kept leaves per name. A
  reader `=m1+m2` passes `MAX_INDEXED_RANGES`, files 12 bytes, and still resolves both names. So the kept leaves
  grow without the cap (`MAX_INDEX_BYTES` = 2^29, `index_memory.rs:27`) seeing them. No limit on the number of
  names turned up.
- Fix. Append: "Kept leaves count toward T-5153's cap as they are kept, at most `MAX_INDEXED_RANGES` a name (past
  it the name is kept as refused, in constant space); past the cap the workbook has no index."

## Non-blocking findings

- N1, lines 1579-1582. The inventory test cannot see state outside `Evaluation` and the fields three functions
  reset. Examples: `groupby_filter_spent` (`evaluation.rs:719`), `area_budget`, the collation thread-locals
  (`functions/util/collation/compared.rs:41-56`), the sheet's lazy column index (`existing_walk.rs:250`). Either
  widen it to every `Model` field and every non-test `thread_local!`/static, or drop "cannot join unnoticed".
- N2, lines 1620-1621 and 1665-1666. The per-name summary must also keep `positional` (`walk.rs:183`: an `@` inside
  a body picks by the reader's row) and the body's calls. Add the mutation "an `@` in a name's body not registered
  as a position reader".
- N3, line 1634. Say whether a refile maintains the name-reader and span-reader lists or rebuilds, as position
  readers do (lines 795-797).
- N4, lines 1604-1612. A name the resolver does not find (`node_frames.rs:181-187`) is not among the outcomes. A
  `RangeReference` to a whole column needs `instantiate`'s refusals.
- N5, lines 1539-1540. "As in Excel (S1; probe V1)": S1's list as fetched has RANDBETWEEN but not RAND (S3 has it),
  and RANDARRAY is in neither; V1 probes RAND. Cite S3 and mark RANDARRAY unprobed. Also, a bare RANDARRAY is a
  dynamic anchor, so T-5132's goal for it waits for 11.3.
- N6, line 1737. T-5134's criterion on main says "the anchor and spill scans run when anchors exist" and "blockers …
  as dependencies". State that the amendment replaces both.
- N7, line 1536. "Each keeps what it has today" is not true of cost. With a list no closure is empty, and
  `edit.rs:743-751` walks it before asking `restricted_pass_allowed`. A declined workbook now builds and holds an
  index it cannot use. Ask first; remember once that the list's closure passes the limit.
- N8, line 1685. Footprint cells "taken as written" are not counted toward the closure limit; count them.

## Checked and holds (by the reviewer, with file:line)

The three refusals (`read_index.rs:630-641`); a clean cell is not `Evaluated` (`read_index/pass.rs:66-98`,
`evaluation.rs:700-706`, `range_values.rs:158-181`); the search flag is set before any cache check, in all six
lookups; the IFS entries are the three named and the floor is 64; range values and running totals charge alike;
order state has two reads only, both flagged; the other rows of the inventory table; 11.2's symbols; the lifetime
rules (name and sheet ops stale the index); E1, E4, E5 of SPEC-0039; 11.3's symbols (a spill read raises a restart,
scans skipped and the assertion, `pass_anchors` and the rank test, a restricted exhaust writes nothing); the S1
quote, V1, and the corpus figures against decision 0422. Lens "semantics must not depend on an optimization": no
finding.

## Not checked

The restricted side of B1 to B3 on running code; the size of the IFS cache's charge difference; T-5166's
"Maintenance" text; the prototype's 6.6 us and the 19-23 ms figures; whether "none turns its dependency tracking
off" is in the research note; the interplay with Designs 9, 10, 12 and 13 beyond the lines that cite 11.
