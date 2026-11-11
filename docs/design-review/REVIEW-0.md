# Design review 0: build-0 (`eed8fccde`, built 2026-11-09 05:33Z)

Reviewer: poc-iconreview-cc. Builder: poc-ui-cc. Crops are in `review-0/`, next to this file.

## Read this first: two owner rulings landed during this review

1. **Icons (IP).** Fluent goes, even though it is MIT. Build our own set: **Lucide** wherever it has a good match (Quick
   depends on `lucide-react ^0.563.0`, `/src/quick/app/package.json:203`), plus **original** icons in Lucide's grammar
   (24 px grid, 2 px stroke, round caps and joins) for what Lucide lacks: conditional formatting, AutoSum, fill handle,
   freeze panes, merge, wrap, borders, number formats, chart types. Ship them in one sprite with a manifest
   (name, source `lucide|original`, licence) and Lucide's ISC licence. Nothing may be a carbon copy of a Microsoft
   glyph; I now check that too.
2. **Look and naming.** Keep Excel's familiar layout and placement. Use Quick/Continuum colours everywhere: no Office
   green or blue accents, and icon accents from our palette. Product UI strings must not say "Excel". A larger
   disambiguation pass comes later; no radical redesign now.

So I do not ask you to fix Fluent icons one by one. Most of build-0's inconsistency comes from the pipeline, not from
the icons, and the pipeline would do the same to Lucide. Each finding below gives the rule the new set must follow.
The rules are collected at the end as **R1–R10**. They are what I will check in build-1.

## Verdict

**Not yet consistent, and two blockers.** The calm canvas, floating ribbon, accent bars, ScreenTips and hover states
are close to Microsoft 365 / Google Sheets quality. What breaks the look is mechanical:

- Drawings are scaled to sizes they were not drawn for, so one row has three stroke weights.
- Chevrons are hairlines.
- About 20 typed glyphs and emoji stand in for icons.
- At common laptop widths the ribbon drops the labels of the commands people need most.

The grid is unreadable in high contrast.

## How this was reviewed (reproducible)

My own captures, independent of your gallery. The tools are on branch `poc/iconreview` in
`apps/collab-sheet/design-review/tools/`:

- `capture.mjs`: **592 screenshots** (74 views × light, dark, forced-colours light, forced-colours dark × 1x, 2x).
  They cover every ribbon tab in Classic and Simplified, the Home tab at 1180/1000/860 px, 26 menus, galleries and
  palettes (plus 3 submenus), the six Format Cells tabs, four context menus, the sheet tabs and status bar, the title
  bar, the review pane, the comment popover, and the hover/focus/pressed/disabled states.
  `node capture.mjs http://127.0.0.1:8710 /tmp/cap/b0` (build-0 served from `build-0/preview`) gave
  `capture: 592 shots … problems: 0`.
- `probe-icons.mjs` reads every ribbon icon's drawing size (viewBox) against its rendered size, so its effective
  stroke.
- `probe-geometry.mjs` checks sub-pixel positions, icon/label centring and hit targets.
- `probe-levels.mjs` and `probe-slack.mjs` read the adaptive shrink levels.
- `probe-forced.mjs` checks the grid's colours in forced colours.
- `probe-undo.mjs` and `probe-misc.mjs` cover the remaining state facts.
- `stroke-measure.mjs`, `master-compare.mjs` and `lucide-compare.mjs` measure and compare masters
  (`@fluentui/svg-icons 1.1.345` and `lucide-static 0.563.0`, both fetched as data).

All numbers below were printed by these tools in this session.

---

## Ranked fixes

### P0: blockers

**P0-1 · The grid is unreadable in forced colours (Windows high contrast).** `review-0/01-hc-grid-unreadable.png`
- **Light forced colours:** black paper, black text, black gridlines, invisible headers.
- **Dark forced colours:** white on white.
- **What still shows:** only cells with their own fill.

