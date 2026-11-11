# Design review 2 (final round): build-2 (poc/ui `c6136c8be`, preview `c6136c8be-mw8zw2rm`)

Reviewer: poc-iconreview-cc. Crops are in `review-2/` (`w*` verify REVIEW-1 items, before/after).

## Final verdict: passes; one P2 and a handful of P3s left as follow-ups

The ribbon and its chrome now meet the owner's bar.
- **Consistent:** one family of 160 icons, one stroke per slot (382 of 382 rendered icons on the rule), whole-pixel
  boxes, no typed glyphs.
- **Stylish:** labels kept at laptop widths, calm launchers, chart pictures in every chart menu.
- **Accessible in every state:** light, dark and both contrast themes; pointer-opened menus without keyboard rings.
- **Our own:** Lucide plus 40 originals, with no carbon copies of Microsoft's icons (new chart originals checked too)
  and no "Excel" in the UI's own strings.

This is the third round. Per the loop, I stop here. The items below are small enough to land as follow-ups without
another review round. I am happy to re-check N4 if you send me a build.

## How this was verified

- **Capture:** build-2 served on :8712. `capture.mjs` gave 600 shots (75 views × 4 themes × 2 DPR), 0 problems.
- **`verify.mjs` (REVIEW-0 items):** 17 checks, 15 pass. The 2 "fails" are the accepted split arrows and the charts
  package's strings (below).
- **`verify-r1.mjs` (REVIEW-1 items):** 15 checks, 14 pass; the one fail is the N5 nit.
- **New this round:**
  - `probe-line-tabs.mjs`, `probe-dialog-paint.mjs` and `probe-dialog-fix.mjs` read pixel profiles, not only boxes;
  - `probe-translates.mjs`;
  - `carbon-compare.mjs` on the 9 new chart originals.

## REVIEW-1 items: fixed or not

| Item | Status | Evidence |
|---|---|---|
| **N1** forced colours: icons, disabled, headers | **Fixed** | 68/68 enabled icons in their control's colour, both schemes; disabled Undo `GrayText` in title bar and ribbon (`rgb(96,0,0)` / `rgb(63,242,63)`); active header 11.31:1 and **8.73:1** (was 1.47:1). `w01-N1-forced-colours.png` |
| **N2** pointer-opened menu focus ring | **Fixed** | Click a cell, right-click: 0 `:focus-visible` items, 0 painted. `w02-N2-menu-focus.png` |
| **N3** launcher over folded names | **Fixed** | 0 px overlap at 860 and 760 px. `w03-N3-launcher-label.png` |
| **N4** Format Cells line samples at 1x | **Not fixed** (see below) | Double is still a grey bar. `w04-N4-double-line-still-blurred.png` |
| **N5** pane tab icons | **Fixed, one nit** | Set icons (`comments`, `activity`, `history`, `agent`, `people`) at 16 px and 1.5 px; five equal tabs. Nit: 55.8 px tabs put 4 of 5 icons at fractional x (1140.11, 1214.7, 1272.5, 1388.09). `w05-N5-pane-tabs.png` |
| **N6** chart-type menus | **Fixed** | 19/19 items have pictures (column 6/6, line 7/7, pie 2/2, scatter 3/3, combo 1/1). `w06-N6-chart-menus.png` |
| **N7** developer items | **Fixed** | File: Open, Download a Copy. The engine line ("rustsheet-wasm · 60 formulas · recalc 48 ms") is in the DOM but hidden (0×0, `probe-status-text.mjs`). `w07-N7-file-menu.png` |
| **N8** launcher weight | **Fixed** | 12 px, secondary ink, 24 × 24 target. `w08-N8-launcher-weight.png` |
| **N9** Comma Style filled | **Fixed as asked, but see N9b** | All strokes now. `w09-N9-comma-reads-as-nine.png` |
| **N10** sheet-tab menu icons | **Fixed** | Insert Sheet, Rename Sheet with icons. |
| Combos 24 px | **Fixed** | 92×24, 20×24, 32×24, 20×24. |
| P2-5 split arrows 18 px | **Accepted** (round 1) | Under WCAG 2.2 2.5.8's equivalent exception (Format Cells; Formulas › AutoSum). |
| P2-6 "Excel" strings | **UI clean; 4 left in the charts package** | `select-data.ts`, `chartex-view.ts` (poc-charts-cc; mailed with the exact strings). |

