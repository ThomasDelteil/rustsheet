//! Probes for the second read of SPEC-0002 Designs 12 and 13 (engine: main at /tmp/wt-design-probe).
//! Each probe prints what today's engine does; none claims the designs are implemented.
use rustsheet_engine::{clock::Clock, types::Cell, Model};

fn model() -> Model {
    Model::new_empty_with_clock(
        "review",
        "en",
        "UTC",
        "en",
        Clock::fixed_unix_millis(1_667_906_008_578).unwrap(),
    )
    .unwrap()
}

fn w(model: &mut Model, row: i32, column: i32, text: &str) {
    model
        .set_user_input(0, row, column, text.to_owned())
        .unwrap();
}

fn v(model: &Model, address: &str) -> String {
    format!(
        "{:?}",
        model.get_cell_value_by_ref(&format!("Sheet1!{address}"))
    )
}

fn kind(model: &Model, row: i32, column: i32) -> String {
    match model.workbook.worksheet(0).unwrap().cell(row, column) {
        None => "none".to_owned(),
        Some(Cell::CellFormula { .. }) => "CellFormula".to_owned(),
        Some(Cell::ArrayFormula { kind, r, .. }) => format!("ArrayFormula({kind:?}, r={r:?})"),
        Some(Cell::SpillCell { a, .. }) => format!("SpillCell(a={a:?})"),
        Some(Cell::EmptyCell { .. }) => "EmptyCell".to_owned(),
        Some(_) => "literal".to_owned(),
    }
}

fn state(model: &Model) -> String {
    let outcome = model.last_recalc_outcome().unwrap();
    format!(
        "restricted_pass_allowed={} spent={} ending={:?}",
        model.restricted_pass_allowed(),
        outcome.spent,
        outcome.ending
    )
}

/// A: a formula the walk does not model, read by a candidate root, reads a range of candidates
/// that are not evaluated yet. Z1 at column 26 of row 1 is after A1 and before nothing of Q.
fn nested_reader(z_row: i32, depth: Option<usize>) {
    let mut m = model();
    if let Some(depth) = depth {
        m.set_max_evaluation_depth(depth);
    }
    for row in 1..=100 {
        w(&mut m, row, 16, &row.to_string()); // P
        w(&mut m, row, 17, &format!("=P{row}*2")); // Q
    }
    w(&mut m, 1, 25, "45000"); // Y1
    // The candidate root: in A1 when Z is in row 1, else one row below Z, after every Q.
    let root_row = if z_row == 1 { 1 } else { z_row + 1 };
    w(&mut m, root_row, 1, &format!("=Z{z_row}+1"));
    w(&mut m, z_row, 26, "=COUNTIF(Q1:Q100,\">0\")+YEAR(Y1)");
    m.evaluate();
    println!(
        "A nested reader: root A{root_row}, Z{z_row}, depth limit {depth:?}: root={} {}",
        v(&m, &format!("A{root_row}")),
        state(&m)
    );
}

/// B: a reader of a circular cell's reader keeps its value; SUM of one range over such cells.
fn circular_readers() {
    let mut m = model();
    w(&mut m, 1, 1, "10"); // A1
    w(&mut m, 1, 2, "=A1"); // B1
    w(&mut m, 1, 3, "=A1+1"); // C1
    w(&mut m, 2, 3, "=A1+2");
    w(&mut m, 3, 3, "=A1+3");
    w(&mut m, 5, 5, "5"); // E5
    w(&mut m, 5, 4, "=SUM(C1:C3)+E5"); // D5
    m.evaluate();
    println!(
        "B no cycle: C1:C3={},{},{} D5={}",
        v(&m, "C1"),
        v(&m, "C2"),
        v(&m, "C3"),
        v(&m, "D5")
    );
    w(&mut m, 1, 1, "=B1"); // A1 <-> B1
    m.evaluate();
    println!(
        "B cycle A1<->B1: A1={} B1={} C1:C3={},{},{} D5={}",
        v(&m, "A1"),
        v(&m, "B1"),
        v(&m, "C1"),
        v(&m, "C2"),
        v(&m, "C3"),
        v(&m, "D5")
    );
    w(&mut m, 5, 5, "7");
    m.evaluate();
    println!(
        "B after E5=7: C1:C3={},{},{} E5={} D5={} (a sum of the stored cells plus E5 would be 43)",
        v(&m, "C1"),
        v(&m, "C2"),
        v(&m, "C3"),
        v(&m, "E5"),
        v(&m, "D5")
    );
}

