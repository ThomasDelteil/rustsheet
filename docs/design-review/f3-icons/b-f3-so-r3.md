# F3 icon originality and provenance review: b-f3-so, round 3 (confirmation)

**Reviewer:** poc-iconreview-cc. **Requested by:** poc-b-f3-so-cx.
**Reviewed:** only `/peers/b-f3-so-astra` (`build/f3/so`, HEAD `9c177e716` plus the working-tree icon files). The
files match the SHA-256s in your request exactly:

| File | SHA-256 (matches yours) |
|---|---|
| `icons/manifest.json` (= `public/icons/manifest.json`) | `46da4b3bb23f787d90faac15ee850f01ade2b0a5eb4b6f08dac21b343de262ca` |
| `icons/sprite.svg` (= `public/icons/sprite.svg`) | `3c9e8e879822964789ab07445af2f81db465301874e1f6ea35d84c2d79d4cc71` |
| `icons/originals.json` | `512391f0020e8a15f238d18f6da79c8b2f98ce33f78ba609b020491b2083610e` |
| `icons/modifications.json` | `eb6bdcda072e503389f518cfc5ee120bebff8cd0778deef46643eb3c34487636` |
| `icons/catalogue.json` | `741788972eecb3dd7eebe5eb56280d46e8724d509a6323965dff39c05f088cc9` |

Crop: `b-f3-so-r3-changed.png`.

## Verdict: passes. Zero copy findings, provenance complete.

- **Counts:** 168 symbols = 168 manifest entries = 168 catalogue entries, made of:
  - 120 unmodified Lucide icons, each drawing Lucide 0.563.0 exactly;
  - 13 recorded Lucide modifications, each naming its base, differing from it, described, and licensed ISC;
  - 35 originals.
- **Provenance:** `provenance-check.mjs` reports no findings.
  - **No original equals or contains a whole Lucide icon.** The only hits are three two-element primitives (Lucide's
    square frame plus one line) in `cf-color-scales`, `cf-data-bars` and `unfreeze`, shared by any table drawing.
  - **The six from round 2 are recorded correctly now:**
    - `freeze-panes`, `freeze-top-row`, `freeze-first-column` as Lucide `grid-3x3`, ISC ("frozen … lines are heavier,
      other grid lines at 45 %");
    - `conditional-format`, `cf-top-bottom`, `cf-highlight` as Lucide `table`, ISC ("accent cell" / "top row" /
      "accent cell and a comparison mark").
- **Microsoft:** all 35 originals rasterised against all 2,871 Fluent icon families. The nearest are unrelated, as in
  round 2. The one changed original, `fill-options`, has its nearest in `square_shadow` (IoU 0.77), a large offset
  shadow square, where ours is a small handle with a chevron. **No copies.**
- **Licence:** the Lucide ISC licence ships.

## Your other changes, verified

1. **CF accent is a token now.** No `#F5C443` remains in `sprite.svg`, `originals.json` or `modifications.json`, and the
   sprite has no hex fills at all. All five accents are `fill="var(--cs-color-accent)"`. Measured with your own
   `fallback.css`, `tokens.css` and `components.css` (`probe-accent-forced.mjs`), accent pixel:

   | Mode | Accent pixel |
   |---|---|
   | normal | `rgb(101,68,149)` (your `#654495`) |
   | forced colours, light | `rgb(55,51,109)` (Highlight) |
   | forced colours, dark | `rgb(0,184,204)` (Highlight) |

   This confirms your observation.

   **Note:** the Highlight mapping lives on `.cs-design .cs-icon` (`components.css:207`). A consumer that sets only
   `[data-qs-design]` (your token scope) and not the `.cs-design` class keeps the brand colour in forced colours (I
   measured `rgb(101,68,149)` there). Consider putting the mapping on `[data-qs-design] .cs-icon` as well.
2. **Stroke fallback `stroke-width="var(--_ds-icon-stroke, 2)"` on every symbol: correct, and better than my round-2
   advice.** I said a literal `2` would be overridden by your slot CSS. It is not: the symbol's own attribute beats the
   value the `<use>` inherits from the slot. You measured that, and so did I.
   - **Engines:** `probe-symbol-stroke.mjs` ran the same page in Chromium 147, Firefox 148 and WebKit 26.4, with
     identical results in all three.
   - **Slotted icons get the slot's value:** 20 px → 1.80 units (1.5 px); 32 px → 1.50 units (2.0 px).
   - **A bare `<use>` gets the fallback:** 1.95 ≈ 2 units. With no attribute it would be 1.05 (SVG's default 1).
   - **A literal `2` would override both slots** (1.95 and 1.97 units).
   - **`var()` in presentation attributes:** it resolves for `stroke-width` and for the accent `fill` in all three
     engines.
3. **The P3 twins are resolved** (`b-f3-so-r3-changed.png`):
   - `fill-options` is now an outlined handle with a menu chevron (`m15.5 17 2 2 2-2`), distinct from the solid
     `grid-fill-handle`;
   - `action-settings` is Lucide `settings` (the cog), distinct from `action-sliders`.

## Nothing blocking

Only the scope note in (1) remains, as an advisory. For D5 you can quote: "F3 icon set (`build/f3/so`, manifest
`46da4b3b…`): zero copy findings against Microsoft (Fluent); provenance complete (120 Lucide exact, 13 recorded ISC
modifications, 35 originals); grammar and fallbacks verified in Chromium, Firefox and WebKit." That is reviewer
poc-iconreview-cc, round 3.

**Reproduce:** tools on branch `poc/iconreview`, `apps/collab-sheet/design-review/tools/`:
- `provenance-check.mjs --icons <icons dir> --lucide <lucide-static 0.563.0> --fluent <@fluentui/svg-icons 1.1.345>`;
- `probe-symbol-stroke.mjs chromium,firefox,webkit`;
- `probe-accent-forced.mjs <fallback+tokens.css> <components.css> <sprite.svg> i-conditional-format data-qs-design cs-design`.
