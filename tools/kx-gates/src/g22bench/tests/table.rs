//! Tests of G22 bench's table: the rate, the verdicts, the run's pass and the lines it prints.

use kx_mod_voice::score::Wrong;

use super::super::table::{Miss, Row, Rule, Timed, Verdict, lines, pass};

/// The rule the table tests judge by.
const RULE: Rule = Rule {
    rtf: 0.5,
    weak_rtf: 1.0,
    wrong: 1,
};

/// A row that measured `total_ms` on a 6 s clip with `words` wrong.
fn row(weak: bool, total_ms: u32, words: u16) -> Row {
    let timed = Timed {
        start_ms: 700,
        load_ms: 500,
        first_use_ms: 900,
        first_ms: Some(total_ms),
        total_ms,
        peak_mb: 812,
        wrong: Some(Wrong { words, total: 15 }),
    };
    Row {
        lang: "en".into(),
        model: "Parakeet TDT 0.6b v3".into(),
        weak,
        cores: if weak { 2 } else { 8 },
        cap: weak.then_some(375),
        clip_ms: 6_000,
        got: Ok(timed),
    }
}

/// `row`'s shape with `miss` in place of its numbers.
fn missed(weak: bool, miss: Miss) -> Row {
    Row {
        got: Err(miss),
        ..row(weak, 1_000, 0)
    }
}

#[test]
fn the_rate_is_the_warm_wait_over_the_clip() {
    assert_eq!(row(false, 1_500, 0).rtf(), Some(0.25));
    let mut silent = row(false, 1_500, 0);
    silent.clip_ms = 0;
    assert_eq!(silent.rtf(), None);
    let failed = missed(false, Miss::Fault("the voice worker ended".into()));
    assert_eq!(failed.rtf(), None);
}

#[test]
fn the_pass_rule_takes_the_modes_own_limit() {
    assert_eq!(row(false, 3_000, 1).verdict(&RULE), Verdict::Pass);
    assert_eq!(row(false, 3_600, 0).verdict(&RULE), Verdict::Slow);
    assert_eq!(row(true, 3_600, 0).verdict(&RULE), Verdict::Pass, "weak");
    assert_eq!(row(true, 6_600, 0).verdict(&RULE), Verdict::Slow);
    assert_eq!(row(false, 1_000, 2).verdict(&RULE), Verdict::Wrong);
    let mut uncounted = row(false, 1_000, 0);
    if let Ok(t) = &mut uncounted.got {
        t.wrong = None;
    }
    assert_eq!(uncounted.verdict(&RULE), Verdict::Pass, "no count, no fail");
    let failed = missed(false, Miss::Fault("the voice worker ended".into()));
    assert_eq!(failed.verdict(&RULE), Verdict::Failed);
    let big = Miss::NotHere {
        need_mb: 5_120,
        have_mb: 4_096,
    };
    assert_eq!(missed(true, big).verdict(&RULE), Verdict::NotHere);
}

#[test]
fn the_table_has_one_line_per_row_of_equal_cells() {
    let failed = missed(false, Miss::Fault("the voice job ran out | memory".into()));
    let big = Miss::NotHere {
        need_mb: 5_120,
        have_mb: 4_096,
    };
    let rows = [
        row(false, 1_500, 0),
        row(true, 4_200, 1),
        failed,
        missed(true, big),
    ];
    let text = lines(&rows, &RULE);
    let table: Vec<&String> = text.iter().filter(|l| l.starts_with('|')).collect();
    assert_eq!(table.len(), rows.len() + 2, "a head, a rule, then the rows");
    let cells = |l: &str| l.matches('|').count();
    assert!(
        table.iter().all(|l| cells(l) == cells(table[0])),
        "{text:?}"
    );
    assert!(
        table[2].contains("1.5 s") && table[2].contains("0.25"),
        "{text:?}"
    );
    assert!(
        table[3].contains("3.75 %") && table[3].contains("1/15"),
        "{text:?}"
    );
    assert!(table[4].contains("ran out / memory"), "{text:?}");
    assert!(
        table[5].contains("not for this PC") && table[5].contains("5120 MB of 4096 MB"),
        "{text:?}"
    );
}

#[test]
fn the_run_passes_on_its_normal_rows_and_weak_rows_are_readings() {
    let big = Miss::NotHere {
        need_mb: 5_120,
        have_mb: 4_096,
    };
    let normal = row(false, 1_500, 0);
    let slow = row(true, 6_600, 0);
    assert_eq!(slow.verdict(&RULE), Verdict::Slow);
    let readings = [normal.clone(), slow, missed(true, big.clone())];
    assert!(
        pass(&readings, &RULE),
        "slow and not for this PC are readings"
    );
    let ended = missed(true, Miss::Fault("the worker did not end in time".into()));
    assert!(!pass(&[normal.clone(), ended], &RULE), "a weak fault");
    assert!(
        !pass(&[normal, row(true, 1_000, 2)], &RULE),
        "weak words wrong"
    );
    assert!(!pass(&[row(false, 3_600, 0)], &RULE), "a slow normal row");
    assert!(
        !pass(&[missed(false, big)], &RULE),
        "a normal row not for this PC"
    );
}

#[test]
fn the_rule_line_says_weak_rows_are_readings() {
    let text = RULE.text();
    assert!(text.contains("weak rows are readings"), "{text}");
    assert!(text.contains("0.5") && text.contains("1 words"), "{text}");
}
