# F3 icon originality and provenance review: b-f3-so, round 4 (the moved artefact)

**Reviewer:** poc-iconreview-cc. **Requested by:** poc-b-f3-so-cx.
**Reviewed:** only `/peers/b-f3-so-astra/apps/collab-sheet/packages/design-system/icons/` (`build/f3/so`, HEAD
`f8f574bf5` plus the working tree). The files match the SHA-256s you gave:

| File | SHA-256 |
|---|---|
| `manifest.json` (= `public/icons/manifest.json`) | `f9a7c78ab189a9200de7516e85608bee60c149979ba4ad7ad38d5621872bd277` |
| `sprite.svg` (= `public/icons/sprite.svg`) | `8580e0e893fdf38a314d6db9f957ec423b349dfb0ce7f9d5feb46cd886dcb71b` |
| `originals.json` | `512391f0020e8a15f238d18f6da79c8b2f98ce33f78ba609b020491b2083610e` (unchanged) |
| `modifications.json` | `eb6bdcda072e503389f518cfc5ee120bebff8cd0778deef46643eb3c34487636` (unchanged) |
| `catalogue.json` | `741788972eecb3dd7eebe5eb56280d46e8724d509a6323965dff39c05f088cc9` (unchanged) |

Crop: `b-f3-so-r4-lost-weights.png`.

## Verdict

**Zero copy findings, and provenance complete, for this exact artefact.** Fix one rendering regression before D5: it is
not a copy issue.

- **Copy evidence:**
  - `provenance-check.mjs` rasterised all 35 originals against all 2,871 Fluent (Microsoft) icon families and all
    1,668 Lucide icons.
  - For every one of the 35, the nearest Fluent icons are the same as in round 3 (35 of 35 unchanged), all unrelated
    drawings.
  - The highest score is `chart-doughnut` against `target` (0.956): concentric rings, inspected in round 2; next
    `cf-data-bars` against `building_shop` (0.879).
  - No copies.
- **Provenance:**
  - 168 symbols = 168 manifest entries = 168 catalogue entries;
  - 120 unmodified Lucide icons, each exactly Lucide 0.563.0;
  - 13 recorded ISC modifications, each named, different from its base and described;
  - 35 originals, none equal to or containing a whole Lucide icon;
  - Lucide's ISC licence ships.

**The regression: removing the inner `stroke-width` attributes took away the deliberate weight of 8 icons.** The
sprite now has 0 inner `stroke-width` attributes and no CSS rule restores them (`components.css` has only the base
`.cs-icon` stroke). But `originals.json` and `modifications.json`, unchanged, still give those 8 entries inner widths: `3.6` on the
heavy lines of `border-bottom-thick`, `border-outside-thick`, `border-top-bottom-thick`, `unfreeze` and the three
freeze icons, and `2.857` on Fill Color's scaled group. So the sprite is no longer a faithful build of your declared geometry.

Measured with `probe-sprite-weights.mjs`, at 96 px through your `components.css`, stroke in 24-units:

| Icon | Shipped | Intended (your JSON) | Effect |
|---|---|---|---|
| `border-bottom` (control) | 2 | 2 | none |
| `border-bottom-thick` | **2** | 3.5 | **identical to `border-bottom`**: "Thick Bottom Border" no longer reads as thick |
| `border-outside-thick` | **2** | 3.5 | **identical to `border-outside`** |
| `border-top-bottom-thick` | **2** | 3.5 | its thick edge drawn thin |
| `freeze-panes`, `freeze-top-row`, `freeze-first-column` | **2** | 3.5 | the frozen line is no longer heavier; only the other lines' 45 % grey remains. The manifest's modification text ("… lines are heavier") no longer describes the sprite |
| `unfreeze` | **2** | 3.5 | its broken frozen lines are no longer heavier |
| `fill-color` | **1.5** | 2 | the `scale(0.7)` group lost its compensating width, so the bucket draws at 0.7× the slot stroke (at 20 px about 1.05 px against the 1.5 px rule) |

**Fix within F1's "no inner stroke-width" rule: put the weight into the geometry.**
1. **Thick edges and frozen lines become filled shapes,** as your handles already are (`data-role="solid"`). For
   example, `border-bottom-thick`'s edge as `<rect x="3" y="16.7" width="18" height="3.6" rx="1.8" fill="currentColor"
   stroke="none" data-role="solid"/>`, and each frozen line likewise. Keep the shapes inside 2–22 (the old 3.6
   strokes' round caps reached x 1.2 and 22.8).
2. **Bake Fill Color's transform into its coordinates.** Apply `translate(3.6 0.6) scale(0.7)` to the path data and
   drop the `<g>`. The bucket's strokes then follow the slot exactly, and "scaled into 2..16" stays true.
3. **Regenerate the sprite from `originals.json`/`modifications.json`** with those changes made there, so the
   geometry files and the sprite agree again. Update the three freeze `modification` texts if the emphasis changes.
4. **Gate it.** A check that the 3 thick borders and their thin twins differ in pixels, and that the freeze and unfreeze
   lines are heavier than the others, would have caught this (`probe-sprite-weights.mjs` does it in 30 lines).

**Quotable for D5, for the copy gate:** "F3 icon set at sprite `8580e0e8…` / manifest `f9a7c78a…`: zero copy
findings against Microsoft (Fluent): 35 originals raster-compared with 2,871 Fluent icon families; provenance complete
(120 Lucide 0.563.0 exact, 13 recorded ISC modifications, 35 originals). Reviewer poc-iconreview-cc, round 4." The
weight regression is a separate reads-as finding, and once fixed the hashes will change again. Send me the new ones and
I will re-confirm both.

**Reproduce:** tools on branch `poc/iconreview`, `apps/collab-sheet/design-review/tools/`:
- `provenance-check.mjs --icons <icons dir> --lucide <lucide-static 0.563.0> --fluent <@fluentui/svg-icons 1.1.345>`;
- `probe-sprite-weights.mjs <components.css> <sprite.svg> <originals.json> <modifications.json>`;
- `weights-sheet.mjs …` for the crop.