/// C: a literal write changes which cells are spill cells while no cell becomes or stops being a
/// dynamic anchor; and a name op makes a plain formula cell an anchor in the pass that runs it.
fn candidacy_moves() {
    let mut m = model();
    w(&mut m, 1, 26, "2");
    w(&mut m, 1, 1, "=SEQUENCE(Z1)");
    m.evaluate();
    println!(
        "C Z1=2: A1={} A2={} A3={} A5={}",
        kind(&m, 1, 1),
        kind(&m, 2, 1),
        kind(&m, 3, 1),
        kind(&m, 5, 1)
    );
    w(&mut m, 1, 26, "5");
    m.evaluate();
    println!(
        "C Z1=5: A1={} A2={} A3={} A5={}",
        kind(&m, 1, 1),
        kind(&m, 2, 1),
        kind(&m, 3, 1),
        kind(&m, 5, 1)
    );

    let mut m = model();
    w(&mut m, 1, 3, "1");
    w(&mut m, 2, 3, "2");
    w(&mut m, 3, 3, "3");
    m.new_defined_name("nm", None, "Sheet1!$C$1").unwrap();
    w(&mut m, 1, 2, "=nm*2");
    m.evaluate();
    println!(
        "C name over one cell: B1 kind={} B2 kind={} B1={}",
        kind(&m, 1, 2),
        kind(&m, 2, 2),
        v(&m, "B1")
    );
    m.update_defined_name("nm", None, "nm", None, "Sheet1!$C$1:$C$3")
        .unwrap();
    println!(
        "C name redefined, before the pass: B1 kind={} B2 kind={}",
        kind(&m, 1, 2),
        kind(&m, 2, 2)
    );
    m.evaluate();
    println!(
        "C name redefined, after the pass: B1 kind={} B2 kind={} B1:B3={},{},{}",
        kind(&m, 1, 2),
        kind(&m, 2, 2),
        v(&m, "B1"),
        v(&m, "B2"),
        v(&m, "B3")
    );
}

/// D: the same cells typed in two orders: `anchor_order` is history.
fn anchor_history(first: &str) {
    let mut m = model();
    let p = |m: &mut Model| w(m, 1, 1, "=SEQUENCE(2)"); // A1, spills A1:A2
    let q = |m: &mut Model| w(m, 1, 3, "=A1:A2*2"); // C1, spills C1:C2
    if first == "A1" {
        p(&mut m);
        m.evaluate();
        q(&mut m);
    } else {
        q(&mut m);
        m.evaluate();
        p(&mut m);
    }
    m.evaluate();
    m.evaluate();
    println!(
        "D typed {first} first: A1:A2={},{} C1:C2={},{} kinds A1={} C1={} {}",
        v(&m, "A1"),
        v(&m, "A2"),
        v(&m, "C1"),
        v(&m, "C2"),
        kind(&m, 1, 1),
        kind(&m, 1, 3),
        state(&m)
    );
}

/// E: a root whose transitive static footprint holds a static cycle that no value closes. Today's
/// on-demand evaluation is what Design 13's fallback prescribes for such a root.
fn static_cycle_read(depth: Option<usize>) {
    let mut m = model();
    if let Some(depth) = depth {
        m.set_max_evaluation_depth(depth);
    }
    w(&mut m, 1, 26, "FALSE"); // Z1
    w(&mut m, 1, 25, "FALSE"); // Y1
    w(&mut m, 1, 1, "=IF(Z1,B5,0)"); // A1
    w(&mut m, 5, 2, "=IF(Y1,C5,1)"); // B5
    w(&mut m, 5, 3, "=IF(Y1,2,B5)"); // C5
    m.evaluate();
    println!(
        "E depth {depth:?} Z1=FALSE: A1={} B5={} C5={} {}",
        v(&m, "A1"),
        v(&m, "B5"),
        v(&m, "C5"),
        state(&m)
    );
    w(&mut m, 1, 26, "TRUE");
    m.evaluate();
    println!(
        "E depth {depth:?} Z1=TRUE:  A1={} B5={} C5={} {}",
        v(&m, "A1"),
        v(&m, "B5"),
        v(&m, "C5"),
        state(&m)
    );
}

