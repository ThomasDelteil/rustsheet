# Transition formula evaluation: what Excel documents, what the corpus shows, and what a design must decide

*poc-design-cc, 2026-11-11. For Taskei P559132976 ("Transition formula evaluation … is not implemented"), whose
owner, the engine lane, wrote that its rules need probing in Excel first. This note is the research. The probe
request follows as its own change. Pages were fetched on 2026-11-11. Nothing here is Excel evidence beyond what
the repository already records: every rule drawn from a page is provisional until a probe answers.*

## In short

- "Transition formula evaluation" is a worksheet option (`sheetPr transitionEvaluation="1"`; BIFF8
  `WsBool.fAltExprEval`). Microsoft documents three effects in one sentence: text strings evaluate as 0, Boolean
  expressions as 0 or 1, and database criteria follow Lotus 1-2-3's rules.
- The engine ignores it. A formula such as `=A1+1` over a text cell is `#VALUE!` here and a number in Excel.
- It is a setting of old files, and where it is on it changes whole sheets: every arithmetic `#VALUE!` root the
  corpus oracle found in the Enron files is on such a sheet.
- Office.js has no property for it, so Excel can only be asked through a file. The oracle tool takes a workbook
  file (`rustsheet-oracle recalc`), so a generated workbook with one such sheet and one control sheet can go
  through excel-graph.

## S1. What Microsoft documents

