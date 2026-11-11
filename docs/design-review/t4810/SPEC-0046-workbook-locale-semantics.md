# SPEC-0046 — Workbook-locale semantics: what a regional setting changes, the model, and the harness's dimension

- **Status:** draft (2026-11-11, T-4810). Nothing in "Excel behavior to match" is evidence yet: every rule there is
  marked with the probe shape that will evidence it, and stays UNVERIFIED until a probe task (T-4812 to T-4816)
  fills it.
- **Milestone / lane:** M17 (decision 0443) / research (this spec), then harness (T-4809, T-4811), research
  (T-4812 to T-4816), engine (T-4817 to T-4821) and api (T-4822).
- **Tasks:** T-4810 (this spec); the fourteen tasks under Rollout.

## Problem

- The engine ships six workbook locales and five formula languages, inherited from IronCalc:
  `locale/locales.bin` holds `en`, `en-GB`, `de`, `es`, `fr`, `it`, and `language/language.bin` holds `en`, `it`,
  `fr`, `de`, `es`. Only en-US has Excel evidence. The eval harness runs every case under `"en"`
  (`WorkbookOptions::default`), and the oracle's canary pins its site to en-US (docs/architecture/oracle.md,
  "Locale is a site setting").
- Nothing says what a locale is supposed to change. Reading the code against the inventory below shows what that
  costs. Each line is a fact of the tree, not of Excel:

  | What | Today | Where |
  |---|---|---|
  | Argument separator | derived from the decimal separator (`,` if it is `.`, else `;`); the locale's own `list` symbol is never read | `expressions/parser/mod.rs` `get_argument_separator_token` |
  | Array constants | the parser reads rows split by `\` in a comma-decimal locale; the printer writes them split by `/`, so a printed formula does not parse back | `parser/mod.rs` `get_column_separator_token`; `parser/stringify.rs` `ArrayKind` |
  | Typed booleans | English only in every language | `Model::set_user_input` |
  | Booleans and errors in criteria | English only | `expressions/utils/mod.rs` `boolean_text`; `token.rs` `english_criterion_error` |
  | NUMBERVALUE's omitted separators | `.` and `,` in every locale | `functions/text/string_format.rs` |
  | DOLLAR, FIXED | `$`, `,`, `.` and parentheses in every locale | `functions/text/string_format.rs` |
  | Format codes | the format lexer takes no locale: `,` and `.` are fixed, and the letters are en-US's | `formatter/lexer.rs` |
  | Built-in date format 14, `[$-F800]`, `[$-F400]` | fixed en-US patterns | `number_format.rs`; `formatter/locale_tag.rs` |
  | Month and day names | CLDR's, which differ from Excel's own (de `Mär` against Excel's `Mrz`) | `locale/mod.rs`; `formatter/date_names.rs` |
  | Collation and case mapping | no locale | `functions/util/collation.rs`; UPPER and LOWER in `functions/text/common.rs`, PROPER in `string_format.rs` |
  | An English function name in another language | `#NAME?` | `test/test_language.rs` |
  | Locale ids | exact match on the engine's ids, so `de-DE` and `en-US` are refused though the facade's spec says `"en-US"` | `locale/mod.rs` `get_locale`; SPEC-0001 `WorkbookOptions` |
  | Stored text | read under the workbook's locale and language: a replica's write (`Input::Stored`) and `Input::StoredFormula` parse a literal with `self.locale` and a formula with the active parser, so a stored `1.5` or `=SUM(1.5,2)` is not read as en-US in a de-DE workbook | `stored_input.rs`; `typed_input.rs`; `workbook/stored.rs` ("`None` when the workbook parses formulas in another locale or language") |
  | The collaborative gate | only a V1 or V2 document's locale is compared with `en-US`; a V3 document's is never read, and `EngineOptions` has no locale field (`deny_unknown_fields`), so the wasm engine is always built en-US | `rustsheet-sheet-ops/src/compat.rs` `legacy`, `evaluate`; `rustsheet-contracts/src/determinism.rs`; `rustsheet-wasm/src/engine.rs` |

- The collaborative sheet's documents have a `locale` key (BCP 47). It is always `en-US` today, and an older
  document that carries another locale is refused as `UnsupportedSemantics`
  (`crates/rustsheet-sheet-ops/src/schema.rs`, `compat.rs`). So a German or French document cannot be opened at
  all until its locale has evidence.
- Eleven tasks wait for this spec's inventory (T-4811 to T-4821, directly or through the probes).

## Goals / non-goals

Goals:
- One inventory of what a regional setting changes in Excel, each item with the probe shape that evidences it,
  what the engine does today and the task that owns it.
- A model of locale and language that a collaborative document can carry: every replica computes the same values.
- The eval harness's locale dimension: how a case names its locale, how the oracle computes under it, and how
  results are reported and gated, so that en-US rows never move.

Non-goals:
- A locale beyond the five of decision 0443 (en-GB, de-DE, fr-FR, es-ES, it-IT).
- Format-code locale tags (`[$-407]`, `[$-de-DE]`). They are a property of a format, not of the workbook, and are
  covered: 841 LCIDs and 935 tags with Excel-generated name tables (T-4508; docs/excel-semantics/number-formats.md,
  "Locale tags localize date names").
