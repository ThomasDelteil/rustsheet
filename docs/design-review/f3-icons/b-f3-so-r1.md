# F3 icon originality review: b-f3-so, round 1

**Reviewer:** poc-iconreview-cc (standing icon originality reviewer).
**Requested by:** poc-b-f3-so-cx.
**Reviewed:** `packages/design-system/icons/{originals.json, catalogue.json, manifest.json}` and `public/icons.svg`, in
the clone mounted at `/peers/b-f3-so-astra`, branch `build/f3/so`, HEAD `eb94f07c0`.
**Crops:** `b-f3-so-r1-sheet.png`, `b-f3-so-r1-provenance.png`, `b-f3-so-r1-readings.png` (this folder).

## Verdict

**No copy findings against Microsoft.** None of the ten originals reproduces a Microsoft (Fluent) glyph's
composition; this holds for CF, merge and number format too (answers below).

Two things must change before D5:
1. **Provenance (must fix):** two "originals" are Lucide drawings, and the manifest records them as
   `source: original, licence: RustSheet`. A third uses Lucide's chart axis.
2. **Reading (must fix):** five originals do not read as what they do at working size.

Grammar is otherwise clean.
- **Live area:** all ten glyphs keep their geometry inside the 2–22 live area (`originals-sheet.mjs`).
- **Fills:** no filled shapes.
- **Sprite:** applies `fill="none"`, a 2 px stroke and round caps and joins to all 66 symbols.
- **Licence:** Lucide's ISC licence ships in `licenses/`.

## Your three questions

- **Conditional formatting: not a copy.**
  - **Against Microsoft:** Fluent's nearest is `table_simple_checkmark` (a table with a filled circular check badge at
    the corner). Yours puts a small open check in the header row: a different composition.
  - **But it reads as "validated" or "checklist", not "format by rule"** (see the readings below).
- **Merge: not a copy.** Fluent's `table_cells_merge` is a rounded frame with an outward double arrow; Lucide's is a
  table with a removed divider. Yours, stubs with inward chevrons, is your own. It does not read yet, though (below).
- **Number format: not a copy.** `format.number.currency` is your own "$". It is narrower than Lucide's (x 8–16) and
  unlike Fluent's `currency_dollar_euro`. The catalogue's `format.number → hash` is Lucide's own.

## Must fix

### 1. Provenance: record Lucide-derived geometry as Lucide (ISC), or redraw

`b-f3-so-r1-provenance.png` overlays yours (black, 2 px) on Lucide 0.468.0 (red centre line). The overlay settles it:

| Icon | Finding | Evidence (Lucide 0.468.0 `icon-nodes.json`) |
|---|---|---|
| `format.borders.all` | **Lucide `grid-2x2`** with `rx` 2 → 1 | Lucide: `rect x3 y3 w18 h18 rx2`, `M12 3v18`, `M3 12h18`. Yours: the same rect with `rx='1'`, `M12 3v18M3 12h18` |
| `view.freeze` | **Lucide `panels-top-left`** exactly, plus two dots | Lucide: `rect x3 y3 w18 h18 rx2`, `M3 9h18`, `M9 21V9`. Yours: the same rect, `M3 9h18M9 9v12` (the same segment), `m4-15h.01m4 0h.01` |
| `insert.chart.combo` | Built on **Lucide `chart-column`'s axis** (`M3 3v16a2 2 0 0 0 2 2h16`; yours starts at y 4); bars and line are yours | as above |

**Fix.** Either:
- record them in `manifest.json` as Lucide-derived, e.g. `source: "lucide"`, `lucideName: "grid-2x2"`,
  `modified: "rx 2 → 1"`, `licence: "ISC"`, with `panels-top-left` "+2 dots" and `chart-column` "axis" the same way;
  or
- redraw them so the geometry is genuinely yours.

Using Lucide is fine. Labelling ISC geometry as "RustSheet" is not. For `borders.all`, plain `grid-2x2` (rx 2) is
better than the modified one: rx 1 breaks Lucide's 2 px corner rule for no gain.

### 2. Reading: five originals do not say what they do

`b-f3-so-r1-readings.png` shows each at 150 px; `-sheet.png` shows them at 24, 20 and 16 px.

