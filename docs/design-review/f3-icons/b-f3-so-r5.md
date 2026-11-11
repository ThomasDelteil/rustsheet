# F3 icon originality and provenance review: b-f3-so, round 5

**Reviewer:** poc-iconreview-cc. **Requested by:** poc-b-f3-so-cx.
**Reviewed:** only `/peers/b-f3-so-astra/apps/collab-sheet/packages/design-system/icons/` (`build/f3/so`, HEAD
`3f0f2695e`). The files match the hashes you gave:

| File | SHA-256 |
|---|---|
| `sprite.svg` (= `public/icons/sprite.svg`) | `518acdb124c0c51ad96740c49a5d9b25d2e76107d88a730dafc4e123e70f39ba` |
| `manifest.json` (= `public/icons/manifest.json`) | `c241ddb722d12ee8293f569a26dd95ddf8f8153dacfc285882de991f07dd327c` |
| `originals.json` | `ad8550c8fde39cd7d6f473ecf10f52323fbf03f161676afda7066def859e1183` |
| `modifications.json` | `3dbc306dc0c715a5e556abe27139cc08e8201a9d72a148444101e071792617e7` |
| `catalogue.json` | `741788972eecb3dd7eebe5eb56280d46e8724d509a6323965dff39c05f088cc9` (unchanged) |

Crops: `b-f3-so-r5-external-sprite.png`, `b-f3-so-r5-weights.png`.

## Verdict

**What you asked about passes:**
- **Weights:** verified at 20 px and 96 px.
- **Originality (R6):** zero copy findings on the latest geometry.
- **Provenance:** complete.

**But this exact sprite has a new P0, not an icon-design issue: it is not well-formed XML, and the package's own
`Icon` cannot load it.** Fix it before D5. The fix is mechanical and changes no geometry.

## P0: the sprite does not parse as XML, so most icons disappear

- **The bad markup.** 16 `<rect>`s in 7 symbols carry `data-role` twice:
  `fill="currentColor" data-role="solid" stroke="none" data-role="solid"`. The 7 symbols are `border-bottom-thick`,
  `border-outside-thick`, `border-top-bottom-thick`, `freeze-panes`, `freeze-top-row`, `freeze-first-column` and
  `unfreeze`.
- **Why it breaks.** A duplicated attribute is a well-formedness error; Python's XML parser reports
  `duplicate attribute: line 79, column 470`. Your `Icon` (`src/icons.tsx`) renders
  `<use href={`${url}#i-${name}`}>` with `spriteUrl = '/icons/sprite.svg'`: an external reference. Browsers fetch it
  as `image/svg+xml` and parse it as XML.
- **Measured** by `probe-external-sprite.mjs`, which serves the sprite and uses seven icons through an external
  `<use>`:

  | Engine | As shipped | With only the duplicates removed |
  |---|---|---|
  | Chromium 147 | 496 ink pixels: only `bold` draws (it precedes the error) | 4,023 (all seven) |
  | WebKit 26.4 | 496: only `bold` | 4,023 |
  | Firefox 148 | **0: no icon at all** | 4,023 |

- **What that means in the product.** The first duplicate is in symbol #21 (`border-bottom-thick`), so in Chromium
  and WebKit the **148 icons from there on (88 %)** do not render, and in Firefox none of the 168 do. Inlined into
  HTML the sprite works, because HTML parsing tolerates the duplicate. That is why an inline gallery would not show
  the break.
- **Cause.** `scripts/icons.mjs:27`:
  `replace(/fill="currentColor"(?! data-role)/g, 'fill="currentColor" data-role="solid"')` only skips elements whose
  `data-role` *immediately* follows the fill. The new rects have `stroke="none"` in between, so a second `data-role`
  is added.
- **Fix.**
  1. Add `data-role="solid"` only when the element has no `data-role` at all: test the whole tag, e.g.
     `/<(\w+)\b(?![^>]*\sdata-role=)([^>]*\sfill="currentColor"[^>]*)>/`. Or drop the auto-tagging, since the
     geometry files already carry it.
  2. **Gate** the generated sprite on two checks:
     - **parse it as XML** (any XML parser; well-formedness fails the build);
     - **render one icon through an external `<use>`** (`probe-external-sprite.mjs` does this in 3 engines).

  Removing the duplicates changes no geometry, so the copy and provenance verdicts below carry over. Send the new
  hashes and I will confirm the parse and render.

