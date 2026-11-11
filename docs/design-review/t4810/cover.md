# T-4810: SPEC-0046, workbook-locale semantics — draft for a design read

*poc-design-cc, 2026-11-11. The spec is `SPEC-0046-workbook-locale-semantics.md` beside this file (451 lines; branch
`agent/design/T-4810-workbook-locale-semantics`, 52a1f2248). It is cut as a CR after T-4808's ruling (decision 0443),
because the locales in scope are that decision's. Nothing in it claims to be evidence of Excel: every rule names
the probe that will evidence it.*

## In one page

- **Why.** The engine ships six locales and five formula languages, and only en-US has Excel evidence. Nothing says
  what a locale should change, so each function decided for itself. Reading the code against the inventory found
  twelve places that disagree with each other or with what Microsoft documents (the spec's "Problem" table). One is
  a defect whatever Excel does: a formula with an array constant, printed under a comma-decimal locale, does not
  parse back.
- **The inventory.** 52 items in seven groups: typed input, text coercion, displayed text, formula language and
  syntax, format codes written in the locale, locale-aware functions, and what must not change. Each has a probe
  shape (five shapes; three exist in the harness today), what the engine does today, and its owner task.
- **The model.** Excel has two settings, the regional format and the display language, and neither is in the file.
  Here both belong to the **document**, because both feed evaluation (`VALUE("1,5")`, `TRUE&""`) and replicas must
  compute the same values. That is Google Sheets' model, not Excel's, where each co-author calculates under their
  own settings. What belongs to the **viewer** is only the language in which formula text is shown and typed, which
  never reaches a value.
- **One finding that changes the harness.** Microsoft documents that writing `Range.values` does not parse a
  locale-shaped string ("stored as text instead of as dates"), and the harness's typed entries are a `values`
  write. Under another locale they must go through `formulasLocal`, after a canary shows that this is typing.
- **Excel's data, not CLDR's.** The locale tables are generated from the probes, as T-4508 did for `[$-407]` tags
  when CLDR's month names turned out not to be Excel's.

## Open questions for the project lead

1. Locale and language belong to the document: every collaborator computes under the document's, not under their
   own. (yes/no; mine: yes. It is the only choice under which replicas converge.)
2. Two settings, with the language defaulting to the locale's; a per-viewer formula language is a later step that
   the first slice must not prevent. (yes/no; mine: yes.)
3. No silent fallback: `de-AT` is refused with the list of supported locales, not computed as `de-DE`. (yes/no;
   mine: yes. A host that wants the mapping writes it knowingly.)
4. Excel's own override of the decimal and thousands separators (Options > Advanced) is not modelled. (yes/no; mine:
   yes; five locales would become five times every separator choice to probe.)
5. The engine keeps what it does today for a field until that field's probe is filled; nothing is "fixed" from
   documentation or memory. (yes/no; mine: yes, directive 1.)
6. What Graph cannot read (R1C1 letters, the localized format code, default sheet and caption names) goes into one
   Script Lab request that a person runs, as the calculation-mode probes do. (yes/no; mine: yes.)

## Questions for the owning lanes

- **Oracle and harness** (T-4809, T-4811; Design 5): can an oracle target's language be set apart from its
  regional format (U2)? Is one suite manifest per locale over a shared case directory (`root`, `oracle_dir`)
  acceptable, and is `formula_local` scored apart from values the right shape?
- **Engine** (T-4817 to T-4821; Designs 3 and 4): is `FormulaSyntax` as locale data, read by lexer, parser and
  printer only, the right single definition? Is a generated text table per locale, beside the `[$-…]` tables,
  where you want the data?
- **Model** (Design 7): a `language` key beside `locale` in the document's metadata, and the refusal in `compat.rs`
  ending locale by locale as each becomes verified.
- **UI**: the first slice shows and accepts formulas in the document's language; is a per-viewer formula language
  wanted for the first release?