1. Support, "Advanced options", under "Lotus compatibility Settings for"
   (https://support.microsoft.com/en-us/office/advanced-options-33244b32-fe79-4579-91a6-48b3be0377c4):
   - "Lotus Compatibility Settings for Select the worksheet in this list box that is affected by the following
     options." So the option is one worksheet's.
   - "Transition formula evaluation Opens and evaluates Lotus 1-2-3 files without losing or changing
     information. When this option is selected, Excel evaluates text strings as 0 (zero), Boolean expressions as
     0 or 1, and database criteria according to the rules that are used in Lotus 1-2-3."
   - "Transition formula entry Converts formulas that are entered in Lotus 1-2-3 release 2.2 syntax to Excel
     syntax and makes names that are defined in Excel behave like names that are defined in Lotus 1-2-3." This
     is the other option of the pair (`transitionEntry`): it is about typing a formula, not about calculating
     one.
2. VBA, `Worksheet.TransitionExpEval`
   (https://learn.microsoft.com/en-us/office/vba/api/excel.worksheet.transitionexpeval): "True if Microsoft
   Excel uses Lotus 1-2-3 expression evaluation rules for the worksheet. Read/write Boolean."
3. VBA, `Worksheet.TransitionFormEntry`
   (https://learn.microsoft.com/en-us/office/vba/api/excel.worksheet.transitionformentry): "True if Microsoft
   Excel uses Lotus 1-2-3 formula entry rules for the worksheet. Read/write Boolean."

Office.js: the type definitions the probe scripts are checked against (`@types/office-js` 1.0.617) hold no
member named after either option.

What the one sentence leaves open:

- **"Text strings as 0":** in arithmetic only, or wherever a number is wanted (a function's numeric argument, a
  comparison with a number, a condition)? Text that looks like a number (`"12"+1`: 13 as in Excel, or 1)? The
  empty string? A text literal in the formula as well as a text cell?
- **"Boolean expressions as 0 or 1":** is the result of `=1<2` the number 1 (so `TYPE` gives 1 and `ISLOGICAL`
  FALSE), or TRUE shown as 1? What do `AND`, `OR`, `NOT`, `ISNUMBER` and the constants `TRUE` and `FALSE`
  return? What does a Boolean typed into a cell count as?
- **"Database criteria according to the rules … in Lotus 1-2-3":** which functions (the D-functions only, or
  `COUNTIF` and its kin too), and which rules (a text criterion matching the whole cell instead of its beginning
  is the usual account, UNVERIFIED)?
- **Whose rules apply across sheets:** a formula on such a sheet that reads a cell of an ordinary sheet, the
  reverse, and a defined name used from both.
- **What else changes.** The corpus (S3) shows `IF`, `ISBLANK`, `VLOOKUP`, `FIXED` and `ROUNDUP` roots
  disagreeing on such sheets. Whether each is one of the three documented effects reaching a function, or a
  fourth, is not known.

## S2. The file formats

- SpreadsheetML: `sheetPr@transitionEvaluation` and `sheetPr@transitionEntry`, both off by default. The
  repository's reader of them is the eval snapshot (`crates/rustsheet-eval/src/xlsx_snapshot/calc.rs`,
  `transition_evaluation`, into `CalcProps::transition_evaluation`).
- BIFF8: `WsBool.fAltExprEval` and `fAltFormulaEntry`. The .xls reader carries both to the sheet properties it
  writes (`crates/rustsheet-xls/src/worksheet.rs`).
- The xlsx importer reads neither (`crates/rustsheet-xlsx/src`: no mention of either attribute).

## S3. How common it is, and what it costs today

All from the repository's own records; I did not rerun the corpus.

- Legacy files (`docs/excel-semantics/xls-import.md`, "Transition formula evaluation (Lotus 1-2-3 rules) is a
  sheet setting"): 32 of 2,519 BIFF8 files set it on at least one sheet. On 5 files saved by desktop Excel, "
  `A1+1` over a text cell cached a number".
- xlsx (`docs/excel-semantics/xlsx-storage.md`, "A sheet with `transitionEvaluation` is calculated by Lotus
  1-2-3 rules"): "In the 10k sample, 414 disagreeing roots in 6 files sit on such sheets. Excel cached numbers
  where the en-US rules give `#VALUE!`, for example `n+"s"/n` with a word as the text, and `IF`, `ISBLANK`,
  `VLOOKUP`, `FIXED` and `ROUNDUP` roots disagree there too. Every arithmetic `#VALUE!` root of the Enron corpus
  (5 files) is on such a sheet." Its status is UNVERIFIED (probe pending).
- The corpus oracle keys these disagreements `lotus|`, class gap (`corpus/harness/DECISIONS.md`, H-14), so they
  are not counted as engine bugs.

## S4. What other engines do

- LibreOffice: the file of its OOXML import that reads a sheet's properties
  (`sc/source/filter/oox/worksheetsettings.cxx`, GitHub mirror, `master`, 2026-11-11) does not mention the
  attribute. I did not look for the option elsewhere in its source.
- Not checked: Google Sheets, Gnumeric, IronCalc upstream.

There is nothing to take from another engine here. The rules are Lotus 1-2-3's as Excel implements them, and
only Excel can say what they are.

## S5. What the engine does today (read on main, 2026-11-11)

- Every sheet is calculated by Excel's rules, whatever the file says.
- Text in arithmetic goes through the number coercions of `crates/rustsheet-engine/src/cast.rs` (`get_number`,
  `cast_to_number`), and the database functions and their criteria are in `functions/database.rs` and
  `functions/criteria.rs`. A rule that depends on the formula's sheet would be a new input of those.

## S6. The probe

A generated workbook with two sheets that hold the same cells: `L` with `transitionEvaluation="1"` and `E`
without. Each row is one formula, entered on both. `rustsheet-oracle recalc -b excel-graph` returns both sheets'
values after Excel for the web's full calculation, so each row answers "what does the option change here" by its
two values. Groups of rows, each for one open point of S1:

| Group | Asks |
|---|---|
| Text in arithmetic | a text cell and a text literal under each operator and unary sign; text that looks like a number; the empty string |
| Text where a number is wanted | numeric arguments of functions (`ROUNDUP`, `FIXED`, `ABS`, `SUM` of an argument and of a range, `AVERAGE` and `COUNT` over a range with text) |
| Comparisons and Booleans | the value and the type (`TYPE`, `ISLOGICAL`, `ISNUMBER`) of a comparison, of `AND`, `OR`, `NOT`, `TRUE`, of an `IS` function; a comparison of text with a number and with text in another case |
| Conditions | `IF` over a text cell, an empty cell, a comparison; `ISBLANK` of each |
| Lookups | `VLOOKUP` and `MATCH` with a text key in another case, a number key against text |
| Database criteria | `DSUM`, `DCOUNT` and `DGET` with a text criterion that is the beginning of a cell, the whole of it, in another case, a comparison written as text; `COUNTIF` and `SUMIF` with the same criteria as controls |
| Across sheets | a formula on `L` reading a text cell of `E` and the reverse; one defined name used from both |

Whether Excel for the web honours the attribute at all is the first thing the answer tells: if `L` and `E` agree
on every row, the web does not, and the only evidence left is the cached values of files saved by desktop Excel,
which the corpus already has.

## S7. What a design must decide

1. **The rule set.** One evaluation mode per sheet, decided by the sheet the formula is on (provisional, the
   probe's last group). Each documented effect is then one place: a text operand's coercion, the type of a
   comparison's and a logical function's result, and the criteria match of the database functions.
2. **Where the mode enters.** The coercions take the mode of the formula being evaluated, not of the cell being
   read. A name's body runs under the mode of the sheet that uses it, unless the probe says otherwise.
3. **Recalculation.** The mode is a property of the sheet, not of a cell. It changes no dependency: a formula
   reads the same cells under either mode. Changing it is a full recalculation, as a change of any calculation
   setting is.
4. **Replicas.** The setting must be in the document and equal on every replica, and an older client that
   ignores it computes other values, so it needs the gate every such setting needs (a required capability,
   `crates/rustsheet-sheet-ops/src/compat.rs`).
5. **Files.** The xlsx importer reads the attribute and the exporter writes it; the .xls reader already carries
   it.
6. **The interface.** The checkbox is in Excel's Advanced options, per sheet. Whether this product needs to let
   a user turn it on, or only to honour it in files that have it, is a question for the project lead.
7. **Effort,** if the probe confirms the three documented effects and no fourth: the coercion and Boolean rules
   with their tests about 1.5 engineer-weeks, the database criteria about 1, import, export and the document
   setting with its gate about 3 days. About 3 engineer-weeks, without an interface.

## Open questions for the project lead

1. Is this in scope, and in which milestone? Recommended: yes, as honouring the setting in files, after the
   probe answers. It is what makes old finance workbooks read as they do in Excel, and the corpus oracle already
   sets 414 roots aside for it.
2. Should a user be able to turn the option on here? Recommended: no. It exists for files that came from Lotus
   1-2-3; honour it, show it, and let it be turned off.
3. Until it is implemented, should a workbook with such a sheet say so when it is opened? Recommended: yes, one
   notice naming the sheets.