- Calendars other than Gregorian, right-to-left sheets, double-byte text functions, BAHTTEXT and PHONETIC under
  another locale, and spelling or proofing.
- Excel's own override of the two separators (File > Options > Advanced, "Use system separators"). See
  Alternatives 5.
- The UI language of the collaborative sheet's chrome. Only what reaches a cell, a formula or a displayed value
  is here.

## Excel behavior to match

### What Microsoft documents

Fetched 2026-11-11; the quotes are verbatim. Documentation is not evidence (directive 1): each statement below is
re-asked by a probe.

- **Excel has two settings, not one.** The system culture gives the number, date and currency conventions:
  `Application.cultureInfo` "Provides information based on current system culture settings", with a `name` such
  as "en-us", a `numberFormat` and a `datetimeFormat`
  (learn.microsoft.com/en-us/javascript/api/excel/excel.cultureinfo). The display language gives the formula
  language: `Range.formulasLocal` is "the formula in A1-style notation, in the user's language and
  number-formatting locale. For example, the English "=SUM(A1, 1.5)" formula would become "=SUMME(A1; 1,5)" in
  German" (learn.microsoft.com/en-us/javascript/api/excel/excel.range). That one sentence names both: the user's
  language (SUMME) and the number-formatting locale (`;` and `1,5`).
- **The separators can be overridden.** "By default, Microsoft Excel uses the system separators that are defined in
  the regional settings in Control Panel" (support.microsoft.com, "Change the character used to separate thousands
  or decimals"); `Application.decimalSeparator` and `thousandsSeparator` are "based on the local Excel settings",
  and `useSystemSeparators` says whether they are the system's
  (learn.microsoft.com/en-us/javascript/api/excel/excel.application).
- **Neither setting is in what the API reads back.** `valuesAsJson` is documented locale-independent: "Data
  returned from this API always aligns with the en-US locale". That an xlsx file stores formulas and number
  formats in their en-US form, and that `Range.formulas` and `numberFormat` read so, is our reading of the file
  format and of the `formulasLocal` and `numberFormatLocal` twins; no page fetched says it, and control G3 asks
  it.
- **A plain value write is not typing.** `Range.values`: "Locale-shaped strings (such as the date "19-8-2025" in
  nl-NL or fr-FR, format DD-MM-YYYY) are stored as text instead of as dates. To ensure dates are stored as dates,
  use a locale-aware API like `formulasLocal`". This matters for the harness: its typed entries are a `values`
  PATCH today (Design 5).
- **Functions that say they follow the locale.** NUMBERVALUE: "If the Decimal_separator and Group_separator
  arguments are not specified, separators from the current locale are used." DATEVALUE: "your computer's system
  date setting may cause the results of a DATEVALUE function to vary from this example". VALUE: "Text can be in any
  of the constant number, date, or time formats recognized by Microsoft Excel." TEXT: "The thousands separator is
  dependent on your regional settings." DOLLAR: "the currency symbol that is applied depends on your local language
  settings." (support.microsoft.com, each function's page.) No page fetched says that TEXT's format letters change
  with the language, which users of German Excel know they do (`TT.MM.JJJJ`); that is unknown U3.

### What other engines do

- **Google Sheets** keeps a locale per spreadsheet: changing it "changes the spreadsheet's default currency, date,
  and number formatting", and "Everyone working on it will see the changes, regardless of their location." The
  language is the account's: "Changing the locale doesn't change your language settings", and function names
  follow the display language unless "Always use English function names" is set (support.google.com/docs/answer/58515).
  *We take this split:* the locale belongs to the document, the formula language to the viewer.
- **HyperFormula** has one option per fact: `language`, `functionArgSeparator`, `decimalSeparator`,
  `thousandSeparator`, `dateFormats`, `timeFormats`, `currencySymbol`, `localeLang` for string comparison, and
  refuses a clash ("both decimalSeparator and thousandSeparator must be different from functionArgSeparator")
  (handsontable.com/docs/hyperformula/guide/i18n-features.html). *We take the explicit separators, as data per
  locale and not as host options* (Alternatives 5).
- **LibreOffice** has a locale setting that "influences settings for numbering, currency and units of measure",
  with "date acceptance patterns" per locale, and lets the default languages be "valid only for the current
  document" (help.libreoffice.org, Options > Language Settings > Languages). *We take the acceptance patterns:* what
  a locale accepts as a typed date is a list, not one format.
