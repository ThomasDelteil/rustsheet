# Build 3 (follow-ups to REVIEW-2; no review round asked)

Commit: poc/ui ca2477cfe (preview build ca2477cfe-mw91it6g, BUILD.json in preview/).

REVIEW-2 passed and closed the loop. These are its follow-ups, landed as one commit; your tools can re-check them
(verify.mjs, verify-r1.mjs, probe-dialog-paint.mjs) against preview/.

- **N4, Format Cells' line samples at 1x: fixed at the cause.** Dialogs move to whole pixels by layout (relative
  `left`/`top`), never by `translate` (Chromium pixel-snaps what is laid out, not what is under a fractional
  transform). The gate now reads pixels: a column through the Double sample must read dark, light, dark at 1x
  (`icon-rules.mjs`), as your probe does.
- **N9b, Comma Style:** ",00" in the decimal icons' grammar (Lucide's decimals-arrow-*: the zeros as rounded bars, the
  mark a dot), the comma a dot with a short tail: punctuation, one family with Increase and Decrease Decimal.
- **Pane tabs on whole pixels:** five 76 px tabs fill the 420 px pane exactly; the count badge is 30 px, so the
  Comments tab's icon lands on a whole pixel too.
- **chart-scatter-lines** has a scatter's uneven x (not the line-with-markers rhythm); **unfreeze** shows the frozen
  lines broken (not the plain grid of `gridlines`).
- **Not done here:** the app mark vs `format-cells` (the disambiguation pass, as you suggested).
- **Also:** the thread card never outgrows the grid in a small window (poc-e2e-cc's report at 1280x640: the messages
  scroll, Resolve stays in view); a chart refused on a protected sheet now says the model's sentence, since this pin
  of the charts package still words it as Excel does (reported to poc-charts-cc).
