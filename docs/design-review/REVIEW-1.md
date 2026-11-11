# Design review 1: build-1 (poc/ui `9c82fa21d`, preview `9c82fa21d-mw8xyg83`)

Reviewer: poc-iconreview-cc. Crops are in `review-1/`. Files named `v*` verify REVIEW-0 items (before/after); files
named `n*` are new findings.

## Verdict

**A large step.** The icon system is now one family and reads as a polished product:
- **Weights:** one stroke weight per slot.
- **Grid:** whole-pixel boxes.
- **Glyphs:** no typed glyphs or emoji.
- **Borders:** a border family that can be told apart.
- **Colour:** accents only where they mean something, in Quick's palette.
- **Layout:** an adaptive ribbon that keeps its labels.

Both P0s are fixed. **No carbon copies of Microsoft icons:** I drew each of the 31 originals and 7 modified Lucide
icons beside Microsoft's drawing of the same idea (`review-1/c01`–`c03`), and none reproduces its composition.

**What remains is narrow:**
- **One P1:** forced colours does not reach the icons, the disabled state or the dark-theme header tokens.
- **Seven P2s:** most are one-line fixes; one is a 1x regression in Format Cells.

I expect build-2 to be the last round.

## How this was verified

Same method as round 0, re-run on build-1 (served from `build-1/preview` on :8711):
- `capture.mjs` gave 592 shots and 0 problems; the File menu was added (`menu-tab-file`).
- `verify.mjs`: 17 checks, 13 pass, 4 fail.
- New probes:
  - `probe-strokes.mjs` (now reads `calc()` widths and screen CTMs);
  - `probe-hc-icons.mjs`, `probe-hc-disabled.mjs`;
  - `probe-menu-open.mjs`, `probe-active.mjs`;
  - `probe-launcher-overlap.mjs`, `probe-line-styles.mjs`;
  - `probe-pane-tabs.mjs`, `probe-pane-rects.mjs`, `probe-targets.mjs`;
  - `carbon-compare.mjs`.

## REVIEW-0 items: fixed or not

| Item | Status | Evidence |
|---|---|---|
| **P0-1** grid in forced colours | **Fixed** | A2:B9 ink 1.00:1 → **21.00:1** in both schemes, 11–12 % of pixels inked (`verify.mjs`). `v01-P0-1-hc-grid.png` |
| **P0-2** replace the set; launcher | **Fixed** | Bundle: no Fluent; Lucide plus `lucide-ISC.txt`; manifest of 146 (108 Lucide, 7 modified, 31 ours). Launcher is Lucide `square-arrow-out-down-right`. `v02-P0-2-icon-set.png` |
| **P1-1** one stroke per slot | **Fixed** | 49/49 large icons at 2 px; AutoSum's label top equals its neighbours'. `v03-P1-1-large-row.png` |
| **P1-2** small slots get small drawings | **Fixed** | 202/202 icons at 20 px draw 1.5 px, Simplified and folded included. `v04-P1-2-simplified.png` |
| **P1-3** chevrons | **Fixed** | 117 chevrons at 12 px, 1.5 px. `v05-P1-3-chevrons.png` |
| **P1-4** typed glyphs and emoji | **Fixed** | 20 (glyph, place) pairs → **0** (`probe-glyphs.mjs`). `v06-P1-4-glyphs.png` |
| **P1-5** borders indistinguishable | **Fixed** | Guides dotted at 45 %, applied edges solid, thick and double distinct. `v07-P1-5-borders.png` |
| **P1-6** labels at laptop widths | **Fixed** | 1366 px: Styles at level 0. 1180 px: 32 px free; only Styles and Editing folded (labelled); Clipboard at level 0. `v08-P1-6-adaptive.png` |
| **P1-7** forced colours re-layout | **Fixed** | Levels at 1440 px identical in light and forced colours. `v09-P1-7-hc-layout.png` |
| **P1-8** half pixels | **Fixed in the ribbon** | 0 of 99 Classic icon boxes off-grid (was 77). **New regression in Format Cells**, see n04. |
| **P1-9** palettes in forced colours | **Fixed** | 70 distinct swatch backgrounds (was 3). `v10-P1-9-hc-palette.png` |
| **P2-1** Undo/Redo state; double dimming | **Fixed** | All four copies disabled at load; disabled icon opacity 1, same token as its label. `v11-P2-1-undo.png` |
| **P2-2** metaphors | **Fixed in the ribbon; chart menus open** | CF is ours (amber), Cell Styles `swatch-book`, vertical alignment ours, Fill `arrow-down-to-line`, Insert/Delete distinct, the CF menu has icons. **The five chart-type menus still have none** (n06). `v12-P2-2-metaphors.png` |
| **P2-3** status metrics | **Fixed** | Own slots, label secondary / value primary. `v13-P2-3-status.png` |
| **P2-4** checked item; mouse-opened menu | **Checked item fixed; open state not** | Lock Cell gets a tile (`v14-P2-4-checked.png`). After a click on a cell, a right-click still opens with a **focus ring** on Cut (n02). |
| **P2-5** targets | **Mostly fixed; accepted trade-off** | Launchers 24 × 24. Split arrows 18 × 24: **accepted**, see below. Combo inputs and arrows measure **20 × 22**, not 20 × 24 (`probe-targets.mjs`): the 24 px wrapper's border is not part of the button. |
| **P2-6** "Excel" strings | **Fixed in the UI**; 4 left in the charts package (owner: poc-charts-cc) | `scan-strings.mjs`: 9 → 4, all in chart-type descriptions ("An Excel chart type Quick cannot draw yet", "That list is not one Excel reads.", …). |
| **P2-7** colour bars | **Fixed** | 1 px edge; defaults #F2B100 / #DB0000 from Quick's palette. `v15-P2-7-fill-bar.png` |
| **P3-1** Name Box font | **Fixed** | UI font (`"Amazon Ember", …`). `v16-P3-1-name-box.png` |
| **P3-2** Format Cells height, close | **Fixed** | 547 px on all six tabs; Close button present. |
| P3-3 selected-tab treatments | Deferred, as agreed | |
| P3-4 reply grabber | **Fixed** | |
| P3-5 app tile | Kept, as agreed | |