**Cause, verified.** `theme.ts` `colorResolver` reads each token through a probe `<span>`'s computed `color`. In forced
colours Chromium forces `color` to `CanvasText`, so every grid token comes back the same. `probe-forced.mjs` printed:

```
--cs-color-grid-cell-bg: as the app reads it rgb(0, 0, 0) | with forced-color-adjust:none rgb(255, 255, 255)
--cs-color-grid-cell-fg: as the app reads it rgb(0, 0, 0) | with forced-color-adjust:none rgb(19, 25, 32)
```

and in dark: `rgb(255, 255, 255)` for paper, ink and lines alike.

**Fix.**
- Give the probe `forced-color-adjust: none`.
- Under `(forced-colors: active)`, build the grid theme from **system colours**: paper `Canvas`, ink `CanvasText`,
  gridlines and headers `GrayText`/`ButtonText` on `ButtonFace`, selection `Highlight`, selected header text
  `HighlightText`. Resolve them through the same probe.
- Re-read the theme when the media query changes.
- Add the app (not only the gallery) to the forced-colours gate.

**P0-2 · Replace the icon set (owner ruling), and drop the one shape that is Office's own.**
- **Every icon is Microsoft's.** All 136 icons in build-0 are Fluent (`icons.generated.ts`).
- **One of them is a copy of Office.** The dialog launcher is, in `ribbon.ts`' own words, "Office's own shape"
  (`launcherGlyph`, `review-0/13-launcher-office-shape.png`). It is a carbon copy by the owner's rule.
- **What to build instead:** Lucide plus originals, with the sprite, manifest and licence above.
- **The launcher's replacement:** must say "more settings for this group" in our own drawing (for example Lucide's
  `square-arrow-out-down-right` or an original), at a 24 × 24 hit area (see P2-5).

### P1: the icon system (build-0 evidence; rule for the new set)

**P1-1 · One stroke weight per slot. Build-0 has three in one row.** `review-0/02-large-row-three-weights.png`,
`03-formulas-scaled-and-misaligned.png`

The generator takes the nearest drawing the set has and scales it. For Fluent Regular I measured strokes of 1 px at
16/20, 1.5 px at 24/28 and 2 px at 32. So in the Classic large row:
- **Stretched 28 px drawings, 1.71 px:** Conditional Formatting, Insert, Delete, Format, Sort & Filter, Previous and
  Next Comment, Protect Sheet, Freeze Panes.
- **Stretched 24 px drawings, ×1.33:** Cell Styles, AutoSum, Financial, Logical.
- **Native 32 px drawings, 2 px:** Paste, Find & Select, Insert Function, Text, Date & Time, Lookup, Math & Trig.
- **Worst case, ×1.6:** More Functions is a 20 px drawing blown up to 32 px.

Sort & Filter visibly lighter than its neighbour Find & Select is the owner's "not consistent".

**Also: the large split button is off the row's baseline.** Formulas › AutoSum's icon sits 2 px lower and its label
6 px lower than every other large button (`probe-large-baseline.mjs`: label top 170 vs 164).

→ **R3/R4.** One stroke per slot, rendered, not inherited from a drawing size. Large split buttons share the large
button's icon and label boxes.

**P1-2 · A small slot must get a drawing meant for it.** `review-0/04-simplified-paste-heavier.png`

When a large control is shown small, the 32/28/24 px drawing is squeezed into 20 px (`system.css` lines 293, 299,
326, 333). This happens in the Simplified ribbon, in Classic below 1438 px, and on every folded group button.
- **Squeezed drawings:** Paste becomes 1.25 px, Conditional Formatting 1.07 px, Cell Styles and AutoSum 1.25 px.
  Their 20 px neighbours are a crisp 1 px.
- **Folded Alignment and Number:** these buttons go the other way: a 20 px drawing at 32 px (×1.6).

→ **R4.** One stroke rule, applied at the size the icon is shown. With Lucide this is natural: one 24-unit drawing,
stroke set per slot.