## Weights: verified (sprite inlined, so measuring the drawing, not the parse)

`probe-weight-pairs.mjs`, through your `components.css`, stroke in 24-units:

| Element | 20 px slot (stroke 1.8) | 96 px (stroke 2) |
|---|---|---|
| `border-bottom` edge (thin twin) | 1.80 | 2.00 |
| `border-bottom-thick` edge | **3.60** | **3.50** |
| `border-outside` / `border-outside-thick` top edge | 1.80 / **3.60** | 2.00 / **3.50** |
| `border-top-bottom` / `border-top-bottom-thick` bottom edge | 1.80 / **3.60** | 2.00 / **3.50** |
| `freeze-panes` frozen row / unfrozen row (45 %) | **3.60** / 1.80 | **3.50** / 2.00 |
| `freeze-top-row` frozen row; `freeze-first-column` frozen column | **3.60**; **3.60** | **3.50**; **3.50** |
| `unfreeze` frozen dash | **3.60** | **3.50** |
| `fill-color` bucket rim; reference `align-left` line | 1.80; 1.80 | 2.00; 2.00 |

- **Heavy elements:** they are 2× the stroke at 20 px. 3.50 at 96 px is 3.6 at this pixel resolution.
- **Fill Color:** it now follows the slot exactly: no group, coordinates baked in.
- **Consistency:** the geometry files carry no inner `stroke-width` any more, so sprite and sources agree
  (`b-f3-so-r5-weights.png`).
- **P3:** the heavy rects are a fixed 3.6 units, so the thick-to-thin ratio depends on the slot. It is 2.0 at 20 px and
  2.4 at 32 px, but only about 1.33 at 12 px (measured 4.0 against 3.0 units, i.e. 2.0 against 1.5 px). If borders
  ever show at 12 px, "thick" will be hard to tell from thin there.

## R6 originality: zero copy findings on this geometry

`provenance-check.mjs` rasterised the 35 originals against all 2,871 Fluent (Microsoft) icon families:
- **Unchanged originals:** 31 keep exactly the nearest neighbours of round 4.
- **Redrawn originals:** `border-bottom-thick`, `border-outside-thick`, `border-top-bottom-thick` and `unfreeze` have
  new, unrelated nearest icons: `tv`, `presence_tentative`, `slide_link`, `building_shop`, at IoU 0.75–0.87.
- **Highest overall:** `chart-doughnut` against `target` (0.956), concentric rings, as before.
- **Against Microsoft's drawings of the same idea:** I also drew the redrawn icons beside these:
  - `border_bottom_thick`, `border_outside_thick` and `border_top_bottom_thick` are dashed rounded squares, against
    our dotted guides with separate solid bars;
  - `table_freeze_*` is an inner corner frame, against our heavy bars on Lucide's grid;
  - `table_dismiss` is a grid with a badge, against our broken heavy lines;
  - `color_fill` and `paint_bucket`, against our Fill Color, which is Lucide's own bucket.

  Different compositions. No copies.

## Provenance: complete

- 168 symbols = 168 manifest entries = 168 catalogue entries.
- 120 unmodified Lucide icons, exact in 0.563.0.
- 13 recorded ISC modifications, each named, different from its base and described.
- 35 originals, none equal to or containing a whole Lucide icon.
- Lucide's ISC licence ships.

**Quotable for D5 (copy gate, this geometry):** "F3 icon set, geometry `originals.json ad8550c8…` /
`modifications.json 3dbc306d…`: zero copy findings against Microsoft (Fluent): 35 originals raster-compared with 2,871
Fluent icon families, redrawn ones also against the same-idea Fluent icons; provenance complete (120 Lucide 0.563.0
exact, 13 recorded ISC modifications, 35 originals); deliberate weights verified at 20 and 96 px. Reviewer
poc-iconreview-cc, round 5." Add the sprite hash once the XML fix is in and I have confirmed it parses.

**Reproduce:** tools on branch `poc/iconreview`, `apps/collab-sheet/design-review/tools/`:
- `provenance-check.mjs`;
- `probe-weight-pairs.mjs <components.css> <sprite.svg> 20|96`;
- `probe-external-sprite.mjs <sprite.svg> i-bold,i-cut,… chromium,firefox,webkit`.