- **IronCalc**, this engine's ancestor, stores the locale in the workbook and treats the language as the viewer's
  (`model.rs`: "the timezone and the locale belong to the workbook while the language can be different for
  different users looking at the same workbook"). The model below keeps that and adds what evaluation needs.

### The inventory

Probe shapes (Design 5 says how the harness carries each):

| Shape | What is observed | Exists today |
|---|---|---|
| **V** | the value of a formula stored in the file in its en-US form, calculated under the locale | for an xlsx case, yes; a spec-built case writes its formulas through `Input::StoredFormula`, which reads them under the suite's locale today (Problem), so not before Rollout 3 |
| **T** | the value, and the number format Excel chose, of a string typed into a cell under the locale | as `{"entered": …}`, sent in a way that is not typing under another locale (U1) |
| **D** | the displayed text of a cell (`Range.text`) under the locale | yes, as `"text": […]` |
| **L** | a stored formula as the locale shows it (`formulasLocal`), and a formula typed in the locale read back in its stored form | no (T-4811) |
| **O** | an Office.js fact Graph cannot read: `cultureInfo`, `numberFormatLocal`, the separators in force | no; a Script Lab request, as `docs/research/t5145-recalc-probe` is |

"Decides" says which of Excel's two settings we believe decides the item: **R** the regional format, **L** the
display language, **?** unknown. Every entry of that column is a belief until its probe is filled.

**A. Typed input** (T-4817)

| Id | Item | Decides | Today | Shape |
|---|---|---|---|---|
| A1 | Decimal and group separators of a typed number (`1.234,5`, `1 234,5`) | R | the locale's symbols (`formatter/parse_formatted.rs`) | T |
| A2 | Date order and separators (`1/2/2001`, `1.2.2001`, `1-2`, two-digit years) | R | day-first when the locale's short pattern starts with `d`; `.` only if that pattern has it (`formatter/date_text.rs`) | T |
| A3 | Month and day names in a typed date (`1 März 2001`, `1 Mrz`) | ? | CLDR's names | T |
| A4 | Times: 24-hour, AM and PM words | R | `is_am_pm_word` takes no locale | T |
| A5 | Currency (`5 €`, `€ 5`, `$5`), and the format the cell gets | R | `$` and the locale's symbol (`cast.rs`) | T |
| A6 | Percent (`5%`, `5 %`) | R | | T |
| A7 | Typed booleans (`WAHR`, `TRUE`) | L | English only | T |
| A8 | Typed error literals (`#WERT!`, `#VALUE!`) | L | by language (`get_error_by_name`) | T |
| A9 | The number format Excel gives a typed entry (a typed date gets a date format: which?) | R | | T, from the fill's log |

**B. Text coercion** (T-4817)

| Id | Item | Decides | Today | Shape |
|---|---|---|---|---|
| B1 | VALUE of number, currency, percent, date and time text | R | the locale (`functions/text/common.rs`) | V |
| B2 | DATEVALUE and TIMEVALUE | R | the locale (`functions/date_and_time.rs`) | V |
| B3 | Arithmetic on text (`"1,5"+0`, `"1.5"+0`) | R | the locale; `$`, `€` and the locale's symbol (`cast.rs`) | V |
| B4 | Criteria (`">1,5"`, `"=1.2.2001"`, `"WAHR"`, `"#NV"`) | R and L | numbers and dates by locale; booleans and errors English only | V |
| B5 | Text to boolean (`IF("WAHR",1,2)`) | L | English only | V |
| B6 | NUMBERVALUE's omitted separators | R | `.` and `,` always | V |
| B7 | Date text as an argument (DAYS, NETWORKDAYS, EDATE) | R | the locale | V |

**C. Displayed text** (T-4818)

| Id | Item | Decides | Today | Shape |
|---|---|---|---|---|
| C1 | General (`1234,5`) | R | the locale's decimal (`formatter/format.rs`) | D |
| C2 | Separators and group sizes of a number format (`#,##0.00`) | R | the locale's | D |
| C3 | The built-in date and time formats, which a file stores as an id with no code (14 to 22, 45 to 47) | R | fixed en-US patterns (`number_format.rs`) | D |
| C4 | `[$-F800]` (long date) and `[$-F400]` (time) | R | fixed en-US strings | D |
| C5 | Names of `mmm`, `mmmm`, `ddd`, `dddd` with no locale tag | ? | CLDR's names of the locale | D |
| C6 | The built-in currency and accounting formats (5 to 8, 37 to 44) | R | | D |
| C7 | AM/PM text | R | | D |
| C8 | Booleans and errors shown in a cell (`WAHR`, `#WERT!`) | L | by language (`cell.rs`); an error with no `Language` field (`#GETTING_DATA`) prints in English (`token.rs`) | D |
| C9 | A value turned to text in a formula (`1.5&""`, `TRUE&""`, `TEXT(1.5,"General")`) | R and L | | V |
| C10 | The code `m/d/yyyy` written out, as a custom format of the file or in `TEXT(x,"m/d/yyyy")` (C3 is the ids with no code) | R | the code as written | D, V |
| C11 | Which character the group separator is where it is a space (fr-FR: U+0020, U+00A0 or U+202F), in displayed text and in typed input | R | CLDR's | D, T |
| C12 | The percent formats 9 and 10 (a space before `%`, its place) | R | fixed en-US | D |

**D. Formula language and syntax** (T-4819)

| Id | Item | Decides | Today | Shape |
|---|---|---|---|---|
| D1 | Function names: the 508 names of `language.bin`, in four languages, against Excel's | L | `lookup_stored` (`functions/xlsx_names.rs`) | L |
| D2 | An English function name typed under another language | L | `#NAME?` | L |
| D3 | The argument separator, and the union operator, which is the same character | R (the system's list separator) | derived from the decimal separator | L |
| D4 | The decimal comma in a formula's literals | R | the lexer, from the locale | L |
| D5 | Array constants: the row and the column separator | R | parser and printer disagree (Problem) | L |
| D6 | Boolean and error literals in a formula (`WAHR`, `#NV`) | L | by language | L |
| D7 | Structured-reference keywords (`[#Kopfzeilen]`, `[#Daten]`, `[@]`) | L | | L |
| D8 | R1C1 letters (de `Z1S1`, fr `L1C1`), in the UI and in values: `ADDRESS(1,1,1,FALSE)`, `INDIRECT("Z1S1",FALSE)`, and INDIRECT of a localized structured-reference keyword | L | | O, and V for the three functions |
| D9 | FORMULATEXT. It returns formula text as a value, so it uses the document's settings and never a viewer's view (Design 1) | ? | English names with the locale's separators (`=SUM(1,123; 2)` under fr, `test/test_fn_formulatext.rs`) | V |
| D10 | The type arguments of CELL and INFO (`CELL("Zeile")`), and their results (`CELL("format")`'s codes, INFO's texts) | L | English only (`functions/information/mod.rs`) | V |
| D11 | `\` in a name where it is also a separator (docs/excel-semantics/references.md, UNVERIFIED) | R | a name character only when the decimal is `.` | L |
| D12 | The separator of an inline list in a data validation (`a;b;c` or `a,b,c`) | R | | O |

**E. Format codes written in the locale** (T-4820)

| Id | Item | Decides | Today | Shape |
|---|---|---|---|---|
| E1 | TEXT's date and time letters (de `TT.MM.JJJJ hh:mm`, fr `jj/mm/aaaa`) | ? (U3) | en-US letters only (`formatter/lexer.rs`) | V |
| E2 | The decimal and group characters inside a code (`TEXT(1234.5,"#.##0,00")`) | R | fixed `,` and `.` | V |
| E3 | Whether the en-US code still works under the locale (`TEXT(x,"yyyy")`) | ? | | V |
| E4 | Colour and condition words, and the name of General (`[Rot]`, `Standard`) | L | | V, O |
| E5 | The code a user sees and types for a cell's format (`numberFormatLocal`) | R and L | none: the facade speaks stored codes | O |

**F. Locale-aware functions** (T-4821)

| Id | Item | Decides | Today | Shape |
|---|---|---|---|---|
| F1 | DOLLAR: symbol, its place, separators, the negative form | R | fixed en-US | V |
| F2 | FIXED: separators | R | fixed en-US | V |
| F3 | UPPER, LOWER, PROPER on the locale's letters (`ß`) | ? | no locale | V |
| F4 | Text comparison and order: `"ä"<"b"`, SORT, MATCH with 1 and -1, `"ß"="ss"` | ? | no locale (`functions/util/collation.rs`) | V |
| F5 | The first day of the week where a function has a default (WEEKDAY, WEEKNUM) | ? | fixed | V |
| F6 | The default name of a new sheet (`Tabelle1`) and of a new table or pivot caption | L | sheet names by language (`new_empty.rs`); pivot captions fixed English (`pivot/layout/render.rs`) | O |
| F7 | The separators ARRAYTOTEXT and VALUETOTEXT write, in both formats | R | fixed en-US | V |

**G. What does not change** (controls; each must be probed to equal en-US)

| Id | Item | Shape |
|---|---|---|
| G1 | Arithmetic, rounding, error propagation, the date system and serials | V |
| G2 | A format with a locale tag (`[$-407]dddd`) shows that tag's names under any workbook locale | D |
| G3 | A stored formula's syntax (function names, separators, numeric literals) and a stored format code read the same under every locale (a file written under de-DE opens under en-US). Its string literals are not syntax: `TEXT(A1,"TT.MM.JJJJ")`, `VALUE("1,5")` and `SUMIF(A:A,">1,5")` keep their text, and their values follow the locale (groups B, E and F) | V |
| G4 | EXACT, FIND, SEARCH, SUBSTITUTE and the wildcards | V |
| G5 | Text functions that count bytes: what `LENB("é")` is under each of the five locales (we expect 1 in all five, none being a double-byte locale; not documented for them) | V |

### Unknowns that decide the design, not only a table entry

| Id | Question | If the answer differs from what Design assumes |
|---|---|---|
| U1 | Under a non-en-US target, does a `formulasLocal` PATCH of `1,5` or `1.2.2001` parse as typing does, and does a `values` PATCH not? | Shape T needs another route (Office.js) for every A item |
| U2 | Which setting decides the language of Graph's `formulasLocal` for the oracle's app-only session: the site's regional settings, the site's language, or a request header? | If the language cannot be set per target, every L item is an O probe |
| U3 | Do TEXT's letters follow the regional format or the display language? | E1 moves between the locale's table and the language's |
| U4 | Do untagged month and day names follow the regional format or the display language? | C5 and A3 move likewise |
| U5 | Is the argument separator the regional list separator, independent of the decimal separator? | If it is derived, `FormulaSyntax` (Design 3) loses a field |
| U6 | Does Excel for the web have the separator override at all? | If it does, the oracle targets must pin it off |

U3 and U4 ask which of the two settings decides an item, so they need a target where the two differ: a site with
de-DE regional settings and English as its language, if U2 shows that a target's language can be chosen. U5 needs
a locale whose list separator is not what its decimal separator implies; we believe none of the five in scope is
one (`,` for en-GB, `;` for the other four), so U5 is asked on a sixth site, or stays a belief at M17's exit and
the "Decides" column says so for the rows that rest on it.

## Design

### 1. The model: a document locale and language, and a viewer's formula view

A workbook has two settings that evaluation reads, and both belong to the document:

- **`locale`**, the regional format: separators, date order and patterns, currency, the list separator, and
  whatever U3 and U4 give it.
- **`language`**, the formula language: function, boolean and error names, and whatever U3 and U4 give it. It
  defaults to the locale's language (`de` for `de-DE`).

Both feed evaluation: `VALUE("1,5")`, `TEXT(x,"TT.MM.JJJJ")` and `TRUE&""` have values that depend on them. So in
a collaborative document they must be the document's, or two replicas compute different values from the same
operations and never converge. We believe Excel does not have this property (each co-author's Excel calculates
under its own settings); no page fetched says so, and nothing here rests on it. Google Sheets has one setting for
the spreadsheet, and that is the only choice a replicated engine has (Alternatives 1).

A third thing is the viewer's and never reaches a value:

- **the formula view**, the language and syntax in which one user reads and types formula text. A German user of
  an en-US document may read `=SUMME(A1; 1,5)`. Formulas are stored and replicated in their en-US form, so a view
  is a pure function from stored text to shown text and back. The first slice has one view, the document's own.
  A per-viewer view is a later step (Rollout 14), and nothing in the first slice may prevent it: no stored or
  replicated formula text is ever in a localized form.

What follows from "the document's":
- Changing `locale` or `language` is an edit of the document. It makes everything owed (SPEC-0002 Design 9): the
  next recalculation is a full one, and its change set lists every value that changed.
- A typed entry is parsed under the document's locale and language into a value or a stored formula, by the
  client that typed it. What is replicated is the value or the en-US formula, never the typed text. A replica
  applies `Input::Stored`. Today it and `Input::StoredFormula` read under the workbook's locale and language
  (`stored_input.rs`); they must read en-US whatever the workbook's settings. That change precedes every
  non-en-US suite and document (Rollout 3).
- **What a document stores is en-US, whatever its locale** (the model lane's condition). The collaborative
  document stores a cell as an input text or a formula text (`sheet_ops::grammar`), and that is canonical today
  only because every document is en-US. Under another locale the client canonicalizes before it plans the write,
  so the planner and the document's gate see en-US grammar only. The gate could not tell otherwise: it parses with
  the engine's lexer and would refuse a de-DE formula, but `1,5` typed as a German number and `1,5` meant as
  en-US text look the same to it.
  - A formula becomes its en-US text: function names, separators, literals.
  - A typed value becomes the canonical literal of what it parsed to, with the number format typing chose
    written as a format: `5 €` is the number 5 and a currency format, `1.2.2001` a serial and a date format. It
    does not become "what one would type in en-US": whether en-US typing reads some text back to the same value
    and format is a probe item (group A), not something to store by.
  - A forced text entry (`'1,5`) stays the text it is.
  - Text that the en-US stored reading would not return as the same text is stored forced: `1,234.5`, `TRUE` or
    `#N/A` typed as text under de-DE would otherwise come back as a number, a boolean or an error.
  - A string literal inside a formula is not canonicalized, and cannot be: `TEXT(A1,"TT.MM.JJJJ")`,
    `VALUE("1,5")` and `SUMIF(A:A,">1,5")` keep their text. So a change of locale rewrites no cell, and it does
    change what such cells compute; that is the full recalculation above.
  - The same holds for every formula a document stores outside a cell: a defined name's body, a conditional
    format's rule, a data validation's formula, a table's calculated column. Each is stored in its en-US form.
- Displayed text (`formatted`, `displayed`) follows the document's locale, so that a cell shows what
  `TEXT(cell, its format)` returns.
- The editor takes what the document's locale writes. In a document whose locale writes `1,5`, a typed value and
  a typed formula both use the decimal comma, and a typed formula uses that locale's argument separator: the
  four characters of `FormulaSyntax` (Design 3), which the facade gives the UI so that the formula bar,
  AutoComplete and the argument tooltip insert the same ones. What `1.5` typed into such a document becomes
  (text, a date, a number) is item A1's probe, not a choice of the UI's. The UI lane asked for this to be
  said, and wants no per-viewer formula language in the first release.

### 2. Identifiers

- A locale is named by a BCP 47 tag with a region: `en-US`, `en-GB`, `de-DE`, `fr-FR`, `es-ES`, `it-IT`. A
  language by its language subtag: `en`, `de`, `fr`, `es`, `it`.
- One function resolves a tag: an exact match, compared without case, or one of the engine's old ids as an alias
  (`en` is `en-US`, `de` is `de-DE`, `fr` is `fr-FR`, `es` is `es-ES`, `it` is `it-IT`). The aliases keep today's
  callers and tests working.
- There is no fallback to a related locale. `de-AT` is an error that lists the supported tags, not a silent
  `de-DE`: a locale with no evidence must not compute as another one unnoticed. A host that wants the mapping
  writes it.
- The facade, the bindings and the collaborative document all speak tags. SPEC-0001's `WorkbookOptions` comment
  (`"en-US"`) becomes true.

### 3. One definition of a formula's syntax

Today three places derive separators from the decimal separator, and two of them disagree (Problem). One value
replaces them:

```rust
/// How formula text is written under a locale. Inside the engine only the lexer, the parser and the printer read it.
/// The facade hands a copy to a UI (`Workbook::formula_syntax`), which reads it and cannot change it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormulaSyntax {
    pub decimal: char,        // 1,5
    pub argument: char,       // SUMME(1; 2), and the union operator
    pub array_column: char,   // between the items of a row of an array constant
    pub array_row: char,      // between rows
}
```

It is data of the locale, filled from the D3 to D5 probes, not computed. Two invariants, asserted by a test for every
locale's row: `decimal` differs from the other three, and `array_column` differs from `array_row`. The lexer's rule:
inside the braces of an array constant the two array separators win over name characters, so that `{WAHR.FALSCH}`
under a locale whose column separator is `.` is two items and not one name; outside braces a character that is both a
separator and a name character is read as today (inventory D11). The printer's separators are the same four, so it
stops writing `/` between rows, which collides with `#N/A` and `#DIV/0!`. The test also asserts that every formula of
the probe corpus printed under a locale parses back to the same tree under it, `=1.5+1` typed under a comma-decimal
locale among them (inventory D4). That property fails on main today for any array constant under a comma-decimal
locale, whatever Excel's separators turn out to be.

### 4. Excel's data, not CLDR's

T-4508's lesson: for `[$-…]` tags the engine first used CLDR's month and day names, and Excel's differ. It now
uses tables generated from Excel (`locale/excel_names.tsv` and its siblings, by `rustsheet-excel-locales`). The
workbook locale goes the same way:

- A locale's facts are a row set of one text table, generated from the M17 probes' sidecars and checked in, as
  the `[$-…]` tables are. The fields: the four characters of `FormulaSyntax`, the group separator and sizes, the
  date order, the accepted typed-date patterns, the patterns of the built-in formats a file stores as an id,
  the long date and time patterns, AM and PM, the currency symbol, its place and the negative forms, and the
  format-code letters.
- Month and day names come from the tables T-4508 already generated, keyed by the locale's LCID. A workbook
  locale and a `[$-407]` tag must not disagree about German month names, so there is one source.
- A language's facts are a second table: function names, boolean and error names, the structured-reference
  keywords, and the names of General and the colours.
- `locales.bin` (CLDR, through IronCalc's generator) stays the source only of a field no probe covers. Each such
  field is listed in docs/excel-semantics/ as UNVERIFIED with its owner, and the list shrinks to nothing as
  the locale's row exits.
- Until a field's probe is filled the engine keeps what it does today. A field changes only with its evidence, in
  the engine task that owns its inventory group.

### 5. The harness's locale dimension (T-4811, T-4809)

- **A suite has one locale.** `suite.toml` gains `locale = "de-DE"` and `language = "de"`, both optional, en-US
  and the locale's language by default. Every existing suite is unchanged, so no en-US row, baseline or gate
  moves.
- **Shared cases, one suite per locale.** The inventory's cases live once, in `eval/suites/locale-inventory/cases`.
  Five manifests (`eval/suites/locale-inventory-de-DE/suite.toml`, and so on) point at them with the existing
  `root` key, name their locale, and keep their sidecars apart with the existing `oracle_dir` key. The same
  typed string then has five recorded answers, which is the probe: `1.234,5` is a number under de-DE and text
  under en-GB. A case that makes sense under one locale only lives in that locale's own suite.
- **The sidecar says what it was computed under.** `source` gains `locale` and `language`. Its field names are
  also the remote oracle's wire format, pinned by a test: the two fields and `formula_local` are added on purpose,
  and an old client drops them. The harness refuses a
  sidecar whose locale is not its suite's, and `rustsheet-oracle fill` refuses a case whose locale has no
  target (T-4809). A sidecar without the field is en-US, which every existing sidecar is.
- **The oracle has a target per locale** (T-4809), each pinned by its own canary: two cells whose en-US answers
  differ, a day-first `DATEVALUE` difference and a decimal-comma `VALUE`, as the en-US canary pins en-US today
  (`tools/src/rustsheet_tools/oracle/canary.py`). The canary's expected values are en-US by construction and are
  checked with no locale, so they become a table per target, or every other fill is refused. The canary also
  records `formulasLocal` of one stored formula, which pins the language.
- **What decides a target's language is not known (U2)**, and nothing in the repository evidences it: the backend
  sends no locale of any kind, and only the regional format is known to follow the site. The probe is small: one
  stored formula, `=SUM(A1,1.5)`, read as `formulasLocal` on a site with de-DE regional settings, with and without
  an `Accept-Language` header, and on a site created in German. It runs before the five sites are made: if the
  language follows the language a site was created with, a site made in English cannot become the de-DE target
  (the oracle lane's caution; a site's creation language is said not to be changeable, which is unverified here).
- **A shared case has one sha256 across locales**, and three places key on it. Each must carry the target, or one
  locale's answer is served to another (the oracle lane's read):
  - the fill's cache, `eval/.cache/oracle/<backend>/<input sha256>`;
  - the remote oracle's requests, named by the workbook's sha256 (decision 0442): the target becomes a key prefix,
    and until then a worker refuses a request that names one;
  - the canary check above.
- **Typed entries.** `{"entered": …}` is sent as a `values` PATCH. Microsoft documents that this does not parse a
  locale-shaped string (above). Under a non-en-US target the backend sends typed entries as `formulasLocal`,
  after the target's canary has shown three things (U1): that the string is parsed as typing parses it, that it
  is not stored as text, and that it is not parsed as en-US. `1.234` shows all three under de-DE: 1234 if typed,
  1.234 if read as en-US, text if not parsed, with `ISNUMBER` and `ISTEXT` beside it. A typed date needs the
  number format Excel chose, which the PATCH's response carries and the fill logs. What a refused typed entry
  answers on this route is evidenced again (decision 0179 knows it for a `values` PATCH only). The en-US path
  does not change.
- **Shape L.** A spec case lists cells under `"formula_local": ["Sheet!A1", …]`. The fill reads each one's
  `formulasLocal` after the recalculation into the sidecar's `formula_local`, and the harness compares it with the
  engine's localized formula text. It is scored apart from values, as displayed text is (`text_expect.rs`), and
  reported as its own block, so that a wrong function name does not hide behind a right value.
- **Reports and gates.** The report has one row per locale suite, and M17's exit row is computed per locale
  (decision 0443 item 4). A locale's suite gates only itself: its baseline is its own, and it joins `make check`
  when its first sidecars land, as any new suite does.
- **The engine adapter** builds the workbook with the suite's locale and language (`harness_options` in
  `crates/rustsheet-eval/src/engine.rs`).

### 6. The public API (T-4822 for the marking; the rest with the engine tasks)

```rust
// crates/rustsheet (the facade)
pub struct WorkbookOptions {
    pub locale: String,    // a BCP 47 tag, "en-US" by default; an old id is accepted as an alias
    pub language: String,  // "" is the locale's language; else a language subtag
    /* … */
}
impl Workbook {
    pub fn locale(&self) -> &str;     // the canonical tag
    pub fn language(&self) -> &str;
    pub fn formula_syntax(&self) -> FormulaSyntax;   // what the editor inserts and accepts (Design 1)
}
impl Txn<'_> {
    /// Changes the document's locale. Everything is owed afterwards.
    pub fn set_locale(&mut self, tag: &str) -> Result<(), EditError>;
    pub fn set_language(&mut self, tag: &str) -> Result<(), EditError>;
}
pub fn supported_locales() -> &'static [&'static str];
pub fn supported_languages() -> &'static [&'static str];
/// A pair of settings the engine can compute under, and whether Excel evidence stands behind that pair.
pub struct SettingsInfo { pub locale: &'static str, pub language: &'static str, pub verified: bool }
pub fn supported_settings() -> &'static [SettingsInfo];
```

- Evidence is per pair, because an oracle target computes one locale with one language (Design 5). `verified` is
  true for en-US with `en` only, until a pair's M17 row exits (the proposed decision 0443, item 5). A locale with
  another language than its own (de-DE with `en`) is verified only if a target computed that pair. The list is
  generated from the milestone's exit rows, not hand-kept. Nothing is refused and nothing warns at run time.
- `get_supported_locales()` (engine and Python) keeps returning ids; the Python binding gains the verified list and
  a docstring that names en-US as the only verified locale.
- A formula is read and written in three forms, each already present or named here: stored (`Input::Stored` and
  `StoredFormula`: en-US once Rollout 3 has changed their reading), typed under the document's settings
  (`Input::Formula`, `Input::Parse`), and shown under the document's settings (`cell().formula`). A per-viewer view
  adds a parameter to the last two (Rollout 14), not a new form.
- xlsx import and export are unchanged: the file carries neither setting, the host gives both at load, and a
  save writes en-US formulas and codes. The collaborative document carries them in its `locale` key and a new
  `language` key.

### 7. The collaborative document

- The `locale` key keeps its meaning. A `language` key joins it, absent meaning the locale's language. It is
  additive: while it is absent the seed and the contract's vectors do not change (the model lane's read).