**P1-3 · Chevrons are hairlines.** `review-0/05-chevrons-0.6px.png`

All 31 chevrons in the Classic ribbon (25 in Simplified) are the 20 px drawing at 12 px: 0.6 px strokes in
`--cs-text-secondary`, the faintest marks on the ribbon. The dialogs' `<select>` chevron (`.cs-input`) is a different
drawing again: two gradient strokes (`system.css:88–91`).

→ **R5.** Chevrons and submenu arrows come from the set (`chevron-down`, `chevron-right`) at the stroke of the icons
beside them. One chevron drawing for split buttons, combos, selects and menus.

**P1-4 · Typed glyphs and emoji instead of icons.** `review-0/06-text-glyphs-and-emoji.png`

These sit beside line icons in the same panes. Emoji are drawn by the platform's colour emoji font (here Noto Color
Emoji). In the activity list "replied" (🗨) and "commented" (💬) do not even match each other. Replace each with a
set icon and let the control's `aria-label` carry the name:

| Where (file:line in `/peers/ui/apps/collab-sheet/ui/src`) | Glyph | Use |
|---|---|---|
| `panels/thread-popover.ts:149` thread header | 💬 | `message-square` |
| `thread-popover.ts:150`, `app/app.ts:311` review close, `chrome/status-bar.ts:43` cancel | × | `x` |
| `thread-popover.ts:160,225` Resolved / Resolve / Reopen | ✓ ↺ | `check`, `rotate-ccw` |
| `thread-popover.ts:174` delete comment | 🗑 (emoji) | `trash-2` |
| `panels/comment-field.ts:28` send | ↑ | `send` or `arrow-up` |
| `panels/activity.ts:163` activity kinds | 🗨 💬 ↺ ✎ | `reply`, `message-square`, `rotate-ccw`, `pencil` |
| `activity.ts:164`, `panels/comments.ts:252` expanders | ▾ | `chevron-down` |
| `chrome/context-menu.ts:137` every submenu arrow | ▸ | `chevron-right` |
| `comments.ts:182` card trail; `comments.ts:198` sort | › ⇅ | `chevron-right`; `arrow-up-down` |
| `comments.ts:205` sort menu check | ✓ | the menus' own check (`--cs-check`) |
| `chrome/sheet-tabs.ts:20` add sheet | + | `plus` |
| `grid/fill-ops.ts:50` Auto Fill Options | ⊞▾ | an original fill-options icon + `chevron-down` |
| `app/app.ts:1112` Auto Fill Options items | ● ○ in the label | the menus' checked state (`menuitemradio`) |
| `panels/presence.ts:30`, `thread-popover.ts:39` agent avatar | ✦ | the agent icon of the set (`bot` or `sparkles`) |
| `app/app.ts:933` status notice | ⚠ | `triangle-alert` |

**P1-5 · The border family cannot be told apart.** `review-0/07-borders-indistinguishable.png`

Bottom, Top, Left, Right and No Border are dashed squares whose applied edge has the same weight and colour as the
guides. The ribbon's Borders button (Bottom Border) reads as "no border". Format Cells' edge buttons have the same
problem. Lucide has no border icons, so these are originals:
- **Two tones:** guides (the dashed frame) in `--cs-text-tertiary`, or the icon colour at about 40 %; the applied edges
  in full icon colour.
- **Thick and double variants:** visibly thicker or doubled.

This is the one place where tone carries meaning, so it is not decoration. Do not copy Office's border pictures:
draw ours on the 24 grid.

**P1-6 · At common laptop widths the ribbon drops the labels people need most.** `review-0/08-adaptive-labels-lost.png`

Levels from `probe-levels.mjs`:
- **1440 px:** everything at full size. The last group ends **3.8 px** before the ribbon body's edge
  (`probe-slack.mjs`).
