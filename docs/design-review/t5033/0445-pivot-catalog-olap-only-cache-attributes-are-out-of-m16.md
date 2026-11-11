# 0445 — Pivot catalog: OLAP-only cache attributes are out of M16; a table slicer's cache stays in (proposed)

- **Status:** proposed (2026-11-11, T-5033). It needs an owner ruling on each of its six groups; until then the 194
  rows keep T-5033 as their owner and stay in scope.
- **Applies:** decision 0274 item 4 ("Ruled out now: OLAP, the Data Model and Power Pivot; external and
  consolidation sources") to catalog rows its `RULES` do not match. It adds no scope and removes none that 0274 did
  not already rule on, except where group 4 says so.
- **Context:**
  - T-4992 gave 194 in-scope rows of eval/catalog/pivot-features.tsv to T-5033: they looked out of scope, and no
    rule matched them. 149 are in family `cache` (stage PV1), 40 in `grouping` (PV3) and 5 in `slicers-timelines`
    (PV5). On main b68c735d5 the pivot session measured 236 PV1 rows with no oracle cell; these 149 are among them.
    While they are in scope, M16's exit row counts each as remaining.
  - Decision 0438 asks, of anything a pivot's definition holds, whether it is an authored input (a shared document
    must keep it) or derived (rebuilt). Each group below says which.
  - **What the standard says.** ISO/IEC 29500-1 section 18.10.1, as the Open XML SDK reference reproduces it
    (learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.spreadsheet, the classes `StringItem`,
    `CacheField`, `Tuples`, `Tuple`, `MemberPropertyIndex`, `MemberPropertiesMap`; fetched 2026-11-11):
    - an item's `b`: "whether this value contains bold formatting on the OLAP server. This attribute applies to
      OLAP-based PivotTables only." `i`, `un` and `st` read alike. `bc` and `fc`: "the background color for this
      value that was provided by the OLAP server. This attribute applies to OLAP-based PivotTables only." `in`:
      "the index to the OLAP serverformat element where the format string for this entry is stored."
    - `cp`: "the number of member property values." `x` (Member Property Index): "an array of indexes to cached
      member property values." `mpMap`: "a mapping to cached member properties." `cacheField@mappingCount`: "the
      number of property mappings for this field." `cacheField@propertyName`: "the name of the property if this
      field is an OLAP property field."
    - `tpls`: "Represents members for the OLAP sheet data entry, also known as a tuple." `tpl`: "Represents an
      OLAP sheet data entry member."
    - an item's `c`: "Specifies the caption for the this item." The text does not say OLAP.
    - `cacheField@sqlType`: "This attribute stores an ODBC data type and applies to ODBC data sources only."
    - `x15:tableSlicerCache` ([MS-XLSX] 2.6.97, in the same reference): "properties of the slicer cache specific to
      table slicer items", with `tableId` "the associated Table" and `column` its column.
  - **What the files on disk hold** (docs/research/pivot-olap-attributes-2026-11-11/, with the script). A scan of
    every workbook under eval/ and crates/, the two fetched pivot corpora included (`pivot-corpus-lo`,
    `pivot-corpus-closedxml`): 97 workbooks with a pivot cache, 128 caches,
    of which 125 have a worksheet source, 1 an external one, 1 a consolidation and 1 a scenario. None of the 128
    carries any attribute or element of groups 1 to 5. Of 12 slicer caches, 3 are a table's.
    - The scan's limit: no cache here is an OLAP cache, so it shows absence where the standard predicts absence and
      has no positive control. The item caption `c` rests on it most: 0 of 125 worksheet caches carry one.
    - Where a caption a user gives an item of a worksheet-source pivot is kept: the table definition's
      `pivotField/items/item@n` (catalog row PV-1081, in scope, owner T-4915), not the cache.
- **Decision (proposed), by group.** "Out" is the catalog scope `out (decision 0445)`.

  | # | Rows | What | Proposed | Authored or derived |
  |---|---|---|---|---|
  | 1 | 84 | OLAP server formatting of an item: `b`, `bc`, `fc`, `i`, `un`, `st`, `in` on `s`, `n`, `e`, `m` in shared items, group items and records | out: the standard says OLAP only | derived: the server supplies them at a refresh |
  | 2 | 46 | Member properties: `cp` on an item; an item's child `x` and `x@v`; `cacheField/mpMap`, `mpMap@v`, `cacheField@mappingCount`, `cacheField@propertyName` | out: OLAP member properties | derived |
  | 3 | 40 | Tuples: `tpls`, `tpls@c`, `tpl`, `tpl@fld`, `tpl@item` under an item | out: OLAP sheet data entry | authored, on an OLAP pivot only |
  | 4 | 18 | The item caption `c` on `b`, `d`, `e`, `m`, `n`, `s` | out, on two grounds that are not the standard's words: no worksheet cache on disk carries it, and a user's caption is kept in `item@n` | derived for an OLAP cache; nothing to author elsewhere |
  | 5 | 1 | `cacheField@sqlType` | round-trip only, as `cacheField@uniqueList` is: an ODBC source's metadata, and external sources are out | derived: the driver reports it |
  | 6 | 5 | `x15:tableSlicerCache@tableId`, `@column`, `@sortOrder`, `@customListSort`, `@crossFilter` | **stay in scope**, stage PV5, owner T-5031 (the slicer-cache probes). A slicer on a Table is not OLAP, a quarter of the slicer caches on disk are one, Excel for the web shows them, and it is the same Insert > Slicer card | authored: which table and column it filters, and how it sorts |

- **Questions for the owner**, one a group, yes or no; the proposal is "yes" to each:
  1. Groups 1 to 3 (170 rows) are out, as OLAP.
  2. Group 4 (18 rows) is out although the standard does not say OLAP of `c`. If the corpus ever shows a
     non-OLAP cache that carries `c`, the rows come back by a new decision.
  3. Group 5 is round-trip only.
  4. Group 6 stays in M16's slicer stage.
- **Consequences**, once ruled:
  - `RULES` in tools/src/rustsheet_tools/pivot_catalog.py gains the patterns for groups 1 to 5, citing this record
    (its scope texts name decision 0274 today, so the function that writes them takes the decision per rule), and
    the catalog is regenerated with its other columns unchanged. The pivot session applies it in T-5033's change
    (agreed 2026-11-11).
  - The five rows of group 6 are re-pointed to T-5031. `scripts/tasks.py validate` fails on a done owner, so the
    re-pointing and T-5033's `done` are one change.
  - M16's exit row loses up to 189 remaining rows, 149 of them from PV1's 236 with no oracle cell.
  - Nothing changes in the engine. An OLAP cache is still imported and exported as it is today.
  - The scan is docs/research/pivot-olap-attributes-2026-11-11/scan.py (counts only). Run over a corpus that
    holds OLAP workbooks it gains the positive control it lacks here, and it is the watch question 2 asks for.
