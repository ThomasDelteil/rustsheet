# Probe request `precision-as-displayed`: what "Set precision as displayed" rounds, and when

*From poc-design-cc, 2026-11-11, for the Excel oracle owner (routed by the coordinator). For Taskei P559132978:
the engine ignores `calcPr fullPrecision="0"`, and its owner asked for a short design and Excel probes before
any code. The research note beside this directory (`../precision-as-displayed-2026-11-11.md`) has what Microsoft
documents (sections S1 and S2), what LibreOffice does (S4) and the questions the documentation leaves open (S6).
A provisional rule below is what Microsoft documents where it documents anything, and says so.*

## Why these cannot be static cases

A case of `eval/suites/probes` is a workbook and its values after one full recalculation, and the case builder
writes no `calcPr fullPrecision` (`tools/`, no setting for it). Most questions here are about a sequence: turn
the option on, change a format, read, calculate, read. Office.js has the option as
`Workbook.usePrecisionAsDisplayed` (ExcelApi 1.9, read and write), and `Range.values` returns what a cell
stores, so Script Lab can ask all of them.

## How

- **Office.js:** run `cases.js`, one case per **new, empty** workbook: `run("A")`, `run("E")`, `run("F")`,
  `run("X")`, `run("O")`, `run("S")`, `run("K")`, `run("W")`, `run("G")`, `run("CF")`. Each logs one JSON line
  with the build, the platform, the calculation mode and the option as it found them (`modeAtStart`,
  `padAtStart`). Every read logs the option, the mode and the application's `calculationState`. Please return
  the lines as `results.jsonl`; it is committed beside this README with a `RESULTS.md`, as the earlier probe
  directories do.
- **The option is destructive.** Microsoft: "Data will permanently lose accuracy when switching this property
  from `false` to `true`" (research note, S1). So `run()` refuses a workbook in which any sheet holds a value
  (`WorkbookNotEmpty`), before it writes anything, and puts the option back as it found it when the case ends or
  fails. Please do not run it beside real data all the same.
- `node check.js` runs every case against a fake of Office.js that keeps a grid, the option and the mode. It
  checks that the script holds exactly the cases named here and the 95 rows listed below; one JSON line per
  case; the option and Automatic mode restored, also when any one sync fails (except the last sync, which is the
  one that restores, failing with its write dropped); the refusal of a workbook with values; the shape of every
  array written; and that the table cases write the rows below where this README says. It needs no Excel and
  says nothing about Excel's values.
- Excel for the web is the set of record: excel-graph is the only oracle (decision 0266 rule 2), and Script Lab
  runs there. Whether the property can be set in Excel for the web is the first thing the run tells (a refused
  write is logged with its error code). If the web refuses it, a run on Excel 365 desktop, en-US, by a listed
  oracle operator who signs it (decision 0427) is the only set, and the rules are marked as desktop evidence.
- The script writes numbers with `Range.values` and formulas with `Range.formulas`. They stand in for typing.
  Where an answer differs when it is typed, say so in `RESULTS.md`.
- A case whose requirement set the host lacks should say so instead of failing silently (the script logs the
  error code).

## A and E: which formats round, and to what

`cases.js` holds one table `T` of 95 rows: a number format, a number, and a formula that gives that number. Row
*i* is sheet row *i* + 1. Column A holds the number as a constant and column B the formula, both in the row's
format. Columns C and D read A and B (`=A2`, `=B2`), in General, set after the formula so that the reader does
not take the format of the cell it reads.

| Case | Steps | Question |
|---|---|---|
| A | write the table; read; turn the option on; read; full calculation; read | For each row: what does the cell store once the option is on (`values`), what does it show (`text`), and what does a reader get? |
| E | turn the option on; A and B: the format, then the number; E and F: the number in a General cell, then the format; read after each step; full calculation; read | Is a number rounded when it is entered into a formatted cell? Is a constant rounded when its cell is formatted afterwards? |