- **1438, 1420, 1366 px:** Styles jumps from level 0 straight to level 2: Conditional Formatting and Cell Styles
  become two unlabelled 20 px icons. Level 1 (labelled small buttons) is passed over. My inference: with these two
  long names it saves no room.
- **1180 px:** Undo, Clipboard, Font, Alignment and Number are all at level 2: Paste and Wrap Text lose their labels.
  Cells and Editing stay at level 1, and **110 px are left empty** at the right.

**Fix.**
- **Styles** goes from full size straight to the folded "Styles" button (as at 1000 px), never to unlabelled icons.
- **The fit re-grows:** after shrinking, give levels back while they fit, so free space is never left while a
  higher-priority group is shrunk.
- **Clipboard** (shrink 9) must keep Paste's label until the last round.

Excel's own order (Styles and Editing first, Clipboard last) is what `shrink` declares; the algorithm does not honour
it at 1180 px.

**P1-7 · Forced colours re-lays out the ribbon.** `review-0/09-hc-ribbon-relayout.png`

At the same 1440 px, high-contrast users get Styles unlabelled (level 2, 50 px; light: level 0, 151 px). In forced
colours `.cs-ribbon__body` gains a 1 px border on each side (`probe-hcborder.mjs`: clientWidth 1422 vs 1424), and
those 2 px tip the fit.

**Fix.** Reserve the border in both modes (a transparent 1 px border at rest), and give the fit a few pixels of
tolerance.

**P1-8 · Icons sit on half pixels.** `probe-geometry.mjs`, Classic, all tabs, 1x:
- **26 of 65 icons:** every one in the two-row groups (Font, Alignment, Number, Charts) has its box at y = n + 0.5.
  Their horizontal strokes (Underline's bar, Strikethrough's line, the alignment bars) smear over two device rows.
- **20 icons:** they start more than 0.1 px off a whole x (large buttons right of Styles: .38, .72, .20; Formulas
  .39, .75; Data .42).
- **Format Cells:** the whole dialog sits at y = 187.5.

**Fix.** Give rows and controls whole-pixel sizes. The ribbon already measures for its fit, so it can round control
widths up to whole pixels at the same time. → **R6.**

**P1-9 · Forced colours empties the colour palettes.** `review-0/18-hc-palette-loses-colours.png`

In forced colours the Fill Color and Font Color palettes draw all 70 swatches as empty boxes, so a high-contrast user
cannot pick a colour by sight. The Cell Styles gallery loses its fills the same way. The ribbon's colour bar already
keeps its colour (`.cs-icon__accent { forced-color-adjust: none }`), so the palette disagrees with the button that
opens it.

**Fix.**
- Give swatches and the gallery's style previews `forced-color-adjust: none`, with a 1 px `ButtonText` edge.
- Mark the current swatch with `Highlight`.

Windows' contrast themes keep content colours visible; only the chrome is recoloured.

### P2: states, metaphors, accessibility

**P2-1 · Undo and Redo: two copies, two states; disabled icons double-dimmed.**
`review-0/10-undo-state-and-disabled-dimming.png`
- **Two states:** the title bar's Undo/Redo are always enabled (`probe-undo.mjs`: `disabled:false`, no
  `aria-disabled`), while the ribbon's copies of the same commands show disabled. Keep one copy (the ribbon's, as
  Microsoft 365 does), or bind both to the command.
- **Double dimming:** disabled ribbon icons get `--cs-text-disabled` (36 % ink) and also `opacity: .55`
  (`system.css:245`), so about 20 % ink, fainter than their own labels (36 %). Drop the opacity: icon and label share
  one disabled token. → **R8.**

**P2-2 · Metaphors that say the wrong thing.** `review-0/11-metaphors-sparkle-lightning.png`
- **Cell Styles uses a sparkle.** In 2026 sparkle means AI, and this app uses `sparkle` for its own agent tab. Take
  Lucide `swatch-book` (a book of style swatches). Reserve sparkle for AI everywhere.
