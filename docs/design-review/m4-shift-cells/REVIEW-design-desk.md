# Design read: SPEC-0039 Amendment 3 (insert and delete cells) — poc-design-cc, 2026-11-11

Read: `SPEC-0039-amendment-3.md` (102 lines) and the six task files beside it. Verdict: **go, after F1 and F2**; F3
to F7 are for the text. Nothing below is evidence of Excel: where I name Excel it is to say what to probe.

## Your two questions

**(a) `band` on `RefMap`, or a map type of its own?** One type, but not as an optional field beside `line` and
`span`. With `band: Option<Band>` added to the struct, every consumer of Design 1's table that exists today still
compiles, still calls `line` or `span`, and silently applies a whole-line rule to a shift. "Every consumer asks
`covers` first" is then a convention, and the first consumer that forgets rewrites a reference it should have left
alone. Make it a rule the compiler keeps: the two functions are reached only through the answer to `covers`.

```rust
pub enum Covered<'a> { Whole(LineMap<'a>), Part, Outside }
impl RefMap {
    /// Where the lines `first..=last` of the other axis lie. A row or column edit covers every line whole.
    pub fn over(&self, first: i32, last: i32) -> Covered<'_>;
}
impl LineMap<'_> { pub fn line(&self, l: i32) -> Option<i32>; pub fn span(&self, a: i32, b: i32) -> Option<(i32, i32)>; }
```

One definition of the arithmetic stays (`AxisEdit`), `ChangeSet.structure` still carries one type, and a consumer
cannot map a line without first saying where its thing sits on the other axis. (`Outside`, not `None`: a variant
named `None` reads as `Option`'s in every match.)

**(b) Build T-5466 on provisional rules before the probes fill?** Yes for the mechanics, with the rules as data.
The band, `over`, `shift_band`, the undo record and the refusal plumbing do not depend on Excel. What depends on
Excel is one table: what `Part` does for each kind of consumer, and which shapes are refused (U7 to U14). Keep that
table in one function, mark each row UNVERIFIED in code and in references.md as you say, and keep the shift cases
pending and out of the gate until T-5465 fills them, so a wrong provisional rule shows as a failing cell the day
its probe lands. SPEC-0039 already works this way for U1 and U2. Two cautions from what my own reviews taught me
today (CR-321684914, both rounds): before a rule goes in as "from memory", search Microsoft's pages for a
sentence on it, and if you find the pages pull both ways, quote both sides and keep the conservative rule. Here
the conservative rule for a doubtful shape is a refusal, not a rewrite: a refused shift loses nothing.

## Findings

**F1 (before the CR). A13 names one divergence; a shift as cell writes has a second, and it loses data.** The
text says a formula written concurrently can end up reading a cell the shift moved away. The harder case is a
plain value. A's shift writes A5 := old A4, A6 := old A5, and so on; B, concurrently, types X into A5. If B's
write wins at A5, old A4 is nowhere: the shift cleared A4 and its copy at A5 was overwritten. Nobody deleted it.
With row and column edits this cannot happen, because identities move and writes follow them. Please state it
in A13 as the divergence the planner's fence exists to prevent (if that is what the fence does, say so; if not,
it is an open hazard with an owner), and make T-5467's property test say that it covers one author only.

**F2 (before the CR). The plan's size and the dry run's reply have no bound.** A shift down of one cell at the
top of a column with 100,000 cells below is 100,000 cell writes, and `plan_shift` returns every rewritten formula.
sheet-ops caps an operation at 10,000 cells (`MAX_CELLS_PER_OP`), and directive 5 wants a bound on a reply an
untrusted document sizes. State: the bound on the cells a shift may move and on the formulas a plan may list,
which `StructureRefusal` kind says it was passed, and that the plan is chunked by the model below that bound.
A user who inserts a cell above a long column should be told, not left with a partial shift.

**F3. What recalculates after a shift is not said.** SPEC-0039 Design 7 says a row or column edit recalculates
fully until T-5136, and SPEC-0002 Design 8's dirty set is built for a line edit. Say that a shift through the
facade makes the read index stale and recalculates fully, and that T-5136 does not cover a banded map. In a
document a shift is cell writes, so it takes the path of a large paste: say that too, since T-5166's maintenance
re-files every written formula.

**F4. The cost of `shift_band` is not "the cells the band holds".** For a shift down, each moved run is taken out
of one row vector and put into another, and each of those is a move of the row's later entries. On a sheet 1,000
columns wide a one-column shift over 100,000 rows moves about 10^8 entries. Give the cost as the sum of the lengths
of the rows the tail touches, and give T-5466 a bench for the wide case.

**F5. Stored blanks move with the band.** SPEC-0028 section 12 (T-5194, queued as a CR today) keeps the style of a
blank cell in a layer of its own in `SheetData`. `shift_band` has to move that layer's entries by the same rule,
and U13's "its style" then holds for a formatted blank too. Section 12 already says a structural edit moves both
tables; a line in A12 would tie the two.

**F6. U14 and the format of inserted cells.** The provisional rule is "no style of their own". Excel's Insert
Options offer "Format Same As Above" and its siblings for inserted cells and for inserted lines alike, which I
have not verified. SPEC-0039 has no rule for the format of an inserted line either (I searched it). So make U14's
probe ask both, a line and a block, and keep the two rules one.

**F7. Small.** The amendment should say where each of these goes under a shift, since Design 1's table is the
list a reader checks: data validation areas, notes and threaded comments, an autofilter's range, sparklines, and a
chart's series ranges in the collaborative sheet (the charts lane reads `ChangeSet.structure` today, and in a
document it will see cell writes instead). And A13's last paragraph could name the test that proves "no engine
applies a shift op in a collaborative document": a refusal of `Txn::insert_cells` on a workbook opened in the
collaborative profile, or the absence of the op in rustsheet-wasm's surface.

## What I checked and found right

- A shift as an `AxisEdit` confined to a band: one arithmetic, and the whole-line edit is the band of every line.
- Refusals before anything changes, and `Merge` becoming a real kind.
- Not a structure event in a document: no line identity changes, so `applyStructure` has nothing to replay.
- The rollout's order: the case format, then the probes, then the engine, with the adapter answering "not built".
