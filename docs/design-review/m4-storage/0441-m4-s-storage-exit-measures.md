# 0441 — M4's storage exit measures

- **Status:** proposed (2026-11-11, T-5430; accepted when SPEC-0045's change is approved).
- **Context:**
  - `docs/milestones.md` gives M4 the exit criterion "≤ 64 B per numeric cell and ≤ 100 B per text cell (repeated
    text shared) on benchmark models". It names no model and no way of counting, as its structural criteria did
    not before decision 0367.
  - The figure depends on both. On `main` (`527f2b8bc`, native aarch64) a million numbers cost 90.4 B each in rows
    of ten after a decode, 143.2 B typed, and 376.1 B typed down one column (SPEC-0045 "Problem"): the row's slack
    and its slot move the figure four-fold.
  - The collaborative client's engine builds its workbook one cell at a time (`Workbook::load_stored`), so the
    typed figure is what a user's browser pays.
  - Exit measures are settled by decision (0085, 0137, 0146, 0187, 0367), and a narrower reading must say so
    (CLAUDE.md directive 2).
- **Decision:**
  1. **The benchmark models** are four workbooks of 1,000,000 cells on one sheet: N10 (numbers, 100,000 rows of
     10), N1 (numbers, one column), T10 (text, every cell distinct, 11 bytes each, 100,000 rows of 10) and R10
     (text, 100 distinct values, 100,000 rows of 10). N10 and N1 are held to 64 B a cell, T10 and R10 to 100 B.
     Two more are reported beside them after a full pass, with no bar, because the criterion names numeric and
     text cells and not formulas: SPEC-0002 "Problem"'s ledger at 100,000 rows, and E10, a number and nine
     lookups that miss in each of 100,000 rows.
     - **N1 is read to the whole byte.** A row costs its 24-byte slot and a cell its 40-byte entry after
       SPEC-0045, which is 64 exactly, and the sheet's 977 chunks add 0.05 B a cell: 64,051,959 bytes for a
       decoded N1, by today's count less the entry's 48 bytes a cell. The criterion is written in whole bytes,
       which do not resolve 0.05. This record reads N1's figure rounded to the nearest byte, which narrows the
       literal "≤ 64" by less than half a byte and says so. The gate still holds N1 at its exact count, so N1
       has no margin: any byte added to an entry or a row slot fails it, until a store with no per-row record
       (SPEC-0045 Part 2).
  2. **Each model is built three ways**: typed (`Model::set_user_input`), loaded cell by cell
     (`Workbook::load_stored`) and decoded (`Model::from_bytes`). Its figure is the largest of the three. This
     reads "on benchmark models" as "however the model came to be", the widest reading.
  3. **The count** is a global allocator's: the bytes requested and not yet freed once the workbook is built, less
     those of an empty workbook, divided by the stored cells, natively on the test host's target. It is exact, so
     the gate compares with no tolerance. The allocator's own overhead (headers, size classes) is not counted: it
     is the system's, and differs by platform.
  4. **The gate** is a test `make check` runs (SPEC-0045 Design 1): the four bars as constants that may only
     fall, and the sizes of `Cell` and of a row's entry asserted at compile time for 64-bit targets and for
     wasm32. The test builds the models at 100,000 cells, the same shapes at a tenth of the rows, so that it
     takes seconds in the test profile; the bench `memory` runs the same code at 1,000,000 cells in release, and
     its figures are the ones quoted. A bar holds at both sizes. wasm32 has no allocator count in `make check`; its entry and row slot are no larger than the native
     ones (40 and 12 bytes against 40 and 24 after SPEC-0045), so a model within its bar natively is within it
     there.
  5. **Two things the gate needs and `main` did not have**, both recorded here:
     - `crates/rustsheet-heap`, a dev-only workspace crate holding the counting allocator that
       `fuzz/src/heap.rs` held for one target. Its `unsafe impl GlobalAlloc` forwards each call to the system
       allocator unchanged, under `#[allow(unsafe_code)]` with that reason, as `rustsheet-wasm`'s allocator is
       (decision 0423). No product crate depends on it.
     - `hashbrown` as a direct dependency of `rustsheet_engine`, at the version `Cargo.lock` already resolves for
       other crates. Its `HashTable` is an index-only table, which the standard `HashMap` does not offer; with it
       a distinct cell text or formula text is held once instead of twice (SPEC-0045 Design 4).
- **Consequences:**
  - `docs/milestones.md` M4 cites this record.
  - No gate weakens: the measure is new. Until SPEC-0045's last task the bars stand at today's figures, so each
    task lowers them and none can raise them.
  - The milestone's scope beyond the bars (columnar copy-on-write chunks, an O(1) snapshot) is SPEC-0045 Part 2 and
    no part of this exit measure. Neither is the performance plan's "1M x 10 ≤ 400 MB"
    (`docs/amazon/plans/performance.md` 4.7): ten million numbers at 42.4 B are 424 MB requested, and that
    budget waits for Part 2.
  - The count is of requested bytes. What a process or a browser tab holds is more (an allocator rounds a
    40-byte row up); SPEC-0045's last task reports one resident figure per model beside the count.