## Remaining

**N4 (P2): the line samples are still blurred at 1x. I diagnosed the cause wrongly in REVIEW-1.**

I said odd lines centred in a 22 px box land on half pixels. Build-2 fixed that geometry: every sample's box is now on
a whole pixel (Double at y = 462.00). Yet the pixels are unchanged: Double is a uniform grey smear,
`139 139 139 139` (`probe-line-tabs.mjs`, with and without reduced motion).

**The real cause** is the dialog's re-snap: `.cs-dialog` gets `translate: 0 0.5px` (`probe-translates.mjs`; the
only fractional CSS translate in the states I probed: at rest, the context menu and Format Cells). Chromium pixel-snaps borders laid out at fractional positions, but not
content under a fractional translate. That is also why build-0's dialog, at y = 187.5 with no translate, drew a crisp
Double.

**The fix, by experiment** (`probe-dialog-fix.mjs`, the same page, CSS injected):

| Dialog | Double sample, pixel rows | Thin sample |
|---|---|---|
| as built (`translate: 0 0.5px`) | `139 139 139 139` (blurred) | `129 129` (blurred) |
| translate removed | `25 255 25` (crisp) | `25` (crisp) |
| layout moved 0.5 px with `margin-top`, no translate | `25 255 25` (crisp) | `25` (crisp) |

**Fix.**
- Drop the translate re-snap for dialogs. If you want the dialog box on whole pixels, move it by layout (`top`,
  `left`, `margin`), never by `translate`/`transform`.
- Change the gate to read pixels: a column through the Double sample must read dark, light, dark. My
  `probe-dialog-paint.mjs` does this in 20 lines.

**N9b (P3): the stroked Comma Style reads as the digit "9"** between % and .00 (`w09`). My REVIEW-1 suggestion caused
this. Better: draw it in the decimal icons' grammar, so the three number-format buttons read as one family, e.g.
"0,0" or ",00" in strokes. Or keep a comma with a Lucide dot head: a zero-length path with round caps at about twice
the stroke, so it reads as punctuation, not a digit.

**P3 nits, for the backlog:**
- **Pane tab icons on fractional x.** Round the five tab widths (56, 56, 55, 56, 56) or centre icons on whole pixels
  (N5).
- **`chart-line-markers` and `chart-scatter-lines` are nearly the same picture** for two different chart types. Give
  the scatter its uneven x spacing (points not on a regular column rhythm).
- **`unfreeze` (ours) is nearly `gridlines`;** a dashed or broken freeze line would say "unfreeze".
- **`app-mark` (the product tile) is nearly `format-cells`.** The brand mark should not double as a command icon. This
  belongs to the disambiguation pass.
- **Gridlines in forced colours use `GrayText`.** That is correct system behaviour, but saturated in some contrast
  themes (green in HC Black). No action now; note it for the disambiguation pass.

## Carbon-copy check, round 2

The 9 new chart originals (100 % column and bar, stacked and 100 % line and area, line with markers, doughnut,
scatter with lines) sit on Lucide's chart axes and differ from Microsoft's chart icons in composition. No copies. The
comma is closest to Microsoft's (it is a typographic comma), see N9b.

## Thank you

Three builds in about three hours: from 136 Microsoft icons scaled to whatever fitted, to a 160-icon family of our
own that holds one rule in every theme, size and state. The gate (`icon-rules.mjs`, 168 checks) means it will stay
that way. Its one blind spot is N4: it checks boxes, and the pixels disagreed.
