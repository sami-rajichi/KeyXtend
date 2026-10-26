//! Tests of `g8predict`.

use super::*;

fn memory(working_mb: f64, commit_mb: f64) -> Memory {
    Memory {
        working_mb,
        commit_mb,
    }
}

fn typed(before: Memory, after: Memory) -> Typed {
    Typed {
        words: 30,
        picked: 30,
        before,
        after,
        note: None,
        stopped: false,
    }
}

/// Both churns: `judged`, and `mixed` from the same start that grew `mixed_mb`.
fn churns(judged: &Typed, mixed_mb: f64) -> Churns {
    let mut mixed = judged.clone();
    mixed.after.working_mb = mixed.before.working_mb + mixed_mb;
    mixed.after.commit_mb = mixed.before.commit_mb + mixed_mb;
    Churns {
        judged: judged.clone(),
        mixed: Ok(mixed),
    }
}

fn ended(gone: Gone) -> Ended {
    Ended {
        gone,
        note: None,
        released: Vec::new(),
    }
}

/// A side read as `m`, with the churns `typed` when predict was on, whose copy went `gone`.
fn side_of(m: Memory, typed: Option<Churns>, gone: Gone) -> Side {
    Side {
        memory: Ok(m),
        typed: typed.map(Ok),
        ended: ended(gone),
    }
}

#[test]
fn the_judged_churn_decides_and_the_mixed_one_is_only_read() {
    let cfg = crate::config::load().expect("loads");
    let limit = cfg.g8.churn_growth_mb;
    let off = Ok(side_of(memory(62.0, 87.0), None, Gone::Quit));
    let on = |c: Churns| Ok(side_of(memory(64.0, 90.0), Some(c), Gone::Quit));
    let within = typed(memory(60.0, 80.0), memory(60.5, 80.5));
    let v = result(&cfg, &on(churns(&within, limit + 2.0)), &off, &Ok(()));
    assert_eq!(v["pass"], true, "the mixed growth is past the limit: {v}");
    let grown = format!("{:.MB_DECIMALS$} MB working set", limit + 2.0);
    let line = v["mixed_reading"].as_str().expect("a reading");
    assert!(line.contains(&grown) && line.contains("30 of 30"), "{line}");
    assert_eq!(v["typing_mixed"]["picked"], 30, "{v}");
    let mut missed = churns(&within, 0.0);
    missed.mixed = Err("no chip offered the ar prefix".into());
    let v = result(&cfg, &on(missed), &off, &Ok(()));
    assert_eq!(v["pass"], true, "a mixed miss never fails: {v}");
    assert_eq!(v["typing_mixed"]["error"], "no chip offered the ar prefix");
    assert!(v["mixed_reading"].is_null(), "no reading without memory");
    let past = typed(memory(60.0, 80.0), memory(60.0 + limit + 0.5, 80.5));
    let v = result(&cfg, &on(churns(&past, 0.0)), &off, &Ok(()));
    assert_eq!(v["pass"], false, "the judged churn grew past it: {v}");
}

#[test]
fn the_result_passes_on_two_clean_quits_and_is_partial_after_a_missed_word() {
    let cfg = crate::config::load().expect("loads");
    let whole = typed(memory(60.0, 80.0), memory(60.5, 80.5));
    let off = Ok(side_of(memory(62.0, 87.0), None, Gone::Quit));
    let on = |t: &Typed| {
        Ok(side_of(
            memory(64.0, 90.0),
            Some(churns(t, 0.0)),
            Gone::Quit,
        ))
    };
    let v = result(&cfg, &on(&whole), &off, &Ok(()));
    assert_eq!(v["pass"], true, "{v}");
    let mut missed = whole.clone();
    missed.picked -= 1;
    missed.note = Some("no chip offered the en prefix".into());
    let v = result(&cfg, &on(&missed), &off, &Ok(()));
    assert_eq!(v["pass"], out::PARTIAL, "{v}");
    assert_eq!(v["typing"]["note"], "no chip offered the en prefix");
    let v = result(&cfg, &on(&whole), &off, &Err("not back".into()));
    assert_eq!(v["pass"], false, "the owner's file did not come back");
}

#[test]
fn an_off_side_never_started_after_a_stuck_copy_fails_the_result() {
    let cfg = crate::config::load().expect("loads");
    let whole = typed(memory(60.0, 80.0), memory(60.5, 80.5));
    let on = Ok(side_of(
        memory(64.0, 90.0),
        Some(churns(&whole, 0.0)),
        Gone::Running,
    ));
    let off = Err(Failed::from(STILL_RUNS.to_string()));
    let v = result(&cfg, &on, &off, &Ok(()));
    assert_eq!(v["pass"], false, "{v}");
    assert_eq!(v["off"]["error"], STILL_RUNS);
    assert!(v["predict_mb"].is_null(), "no off reading");
}

#[test]
fn the_predict_part_is_the_on_reading_less_the_off_reading() {
    let got = less(memory(64.5, 90.0), memory(62.5, 87.0));
    assert_eq!(got, memory(2.0, 3.0));
    let line = reading(less(memory(64.5, 90.0), memory(62.3, 87.0)), 2.0);
    assert!(line.contains("2.2 MB working set"), "{line}");
    assert!(
        line.contains("3.0 MB commit") && line.contains("about 2.0 MB"),
        "{line}"
    );
}

#[test]
fn a_predict_part_below_nothing_still_reads_as_a_number() {
    let line = reading(less(memory(60.0, 80.0), memory(60.5, 80.0)), 2.0);
    assert!(line.contains("-0.5 MB working set"), "{line}");
}

#[test]
fn a_copy_that_had_to_be_ended_fails_the_run() {
    assert!(passed(true, true, [true, true], true));
    assert!(
        !passed(true, true, [true, false], true),
        "the off copy was ended"
    );
    assert!(
        !passed(true, true, [true, true], false),
        "the owner's file did not come back"
    );
    assert!(!passed(false, true, [true, true], true), "no reading");
    assert!(!passed(true, false, [true, true], true), "the churn grew");
}

#[test]
fn the_typing_churn_passes_within_the_growth_limit_and_fails_past_it() {
    let within = typing_parts(&typed(memory(60.0, 80.0), memory(62.0, 82.5)), 3.0);
    assert!(budget::all_pass(&within));
    assert_eq!(within.map(|p| p.got), [Some(2.0), Some(2.5)]);
    let past = typing_parts(&typed(memory(60.0, 80.0), memory(63.5, 81.0)), 3.0);
    assert_eq!(
        past.map(|p| p.pass),
        [false, true],
        "the working set grew past it"
    );
    let commit = typing_parts(&typed(memory(60.0, 80.0), memory(60.0, 83.1)), 3.0);
    assert_eq!(
        commit.map(|p| p.pass),
        [true, false],
        "the commit grew past it"
    );
}