- **Conditional Formatting uses table + lightning.** Lightning is Flash Fill's metaphor in spreadsheets. Draw an
  original: a grid whose cells are highlighted by a rule, with the highlight in an accent from **our** palette (one of
  the three places a colour accent is allowed).
- **Top/Middle/Bottom Align use object-alignment glyphs** (two bars against a line). Draw originals: text lines placed
  top, middle and bottom in a cell box. Lucide's `align-*-vertical` are object alignment too.
- **Fill (Editing) is a bare down arrow.** It reads as "move down" or "download". Lucide `arrow-down-to-line` is
  closer: fill down to the end.
- **Insert and Delete (Cells) are near twins at 20 px**, a row of cells with lines. In narrow layouts they are icon
  only. The new pair must differ in silhouette (plus vs minus/x), not in a 3 px detail.
- **The Conditional Formatting menu has no icons**, while every sibling menu has them. Give its five families icons
  (rules, top/bottom, data bars, colour scales, icon sets), drawn by us.
- **The five chart-type menus have no icons either** (Clustered/Stacked/100 % Column and Bar, Line, Area, Pie,
  Doughnut, Scatter, Combo). Each item needs its chart-type original: these menus are where users tell the types
  apart, and the split buttons that open them already show one.

**P2-3 · Status bar metrics run together.** `review-0/12-status-metrics-run-together.png`

"Average: 26,868.75 Count: 24 Sum: 644,850" is separated by single spaces. Give each metric its own slot, 16–24 px
apart, with the label in `--cs-text-secondary` and the value in primary.

**P2-4 · A checked menu item looks like the current one.** `review-0/14-checked-looks-like-hover.png`

When a menu has icons, a checked item (Format › Lock Cell) loses its check mark and becomes a tinted row
(`system.css:132–133`). The tint is the same family as hover and selection. Keep a check slot: the check, then the
icon, or a tinted tile behind the icon only.

**Related.** A menu opened with the mouse shows its first item filled, as if hovered: `.cs-menu__item:focus` has the
hover background (`system.css:123`). Style only `:focus-visible`, so the item is focused for the keyboard but not
painted for the mouse.

**P2-5 · Targets under 24 × 24 (WCAG 2.2, 2.5.8).** `review-0/15-small-targets.png`

`probe-geometry.mjs` found 15 of 82 ribbon targets undersized:
- the split arrows of Borders, Fill Color, Font Color, Accounting and AutoSum (14 × 24);
- the three launchers (16 × 16);
- the font and size combos (22 px tall) and their arrows (18 × 22);
- the View tab's checkboxes (16 × 16).

**Fix.** Make arrow halves at least 20 px wide, with 24 px of hit area. Launchers can grow their hit area to 24 × 24
without moving the glyph: nothing else is near them. Checkboxes take their label as part of the target.

**P2-6 · "Excel" in product UI strings (owner ruling).** `scan-strings.mjs` over the built bundle found **9 string
literals**.
- **Two copy Excel's own messages word for word.** Rewrite both in our words:
  - the Sort Warning, "Microsoft Excel found data next to your selection. Since you have not selected this data, it
    will not be sorted." (`app/app.ts:1476`);
  - the insert error, "Microsoft Excel can't insert new cells because it would push non-empty cells off the end of the
    worksheet…".
- **App strings that name it:**
  - "Align the text to the bottom of the cell (Excel's default)." (`app/ribbon-tabs.ts:126`)
  - "Give the cells one of Excel's built-in styles…" (`ribbon-tabs.ts:157`)
  - "This text overflows into the empty cells to its right, as in Excel" (in `index-*.js`)
- **Drop target:** "Drop an Excel workbook to add its sheets" (`app/app.ts:1052`, matched in source).
- **Design gallery** (`design.html`):
  - "Excel-grade sheets"
  - "…the cells keep Excel's (Calibri…)"
  - "…Excel's mini toolbar…"
  - "…fields in Excel's layout…"

