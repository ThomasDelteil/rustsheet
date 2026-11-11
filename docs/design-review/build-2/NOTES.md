# Build 2 for design review (poc-ui-cc)

Commit: poc/ui c6136c8be (preview build c6136c8be-mw8zw2rm, BUILD.json in preview/). Everything REVIEW-1 asked for is in, and each new rule is a
gate in `scripts/icon-rules.mjs` (now 168+ checks on the live app).

## P1
- **N1 forced colours reach the icons: fixed.** The controls opt out of forced colours (to draw Highlight themselves)
  and their icons inherited that opt-out, so they kept the app's ink. In forced colours `--cs-icon` is now
  `currentColor` (the control's ButtonText, HighlightText when pressed, CanvasText in panes) and `--cs-text-disabled`
  is `GrayText`; the forced-colours token block now out-ranks the dark theme's own declarations (the active header was
  1.47:1 in the dark contrast theme). Gate: in forced colours (light and dark system schemes, with the light and the
  dark app theme), every ribbon icon's stroke equals its control's colour, disabled ones GrayText, and the active and
  selected headers read at 4.5:1 or more.

## P2
- **N2 pointer-opened menus: fixed.** A menu opened by the pointer focuses itself (no item shows the keyboard's
  ring); one opened by the keyboard (Shift+F10, the menu key, Alt+Down, Enter on a menu button) focuses its first
  item. The first ArrowDown enters the items, ArrowUp the last. One rule for context menus and ribbon popups (the
  last input's modality). Gate: click a cell, right-click: no `:focus-visible` item; ArrowDown enters.
- **N3 launchers over folded group names: fixed.** Hidden on folded groups (the dialog is a launcher away in the
  group's popup and through Ctrl+1). Gate: no launcher overlaps its group's name at 1440, 1180, 1000, 860, 760 px.
- **N4 line samples: fixed.** Each sample's box (border included) is made even, so odd lines sit on whole pixels and
  Double is two lines again. Gate: every sample's y on a whole pixel.
- **N5 the pane's tab icons: fixed.** The set's `comments`, `agent`, `people`, and Lucide's `activity` and `history`
  (added to the sprite, now 160 icons), at the 16 px slot with the 1.5 px rule; the five tabs share the row evenly.
  Gate: the pane's icons are the set's at 16 px, 1.5 px.
- **N6 chart-type menus: fixed.** Every type has its picture: Lucide's `chart-column`, `chart-column-stacked`,
  `chart-bar`, `chart-bar-stacked`, `chart-line`, `chart-area`, `chart-pie`, `chart-scatter`, `chart-spline`, and 9
  of ours on Lucide's chart axes (100 % column and bar, stacked and 100 % line, line with markers, stacked and 100 %
  area, doughnut, scatter with lines).
- **N7 diagnostics in product chrome: fixed.** The stress test is in the File menu only with `?debug=1`; the status
  bar names a recalculation's time only when it took a second or more (for six seconds), and every time with
  `?debug=1`.
- **Combos 24 px: fixed.** Their edge is an inset shadow (an outline in forced colours), so the field and the arrow are
  the full 24 px target.

## P3
- N8 launcher weight: 12 px in the secondary ink, 24 px target kept. N9 Comma Style: the head is a ring, all strokes.
  N10 the sheet tab menu: `insert-sheet` and `rename`.

## Also in this build
- No UI sentence of Excel's left: the film review found the protected-cell refusal; it and the wrong-password
  message, Go To's "Reference isn't valid", "There's a problem with this formula", the cut/paste shape message and
  "To prevent possible loss of data" are in our words (the model lane reworded its copy too).
- Range comments (one thread on a range, outlined while open) and QuickSheet in the page are new product features.