- Today only a V1 or V2 document's locale is compared with `en-US` (`compat.rs`, `legacy`); a V3 document's is
  never read, and the wasm engine is built from `EngineOptions`, which has no locale field. So a current client
  would read a de-DE document as readable and compute it as en-US, and replicas would diverge. Before any client
  may write another locale:
  - a locale other than `en-US`, or a present `language` key, raises the document's reader floor. A required
    capability alone is a write gate: `evaluate` keeps a document with an unknown required capability readable,
    read-only (`UnsupportedRequiredCapabilities`), and such a client would still show values computed as en-US.
    A newer `minimumReader` is unreadable in `compat.rs`, and I-21's floor contract opens the document
    read-only with every cell unloaded (`apps/collab-sheet/decisions/I-21-floor.md`, D-I21-01). The capability
    is added too, as the write gate. Which floor applies, and a test of an older reader opening such a document,
    are part of this step;
  - `compat.rs`'s `evaluate` checks both keys against `supported_settings()` and accepts a pair when it is
    verified. Until then the refusal is right: opening the document would change what it computes;
  - `EngineOptions` and `CalcIdentity` (`rustsheet-contracts`) gain both fields, with a bump of `semantics`, and
    the wasm engine builds its workbook from them;
  - both keys are bounded in length, as V2 bounds `locale` (directive 5), and `get_locale` no longer echoes an
    unbounded input in its error.