/// F: an anchor that reads, inside its static footprint, an anchor ranked after it.
fn anchor_reads_later_anchor() {
    let mut m = model();
    w(&mut m, 1, 5, "1"); // E1
    w(&mut m, 2, 5, "2"); // E2
    w(&mut m, 1, 26, "0"); // Z1
    w(&mut m, 1, 1, "=IF(Z1=1,C1:C2,E1:E2)"); // A1
    w(&mut m, 1, 3, "=E1:E2*2"); // C1
    m.evaluate();
    m.evaluate();
    println!(
        "F Z1=0: A1:A2={},{} kinds A1={} C1={} {}",
        v(&m, "A1"),
        v(&m, "A2"),
        kind(&m, 1, 1),
        kind(&m, 1, 3),
        state(&m)
    );
    w(&mut m, 1, 26, "1");
    m.evaluate();
    println!(
        "F Z1=1: A1:A2={},{} {}",
        v(&m, "A1"),
        v(&m, "A2"),
        state(&m)
    );
    m.evaluate();
    println!(
        "F Z1=1 again: A1:A2={},{} {}",
        v(&m, "A1"),
        v(&m, "A2"),
        state(&m)
    );
}

/// G: a candidate that an unread IF branch names, which reads a formula off the allowlist
/// placed before it.
fn moved_candidate(z1: &str) {
    let mut m = model();
    w(&mut m, 1, 26, z1); // Z1
    w(&mut m, 2, 25, "45000"); // Y2
    w(&mut m, 1, 1, "=IF(Z1,C3,0)"); // A1
    w(&mut m, 2, 2, "=YEAR(Y2)"); // B2
    w(&mut m, 3, 3, "=B2+1"); // C3
    m.evaluate();
    println!(
        "G Z1={z1}: A1={} B2={} C3={} {}",
        v(&m, "A1"),
        v(&m, "B2"),
        v(&m, "C3"),
        state(&m)
    );
}

/// H: the listed leaf functions read a cell through an argument.
fn leaf_arguments() {
    let mut m = model();
    w(&mut m, 5, 3, "=IF(TRUE,\"UTC\",\"\")"); // C5
    w(&mut m, 1, 2, "=TODAY(C5)"); // B1
    w(&mut m, 2, 2, "=NOW(C5)"); // B2
    w(&mut m, 6, 3, "=1+2"); // C6
    w(&mut m, 3, 2, "=RANDBETWEEN(C6,C6)"); // B3
    m.evaluate();
    println!(
        "H C5=UTC: B1={} B2={} B3={} {}",
        v(&m, "B1"),
        v(&m, "B2"),
        v(&m, "B3"),
        state(&m)
    );
    w(&mut m, 5, 3, "=IF(TRUE,\"No/Such\",\"\")");
    m.evaluate();
    println!("H C5=No/Such: B1={} B2={}", v(&m, "B1"), v(&m, "B2"));
}

/// I: the total above its data.
fn total_on_top(total_row: i32) {
    let mut m = model();
    for row in 1..=10_000 {
        w(&mut m, row, 1, &row.to_string());
        w(&mut m, row, 2, &format!("=A{row}*2"));
    }
    w(&mut m, total_row, 3, "=SUM(B1:B10000)");
    m.evaluate();
    println!(
        "I total in C{total_row}: {} {}",
        v(&m, &format!("C{total_row}")),
        state(&m)
    );
}