What the rows ask, with our provisional rule. "Rounds as shown" means: the cell stores the number its text
shows.

| Rows (format) | Question | Our provisional rule |
|---|---|---|
| `0.00`, `0.0`, `0`, `#,##0.00`, `000`, `00.0` | Rounds as shown? Ties (2.675, 0.125, 0.5, 2.5, -0.5) and doubles just under a tie (1.005, 8.325, 0.285, 0.49999999999999994): as ROUND gives, or as the binary value falls? | rounds as shown, as Microsoft documents ("to whatever format, including decimal places, is displayed"); a tie as the cell's text shows it |
| `0` with 1.2345678901234567E+19 and 123456789012345.6 | Is a number shown with 15 digits and zeros stored as those digits? | yes, the text's number |
| 16 and 18 decimals with `=0.1+0.2`, `=1/3` | Does a format with more decimals than a double holds change the bits (0.30000000000000004 to 0.3)? | UNVERIFIED: yes if the rule is the text's number; no rule in the documentation |
| `General` | Unchanged? | UNVERIFIED: unchanged. The documentation speaks of formats with decimal places; LibreOffice leaves General alone unless its decimals are limited |
| `0%`, `0.0%`, `0.00%` | 0.12345 in `0.0%` stores 0.123? | rounds as shown |
| `0.00E+00`, `0E+0`, `0.0E+0`, `##0.0E+0` | 12345 in `0.00E+00` stores 12300? | rounds as shown (LibreOffice: the same) |
| `# ?/?`, `# ??/??`, `# ?/8`, `# ?/2`, `?/?` | 0.3 in `# ?/?` stores 2/7? 0.04 stores 0? | UNVERIFIED: the fraction's value (LibreOffice: the same) |
| `#,##0,`, `0.0,,`, `0.00,"K"` | 1234567 in `#,##0,` stores 1235000? | UNVERIFIED: rounds as shown (LibreOffice: the same) |
| dates and times | 45000.75 in `m/d/yyyy` stays 45000.75, or becomes 45000? 0.5004 in `h:mm`? | UNVERIFIED: unchanged (LibreOffice leaves dates and times alone) |
| `@` | In A the cell is Text before it is written, so the constant and the formula's own text are both stored as text: a control. In E, columns E and F get the Text format after the number: unchanged? | unchanged |
| two sections, a condition | The section the number selects decides: -1.2345 in `0.00;-0.0` stores -1.2? | UNVERIFIED: yes |
| `0.00;;`, `;;;`, `"abc"`, `#,###` with 0.4, `#.##` with 0.001 | A section that shows no digit, or nothing at all: unchanged, or 0? | UNVERIFIED: unchanged where the section has no digit placeholder; 0 where its placeholders show none |
| `0.0;-0.0;"zero"` with 0.04 and -0.04 | Rounds to 0 by the first or second section: 0, and is -0.04 stored as 0 or -0? | UNVERIFIED: 0 |
| literals, colours, padding, a locale tag, the accounting format | Rounds as shown, whatever else the section prints? | rounds as shown |
| `#.##`, `0.??`, `?.?`, `#` | Optional digits round as `0` digits do? | rounds as shown |

## The rows of the table

Every row of `T`, as `cases.js` holds it; `node check.js` fails when the two differ. The number is the
JavaScript double the script writes as the constant; the formula is what it writes beside it.