| Icon | What it reads as | Why (measured on the 24 grid) | What to do |
|---|---|---|---|
| `home.cells.merge` | **"×" (delete, cancel)** | The two chevrons overlap: one's apex is at (13,12) pointing right, the other's at (11,12) pointing left, so they cross over x 11–13 and form a bold × between the stubs | Keep the inward idea but do not cross: e.g. `>` and `<` meeting a short central bar, with a 2-unit gap each side; or two cells whose divider dissolves |
| `home.cells.wrap` | A hook with a backwards arrowhead | The return line's tail travels left and ends at (11,16), but the chevron's apex is at (12,16) **pointing right**, against its own line | Point the head along the return: e.g. a separate `M14 13l-3 3 3 3` (apex at (11,16), pointing left) |
| `format.borders.bottom` | **A barcode or bar chart** | Five vertical bars (x 4–20, y 3–15) over a baseline: nothing says "cell" or "edge" | Draw the cell: a frame of Lucide-style dots (`h.01` segments) or a light frame, with the bottom edge solid. All the border icons should share that frame (today `borders.all` is a solid grid and `borders.bottom` is bars: two families) |
| `format.conditional` | **"Validated" or "checklist"** | A table with a 3 × 2-unit check (x 11–14, y 6–8) in its header. A grid with a check is the picture people know for data validation (my judgement; Office's ribbon artwork is not on disk to compare). At 16 px the check is about 1.3 px tall and blurs | Show a rule's effect: one or two cells highlighted (a colour accent is allowed for CF, from your palette's tokens), or a cell highlight with a small comparison mark. Not a check |
| `grid.fill-handle` | **Copy** (two overlapping squares) | A 13-unit square with a 7-unit square overlapping its bottom-right corner, plus a 3-unit stroke inside the small square (y 18) that is noise at 16 px | Make the handle small and solid at the corner, the way the grid draws it (a filled handle is semantic, like a colour bar), optionally with a short diagonal drag stroke. Drop the inner stroke. Use `rect rx 2` for the cell, not a sharp-cornered path |

## Should fix (grammar)

- **`home.formula.autosum`: the + and − crowd the Σ.** Lucide keeps 2 units between distinct elements.
  - **Plus:** about 0.8 units of clearance from the Σ's top bar ((15,6) to (17,8)).
  - **Minus:** about 0.2 units from the Σ's bottom bar ((15,18) to (17,17)). At 16 px they touch.
  - **Fix:** move the marks out or drop the minus; Σ alone says AutoSum.
- **`format.number.currency`: narrow.** It is only 8 units wide (x 8–16) and reads thin at 16 px. Widen it towards
  6–18.
- **`grid.fill-handle`: sharp corners.** The cell is a sharp-cornered path (`M4 4h13v13H4z`) where Lucide uses
  `rect rx 2` (see above).

## Advisory (catalogue: the 56 Lucide mappings)

- **Version.** You pin `lucide-static 0.468.0`; Quick's app is on `lucide-react ^0.563.0`. Five of your names do not
  exist in 0.563.0, where they were renamed:
  - `align-left`, `align-center` and `align-right` are now `text-align-start`, `-center` and `-end`;
  - `filter` and `filter-x` are now `funnel` and `funnel-x`.

  If the goal is to look native to Quick, move to Quick's line and re-map.
- **Reads-as notes:**
  - `home.cells.format-cells → square-pen` reads as "edit".
  - `insert.chart.line → chart-no-axes-combined` draws bars and a line, so it reads as a combo chart, next to your
    real combo. Lucide `chart-line` or `chart-spline` say "line".
  - `view.toggle-grid → layout-grid` reads as "grid view / apps", not gridlines (Lucide `grid-3x3` is closer).
  - `action.magic → sparkles`: keep sparkle for AI only.

## How this was checked (reproducible)

Tooling is on branch `poc/iconreview`, `apps/collab-sheet/design-review/tools/`. Reference sets were fetched as data:
`npm pack lucide-static@0.468.0` (your pin), `lucide-static@0.563.0` and `@fluentui/svg-icons@1.1.345`.

```
node originals-sheet.mjs --originals <your originals.json> --lucide <lucide 0.468.0> --fluent <fluent> --map "..." --out sheet.png
node overlay-compare.mjs --originals <your originals.json> --lucide <lucide 0.468.0> --pairs "format.borders.all=grid-2x2;view.freeze=panels-top-left;insert.chart.combo=chart-column;home.cells.wrap=wrap-text" --out overlay.png
```

What I did not check: how the icons are used in a UI (only the package's files), and colour usage (the originals carry
none).

When you have changes, send me the new commit and I will do round 2.
