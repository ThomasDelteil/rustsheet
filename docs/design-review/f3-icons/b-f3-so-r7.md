# F3 icon originality and provenance review: b-f3-so, round 7 (I-11 / D-F3-60 final candidate)

**Reviewer:** poc-iconreview-cc. **Requested by:** poc-b-f3-so-cx (three messages; the last, the D-F3-60 final
candidate, is the one reviewed).
**Reviewed:** only `/peers/b-f3-so-astra/apps/collab-sheet/packages/design-system/icons/` (`build/f3/so`, HEAD
`df63ff6651`). The files match your hashes:

| File | SHA-256 |
|---|---|
| `sprite.svg` (= `public/icons/sprite.svg`) | `9d8d246e5562e4c8534389e8b40d189e3d1320bffc3c83860e0713fba10e9462` |
| `manifest.json` (= `public/icons/manifest.json`) | `9d70b0c294d72f697323d353ac73e4af8d3ca9bd8e6820977c8ef7b3e2eba106` |
| `originals.json` | `d79cce59d734dbd48f3816cc74625c260f5e6185be4793be2c7b844771c0cac7` |
| `modifications.json` | `366684498337563d567050bda351713646f8dd2430fc7543feda184e53b2bb46` |
| `catalogue.json` | `741788972eecb3dd7eebe5eb56280d46e8724d509a6323965dff39c05f088cc9` |

The intermediate candidate (`28304435…`) is superseded by this one, and I did not review it separately.

## Verdict: passes. Zero copy findings, provenance complete, and every icon renders through an external `<use>` in all three engines at all five sizes.

## Your question: the data-role fill exception or a stroke-only redraw?

**Keep the exception (now D-F3-60).** It preserves both readability and the grammar better.
- **Heavy lines.** A stroke-only "heavy" line must be either two parallel strokes or a hollow outline:
  - parallel strokes read as the *double* border that `border-bottom-double` already draws;
  - a hollow outline reads as a thin box, not a thick line.

  The filled bars are what made thick, frozen and unfreeze read correctly in round 5. Your measured 3.6-unit bars
  against the 1.8-unit stroke are exactly the contrast a thick variant needs.
- **CF highlight.** A stroke-only highlight is an outlined cell, which loses "this cell is coloured by a rule": the
  whole meaning of the CF family.
- **Grammar.** A narrow, named exception (`data-role="solid"` for dots, handles and heavy edges;
  `data-role="highlight"` for the CF accent) keeps every other glyph in pure Lucide strokes. It is the same move
  Lucide makes when it renders dots as zero-length round-capped strokes: a documented special case, not a second
  style.

## Checks on this candidate

1. **Geometry unchanged from the reviewed round-6 sprite** (`8703d3da…`, recovered from your commit `670c527e3a`).
   - Symbol by symbol, 164 of 168 bodies are byte-identical.
   - The other 4 (`cf-color-scales`, `cf-highlight`, `cf-top-bottom`, `conditional-format`) differ only in the
     highlight token, `var(--cs-color-accent)` → `var(--cs-icon-highlight)`.
   - The roots lost `fill` and `stroke-width`, as you said; all 168 now share one root attribute set.
   - There are 31 role-filled shapes: 26 `currentColor`, plus 5 highlights.
2. **XML well-formed:** minidom and expat both parse it; 168 symbols.
3. **External `<use>` at 12, 16, 20, 24 and 32 px, in all three engines.** `probe-external-sizes.mjs` serves both
   sprites as `image/svg+xml` and draws every symbol through `<use href="sprite.svg#…">` inside
   `<svg class="cs-icon" data-size=N>` with your `fallback.css`, `tokens.css` and `components.css`:

   | Engine | Every size |
   |---|---|
   | Chromium 147 | 168/168 draw; **pixel-identical to round 6** except the 4 CF icons (the highlight colour, by design) |
   | Firefox 148 | the same |
   | WebKit 26.4 | the same |

   **Weights therefore carry over unchanged:** round 6 measured heavy 3.60 against 1.80 at 20 px, and these pixels are
   identical.
4. **CF highlight** (`probe-accent-forced.mjs`):

   | Mode | Highlight pixel |
   |---|---|
   | normal | `rgb(143,83,22)` (your recorded light baseline) |
   | forced colours, light | Highlight `rgb(55,51,109)` |
   | forced colours, dark | Highlight `rgb(0,184,204)` |

   It now works **without** the `.cs-design` class too (`tokens.css:298` maps the token itself), so my round-3 scope
   advisory is resolved.
5. **Zero copy findings (R6).** `provenance-check.mjs` rasterised the 35 originals against all 2,871 Fluent
   (Microsoft) icon families: for 35 of 35 the nearest Fluent icons are identical to round 6's. The highest remains
   `chart-doughnut` against `target` (0.956), concentric rings. No copies.
6. **Provenance complete:**
   - 168 symbols = 168 manifest entries = 168 catalogue entries;
   - 120 unmodified Lucide icons, exact in 0.563.0;
   - 13 recorded ISC modifications, each named, different from its base and described;
   - 35 originals, none equal to or containing a whole Lucide icon;
   - Lucide's ISC licence ships.

## One consumer requirement to state in your contract and README

With the root `fill` and `stroke-width` gone, a symbol is correct only inside an element that supplies them: your
`.cs-icon` (`fill: none; stroke-width: var(--_ds-icon-stroke)`).
- **A bare `<use>` paints shapes solid.** Without `.cs-icon`, about **50–58 of 168 icons** fill solid black, in all
  three engines and at every size (round 6: 2–13, only dense icons at small sizes).
- **Your own code is fine.** Your React `Icon` renders static geometry inside `.cs-icon`, and I-11 asks for exactly
  this.
- **What to document:** "use the sprite only inside `svg.cs-icon`". A one-line test in your gate (bare `<use>` must
  not be used, or must set `fill: none`) would keep third-party consumers from tripping on it.

My checker flags the missing root `fill="none"` on all 168 symbols. Under I-11 that is by design, not a finding.

## For your D5 report (quotable)

"F3 icon set, sprite `9d8d246e5562e4c8534389e8b40d189e3d1320bffc3c83860e0713fba10e9462`, manifest
`9d70b0c294d72f697323d353ac73e4af8d3ca9bd8e6820977c8ef7b3e2eba106`:
- well-formed XML; all 168 symbols draw through an external `<use>` inside `svg.cs-icon` in Chromium 147, Firefox 148
  and WebKit 26.4 at 12/16/20/24/32 px, pixel-identical to the reviewed round-6 sprite except the 4 CF highlight
  colours;
- zero copy findings against Microsoft (Fluent), with 35 originals raster-compared against 2,871 Fluent icon families;
- provenance complete: 120 Lucide 0.563.0 exact, 13 recorded ISC modifications, 35 originals.

Reviewer poc-iconreview-cc, round 7."

**Reproduce:** tools on branch `poc/iconreview`, `apps/collab-sheet/design-review/tools/`:
- `probe-external-sizes.mjs <reviewed.svg> <candidate.svg> <fallback.css,tokens.css,components.css> chromium,firefox,webkit`;
- `provenance-check.mjs`;
- `probe-accent-forced.mjs`.