"xlsx workbook", "spreadsheet" or "our default" say the same. I will re-run the scan on build-1.

**P2-7 · The Fill Color bar disappears in its own default.** The yellow bar (`#FFFF00`) on white is 1.07:1, so it has
no visible edge.
- **Hairline:** give the colour bars a `--cs-line` edge (as the palette swatches already have).
- **Default colour:** per the owner's colour ruling, the initial accents (`initial: "#FFFF00"` and `"#FF0000"` in
  `ribbon-tabs.ts`) come from our palette.

### P3: polish

- **P3-1 Name Box font.** The Name Box uses the monospace stack (`SF Mono, Monaco, Menlo, Consolas`). It reads as a
  code field (`review-0/16-monospace-name-box.png`). Use the UI font.
- **P3-2 Format Cells size.**
  - **Height:** it changes per tab (`probe-misc.mjs`: 485 px; Font 547; Fill 494). The dialog re-centres and OK/Cancel
    jump up to 31 px. Fix the body height to the tallest tab.
  - **Close button:** there is none in the header (`dismiss`, now `x`).
- **P3-3 Selected tab treatments.** The ribbon and dialogs use the accent underline; the review pane uses a filled pill
  (Quick's sidebar). Fine for now, but pick one when the pane is restyled.
- **P3-4 Comment reply resize grabber.** The reply box shows the native textarea resize grabber inside a designed
  field. Hide it, and let the field grow with its text.
- **P3-5 Title bar app tile.** It is the only filled-tile icon on the page (white glyph on primary). Fine as the
  product mark: keep it the single exception.

---

## Rules for the new set: what I will check in build-1

- **R1 Source.**
  - **Lucide:** `lucide-static`/`lucide` at 0.563.x (Quick's line), for everything Lucide has.
  - **Originals:** everything else, drawn on Lucide's 24 grid with a 2 px stroke and round caps and joins.
  - **Packaging:** one sprite; a manifest `{name, source: lucide|original, licence}`; Lucide's ISC licence in
    `licenses/`.
  - **What must not remain:** no Fluent paths, no Office shapes.
- **R2 Carbon-copy test.**
  - **Depict the action, not Microsoft's picture.** Common metaphors (scissors, clipboard, magnifier, brush) are fine
    in our drawing. A Microsoft-specific composition is not: the launcher's corner arrow, Office's Format-Painter brush
    or Wrap-Text loop, a grid-with-lightning, Excel's border pictures.
  - **How I check:** I render each original beside Microsoft's equivalent from the Fluent set and flag close matches.
- **R3 Stroke per slot.** Lucide's default scales the stroke with size, so a 32 px icon gets 2.67 px and looks clumsy
  (`review-0/17-lucide-stroke-policy.png`). Recommended:
  - **20 px slots:** 1.5 px (ribbon small buttons, Simplified, menus, mini toolbar, panes).
  - **32 px slots:** 2 px (Classic large buttons, folded groups).
  - **12 px chevrons:** 1.5 px.
  - **Implementation:** `stroke-width` in 24-units = px × 24 / size, or `vector-effect: non-scaling-stroke`.
  - **Allowed variation:** one rule, no per-icon exceptions except deliberate weight (Bold, thick borders).
- **R4 One drawing, any slot.** A control shown small draws its icon at the small slot with the small stroke. No
  drawing is ever stretched or squeezed. A test asserts every rendered icon's effective stroke per slot.
- **R5 Chevrons and arrows from the set.** One chevron for split buttons, combos, selects and menus; `chevron-right`
  for submenus.
- **R6 Whole-pixel boxes.** Every icon box starts on a whole CSS pixel (x and y), in Classic, in Simplified and in
  dialogs. The test checks this too.
- **R7 Colour.**
  - **Monochrome by default:** icons are `currentColor`.
  - **Accents in three places only:** Fill Color and Font Color (the bar shows the colour that will be applied, with a
    hairline edge) and Conditional Formatting (accent from our palette).
  - **No Office green or blue anywhere in the chrome.**
- **R8 States.**
  - **Disabled:** one token for icon and label.
  - **Hover, pressed, selected:** as now. They are good.
  - **Focus:** ring on `:focus-visible` only.
  - **Forced colours:** icons in `ButtonText`/`CanvasText`, selected in `HighlightText` on `Highlight`, disabled in
    `GrayText`; the colour bars keep their colour (`forced-color-adjust: none` on the bar only).
- **R9 Metaphors.**
  - **Sparkle only for AI.**
  - **Distinct silhouettes** for pairs that appear icon-only (Insert/Delete, Top/Bottom Border, Increase/Decrease
    Decimal).
  - **Every icon** says what the command does.
- **R10 No typed glyphs or emoji** in any icon position (the P1-4 table).

### Lucide names I checked exist in `lucide-static 0.563.0` (suggestions, not orders)

| Command | Lucide | Command | Lucide |
|---|---|---|---|
| Undo / Redo | `undo-2` / `redo-2` | Sort A→Z / Z→A | `arrow-down-a-z` / `arrow-up-z-a` |
| Paste / Cut / Copy | `clipboard-paste` / `scissors` / `copy` | Filter / Clear filter | `funnel` / `funnel-x` |
| Format Painter | `paintbrush` | Find / Replace | `search` / `replace` |
| Grow / Shrink Font | `a-arrow-up` / `a-arrow-down` | AutoSum | `sigma` |
| Bold / Italic / Underline / Strike | `bold` / `italic` / `underline` / `strikethrough` | Fill / Clear | `arrow-down-to-line` / `eraser` |
| Fill Color / Font Color | `paint-bucket` / `baseline` (+ our colour bar) | Cell Styles | `swatch-book` |
| Align left / centre / right | `text-align-start` / `text-align-center` / `text-align-end` | Format (cells) | `table-properties` |
| Wrap Text | `text-wrap` | Merge | `table-cells-merge` |
| Indent less / more | `list-indent-decrease` / `list-indent-increase` | Charts | `chart-column`, `chart-bar`, `chart-line`, `chart-pie`, `chart-scatter`, `chart-area`, `chart-column-stacked` |
| Accounting / Percent | `dollar-sign` / `percent` | Insert Function | `square-function` |
| Decimals more / fewer | `decimals-arrow-left` / `decimals-arrow-right` (check which is which) | Comments | `message-square-plus`, `message-square`, `messages-square` |
| Protect sheet / workbook | `shield` / `file-lock` | Theme / panel / people | `moon`, `sun`, `panel-right`, `users` |

**Needs originals** (no Lucide match): borders (P1-5), vertical alignment (P2-2), conditional formatting, freeze panes
(three variants), Comma Style, Insert/Delete cells as a pair, Auto Fill Options, and the combo chart.

## What I would steal, and what I would avoid

- **Steal from Microsoft 365:**
  - Icons hinted per size: each slot gets a drawing at its own weight.
  - Two-tone border pictures (guides grey, applied edge dark).
  - Colour bars under Fill and Font colour.
  - An adaptive ribbon that keeps the most-used labels longest.
- **Steal from Google Sheets:**
  - A single icon family at a single size and weight across the whole toolbar.
  - Borders offered as a compact 2D grid of choices rather than a long list. Optional, later.
- **Avoid:**
  - Mixing families or emoji.
  - Scaling drawings to arbitrary sizes.
  - Sparkle for non-AI.
  - Any composition that is Microsoft's picture redrawn.

## For build-1, please include

- The preview bundle as now.
- The icon sprite and its manifest.
- A one-line note per P0/P1 item: fixed, or why not.

I will re-run the same 592-shot capture and the probes, and verify each item with before/after crops.
