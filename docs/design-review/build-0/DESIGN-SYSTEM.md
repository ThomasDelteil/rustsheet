# The RustSheet design system (collab sheet UI, D-UI-55)

The owner's directive (2026-11-09): build a design system for the grid product that works across the board, then
restyle everything onto it, with a living gallery whose screenshots are a gate. The bar is Microsoft 365 or Google
Sheets polish, in Quick's brand. This is that system: what it is, how it is built, and how it is kept honest.

**Where it lives** (`apps/collab-sheet/ui`)

| Part | File |
|---|---|
| Tokens: Continuum (layer 1, Quick's values) and the app's semantic tokens (layer 2) | `src/design/theme.css` |
| Components: every control, every state | `src/design/system.css` |
| Builders: buttons, icon buttons, badges, avatars | `src/design/primitives.ts` |
| Icons: the vendored set, inline SVG | `src/design/icons.ts`, `src/design/icons.generated.ts`, `scripts/icons.mjs` |
| The ribbon, menus, palettes, galleries, dialogs, ScreenTips | `src/chrome/{ribbon,context-menu,color-picker,cell-styles,dialog,screen-tip}.ts` |
| The living gallery | `src/design/gallery.ts`, `design.html` (dev), `preview/design.html` (static preview) |
| The gate | `scripts/design.mjs`; references in `test/visual/*.png` |

Open the gallery at `/design.html` on the dev server, or `design.html` next to the static preview's `index.html`.

## 1. Principles

1. **One definition per component.** The app and the gallery call the same builders (`Ribbon`, `menuElement`,
   `dialogElement`, `colorPalette`, `cellStylesGallery`, `button`, `iconButton`, `screenTipContent`). What the gallery
   shows is what ships; there is no "gallery copy" to drift.
2. **Components read semantic tokens only.** A component says what a colour is *for* (a panel, a line, a control's
   hover); the theme says what it is. `system.css` contains no colour literal: a unit test (`test/theme.test.ts`) fails
   on any `#hex` or `rgb()` outside `theme.css`, and on layer-2 colours that are not Continuum tokens or mixes of them.
3. **Quick's brand through Continuum.** Layer 1 is Continuum's token names with Quick's values in a zero-specificity
   `:where()` block, so inside Quick, Quick's theme (and an org's brand theme) wins. The accent is Quick's primary,
   never Excel's green.
4. **Excel's layout and behaviour, Fluent's craft.** The ribbon, Format Cells and menus follow Microsoft 365 Excel's
   structure and words; the visuals (rounded panels on a canvas, accent bars, subtle controls, 20/32 px icons) follow
   Fluent 2. Microsoft's proprietary Office artwork is not used.
5. **Every state, visible and accessible.** Hover, pressed, keyboard focus, disabled (still focusable, with its
   reason) and selected exist for every interactive component, in light, dark and forced colours.

## 2. Tokens

Two layers (unchanged since D-UI-42, extended here):

- **Layer 1, Continuum (`--qs-*`)**: Quick's values for its palette (content, main, foregrounds, dividers, primary,
  status, categorical colours, radii, shadows, fonts). Light in `:where(:root, [data-theme="light"])`, dark in
  `:where(:root[data-theme="dark"], [data-theme="dark"])`.
- **Layer 2, the app (`--cs-*`)**: mappings onto layer 1, declared on `:root` and on any `[data-theme]` element, so a
  subtree can show the other theme (the gallery shows both side by side).

The semantic set added for the design system:

| Family | Tokens | Use |
|---|---|---|
| Surfaces | `--cs-surface-app`, `-panel`, `-raised`, `-sunken` | the canvas; the ribbon, formula bar and panes on it; menus and dialogs; tab tracks. Dark: the canvas is darkest, raised surfaces lighter (Fluent's rule) |
| Text | `--cs-text-primary`, `-secondary`, `-tertiary`, `-disabled`, `-accent`, `-on-accent`, `-danger` | |
| Icons | `--cs-icon`, `--cs-icon-accent` | neutral glyphs a shade lighter than text |
| Lines | `--cs-line-subtle`, `--cs-line`, `--cs-line-strong` | dividers, borders |
| Controls | `--cs-control-bg`, `-hover`, `-pressed`, `-selected`, `-selected-hover`, `-selected-fg`, `-border`, `-border-hover`, `-border-focus`; `--cs-accent-hover`, `--cs-accent-pressed` | every button, tab, menu item and field |
| Focus | `--cs-focus-width` (2 px), `--cs-focus-offset` (1 px), `--cs-focus-ring` | keyboard focus only (`:focus-visible`) |
| Type | `--cs-text-100..600` (11, 12, 13, 14, 16, 20 px), `--cs-leading-100..600`, `--cs-weight-regular/semibold` | 13 px is the UI's body; cells keep Excel's Calibri |
| Radii | `--cs-radius-sm` (4), `-md` (6, Quick's), `-lg` (8), `-xl` (12), `-pill` | controls, menus and panels, dialogs |
| Elevation | `--cs-elevation-panel`, `-flyout`, `-dialog`, `-toast` | |
| Motion | `--cs-duration-fast/normal`, `--cs-ease-standard/decelerate` (all 0 under reduced motion) | |
| Sizes | `--cs-size-control-sm/md/lg` (24/28/32), `--cs-size-icon-sm/lg` (20/32), `--cs-ribbon-row` (24), `--cs-ribbon-large-w`, `--cs-ribbon-simplified-row` (32) | |
| Grid | `--cs-color-grid-*` (paper, ink, gridlines, headers: active, selected, filtered; selection, fill handle, copy marquee, frozen line, comment marker, agent tint), `--cs-presence-0..6`, `--cs-presence-self`, `--cs-ref-0..5` | read by the canvas through `theme.ts` |
| Shapes | `--cs-check` | a mask whose colour is the element's background (a token) |

## 3. Components

Each has every state: **hover** (`:hover`), **pressed** (`:active`), **focus** (`:focus-visible`), **disabled**
(`:disabled` or `aria-disabled="true"`, kept focusable with its reason as `aria-description` and in its ScreenTip),
**selected** (`aria-pressed`, `aria-selected`, `aria-checked` or `aria-expanded`). In the gallery each state is also
selectable as `data-state="hover|pressed|focus"` so all can be shown at once.

| Component | Builder / classes | Variants | Notes |
|---|---|---|---|
| Button | `button()` / `.cs-btn` | default (outlined), primary, subtle (`--ghost`), danger; `--sm` | a toggle shows `aria-pressed` as selected |
| Icon button | `iconButton()` / `.cs-iconbtn` | 32 px, `--sm` 24 px | its label is its name and tooltip |
| Ribbon | `Ribbon` / `.cs-ribbon` | Classic, Simplified (Ribbon Display Options; kept per browser) | tabs with the accent bar, groups with names and dialog launchers, adaptive shrinking (below), show tabs only (Ctrl+F1) |
| Ribbon controls | `.cs-rbtn--large/--small`, `.cs-rsplit--large/--small`, `.cs-rmenu`, `.cs-ribbon__combo`, `.cs-ribbon__select`, `.cs-ribbon__check` | large (32 px icon over its label), small (20 px icon, label at its right or icon-only), split (action + arrow, each half its own hover), menu, gallery, combo box, list, checkbox | declared as data (`src/app/ribbon-tabs.ts`) over command ids |
| Menu | `menuElement()`, `showContextMenu()` / `.cs-menu` | icons, shortcuts, headings, separators, checked, disabled with reason, submenus that flip left | arrows, Enter, Esc, ArrowRight/Left |
| Popup | `showPopup()` / `.cs-popup` | a palette, a gallery, a folded ribbon group | one at a time with the menus; outside click or Esc closes |
| Colour palette | `colorPalette()` / `.cs-palette` | Fill (No Fill) and Font (Automatic) | Office's theme colours with five tints each (lumMod/lumOff), standard colours, More Colors; arrow keys move in the grid |
| Gallery | `cellStylesGallery()` / `.cs-gallery` | Cell Styles | each style drawn as a cell with it is (Automatic ink as the grid draws it) |
| Fields | `.cs-input` (text, select, number spinner, textarea), `.cs-check`, native checkbox and radio in the accent | | focus is the accent along the bottom edge (Fluent's); invalid is `aria-invalid` |
| Tabs | `.cs-tablist` / `.cs-tabitem` (and `.cs-dialog__tab`) | dialogs, panes | the accent bar under the current one |
| Sheet tabs | `SheetTabs` / `.cs-tabs`, `.cs-tab` | | the shown sheet hangs in the grid's own paper with the accent bar |
| Dialog | `dialogElement()`, `showDialog()` / `.cs-dialog` | Format Cells is the reference | title, tabs, fields in Excel's layout, OK (primary) and Cancel; focus kept inside; Enter is OK, Esc Cancel |
| Task pane | the Review pane (`.cs-review`, Quick's `qs-*` components) | | Quick's own sidebar CSS on the same tokens |
| ScreenTip | `ScreenTips`, `screenTipContent()` / `.cs-screentip` | name and shortcut, what it does, why unavailable | after 450 ms of hover, at once on keyboard focus; replaces the browser's title tooltips on the ribbon |
| Toast | `toast()` / `.cs-toast` | info, error | |
| Status bar | `StatusBar` / `.cs-status` | | |
| Badges, chips, avatars | `badge()`, `.cs-chip`, `avatar()` | neutral, accent, agent, danger; people's colours with readable ink | |

### The ribbon's layout

- **Classic** (default): Microsoft 365's. Large buttons (Paste, Conditional Formatting, Cell Styles, Insert, Delete,
  Format, Sort & Filter, Find & Select), small ones three to a column (Cut, Copy, Format Painter; AutoSum, Fill,
  Clear) or two rows of them (Font, Alignment, Number), each group named underneath; the Font, Alignment and Number
  groups' launchers open Format Cells on that tab.
- **Simplified**: one row of small controls; groups set apart by dividers; labels only where they carry meaning
  (Wrap Text, the menus).
- **Adaptive**: when the window narrows, groups give up room in Excel's order (`shrink`, lowest first: Styles, Editing,
  Cells before Font and Clipboard): large buttons become small ones, small ones lose their labels, and at last a group
  folds into one button that opens it as a popup. A step that would not save room is passed over; folding waits for
  its own round, so the groups that fold first are the ones Excel folds first. Measured, not estimated.

### Icons

Fluent UI System Icons (Microsoft's open set, MIT; `@fluentui/svg-icons` 1.1.345), 136 of them vendored as path data
(67 KiB) by `scripts/icons.mjs`, which reads the package as untrusted data (path syntax only). Drawn as inline SVG in
the current text colour, so every state and theme recolours them for free; Fill Color and Font Color draw the set's
accent layer in the colour they apply. The licence ships with the bundle (`licenses/fluentui-system-icons-MIT.txt`)
and its notice stays in the minified code (`/*! @license ... */`).

## 4. Accessibility

- **Keyboard.** The ribbon's tab bar is one Tab stop; arrows move between tabs, Down goes into the tab's controls,
  F6 and Shift+F6 cycle the worksheet, ribbon, task pane and sheet tabs. Menus and the palette are arrow-driven; Esc
  returns focus where it came from. Dialogs keep focus inside. Every ribbon command has its Excel shortcut where
  Excel has one, shown in its ScreenTip and menu item (`aria-keyshortcuts`).
- **ARIA.** Tab lists hold only tabs (axe's `aria-required-children` caught the first cut); toggles are
  `aria-pressed`, menu items `menuitem`/`menuitemcheckbox`, popups `aria-haspopup` + `aria-expanded`; disabled
  controls are `aria-disabled` with `aria-description` (their reason). `scripts/a11y.mjs` (axe-core) gates it:
  no critical or serious finding.
- **Contrast.** `scripts/design.mjs` checks every text in the gallery, light and dark, against what it is drawn on
  (backgrounds composited): WCAG 2.2 AA, 4.5:1, or 3:1 for large text; disabled controls are exempt as WCAG exempts
  them. It found and this system fixed: white on Quick's dark primary (3.3:1) and dark error (2.8:1), now the dark
  surface's colour (5:1, 6:1); the accent as text in the dark (4.4:1, now a lighter mix); the danger badge (4.2:1);
  people's initials (now the app's readable ink).
- **High contrast.** A `forced-colors` block keeps every control's edge, the selected and focused ones in the
  system's Highlight, icons in the text colour; the gate renders the gallery in forced colours and compares it too.

## 5. The gate

`node scripts/design.mjs` (after `npm run build`, with the dev server up) does three things and exits 1 on any
failure:

1. **Visual**: each of the gallery's 12 sections, light and dark, is screenshot and compared with its reference in
   `test/visual` (pixelmatch, threshold 0.1, at most 0.2 % of pixels may differ). A change of look fails until its new
   references are reviewed and recorded with `--update` (the diff images go to `/tmp/collab-sheet/design-diff`).
2. **Contrast**: as above.
3. **Forced colours**: the light gallery in forced colours, against its own references.

It runs in `npm run e2e:all` (`e2e:design`).

## 6. What we took and what we avoided

- **Took from Microsoft 365 / Fluent 2**: the floating rounded ribbon on a neutral canvas; the accent bar under the
  current tab; subtle buttons that fill on hover; split buttons whose halves hover separately; 20/32 px icons; rich
  ScreenTips with the shortcut and the reason; the bottom-edge focus of text fields; Classic and Simplified layouts;
  Excel's group order and words; Office's colour palette and built-in cell styles (from an Excel-saved workbook's
  styles part).
- **Took from Google Sheets**: a calm, low-chrome canvas and a single brand accent; menus with a check in the icon's
  place.
- **Avoided**: Office's proprietary artwork and Excel's green; browser `title` tooltips on the ribbon (late, plain,
  cannot say why a command is unavailable); colour literals in components; a gallery built from copies; disabled
  controls that cannot be focused (their reason would be unreachable).

## 7. Adding a component

1. Tokens first: reuse a semantic token; add one to layer 2 only when a meaning is missing, mapped onto layer 1.
2. Write its CSS in `system.css` with all five states (and its `data-state` twins), its builder in `design/` or
   `chrome/`, and use the builder everywhere.
3. Add it to the gallery (`gallery.ts`), in every state.
4. `npm run build && node scripts/design.mjs --update`, look at the new references, commit them with the code.
