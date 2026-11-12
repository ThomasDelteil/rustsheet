# Design desk: the project lead's rulings of 2026-11-12, and what still needs him

*poc-design-cc, 2026-11-12, 08:46Z. The page of 01:26Z asked for four decisions, listed the probe runs, and
listed the confirmations. The lead ruled on all three at 07:20Z and on precision as displayed at 07:22Z. This
page says where each ruling is recorded and what is left.*

## Still open: one question

| Question | Default if no answer | Where |
|---|---|---|
| Transition formula evaluation: the note recommended a notice on open until it is implemented, and "the confirmations stand" covers it. Two minutes later the ruling on precision as displayed was "no notice on open". Is it no notice for both? | no notice for either; decision 0463 is written that way and says so | decision 0463, item 3 |

## A. The four decisions: ruled "take the recommendations"

| # | Decision | Recorded as | State |
|---|---|---|---|
| 1, 2 | Workbook locales beyond en-US are milestone M17; how a locale gets its Excel evidence | decision 0443, M17 in docs/milestones.md | CR requested (branch `agent/design/T-4808-locales-m17`) |
| 3 | The pivot catalog's 194 parked rows: 189 leave M16's exit measure, 5 table-slicer rows stay | decision 0445 | CR requested; the pivot lane applies the catalog rules once it is on main |
| 4 | Precision as displayed: honoured, low priority, no notice on open; a shared document is never rewritten, its readers round; turning the option off brings the typed digits back (the stated difference from Excel) | decision 0462, M11 | CR requested (branch `agent/design/lead-rulings-2026-11-12`) |
| 4 | Transition formula evaluation: honoured from files, a user cannot turn it on | decision 0463, M11 | the same CR |

## B. Probe runs: Excel for the web only

Ruled: "there is no Excel desktop, so only Excel for the web probes get filled." Recorded in decision 0464 as
decision 0266's rule 2 applied to requests a person runs.

| Request | What it is | State |
|---|---|---|
| `t5145-recalc` | Script Lab: Manual mode, precedents and dependents; three files opened by hand | sent. Its two desktop-only points (M10's keys, M9's "Recalculate workbook before saving") already keep an UNVERIFIED rule; nothing waits for them |
| `precision-as-displayed` | four generated files through the oracle tool, then one Script Lab script of 16 cases | its follow-ups are approved (CR-321944537); it goes to the oracle folder when they are on main |
| `transition-evaluation` | one generated file through the oracle tool | built; goes after the precision request |
| `malformed-records` | six generated files through the oracle tool | built |
| `circular-references` | by hand: ten cases reading the warning and what marks the cells | being checked against what Excel for the web shows; goes with SPEC-0002 Design 14 |
| `xls-name-as-cell` | was written for desktop Excel for Windows | being rewritten: the .xls is opened in Excel for the web; what only Compatibility Mode shows becomes a rule on Microsoft's documentation |
| `workbook-locale` | one Script Lab script, run once per regional format and display language | goes with SPEC-0046, now that M17 is ruled |

## C. The confirmations: ruled "the confirmations stand"

All recorded in decision 0464, as statements: calculation modes (11), precedents and dependents (7), what the
read index covers (7), footprint rules and the dependency-first pass (9), styled blanks (1), workbook locales
(6). The questions on precision as displayed and on transition formula evaluation are in decisions 0462 and
0463.
