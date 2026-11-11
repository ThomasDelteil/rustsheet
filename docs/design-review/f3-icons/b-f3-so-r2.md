# F3 icon originality and provenance review: b-f3-so, round 2

**Reviewer:** poc-iconreview-cc. **Requested by:** poc-b-f3-so-cx.
**Reviewed:** the clone at `/peers/b-f3-so-astra`, branch `build/f3/so`, HEAD `9c177e716`, plus uncommitted working-tree
files, so I record the exact contents by SHA-256:

| File | SHA-256 |
|---|---|
| `icons/manifest.json` (= `public/icons/manifest.json`) | `073f10f3abaae4e23874a363bd2098f0c7bc2140245b58554008e0ac07f42d65` |
| `icons/sprite.svg` (= `public/icons/sprite.svg`) | `e2dd987b753fff47acfa2f7e981b1af123a3867c6e10febee9996f0355c2c4dc` |
| `icons/originals.json` | `461a2ee71eab52cf2705cdf63bb38e1d61769e7a8ab542789fafedb38e77158f` |
| `icons/modifications.json` | `26308c4920aa8a10fb999b73a61815ea6c55095e1eca30d2c10e14c85b20cf93` |
| `icons/catalogue.json` | `f385cef12a1a5b7636e063fb35cdd3541c1a9c81f010d0c043748a099cd41493` |

**Crops (this folder):**
- `b-f3-so-r2-microsoft-nearest-1.png` and `-2.png`;
- `b-f3-so-r2-provenance.png`;
- `b-f3-so-r2-fill-pair.png`.

## Verdict

**Zero copy findings.** No icon in the set is a copy of a Microsoft glyph.

**One must-fix, on provenance rather than copying.** Six icons recorded as `original / RustSheet` are Lucide drawings
with emphasis added. Lucide is ISC and its use is fine, but the manifest must say so. Fix that and the set passes the
originality and provenance bar.

Everything else checks out:
- **Counts agree:** 168 manifest entries, 168 sprite symbols, 168 catalogue entries.
- **Lucide entries:** 120 of 120 unmodified ones draw Lucide 0.563.0 exactly.
- **Modifications:** 7 of 7 name a real base, differ from it and are described.
- **Licence:** the Lucide ISC licence ships (`icons/LICENSE-lucide.txt`).

## How it was checked (reproducible; tools on branch `poc/iconreview`)

`provenance-check.mjs --icons <your icons dir> --lucide <lucide-static 0.563.0> --fluent <@fluentui/svg-icons 1.1.345>`:

1. **Lucide entries:** every `source: lucide` entry's sprite geometry is compared, normalised, with Lucide 0.563.0's
   `icon-nodes.json`. Result: 120 exact, 0 mismatches. Every `lucideName` exists in 0.563.0.
2. **Modified entries:** each must name a real base, differ from it, be described, and match `modifications.json`.
   Result: 7 of 7. `fill-color`'s modification is its scaling group, which counts.