**Your call on the split arrows (18 px): accepted.** WCAG 2.2's 2.5.8 has an "equivalent" exception, and every
undersized arrow's function has a full-size equivalent on the page:
- **Borders, Fill Color, Font Color, Accounting:** Format Cells, via the 24 × 24 launchers and Ctrl+1.
- **AutoSum's menu:** the Formulas tab's large AutoSum.

Keeping the 1440 px Home tab labelled is worth more than 2 px. **Please do grow the font name/size combos to 24 px tall**
(the wrapper already is): that costs no width.

## New findings, ranked

### P1

**N1 · Forced colours stops at the icons.** `review-1/n01-hc-icons-and-headers.png`

Three symptoms of one gap: the forced-colours mapping covers the grid and the controls' text, but not `--cs-icon`,
`--cs-text-disabled`, or the tokens the dark theme re-declares.

1. **Icons do not take the system colour.** `probe-hc-icons.mjs`, enabled and not pressed: **65 of 68** ribbon icons
   keep the app's ink, `color(srgb … / 0.82)`, while their labels are `ButtonText` (`rgb(0,0,0)` / `rgb(255,255,255)`).
   In the dark contrast theme the icons are visibly dimmer than their own labels.
2. **Disabled ribbon controls ignore `GrayText`.** They stay at the app's 36 % ink (`ribbon-undo`,
   `ribbon-filter-clear`). The title bar's disabled Undo/Redo use the system's `GrayText`: `rgb(96,0,0)` in light,
   `rgb(63,242,63)` in dark. So the same command looks different in two places, and the user's own disabled colour is
   not honoured.
3. **The active row/column headers are unreadable in the dark contrast theme.** Ink is `rgb(129,118,254)` (Quick's
   dark accent) on the Highlight fill `rgb(0,184,204)`: **1.47:1** (light contrast theme: 11.31:1). The dark theme's
   `--cs-color-grid-header-active-fg: var(--qs-primary-default)` (`:root[data-theme="dark"]`) out-ranks the
   forced-colours mapping.

**Fix.** In the `(forced-colors: active)` block:
- set `--cs-icon` and the icon stroke to `ButtonText` (or `currentColor` of the control);
- set `--cs-text-disabled` to `GrayText`;
- declare the grid tokens with at least `:root[data-theme]` specificity, so the theme's re-declarations cannot win.

**Gate.**
- Every enabled icon's stroke equals its control's colour in forced colours.
- Disabled icons are `GrayText`.
- The active header's ink is at least 4.5:1, in both contrast themes and both app themes.

### P2

**N2 · A mouse-opened context menu shows the keyboard focus ring.** `review-1/n02-menu-focus-ring-on-click.png`

Click a cell, then right-click: the menu opens with Cut focused, `:focus-visible` true, fill plus ring
(`probe-menu-open.mjs`). Why:
- After a click, the grid itself matches `:focus-visible` (`probe-active.mjs`: `DIV grid focus-visible=true`; it is
  focused by script).
- Chromium passes that to an element the script focuses next.

My round-0 capture missed it because an earlier click on the status text had moved focus off the grid.

**Fix.** On a pointer-opened menu, focus the menu container (`role="menu"`, `tabindex="-1"`); the first ArrowDown enters
the items. On a keyboard open (Shift+F10, the Menu key, Alt+↓), focus the first item as now. Do not rely on
`:focus-visible` inheritance.

**N3 · The launcher covers the group name on folded groups.** `review-1/n03-launcher-over-label.png`

`probe-launcher-overlap.mjs` measured:
- **Alignment:** 13.5 px (at 860 and 760 px);
- **Number:** 12.6 px (860 px);
- **Font:** 4 px (760 px).

**Fix.** Hide the launcher on a folded group (its popup can carry it), or reserve its width in the label row.

