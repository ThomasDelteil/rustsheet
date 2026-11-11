# Precision as displayed: what Excel documents, what others do, and what a design must decide

*poc-design-cc, 2026-11-11. For Taskei P559132978 ("Precision as displayed is not implemented"), whose owner,
the engine lane, asked for a short design and Excel probes before any code. This note is the research; the probe
request is `precision-as-displayed-probe/`; the design is not written into a spec yet (S7 says what it must
decide and what I recommend). Pages were fetched on 2026-11-11. Nothing here is Excel evidence: every rule drawn
from a page is provisional until the probe answers.*

## In short

- "Set precision as displayed" is a workbook option (`calcPr fullPrecision="0"`). With it on, every numeric cell
  stores the number its format shows. Microsoft documents that constants lose their digits for good.
- It makes a cell's number format an input of calculation. Today the engine, the read index and the planned
  format-only skip (T-5185) all assume a format never changes a value.
- The option can be set and read from Office.js (`Workbook.usePrecisionAsDisplayed`, ExcelApi 1.9), so one
  Script Lab script can ask Excel nearly everything the documentation leaves open. That script is the probe
  request.
- It is rare: 3 of the 2,120 workbooks of the ontology census and 15 of 2,519 legacy .xls files. Where it is on,
  it changes many cells: one such .xls file differs from Excel in 221 cells.
- LibreOffice implements it with one function of (value, format), applied to a formula's result and to a
  constant when it is read. Its constants keep their digits. That is the structure to copy and the one behaviour
  not to.

## S1. What Microsoft documents

