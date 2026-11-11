# F3 icon originality and provenance review: b-f3-so, round 6 (final confirmation)

**Reviewer:** poc-iconreview-cc. **Requested by:** poc-b-f3-so-cx.
**Reviewed:** only `/peers/b-f3-so-astra/apps/collab-sheet/packages/design-system/icons/` (`build/f3/so`, HEAD
`6fc3765f2`). All five files match the SHA-256s you gave:

| File | SHA-256 |
|---|---|
| `sprite.svg` (= `public/icons/sprite.svg`) | `8703d3da15c977a77ce00c18344f4131a5af1c7431694fcc59a3ae6ed2822100` |
| `manifest.json` (= `public/icons/manifest.json`) | `99cfcb65f29922af774610e6b5adf0e846249c155c92ebfb6c8292ec11426b8c` |
| `originals.json` | `ad8550c8fde39cd7d6f473ecf10f52323fbf03f161676afda7066def859e1183` |
| `modifications.json` | `3dbc306dc0c715a5e556abe27139cc08e8201a9d72a148444101e071792617e7` |
| `catalogue.json` | `741788972eecb3dd7eebe5eb56280d46e8724d509a6323965dff39c05f088cc9` |

## Verdict: passes. The P0 is fixed, with zero copy findings and complete provenance.

1. **XML well-formed.**
   - The sprite parses with two independent XML parsers (Python's minidom and expat via ElementTree): 168 symbols.
   - No tag carries `data-role` more than once (the round-5 defect).
2. **External references render in all three engines.** `probe-external-all.mjs` serves the sprite as
   `image/svg+xml` and draws every symbol through `<use href="/icons/sprite.svg#…">`, counting ink per icon:

   | Engine | This sprite (`8703d3da…`) | Control: round-5 sprite (`518acdb1…`) |
   |---|---|---|
   | Chromium 147 | **168 of 168** | 21 of 168 |
   | Firefox 148 | **168 of 168** | 0 of 168 |
   | WebKit 26.4 | **168 of 168** | 21 of 168 |

   The control shows the test detects the old break. (In round 5 I said 148 icons were missing in Chromium and
   WebKit; precisely, 147 were missing and `border-bottom-thick` drew only its guides.) Your React `Icon` no longer
   depends on this, since it renders static geometry, but the published sprite is now safe for any consumer that
   references it.
3. **Zero copy findings (R6).**
   - `provenance-check.mjs` rasterised the 35 originals against all 2,871 Fluent (Microsoft) icon families and all
     1,668 Lucide icons. The geometry files are unchanged since round 5, and for **35 of 35** originals the nearest
     Fluent icons are identical to round 5's.
   - Round 5's visual check of the redrawn icons against Microsoft's same-idea icons (`border_*_thick`,
     `table_freeze_*`, `table_dismiss`, `color_fill`) therefore stands.
   - The highest similarity remains `chart-doughnut` against `target` (0.956), concentric rings, judged generic in
     round 2. No copies.
4. **Provenance complete.**
   - 168 symbols = 168 manifest entries = 168 catalogue entries.
   - 120 unmodified Lucide icons, exact in 0.563.0.
   - 13 recorded ISC modifications, each named, different from its base and described.
   - 35 originals, none equal to or containing a whole Lucide icon.
   - Lucide's ISC licence ships.
5. **Weights hold** (`probe-weight-pairs.mjs`, 20 px slot, stroke 1.8 units):
   - thick edges, frozen lines and unfreeze dashes 3.60 units;
   - thin twins, the unfrozen 45 % lines, Fill Color's rim and the reference line all 1.80.

## The new manifest field (D-F3-39 `mirrorInRtl`)

It is present on all 168 entries, 18 of them `true`:
- `chevron-left`, `chevron-right`, `undo`, `redo`, `send`, `indent-decrease`, `indent-increase`, `comment-next`,
  `comment-previous`, `fill-left`, `fill-right`, `external-link`, `launcher` and `go-to` carry a horizontal direction.
- `chevron-up`, `chevron-down`, `fill-up` and `fill-down` are symmetric about the vertical axis, so mirroring them is
  a harmless no-op. Mark them `false` if you want the field to mean "changes in RTL".
- Not an originality matter. Which commands should read as logical rather than physical in RTL is a product call for
  your D-F3-39.

## For your D5 report (quotable)

"F3 icon set, sprite `8703d3da15c977a77ce00c18344f4131a5af1c7431694fcc59a3ae6ed2822100`, manifest
`99cfcb65f29922af774610e6b5adf0e846249c155c92ebfb6c8292ec11426b8c`:
- well-formed XML; all 168 symbols render through an external `<use>` in Chromium 147, Firefox 148 and WebKit 26.4;
- zero copy findings against Microsoft (Fluent), with 35 originals raster-compared against 2,871 Fluent icon families;
- provenance complete: 120 Lucide 0.563.0 exact, 13 recorded ISC modifications, 35 originals.

Reviewer poc-iconreview-cc, round 6 (final)."

**Reproduce:** tools on branch `poc/iconreview`, `apps/collab-sheet/design-review/tools/`:
- `probe-external-all.mjs <sprite.svg> chromium,firefox,webkit`;
- `provenance-check.mjs`;
- `probe-weight-pairs.mjs <components.css> <sprite.svg> 20`.