| Sheet row | Format | Number | Formula |
|---|---|---|---|
| 2 | `0.00` | 1.2345 | `=1.2345` |
| 3 | `0.00` | 2.675 | `=2.675` |
| 4 | `0.00` | 1.005 | `=1.005` |
| 5 | `0.00` | -1.005 | `=-1.005` |
| 6 | `0.00` | 0.125 | `=0.125` |
| 7 | `0.00` | 0.135 | `=0.135` |
| 8 | `0.00` | 0.285 | `=0.285` |
| 9 | `0.00` | 8.325 | `=8.325` |
| 10 | `0.00` | 1234567.891 | `=1234567.891` |
| 11 | `0.00` | 0.004 | `=0.004` |
| 12 | `0.00` | -0.004 | `=-0.004` |
| 13 | `0.00` | 123456789012.345 | `=123456789012.345` |
| 14 | `#,##0.00` | 1234.5678 | `=1234.5678` |
| 15 | `0.0` | 0.25 | `=0.25` |
| 16 | `0.0` | 0.35 | `=0.35` |
| 17 | `0.0` | -0.25 | `=-0.25` |
| 18 | `0.0` | 0.05 | `=0.05` |
| 19 | `0` | 0.5 | `=0.5` |
| 20 | `0` | 1.5 | `=1.5` |
| 21 | `0` | 2.5 | `=2.5` |
| 22 | `0` | -0.5 | `=-0.5` |
| 23 | `0` | -2.5 | `=-2.5` |
| 24 | `0` | 0.49999999999999994 | `=0.49999999999999994` |
| 25 | `0` | 12345678901234567000 | `=12345678901234567000` |
| 26 | `0` | 123456789012345.6 | `=123456789012345.6` |
| 27 | `000` | 1.5 | `=1.5` |
| 28 | `00.0` | 1.25 | `=1.25` |
| 29 | `0.0000000000000000` | 0.30000000000000004 | `=0.1+0.2` |
| 30 | `0.000000000000000000` | 0.3333333333333333 | `=1/3` |
| 31 | `0.00000000000000` | 0.30000000000000004 | `=0.1+0.2` |
| 32 | `General` | 1.23456789012345 | `=1.23456789012345` |
| 33 | `General` | 0.30000000000000004 | `=0.1+0.2` |
| 34 | `General` | 123456789.123456 | `=123456789.123456` |
| 35 | `General` | 12345678901234567000 | `=12345678901234567000` |
| 36 | `General` | 0.000001234567890123 | `=0.000001234567890123` |
| 37 | `0%` | 0.125 | `=0.125` |
| 38 | `0%` | 0.135 | `=0.135` |
| 39 | `0%` | 0.12345 | `=0.12345` |
| 40 | `0.0%` | 0.12345 | `=0.12345` |
| 41 | `0.00%` | 0.123456 | `=0.123456` |
| 42 | `0.00E+00` | 12345 | `=12345` |
| 43 | `0.00E+00` | 0.00012345 | `=0.00012345` |
| 44 | `0.00E+00` | -98765 | `=-98765` |
| 45 | `0E+0` | 15000 | `=15000` |
| 46 | `0.0E+0` | 0.000999 | `=0.000999` |
| 47 | `##0.0E+0` | 12345 | `=12345` |
| 48 | `##0.0E+0` | 123456 | `=123456` |
| 49 | `# ?/?` | 0.3 | `=0.3` |
| 50 | `# ?/?` | 2.375 | `=2.375` |
| 51 | `# ?/?` | -0.3 | `=-0.3` |
| 52 | `# ?/?` | 0.04 | `=0.04` |
| 53 | `# ??/??` | 0.3 | `=0.3` |
| 54 | `# ??/??` | 3.14159 | `=3.14159` |
| 55 | `# ?/8` | 0.3 | `=0.3` |
| 56 | `# ?/2` | 0.3 | `=0.3` |
| 57 | `?/?` | 2.375 | `=2.375` |
| 58 | `#,##0,` | 1234567 | `=1234567` |
| 59 | `#,##0,` | 499 | `=499` |
| 60 | `#,##0,` | 500 | `=500` |
| 61 | `0.0,,` | 123456789 | `=123456789` |
| 62 | `0.00,"K"` | 1234567 | `=1234567` |
| 63 | `m/d/yyyy` | 45000.75 | `=45000.75` |
| 64 | `yyyy` | 45000.75 | `=45000.75` |
| 65 | `d-mmm-yy` | 45000.75 | `=45000.75` |
| 66 | `m/d/yyyy h:mm` | 45000.123456 | `=45000.123456` |
| 67 | `h:mm` | 0.5004 | `=0.5004` |
| 68 | `h:mm:ss` | 0.50001 | `=0.50001` |
| 69 | `[h]:mm` | 1.5004 | `=1.5004` |
| 70 | `mm:ss.0` | 0.000012 | `=0.000012` |
| 71 | `@` | 1.2345 | `=1.2345` |
| 72 | `0.00;-0.0` | 1.2345 | `=1.2345` |
| 73 | `0.00;-0.0` | -1.2345 | `=-1.2345` |
| 74 | `0.0;(0.00)` | -1.2345 | `=-1.2345` |
| 75 | `[>=100]0;0.00` | 150.6 | `=150.6` |
| 76 | `[>=100]0;0.00` | 50.678 | `=50.678` |
| 77 | `0.00;;` | -1.2345 | `=-1.2345` |
| 78 | `0.00;;` | 0 | `=0` |
| 79 | `;;;` | 1.2345 | `=1.2345` |
| 80 | `0.0;-0.0;"zero"` | 0.04 | `=0.04` |
| 81 | `0.0;-0.0;"zero"` | -0.04 | `=-0.04` |
| 82 | `0.0;-0.0;0.000` | 0 | `=0` |
| 83 | `0.0" kg"` | 1.2345 | `=1.2345` |
| 84 | `"abc"` | 1.2345 | `=1.2345` |
| 85 | `[Red]0.0` | 1.2345 | `=1.2345` |
| 86 | `[$-409]0.00` | 1.2345 | `=1.2345` |
| 87 | `$#,##0.00_);[Red]($#,##0.00)` | -1234.5678 | `=-1234.5678` |
| 88 | `_($* #,##0.00_);_($* (#,##0.00);_($* "-"??_);_(@_)` | 0.004 | `=0.004` |
| 89 | `_($* #,##0.00_);_($* (#,##0.00);_($* "-"??_);_(@_)` | 1234.5678 | `=1234.5678` |
| 90 | `#.##` | 1.2345 | `=1.2345` |
| 91 | `#.##` | 0.001 | `=0.001` |
| 92 | `0.??` | 1.5 | `=1.5` |
| 93 | `0.??` | 1.2345 | `=1.2345` |
| 94 | `#,###` | 0.4 | `=0.4` |
| 95 | `#` | 0.4 | `=0.4` |
| 96 | `?.?` | 0.04 | `=0.04` |