**N4 · Regression: Format Cells' line styles blur at 1x; the Double style becomes a grey bar.**
`review-1/n04-double-line-regression.png`

The dialog now snaps to whole pixels (top 157; was 187.5). But each line sample is centred in a 22 px box, so every
odd-width line lands on a half pixel (`probe-line-styles.mjs`):
- **1 px lines** (hair, dotted, dash-dot, dash-dot-dot, dashed, thin): y = n + 0.5.
- **3 px lines** (thick, double): y = n + 0.5.

Build-0's half-pixel dialog happened to cancel this.

**Fix.** Centre the samples on whole pixels. Use a box whose height parity matches the line (e.g. 21 or 23 px for
1 and 3 px lines), or position the sample with an explicit whole-pixel offset. Gate: the samples' y on whole pixels.

**N5 · The review pane's tab icons are a second family.** `review-1/n05-pane-tabs.png`

The five tabs (comments, activity, versions, agent, people) draw inline 16-unit SVGs:
- **Outside the sprite:** no `data-icon`, not in the manifest.
- **Size and weight:** at 12 px, so **1.13 px** strokes, the only icons off the 1.5 px rule (`probe-strokes.mjs`: 377
  of 382 on rule, these 5 are the rest).
- **Uneven tabs:** three widths (94.1, 60.5, 32 px).
- **Off the pixel grid:** icon x at 1161.2, 1257.3, 1319.8.

**Fix.** Use the set's `comments`, `agent` and `people`, and add Lucide `activity` and `history` to the sprite (both
exist in 0.563.0). Draw them at 16 px with the 1.5 px rule. Give the tabs equal widths (or equal padding), with the
count badge inside its own tab's width.

**N6 · The chart-type menus still have no icons.** (Open from P2-2.) `review-1/n06-chart-menus-no-icons.png`

Clustered/Stacked/100 % Column and Bar, Line, Stacked Line, Line with Markers, Area, Pie, Doughnut, Scatter (3) and
Combo are text only. The CF menu next to them now has icons. These menus are where people tell chart types apart, so
each item needs its own picture. Lucide covers several: `chart-column`, `chart-column-stacked`, `chart-bar`,
`chart-bar-stacked`, `chart-line`, `chart-area`, `chart-pie`, `chart-scatter`, `chart-spline`. The 100 % and doughnut
variants are originals. If the menus belong to the charts package, please pass this to poc-charts-cc.

**N7 · Developer items in product chrome.** `review-1/n09-debug-in-product.png`
- **File menu:** File › "Add Stress Test Sheets" (with a calculator icon) is a test command in a user menu.
- **Status bar:** "Calculated in 49 ms" is diagnostic text.

Move both behind a developer flag, or into a diagnostics view. If recalculation must show, show its state only while
it is pending or running, as spreadsheets usually do.

### P3

- **N8 · Launchers are as heavy as commands.** `review-1/n07-launcher-weight.png` A 14 px boxed arrow at full icon ink
  sits next to a 12 px secondary group name. Draw it at 12 px in `--cs-text-secondary`, and keep the 24 × 24 target.
- **N9 · Comma Style is the set's only filled form.** `review-1/n08-comma-filled.png` It is `<circle … fill="currentColor"/>`
  plus a stroked tail. Draw the head as a ring, or show "1,000" in strokes, so it obeys the stroke grammar like its 145
  siblings. It also sits very close to Microsoft's own comma glyph; a typographic comma is generic, but a stroked
  version is safer.
- **N10 · Sheet-tab context menu has no icons** (Insert Sheet, Rename Sheet) while every other menu does. Give it
  `insert-sheet` and `rename`.

## Carbon-copy check (owner rule)

`carbon-compare.mjs` drew each original and each modified Lucide icon at 48 px beside Microsoft's (Fluent) drawing of
the same idea (`review-1/c01-carbon-originals.png`, `c02-carbon-borders.png`, `c03-carbon-modified-lucide.png`):
- **Alignment (3), Conditional Formatting and its five families, Fill Options, the four freeze icons, chart combo,
  comment previous/next, delete rows/columns, fill/font colour, the launcher:** each is a different composition from
  Microsoft's.
- **The border family:** follows the industry convention (dotted guides, solid applied edge, as Google Sheets and
  LibreOffice also draw it), on our own dot grid. That is a judgement: Office's ribbon artwork is not on disk to
  compare against.
- **Lucide's own icons:** not copies, even where the metaphor coincides with Fluent's (Wrap Text's return arrow is in
  every family).

**Verdict: no carbon copies.** One caution: N9.

## For build-2

1. **Fixes:** N1–N7, and the 24 px combos.
2. **Gate additions** (your `icon-rules.mjs` is the right place):
   - forced-colours icon and disabled colours;
   - the active header in both contrast themes with the dark app theme;
   - line-style samples on whole pixels;
   - no launcher overlapping its label;
   - review pane icons from the set;
   - a pointer-opened menu with no `:focus-visible` item.

I will re-run the capture, `verify.mjs` and the probes above, and verify each item with a crop.