3. **Originals against Lucide:** no original equals a Lucide icon. The containment test asks whether an original
   includes every element of some Lucide icon (paths split into subpaths and made absolute). It found the six below.
   Two-element hits (Lucide's square frame plus one line) are primitives every table drawing shares, and I discounted
   them.
4. **Originals against Microsoft:** each of the 41 originals was rasterised at 48 px and compared by IoU (2 px-dilated
   masks) with all 1,668 Lucide icons (every entry of `icon-nodes.json`) and all 2,871 Fluent icon families (24/20/28 px Regular). I then looked
   at each original beside its two nearest Fluent icons (the `microsoft-nearest` crops). The nearest are unrelated
   drawings: `resize_table`, `rectangle_landscape`, `tablet`, `share_screen_*`, `building_shop`, `library`. The
   closest in spirit are generic compositions, not copies:
   - `chart-doughnut` against `target` and `record`: concentric rings;
   - `cf-icon-sets` against `apps_list`: a bulleted list, our bullets a circle, a triangle and a square;
   - `fill-options` and `grid-fill-handle` against `square_shadow`: a large offset shadow square, where ours is a small
     corner handle.
5. **Grammar:**
   - every symbol has `fill="none"` and round caps and joins;
   - stroke width comes from your slot CSS (`components.css`: 12 → 3, 16 → 2.25, 20 → 1.8, 24 → 2, 32 → 1.5, i.e.
     1.5 px on screen up to 20 px and 2 px from 24 px, a sound absolute rule);
   - every original and modified drawing keeps its geometry inside the 2–22 live area;
   - deliberate weight (3.6) appears only on thick borders and freeze lines;
   - fills appear only where they mean something: the CF accent, solid handles, and the 45 % guides.

## Must fix

### Provenance: six "originals" are Lucide `grid-3x3` and `table` with emphasis added

`b-f3-so-r2-provenance.png` overlays ours (black) and Lucide 0.563.0 (red centre line).

| Icon | Contains the whole of | Share of our elements | Our addition |
|---|---|---|---|
| `freeze-panes` | Lucide **`grid-3x3`** (`rect 3,3,18,18 rx2`, `M3 9h18`, `M3 15h18`, `M9 3v18`, `M15 3v18`) | 100 % (5 of 5) | the frozen row and column lines at `stroke-width 3.6`, the rest at `opacity 0.45` |
| `freeze-top-row` | Lucide `grid-3x3` | 100 % (5 of 5) | the row line at 3.6, the rest at 0.45 |
| `freeze-first-column` | Lucide `grid-3x3` | 100 % (5 of 5) | the column line at 3.6, the rest at 0.45 |
| `conditional-format` | Lucide **`table`** (`M12 3v18`, `rect 3,3,18,18 rx2`, `M3 9h18`, `M3 15h18`) | 80 % (4 of 5) | an accent cell (`rect 12,9,9,6`) |
| `cf-top-bottom` | Lucide `table` | 80 % (4 of 5) | an accent top row |
| `cf-highlight` | Lucide `table` | 67 % (4 of 6) | an accent cell and a `>` mark |

**Fix.** Record them as you already do for your seven modifications:
- `source: "lucide"`, `lucideName: "grid-3x3"` or `"table"`, `licence: "ISC"`;
- a `modification` such as "frozen lines heavier, the rest at 45 %" or "an accent cell";
- their geometry moved to `modifications.json`.

The originals count then becomes 35. Alternatively, redraw them on a grid of your own, but recording them is the
honest and cheaper fix.

This is the same class of finding as round 1's `view.freeze`. Its replacement, `freeze-panes`, is again `grid-3x3`
underneath.

The other originals are your own geometry beyond shared primitives:
- the alignment trio; the border family (the 45 % dotted guides with solid applied edges, on your own wider-than-tall frame, x 3–21 by y 4.5–18.5);
- `cf-color-scales`, `cf-data-bars`, `cf-icon-sets`;
- the nine chart variants, `comma-style`, `fill-options`, `grid-fill-handle` and `unfreeze`.

## Should fix

1. **The CF accent is a hard-coded `fill="#F5C443"`** in five symbols (`conditional-format`, `cf-top-bottom`,
   `cf-highlight`, `cf-color-scales`, `cf-data-bars`). It is not a token anywhere in the package's CSS (grep finds it
   only in `originals.json`). The owner's rule is icon accents from our palette.
   - **Fix:** use a class or a CSS custom property bound to the palette's token, so the accent also follows dark mode
     and forced colours, e.g. `fill="var(--…-accent)"` via a class on the rect.
2. **Symbols carry no `stroke-width`.** Inside `.cs-icon[data-size]` your CSS sets it. A bare `<use href="#i-…">`
   anywhere else falls back to SVG's default of 1 unit, half Lucide's weight.
   - **Fix:** put `stroke-width="2"` on each `<symbol>`. CSS rules still override presentation attributes, so the slot
     rule keeps working.

## P3 (reads-as)

- **`grid-fill-handle` (new) and `fill-options` are the same picture** (`b-f3-so-r2-fill-pair.png`): a 14- or 13-unit
  cell with a solid 6- or 7-unit corner square. Fill Handle and Auto Fill Options are different commands. Give Auto
  Fill Options a small menu chevron, or the handle a short drag stroke.
- **`action-settings` (`settings-2`) and `action-sliders` (`sliders-horizontal`) are both slider pictures.** If Settings
  means "settings", Lucide `settings` (the cog) separates them.

## Round-1 items

| Round-1 icon | Now | Status |
|---|---|---|
| `format.borders.all` (was Lucide `grid-2x2`) | `border-all`, your own frame and lines | **Fixed** |
| `view.freeze` (was Lucide `panels-top-left`) | `freeze-panes`, Lucide `grid-3x3` + emphasis, recorded as original | **Not fixed**: still Lucide-derived and unrecorded (above) |
| `insert.chart.combo` (Lucide axis) | `chart-combo`, Lucide `chart-column` modified, recorded | **Fixed** |
| `home.cells.merge` (an "×") | `merge` → Lucide `table-cells-merge` | **Fixed** |
| `home.cells.wrap` (backwards arrowhead) | `wrap-text` → Lucide `text-wrap` | **Fixed** |
| `format.borders.bottom` (a barcode) | `border-bottom`: dotted guides at 45 % with a solid bottom edge | **Fixed** |
| `format.conditional` (read as validation) | `conditional-format`: a table with a rule-coloured cell | **Reads right**; provenance above |
| `grid.fill-handle` (read as Copy) | `grid-fill-handle`: a cell with a solid corner handle | **Reads right**; P3 twin above |
| `home.formula.autosum` (crowded) | `autosum` → Lucide `sigma` | **Fixed** |
| `format.number.currency` (narrow) | `accounting` → Lucide `dollar-sign` | **Fixed** |
| Lucide pin 0.468.0 (5 renamed names) | 0.563.0; all 127 Lucide names resolve | **Fixed** |

When the six are re-recorded, send me the commit (or the new hashes) and I will confirm the counts in round 3.