## The other Office.js cases

| Case | Steps | Question | Our provisional rule |
|---|---|---|---|
| F | A1 = 1.23456 and B1 `=1.23456`, both `0.000`; C1 `=A1*1000`, D1 `=B1*1000`; option on; read; F9; read; A1:B1 to `0.00`; read; F9; read; full; read; A1:B1 to `0.0000`; read; F9; read; full; read | Does a format change alone round a constant, and a formula's value? At once, at F9, or at a full calculation? After `0.00`, is B1 1.23 (calculated again from 1.23456) or 1.24 (its stored 1.235 rounded again)? After `0.0000`, is A1 still 1.24 or 1.235 (digits gone) and B1 1.2346 (calculated again)? Do C1 and D1 follow at once? | UNVERIFIED. A constant: rounded at the format change, and its digits are gone ("permanently"). A formula: calculated again and rounded, with its readers, at the format change |
| X | as F, in Manual mode from before the option is turned on | Is a constant rounded at once in Manual mode? Are formulas and readers stale until F9? | UNVERIFIED: constants at once; formula cells and readers at F9 |
| O | A1 = 1.23456 and B1 `=1.23456` in `0.00`, C1 `=A1*1000`, D1 `=B1*1000`; option on; read; option off; read; F9; read; full; read | With the option off again: A1 stays 1.23 (documented). Is B1 1.23456 again at once, at F9, or only at a full calculation? | A1 stays 1.23 ("the original underlying values cannot be restored"). B1: UNVERIFIED, at once |
| S | A1 `=SEQUENCE(4)/3` spilling over A1:A4 with formats `0.0`, `0.00`, General, `0%`; C1 `=SUM(A1#)`, C2 `=A2*3`, C3 `=SUM(A1:A4)` | Is each spilled cell rounded by its own format? Do `A1#` and `A1:A4` give the same sum? | UNVERIFIED: each cell by its own format; the same sum |
| K | `0.00` cells holding TRUE, the text `1.2345`, `=1/0`, nothing, `="1.2345"`, `=1.2345>1`; readers `=A1*1` and so on | Only numbers are rounded: is the text still `1.2345` to its reader? | only numbers |
| W | option on; into `0.00` cells: a number, text Excel reads as a number, the same through `formulas`, a paste of values from a General cell, a paste of the cell with its General format | Is every way of writing a number rounded? Does a paste that brings its own format keep the digits? | UNVERIFIED: rounded by the format the cell has after the write |
| G | General cells in a narrow column (`columnWidth = 40`), holding 123456789.123456 (constant and formula) and 0.1234567890123 | Does the option follow what a narrow General cell shows? | UNVERIFIED: no, General is unchanged |
| CF | A1 = 1.23456 and A2 `=1.23456`, General, with a conditional format (`=TRUE`) whose number format is `0.0` | Does the option follow the format the cell shows, or the cell's own? | UNVERIFIED: the cell's own format |

