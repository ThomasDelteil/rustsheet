# Which pivot caches carry the attributes decision 0445 rules on (2026-11-11)

For T-5033. The question: of the 194 catalog rows that T-4992 parked as "probably OLAP only", does any occur in a
cache that is not an OLAP cache? If one does, it is not OLAP only, whatever the standard says.

## Method

`scan.py` (standard library only) opens every xlsx and xlsm under the roots it is given, parses each
`xl/pivotCache/pivotCacheDefinition*.xml` and its records part, and counts by kind of cache:
- on an item (`s`, `n`, `e`, `m`, `b`, `d`): the attributes `c`, `cp`, `b`, `bc`, `fc`, `i`, `un`, `st`, `in`, and
  the children `x` and `tpls`;
- on a cache field: `mappingCount`, `propertyName`, `sqlType`, and the child `mpMap`;
- slicer caches, and which are a table's (`x15:tableSlicerCache`).

A cache is counted as OLAP when its definition has `cacheHierarchies`, and otherwise by its `cacheSource` type. It
prints counts and at most two file names a line, never cell content; `--no-names` leaves the names out.

    python3 -I docs/research/pivot-olap-attributes-2026-11-11/scan.py eval crates

## Result (main dfa450198, with `pivot-corpus-lo` and `pivot-corpus-closedxml` fetched)

    workbooks with a pivot cache: 97; caches: 128; by kind: {'worksheet': 125, 'external': 1, 'consolidation': 1, 'scenario': 1}
    slicers: {'slicer caches': 12, 'of a table (x15:tableSlicerCache)': 3}

No further line is printed: none of the 128 caches carries any of the attributes or elements.

Without the two fetched corpora the same scan sees 14 workbooks and 22 caches, all with a worksheet source, and
finds none either.

## What it shows and what it does not

- It shows absence in 125 worksheet-source caches saved by Excel, which is where the standard predicts absence.
- It has no positive control: no workbook on disk has an OLAP cache, so the scan never finds what it looks for. A
  corpus that holds OLAP workbooks would give one; the corpus nightly's store is the place.
- The item caption `c` is the one attribute the standard does not call OLAP only. For it this scan is the only
  evidence, with the fact that a user's caption for an item of a worksheet-source pivot is stored elsewhere
  (`pivotField/items/item@n`).
- 3 of 12 slicer caches are a table's: table slicers are real in the files we have.
