"""Counts the probes of Design 13's traversal as its text describes the structure (spec lines 2370-2374):
a per-sheet list of formula positions by column, sorted by row; per column, the next candidate not
yet evaluated at or after a row (union-find with path compression); a rectangle visits the list's
columns that meet it.

Document: W columns by H rows of formula cells. Row 1 holds leaves over constants. The cell at
(r, c), r >= 2, reads one rectangle: the whole of row r-1, columns 1..W (`=SUM($A{r-1}:$<W>{r-1})`).
F = W*H candidates, R = W*(H-1) rectangles walked. A probe is one visit of a list column by a
rectangle (its binary search for the first row at or after the top, then one successor find).
"""

import bisect
import math
import sys


def run(width, height):
    columns = list(range(1, width + 1))  # the list's columns, sorted
    rows = {c: list(range(1, height + 1)) for c in columns}
    parent = {c: list(range(height + 1)) for c in columns}  # successor forest per column
    evaluated = set()
    probes = 0
    finds = 0

    def find(column, index):
        nonlocal finds
        forest = parent[column]
        root = index
        while forest[root] != root:
            root = forest[root]
            finds += 1
        while forest[index] != root:
            forest[index], index = root, forest[index]
        return root

    def footprint(cell):
        row, _ = cell
        return [] if row == 1 else [(row - 1, 1, row - 1, width)]

    def next_unevaluated(rect, cursor):
        """The next candidate of `rect` not evaluated, in placement order from `cursor`."""
        nonlocal probes
        top, left, bottom, right = rect
        start = bisect.bisect_left(columns, max(left, cursor))
        for column in columns[start:]:
            if column > right:
                break
            probes += 1
            index = find(column, bisect.bisect_left(rows[column], top))
            if index < height and rows[column][index] <= bottom:
                return (rows[column][index], column)
        return None

    def evaluate(cell):
        row, column = cell
        evaluated.add(cell)
        index = row - 1
        parent[column][index] = index + 1  # passed once from now on

    for row in range(1, height + 1):
        for column in columns:
            root = (row, column)
            if root in evaluated:
                continue
            stack = [[root, 1]]
            while stack:
                cell, cursor = stack[-1]
                pushed = False
                for rect in footprint(cell):
                    found = next_unevaluated(rect, cursor)
                    if found is not None:
                        stack[-1][1] = found[1]
                        stack.append([found, 1])
                        pushed = True
                        break
                if not pushed:
                    stack.pop()
                    evaluate(cell)
    formulas = width * height
    rectangles = width * (height - 1)
    claimed = formulas * 4 + rectangles * math.log2(formulas)  # F*alpha(F) + R*log F, alpha <= 4
    return formulas, rectangles, probes, finds, claimed


if __name__ == "__main__":
    height = 20
    print("W H F R probes finds claimed(F*4+R*log2F) probes/claimed probes/F")
    for width in (50, 100, 200, 400, 800):
        f, r, probes, finds, claimed = run(width, height)
        print(
            width,
            height,
            f,
            r,
            probes,
            finds,
            round(claimed),
            round(probes / claimed, 2),
            round(probes / f, 1),
        )
    sys.exit(0)