- A change of the document's locale is one operation on the document's metadata. Every replica applies it and
  recalculates fully. The metadata is outside every undo root, as the calculation settings are, so Ctrl+Z does
  not take a locale change back. That is intended: it is a setting of the document, and changing it back is
  another change. Stored inputs are en-US (Design 1), so a change of locale rewrites no cell.

## Alternatives considered

1. **The viewer's locale decides evaluation**, as in Excel. Two replicas then hold different values for
   `=VALUE("1,5")` from the same operations. A replicated engine cannot do it. Excel can because it does not
   promise that co-authors see the same calculated values.
2. **One setting, the locale, that implies the language.** Excel has two, and so have Google Sheets and IronCalc.
   A German user who keeps English function names is common. The default (the locale's language) gives the
   one-setting behaviour to whoever wants it.
3. **CLDR as the data source.** It is what `locales.bin` holds, and T-4508 measured the difference for names.
   Excel's conventions are Windows's and Excel's own, not CLDR's.
4. **A copy of every inventory case per locale.** Five copies of each case to keep in step. The shared directory
   with one manifest per locale uses two keys the manifest already has.
5. **Separators as host options**, as HyperFormula has and as Excel's override is. Every replica would have to
   hold the same options, so they would be document settings with more states to probe: five locales become five
   times every separator choice. If a customer needs the override, it is a decision of its own, after U6.
6. **Refuse the unverified locales until they exit.** The proposed decision 0443, item 5, rejects it: callers pass
   a locale today.
7. **Store the locale in the xlsx**, as a custom property. Excel would ignore it, and a file opened in Excel
   computes under that user's settings whatever it says. The host that loads the file knows its user.

## Test plan

- **The inventory suites** (Design 5): every item of groups A to G has at least one case in
  `locale-inventory`, of the shape its row names, and the five locale suites fill them. Group G's cases must
  equal their en-US sidecars cell for cell.
- **Round trip of formula text:** for every formula of the probe corpus and each locale, print under the locale,
  parse under it, and compare trees; and print in en-US, which must equal the stored text.
- **One source for names:** a test that the workbook locale de-DE and the tag `[$-407]` give the same month and
  day names for every month and day.
- **Tag resolution:** each tag and each alias resolves; `de-AT`, `DE_de` and the empty string are errors that list
  the supported tags.
- **Determinism:** two workbooks built from the same operations under the same locale and language hold the
  same values; `scripts/determinism.py` runs the locale suites natively and in wasm.
- **en-US is unmoved:** `rustsheet-eval check` over every existing suite reports no change with the locale
  dimension in place and no locale suite present.
- **The marking** (T-4822): `supported_locales()` reports exactly en-US as verified while no M17 row has exited,
  and the test reads the same exit rows the report computes.

## Rollout

Each step is an existing task, S or M. A step relies only on the designs named.

1. **T-4808** (the proposed decision 0443): M17 exists. Needs the project lead's four answers.
2. **T-4822**: the marking of Design 6 (`supported_settings()` and the docs), as its criteria say. It may start at
   once.
3. Not yet a task (engine; filed when T-4808 is ruled): the stored reading is en-US. `Input::Stored` and
   `Input::StoredFormula` read en-US whatever the workbook's settings; tags with aliases (Design 2), and the
   keyed load path compares canonical tags (`KeyLexer::reads` compares `"en"` today, so a tag `en-US` would
   silently lose it); `Txn::set_locale` and `set_language` make everything owed (`Model::set_language` does not
   recalculate today). It precedes every non-en-US suite and document.
4. **T-4809**: an oracle target per locale with its canary (Design 5). It settles U1, U2 and U6 before anything
   else is filled, because they decide which shapes Graph can serve.
5. **T-4811**: the suite's `locale`, the sidecar's fields, shape L, the report rows (Design 5).
6. A Script Lab request for the O items (D8, D12, E4, E5, F6, and whatever U1 and U2 push there), written with
   T-4812.
7. **T-4812 to T-4816**: the inventory's cases, written once (T-4812, de-DE, writes the shared cases; the others
   fill them and add what their locale alone needs), each recording its answers in docs/excel-semantics/ and
   generating its locale's rows of the tables of Design 4.
8. **T-4819**: `FormulaSyntax` (Design 3) and the language table. It fixes the array-constant round trip under
   every locale.
9. **T-4817**: typed input and coercion, groups A and B.
10. **T-4818**: displayed text, group C.
11. **T-4820**: format codes written in the locale, group E.
12. **T-4821**: the locale-aware functions, group F.
13. Not yet a task (the model and contracts lanes): the collaborative gate of Design 7, before any client may
    write a locale other than en-US. It is filed when the collaborative sheet asks for its first locale.
14. Not yet a task: a per-viewer formula view (Design 1). It is filed when the collaborative sheet asks for it.

The engine tasks of steps 8 to 12 each depend on all five probe tasks today. If the collaborative sheet needs one
locale first (the proposed decision 0443, question 4), their `depends_on` is cut to that locale's probe task, and
the other locales follow through the same steps.