## By hand

- **SAVE.** `run("SAVE")` in a new workbook builds A1 = 1.23456 (`0.00`), B1 `=1.23456` (`0.00`), C1 `=A1*1000`,
  turns the option on and **leaves it on**. Save the workbook as .xlsx and unzip it. What do `xl/workbook.xml`'s
  `calcPr` (`fullPrecision`, any other attribute) and the `<v>` of A1, B1 and C1 hold? Provisional rule:
  `fullPrecision="0"`, and 1.23, 1.23 and 1230.
- **READ.** `python3 -I generate.py <dir>` writes two files that say the option is on while their cells hold
  more digits than their formats show, as a writer other than Excel can leave them: A1 = 1.23456 (`0.00`), B1
  `=A1*1000` cached 1234.56, C1 `=1.23456` (`0.00`) cached 1.23456, D1 `=C1*1000` cached 1234.56. Open each in a
  new Excel session and call `run("READ")`, which reads A1:D1, calculates (F9), reads, calculates fully and
  reads, and writes nothing else.
  - `pad-unrounded.xlsx` (sha256 `87af2ead7704e2d989fcf94315ae65935c0e5fc03af629fb7228953ea68b1443`).
  - `pad-unrounded-fullcalc.xlsx`, the same with `fullCalcOnLoad="1"` (sha256
    `b15af90cfe59add9c0bda05f6f02d1477de3690dfda8917a532597cef8968ff1`).

  Does Excel round the constant A1 when it loads the file, at a calculation, or never? Are the cached formula
  values kept until something calculates? Provisional rule: UNVERIFIED. The design assumes the worst case for a
  reader of files: Excel does not round a constant on load, so a file's constants are taken as they are. The
  same two files can go through excel-graph for Excel for the web's values after its full calculation.

## What the answers decide

SPEC-0002 and SPEC-0028 treat a cell's number format as invisible to calculation (T-5185 plans to skip the
recalculation of a format-only edit). With the option on, a format is an input of every numeric cell. The design
(research note, S7) needs to know: the rounding function (A, G, CF), when a constant loses its digits (E, F, W),
whether a format change is a recalculation (F, X), what turning the option off owes (O), whether a spilled cell
follows its own format (S), and what a file may hold (SAVE, READ).