1. Support, "Change formula recalculation, iteration, or precision in Excel", section "Change the precision of
   calculations in a workbook"
   (https://support.microsoft.com/en-us/office/change-formula-recalculation-iteration-or-precision-in-excel-73fc7dac-91cf-4d36-86e8-67124f6bcce4):
   - "When a formula performs calculations, Excel usually uses the values stored in cells referenced by the
     formula. For example, if two cells each contain the value 10.005 and the cells are formatted to display
     values in currency format, the value $10.01 is displayed in each cell. If you add the two cells together,
     the result is $20.01 because Excel adds the stored values 10.005 and 10.005, not the displayed values."
   - "When you change the precision of the calculations in a workbook by using the displayed (formatted) values,
     Excel permanently changes stored values in cells from full precision (15 digits) to whatever format,
     including decimal places, is displayed. If you later choose to calculate with full precision, the original
     underlying values cannot be restored."
   - The steps: File, Options, Advanced, "When calculating this workbook", "select the workbook you want and
     then select the Set precision as displayed check box." So the option is one workbook's.
2. Support, "Set rounding precision"
   (https://support.microsoft.com/en-us/office/set-rounding-precision-e5d707e3-07a8-4df2-810c-218c531eb06a):
   "This option forces the value of each number in the worksheet to be at the precision that is displayed on the
   worksheet." Its note: "Using the Set precision as displayed option can have cumulative calculation effects".
3. Microsoft Learn, "Floating-point arithmetic may give inaccurate result in Excel"
   (https://learn.microsoft.com/en-us/troubleshoot/microsoft-365-apps/excel/floating-point-arithmetic-inaccurate-result),
   "Method 2: Precision as displayed": "This option forces the value of each number in the worksheet to be the
   displayed value." And: "if you choose a number format that shows two decimal places, and then you turn on the
   Precision as displayed option, all accuracy beyond two decimal places is lost when you save your workbook.
   This option affects the active workbook including all worksheets. You can't undo this option and recover the
   lost data."
4. Office.js, `Excel.Workbook.usePrecisionAsDisplayed`
   (https://learn.microsoft.com/en-us/javascript/api/excel/excel.workbook): "True if calculations in this
   workbook will be done using only the precision of the numbers as they're displayed. Data will permanently
   lose accuracy when switching this property from `false` to `true`." `boolean`, read and write, "API set:
   ExcelApi 1.9".
5. VBA, `Workbook.PrecisionAsDisplayed`
   (https://learn.microsoft.com/en-us/office/vba/api/excel.workbook.precisionasdisplayed): "True if calculations
   in this workbook are done by using only the precision of the numbers as they're displayed. Read/write
   Boolean."

What the pages agree on: the option is per workbook; stored values change, permanently for data that is there
when it is turned on; the rule is "the displayed value". What none of them says:

- which formats count (General, dates and times, fractions, scientific, a scaling comma, a section with no
  digit);
- whether "displayed" means the number format alone, or what the cell shows (a narrow column, a conditional
  format's number format);
- when a constant typed later, or a cell formatted later, is rounded;
- what a formula cell does when its format changes, or when the option is turned off;
- page 3's "when you save your workbook" against page 1's "permanently changes": whether the loss is at the
  switch or at the save;
- what Excel does with a file that says the option is on and holds unrounded numbers.

## S2. The file formats

- SpreadsheetML: `calcPr@fullPrecision`, schema default true; false "indicates the application uses the display
  values of the referenced cells when performing calculations" (ECMA-376 Part 1, as quoted in
  `recalc-modes-and-dependency-tracing-2026-11-11.md`).
- BIFF8: the `Precision` record. The repository reads it in two places: the .xls reader
  (`crates/rustsheet-xls/src/globals.rs`, `rt::PRECISION` into `full_precision`, written out as
  `fullPrecision="0"` by `book.rs` `write_calc`) and the corpus tagger
  (`crates/rustsheet-corpus/src/tag/biff/records.rs`, record 0x000E with value 0).

## S3. How common it is, and what it costs today

- The ontology census (`corpus-ontology-census-2026-11-09.md`, "Files per id"): `wb.calc.precisionAsDisplayed`
  in 3 of 2,120 workbooks.
- The .xls scoreboard (`docs/excel-semantics/xls-import.md`, "Precision as displayed"; T-5295): 15 of 2,519
  BIFF8 files set it, and on 3 files saved by desktop Excel the cached values are the displayed roundings
  ("12.24 where the full value is 12.2399…"). That entry is the only Excel evidence in the repository, and it is
  about cached values in files.
- One such file "differs in 221 cells" (T-5308's Outcome).
- The fetched eval suites are not in this container, so I could not count the affected cells of the three census
  workbooks.

## S4. What other engines do

**LibreOffice Calc** ("Precision as shown"; help: "Specifies whether to make calculations using the rounded
values displayed in the sheet", https://help.libreoffice.org/latest/en-US/text/shared/optionen/01060500.html).
Source read from the GitHub mirror, `master`, on 2026-11-11:

- One function, `ScDocument::RoundValueAsShown(double fVal, sal_uInt32 nFormat, …)`
  (`sc/source/core/data/documen4.cxx`):
  - date, time and date-time formats: the value is returned unchanged;
  - the standard (General) format: rounded to the document's "decimal places for General" setting, and unchanged
    when that is unlimited ("#i115512# no rounding for automatic decimals");
  - otherwise the precision is that of the subformat the value selects (`GetSubformatIndex`), then: percent adds
    2; scientific subtracts the value's decimal exponent, with a correction for engineering notation; a fraction
    returns the fraction's value (`GetRoundFractionValue`); number and currency subtract the thousands divisors
    ("tdf#106253 Thousands divisors for format \"0,\"").
- It is called in two places:
  - `ScFormulaCell::InterpretTail` (`formulacell.cxx`, "Precision as shown?"): a formula's numeric result, when
    there is no error and the format is not a date or time;
  - `ScInterpreter::GetValueCellValue` (`interpr4.cxx`): a value cell **when a formula reads it** (`if (
    bCalcAsShown && fOrig != 0.0 )`).
- So LibreOffice never changes a stored constant. Turning the option off gives the digits back, where Microsoft
  says "the original underlying values cannot be restored".

**Not checked:** Google Sheets, Gnumeric, IronCalc upstream. In this repository nothing reads the flag but the
readers above and the eval snapshot (`crates/rustsheet-eval/src/xlsx_snapshot/calc.rs`,
`precision_as_displayed`).

**What I would take, and what I would leave:**

- Take: one pure function of (number, format) with two callers, and its list of format kinds. The list is the
  probe's row groups.
- Take, for a collaborative document: what the engine holds for a constant is a function of what the document
  holds (S7, "Replicas").
- Leave: a precision computed beside the formatter. LibreOffice's function carries two bug references for cases
  where the count of decimals and the display disagreed. Here the rounded number should come from the digits the
  formatter itself prints (`formatter/format.rs`, `format_parsed`), so the two cannot disagree at a tie.
- Leave: constants that keep their digits, if the probe confirms Microsoft's "permanently".

## S5. What the engine does today (read on main, 2026-11-11)

- Nothing reads `fullPrecision`: the xlsx importer's `calcPr` handling takes the iteration settings only
  (`crates/rustsheet-xlsx/src/import/workbook.rs`), and `docs/excel-semantics/number-formats.md` says so
  ("Precision as displayed … is not implemented", UNVERIFIED, citing T-5185).
- A cell that exists carries its own style index (`Model::get_cell_style_index`, `model.rs`), and its number
  format is `get_style_for_cell(…).num_fmt`. The formatter parses a code into sections (`formatter/parser.rs`,
  `ParsePart::{Number, Date, Error, General}`); a number section knows its decimals (`precision`), its percent
  signs (`percent`), its scaling commas (`comma`), whether it is scientific, and its fraction.
- The replicas' engine settings are `EngineOptions` (`crates/rustsheet-contracts/src/determinism.rs`): "Every
  field is set from the document's `calc` identity (I-02), never from the host". It has no field for this option
  and refuses unknown fields.
- T-5185 (blocked) plans that a transaction of number formats alone runs no recalculation, and its task file
  already says the skip must exclude a workbook with this option.

## S6. The questions, and the probe

`precision-as-displayed-probe/README.md` lists them with a provisional rule each. In groups:

| Group | Cases | What it settles |
|---|---|---|
| The rounding function | A (95 rows of format and number), G, CF | which formats round, to what, at ties, and whether "displayed" means the format or the cell |
| When a constant loses digits | E, W, F | at entry, at a later format change, by each way of writing |
| Whether a format change calculates | F, X | a formula cell and its readers, in Automatic and Manual mode |
| Turning it off | O | when formula cells get their digits back |
| Spills and kinds | S, K | each spilled cell by its own format; only numbers |
| Files | SAVE, READ | what Excel writes, and what it does with a file whose numbers are not rounded |

## S7. What a design must decide

1. **The function.** `shown(number, format) -> number`, pure, defined from the digits the formatter prints for
   the section the number selects. Provisional by kind: fixed decimals, percent, scientific, scaling commas and
   fractions round to what is shown; General, dates, times and Text leave the number alone. Tests that need no
   Excel: `format(shown(v, f), f) == format(v, f)` and `shown(shown(v, f), f) == shown(v, f)` over generated
   formats and doubles.
2. **Where it is applied.** Three places store a number in a cell: the evaluator, for a formula's result (a
   spill cell and an array member by their own formats); a write of a constant; and the load of a file. Two
   events change what a stored number shows: a change of the cell's number format, and the option itself.
3. **Recalculation.** With the option on, a formula cell's value is a function of its reads and of its own
   format. So a format change of a formula cell is a root of the dirty set (SPEC-0002 Design 3), a format change
   of a constant is a write of that constant, turning the option on or off owes a full calculation, and T-5185's
   skip does not apply. If the probe's CF case shows that a conditional format's number format counts, a cell's
   value would also depend on that rule's inputs, which the read index does not file: that outcome needs its own
   design.
4. **Replicas.** A document setting, in the `calc` identity, and a field of `EngineOptions`. An older client
   would ignore the setting and compute other values, so the document must name it as a capability such a client
   lacks, which makes it refuse the document (`crates/rustsheet-sheet-ops/src/compat.rs`,
   `UnsupportedRequiredCapabilities`). Formats are already replicated; the function is pure and uses no platform
   arithmetic. For constants there are three ways, and one is wrong:
   - *The engine alone rounds its copy.* Wrong: a replica that joins later computes from the document's digits.
   - *The engine always holds `shown(document number, document format)`, and the document keeps its digits*
     (LibreOffice's way). Replicas agree. Turning the option off, or widening a format, brings digits back,
     which Microsoft says cannot happen.
   - *The same, and every command that Excel documents as destructive also writes the rounded numbers to the
     document, in its own transaction* (turning the option on, writing a constant, changing its format).
     Replicas agree, a sequence of edits by one user gives Excel's values, and undo of that transaction restores
     the digits, which Excel cannot do and which contradicts no Excel sequence. Recommended.
5. **Files.** Import reads the flag and takes the numbers as the file has them, until READ says what Excel does.
   Export writes the flag; the numbers it writes are already rounded.
6. **The interface.** One checkbox, in the place Excel has it, with Excel's warning before it is turned on.
7. **Effort,** if the probe confirms the provisional rules: the function and its tests about 1 engineer-week;
   the three stores, the two events and the recalculation rules about 1.5; import and export 2 days; the
   document setting, the engine option and the gate about 1 across the engine and model lanes; the checkbox and
   its warning 2 to 3 days. About 4 engineer-weeks.

## Open questions for the project lead

1. Is this in scope, and in which milestone? Recommended: yes, after the probe answers; it is a documented Excel
   option, and where it is on, most numeric cells are wrong today.
2. Until then, should a workbook with the option on say so to the user? Recommended: yes, a notice on open
   ("this workbook rounds numbers as displayed; values are calculated at full precision here"). The importer
   would read one more attribute of `calcPr` for it.
3. Constants in a shared document: the third way in S7.4? Recommended: yes.
