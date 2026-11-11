# Build 1 for design review (poc-ui-cc)

Commit: poc/ui 9c82fa21d (preview build 9c82fa21d-mw8xyg83, BUILD.json in preview/). The rules R1-R10 of REVIEW-0 are now a gate in the build:
`scripts/icon-rules.mjs` (151 checks on the live app, light; listed per rule below). The sprite and manifest are in
`icons/` (also served as `preview/icons/icons.svg` and `preview/icons/manifest.json`); Lucide's licence is
`preview/licenses/lucide-ISC.txt`. Screenshots: `shots/` (both themes), the gallery references `gallery/` (light, dark,
forced colours), `55-forced-colours-app.png`. DESIGN-SYSTEM.md section "Icons (D-UI-61)" describes the system.

## P0
- **P0-1 grid in forced colours: fixed.** The grid's tokens become system colours under `(forced-colors: active)`
  (Canvas, CanvasText, GrayText, ButtonFace/ButtonText, Highlight/HighlightText; theme.css end), the probe in
  `theme.ts` has `forced-color-adjust: none`, and the theme is re-read when the media query changes. The app is in
  the gate now: icon-rules reads the grid's theme in emulated forced colours (ink and headers at least 4.5:1, gridlines
  distinct) and shoots `55-forced-colours-app.png`.
- **P0-2 icon set and launcher: fixed.** 146 icons, no Fluent path left: 108 Lucide 0.563.0, 7 Lucide changed for us
  (marked `lucide*` in the gallery, `modified` in the manifest), 31 our own (borders, vertical alignment, Conditional
  Formatting and its five families, freeze panes, Comma Style, Auto Fill Options). The launcher is Lucide's
  `square-arrow-out-down-right` at 14 px in a 24 x 24 target, at the group's corner.

## P1
- (To fit 1440 px with the larger targets: large buttons 48 px minimum (were 52), group padding 4 px, font box 112 px,
  number format box 120 px.)
- **P1-1 one stroke per slot: fixed.** Stroke is set by the slot on screen (1.5 px at 12-20 px, 2 px at 32 px); one
  24-unit drawing per icon. AutoSum's large split now shares the large button's boxes (icon and label tops equal to
  Insert Function's: checked).
- **P1-2 small slots: fixed.** A large control shown small takes the 20 px slot and its 1.5 px stroke (no squeezed
  drawing; checked in Classic 1440/1180/1000 and Simplified).
- **P1-3 chevrons: fixed.** The set's `chevron-down` at 12 px, 1.5 px, for split buttons, menu buttons, combos, the
  dialogs' selects (`selectField`), the panes' expanders; `chevron-right` for submenus and card trails (47 chevrons
  checked).
- **P1-4 typed glyphs and emoji: fixed.** All 15 rows of the table: icons from the set or menu states (Auto Fill
  Options are checked radio items; the comments sort uses the menus' check). Gate: no glyph or emoji in any button,
  menu item or avatar.
- **P1-5 borders: fixed.** Our own family on a wider-than-tall cell: what is not applied is dotted at 45 % opacity, the
  applied edges full; thick variants heavy, doubles doubled; inside lines only when applied. Format Cells' edge
  buttons have their own `border-inside-h` / `-v`.
- **P1-6 labels at laptop widths: fixed.** Styles never shows unlabelled icons (`keepLabels`: full, labelled, or
  folded); after the shrinking rounds, the groups that shrink last get their room back first, at the expense of the
  ones that shrink before them, so a group people need more is never smaller than one they need less. Editing gives
  way before Styles (both are "first" in Excel's order; Conditional Formatting is used more, and stays labelled at
  1280 px, the e2e demo's width). Measured (Home; levels 0 full, 1 labelled small, 2 icons, 3 folded): 1440 px
  everything full (5 px spare); 1366 and 1280 Editing icons, Styles full; 1180 Styles and Editing folded; 1100 Cells
  labelled small; 1000 Alignment icons, Cells icons; 900 Number folded. Clipboard, Font and Paste's label stay full
  down to 900 px. Gate at 1440, 1438, 1366, 1280,
  1180, 1000 px: Styles never at level 2, and no more than 80 px left free while a group is shrunk.
- **P1-7 forced colours re-layout: fixed.** The ribbon body reserves the 1 px edge at rest; the fit has 2 px of
  slack. Gate: the levels at 1440 px are the same in forced colours.
- **P1-8 half pixels: fixed.** The fit rounds control widths to whole pixels (large ones to even widths, so centred
  icons land on pixels), the two-row groups sit at whole offsets, dialogs re-snap after centring (and after their
  entrance animation). Gate: every icon box on whole CSS pixels in Classic (3 widths x 6 tabs), Simplified (6 tabs)
  and Format Cells (6 tabs).
- **P1-9 palettes in forced colours: fixed.** Swatches and style previews keep their colours (`forced-color-adjust:
  none`) with a ButtonText edge; the current swatch and hover in Highlight.

## P2 and P3
- P2-1: the title bar's Undo/Redo run the same commands as the ribbon's and show their state; disabled icons use
  the one disabled token (no extra opacity).
- P2-2: Cell Styles is `swatch-book`; Conditional Formatting is ours (a rule-coloured cell, in Continuum's amber), and
  its menu's five families have icons; vertical alignment is text lines in a cell; Fill is `arrow-down-to-line`;
  Insert/Delete are a grid with a plus / with a cross; sparkle only for the agent.
- P2-3: each status metric has its own slot (label secondary, value primary, 20 px apart).
- P2-4: a checked item in an icon menu keeps a check (a tile behind its icon, or the check in its icon's place);
  menu items paint as hovered only for the mouse, focused only for the keyboard (`:focus-visible`).
- P2-5: launchers 24 x 24; combo arrows 20 x 24; checkboxes are their labels. Split arrows 18 x 24 (were 14): the
  full 20 cost the 1440 px Home tab its Styles labels, so I chose labels at 1440 over the last 2 px; your call.
- P2-6: no product string says Excel or quotes it (sort warning, insert refusal, Find's messages, the import
  refusals, tips, the gallery). Remaining 4 are in the charts package's layer (reported to poc-charts-cc).
- P2-7: colour bars have a 1 px edge; Fill and Font Color default to our palette's amber (#F2B100) and red (#DB0000).
- P3-1 Name Box in the UI font; P3-2 Format Cells has one height on every tab (checked) and every dialog a Close
  button; P3-4 the reply field grows with its text, no grabber; P3-5 the app tile stays the one filled tile.
- Not changed: P3-3 (selected-tab treatments), as you suggested, until the pane is restyled.
