# Draft for T-4808: workbook locales beyond en-US are milestone M17 (needs the project lead's ruling)

*poc-design-cc, 2026-11-11. A decision brief. It becomes `docs/decisions/NNNN` and the M17 section of
docs/milestones.md once the four questions at the end are answered. It is cut after CR-321667084 (decision 0437) is
on main, because both edit the phase table.*

## Why now

- The owner asked for this decision on 2026-11-07 (T-4808). Fourteen tasks wait behind it: the oracle sites
  (T-4809), the locale-semantics spec (T-4810), the harness dimension (T-4811), five probe tasks (T-4812 to T-4816),
  five engine tasks (T-4817 to T-4821) and the API's interim status (T-4822).
- docs/status.md, "Needs owner": docs/milestones.md has no M17 heading, and until it has one `make progress-report`
  refuses the run, so the dashboard cannot be regenerated. T-5018 (the guard against the next such PR) waits too.
- The collaborative sheet's documents have a `locale` key (BCP 47; `crates/rustsheet-sheet-ops/src/schema.rs`). Today
  it is always `en-US`, and an older document that carries another locale is refused as `UnsupportedSemantics`
  (`compat.rs`). So a German or French document cannot be opened until that locale has evidence behind it.

## What is true today (read in the repository)

- The engine ships six workbook locales (`locale/locales.bin`: en, en-GB, de, es, fr, it) and five formula
  languages (`language/language.bin`: en, it, fr, de, es), inherited from IronCalc. `get_supported_locales()`
  (engine, and the Python binding) lists all six with no caveat.
- Only en-US has Excel evidence. The parity bar is en-US Excel (CLAUDE.md). The eval harness runs every case under
  `"en"`. The oracle calculates in its SharePoint site's regional settings, and its canary pins the site to en-US
  (docs/architecture/oracle.md, "Locale is a site setting").
- Format-code locale tags (`[$-40C]`) are another axis and are covered already (T-4508).
- xlsx stores formulas and number formats locale-neutral, so a workbook's locale changes typed input, text
  coercion, displayed text, the formula language as typed and shown, format codes written in the locale, and a few
  locale-aware functions (T-4810's inventory). It does not change stored formulas.

## The proposed decision

1. **M17, "Workbook locales beyond en-US", is a milestone.** Its locales are the five the engine ships beyond
   en-US: en-GB, de-DE, fr-FR, es-ES and it-IT. Another locale needs a new decision.
2. **Where it sits.** Since decision 0437 no phase gates work. M17 gets a row of its own in the phase table, after
   Phase 3, and is active when the lead directs it.
3. **Evidence.** One Excel for the web site per locale, each pinned by its own canary whose en-US answer differs
   (a day-first `DATEVALUE`, a decimal-comma `VALUE`), as T-4809 describes. Creating the five sites and granting the
   oracle app access is an owner step. A sidecar records the locale it was computed under, and a fill refuses a
   case whose locale has no site.
4. **Exit criteria**, measured per locale as M10's are: every item of T-4810's inventory has oracle-backed probe
   cases under that locale, and the locale's probe row is at visible parity (not visibly off), with accepted
   deviations left out. The milestone exits when all five rows do. en-US's rows do not move.
5. **Until a locale's row exits, the locale stays callable and is marked unverified.** The API docs, the Python
   docstrings and the README name en-US as the only Excel-verified locale and language, and the API can be asked:
   `get_supported_locales()` keeps its list, and a second call says which of them are verified (en-US only, until
   a row exits). Nothing is refused and nothing warns at run time. Refusing would break callers that pass a locale
   today, and the collaborative sheet's documents that already carry one.

## What it costs

- Five oracle sites to create and keep (an owner step), and one fill per locale per probe case.
- T-4810's spec, then about one probe task and one slice of each of five engine tasks per locale.
- No change to en-US behaviour, to stored files, or to any gate: a new locale's rows are new rows.

## Questions for the project lead

1. Are the five locales the engine ships (en-GB, de-DE, fr-FR, es-ES, it-IT) the scope of M17, no more? (yes/no;
   mine: yes.)
2. Evidence by one Excel for the web site per locale, which you create (five sites, the oracle app granted on
   each)? Or can the oracle service that is being set up take a locale per request? (sites/service; mine: whichever
   exists first; the decision names "an oracle target per locale" and T-4809 builds it. Before any site is made,
   one small probe should run (SPEC-0046, unknown U2): what decides the language of a site's formulas. The oracle
   lane cautions that a site's language may be fixed when the site is created, so a site made in English might
   never serve German function names. That caution is not verified here.)
3. Interim status: callable and marked unverified, with an API query for it, rather than refused? (yes/no; mine:
   yes.)
4. With UI-first as the direction: is one locale wanted for the collaborative sheet's first release, and which?
   (mine: de-DE first, since its separators and date order differ most from en-US; the others follow the same
   probes.)