/// J: Design 12's footprints, as the engine reads them.
fn footprints() {
    let mut m = model();
    for row in 1..=4 {
        w(&mut m, row, 1, &row.to_string()); // A1:A4 = 1..4
        w(&mut m, row, 2, &(row * 10).to_string()); // B1:B4 = 10..40
    }
    for column in 3..=4 {
        w(&mut m, 1, column, &column.to_string()); // C1, D1 = 3, 4
    }
    for row in 1..=10 {
        w(&mut m, row, 6, &(row * 100).to_string()); // F1:F10 = 100..1000
    }
    for column in 8..=11 {
        w(&mut m, 1, column, &(column * 1000).to_string()); // H1:K1
    }
    w(&mut m, 20, 1, "=SUMIF(A1:A3,\">0\",F1:F10)"); // F1:F3 only
    w(&mut m, 21, 1, "=SUMIF(A1:A3,\">0\",F1)"); // F1:F3
    w(&mut m, 22, 1, "=LOOKUP(99,A1:A4,H1)"); // one-cell result: reads right, K1
    w(&mut m, 23, 1, "=LOOKUP(99,A1:D2,F1:F2)"); // 2 by 4: along the first row, 4 long; F4
    w(&mut m, 24, 1, "=LOOKUP(99,A1:B4,H1)"); // 4 by 2: along the first column, 4 long; K1
    w(&mut m, 25, 1, "=LOOKUP(99,A1:A4,F1:G2)"); // 2-D result
    w(&mut m, 26, 1, "=LOOKUP(99,A1:B4)"); // array form: last column, B4
    m.evaluate();
    for row in 20..=26 {
        println!("J A{row}={}", v(&m, &format!("A{row}")));
    }
}

/// L: an anchor reads a plain formula cell that reads where a later anchor spills. Today the
/// plain cell runs inside the anchor, so the restart names the anchor as the reader.
fn plain_cell_between_anchors() {
    let mut m = model();
    w(&mut m, 1, 5, "10"); // E1
    w(&mut m, 2, 5, "20"); // E2
    w(&mut m, 1, 1, "=C3+E1:E2"); // A1, an anchor with a modelled formula, spills A1:A2
    w(&mut m, 3, 3, "=G2+1"); // C3, a plain formula cell
    w(&mut m, 1, 7, "=SEQUENCE(2)"); // G1, an anchor placed after A1, spills G1:G2
    m.evaluate();
    println!(
        "L first evaluation: A1:A2={},{} C3={} G2={} kinds A1={} C3={} G1={} G2={} {}",
        v(&m, "A1"), v(&m, "A2"), v(&m, "C3"), v(&m, "G2"),
        kind(&m, 1, 1), kind(&m, 3, 3), kind(&m, 1, 7), kind(&m, 2, 7), state(&m)
    );
    m.evaluate();
    println!("L second evaluation: A1:A2={},{} C3={} {}", v(&m, "A1"), v(&m, "A2"), v(&m, "C3"), state(&m));
}

/// M: an anchor whose static footprint names a plain cell that it reads only on one branch; the
/// plain cell reads a second anchor's spill, and the second anchor reads the first one's spill.
fn unread_branch_between_anchors(z1: &str) {
    let mut m = model();
    w(&mut m, 1, 26, z1); // Z1
    w(&mut m, 1, 5, "10"); // E1
    w(&mut m, 2, 5, "20"); // E2
    w(&mut m, 1, 1, "=IF(Z1,C3,0)+E1:E2"); // A1, spills A1:A2
    w(&mut m, 3, 3, "=G2+1"); // C3
    w(&mut m, 1, 7, "=A2+SEQUENCE(2)"); // G1, spills G1:G2, reads A1's spill
    for round in 1..=2 {
        m.evaluate();
        println!(
            "M Z1={z1} evaluation {round}: A1:A2={},{} G1:G2={},{} C3={} kinds A1={} G1={} {}",
            v(&m, "A1"), v(&m, "A2"), v(&m, "G1"), v(&m, "G2"), v(&m, "C3"),
            kind(&m, 1, 1), kind(&m, 1, 7), state(&m)
        );
    }
}

fn main() {
    nested_reader(1, None);
    nested_reader(200, None);
    nested_reader(1, Some(2));
    nested_reader(200, Some(2));
    circular_readers();
    candidacy_moves();
    anchor_history("A1");
    anchor_history("C1");
    static_cycle_read(None);
    static_cycle_read(Some(1));
    anchor_reads_later_anchor();
    moved_candidate("FALSE");
    moved_candidate("TRUE");
    leaf_arguments();
    total_on_top(1);
    total_on_top(10_001);
    footprints();
    plain_cell_between_anchors();
    unread_branch_between_anchors("FALSE");
    unread_branch_between_anchors("TRUE");
}
