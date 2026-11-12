# Design desk: what needs the project lead, in one place

*poc-design-cc, 2026-11-12, 01:26Z. Every question below was already sent with its change or draft; this page only
gathers them, in the order of what they hold up. Each has a recommended answer where it was asked. Where the
recommendation stands, nothing needs doing: the designs are written to it.*

## A. Four decisions that hold up other work

| # | Decision | Recommended | What waits for it | Where |
|---|---|---|---|---|
| 1 | Workbook locales beyond en-US are milestone M17, with the five locales the engine ships (en-GB, de-DE, fr-FR, es-ES, it-IT); interim status "callable, marked unverified"; one locale first | yes; de-DE first | 14 tasks (T-4809 to T-4822), the locale spec SPEC-0046 and its probe; `make progress-report`, which refuses the run while docs/milestones.md has no M17 heading | `t4808/decision-draft.md` (4 questions) |
| 2 | How a locale gets Excel evidence: one Excel for the web site per locale, which only the lead can create, or the oracle service taking a locale per request | whichever exists first; one small probe before any site is made (does a site's language follow its creation?) | every probe of M17 | the same, question 2 |
| 3 | The pivot catalog's 194 parked rows: 189 are OLAP or ODBC only and leave M16's exit measure; 5 rows of a table slicer's cache stay | yes | the pivot lane's M16 exit list | `t5033/README.md` (4 questions) |
| 4 | Precision as displayed, and transition formula evaluation: in scope, and in which milestone? Both are Excel options that old workbooks carry and the engine ignores (wrong values on those workbooks today) | yes for both, each after its probe answers; honour transition evaluation in files without letting a user turn it on | the engine lane's two items (Taskei P559132978, P559132976) | `precision-as-displayed/research-note.md`, `transition-evaluation/research-note.md` |

## B. Probe runs that only a person with Excel can do

Each is ready or will be within a day; none needs more than Excel for the web unless it says so.

| Request | What it is | Effort for the person | State |
|---|---|---|---|
| `t5145-recalc` | Script Lab: Manual mode (17 cases), precedents and dependents (2 runs); three files opened by hand (M9, M10) | about 25 script runs in new workbooks | sent (fork branch); cases M17 to M20 and three P cases were added since, on main |
| `circular-references` | by hand: ten cases, each a few formulas, reading the warning, the status bar and the Error Checking menu | about 30 minutes | with SPEC-0002 Design 14, which goes to review when T-5145 has landed |
| `precision-as-displayed` | four generated files through the oracle tool (`rustsheet-oracle recalc`), then a Script Lab script of 16 cases | one command per file; 16 script runs | in review; it goes to the oracle folder when its follow-ups land |
| `transition-evaluation` | one generated file through the oracle tool | one command | built; goes after the precision request |
| `malformed-records` | six generated files through the oracle tool, returning the workbooks Excel writes back | one command per file | built |
| `xls-name-as-cell` | by hand in desktop Excel for Windows: an .xls with ten names such as `TAX2024` and ten formulas, saved as .xlsx; what Excel does to the names and their uses | about 20 minutes; the one request here that needs desktop Excel | in review (Taskei P560018445) |
| `workbook-locale` | one Script Lab script, run once per regional format and display language | per locale | held until decision 1 |

## C. Confirmations: the recommendation stands unless you object

| Topic | Questions | The ones most worth a look | Where |
|---|---|---|---|
| Calculation modes (SPEC-0002 Design 9, on main) | 11 | a read of a stale cell in OnDemand mode fails rather than returning a stale value; a save while cells are owed fails; a structural edit in Manual mode calculates nothing until F9 | the description of CR-321724568 |
| Precedents and dependents (Design 10, on main) | 7 | the answer is the written relation, with the formulas it cannot speak for named; two caps, each a refusal | the description of CR-321762904 |
| What the read index covers (Design 11, approved) | 7 | a whole-column range and a union stay "always dirty" for now; a workbook with a circular reference still recalculates fully | `t5145/section-4-coverage.md` |
| Footprint rules and the dependency-first pass (Designs 12 and 13) | 9 | a dynamic array that reads formula cells (a FILTER or SORT over a computed column) still recalculates fully at every edit in this slice; a workbook whose formulas read very wide ranges of formulas keeps today's pass (refused, not charged: the draft's two reviewers ruled differently, and the refusal is recommended); after typing a formula or inserting a row in a workbook whose totals sit above their data, the next recalculation is a full one; Design 13 changes which cell draws which RAND value | `t5145/section-5-footprint-rules-and-dependency-first.md` |
| A styled blank cell changes no work (SPEC-0028 section 12, on main) | 1 | the used range keeps counting formatted blanks, as Excel's does | `t5194/cover.md` |
| Workbook-locale semantics (SPEC-0046, draft) | 6 | locale and language belong to the document, not to the viewer: every collaborator computes under the document's | `t4810/cover.md` |
| Precision as displayed | 3 more | constants in a shared document; turning the option on is not undoable, as in Excel | `precision-as-displayed/research-note.md` |
| Transition formula evaluation | 2 more | a notice on open until it is implemented | `transition-evaluation/research-note.md` |
