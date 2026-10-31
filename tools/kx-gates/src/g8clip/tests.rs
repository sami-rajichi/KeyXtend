use super::*;
use crate::config::load;

fn mem(working_mb: f64, commit_mb: f64) -> Memory {
    Memory {
        working_mb,
        commit_mb,
    }
}

/// Readings within every shipped limit: 20 MB empty, 40 MB filled, 60 MB open, 1 MB growth over the rounds.
fn calm() -> Readings {
    Readings {
        empty: mem(20.0, 30.0),
        filled: mem(40.0, 50.0),
        open: mem(60.0, 70.0),
        before: mem(55.0, 65.0),
        last: mem(56.0, 66.0),
    }
}

/// The part named `name` among `parts`.
fn part<'a>(parts: &'a [Part], name: &str) -> &'a Part {
    parts.iter().find(|p| p.name == name).expect(name)
}

#[test]
fn the_most_takes_each_figure_on_its_own() {
    assert_eq!(most(mem(1.0, 9.0), mem(5.0, 2.0)), mem(5.0, 9.0));
}

#[test]
fn calm_readings_pass_every_part_against_the_shipped_limits() {
    let g = load().expect("loads").g8;
    let got = parts(&g, &calm());
    assert_eq!(got.len(), 6);
    assert!(budget::all_pass(&got), "{got:?}");
    assert_eq!(part(&got, "rounds_working_mb").got, Some(1.0));
    assert!(
        part(&got, "open_commit_mb")
            .limit
            .total_cmp(&g.commit_mb)
            .is_eq()
    );
}

#[test]
fn memory_past_a_limit_or_growth_over_the_rounds_fails_its_part() {
    let g = load().expect("loads").g8;
    let mut big = calm();
    big.open.working_mb = g.memory_mb + 1.0;
    let got = parts(&g, &big);
    assert!(!part(&got, "open_working_mb").pass);
    assert!(part(&got, "filled_working_mb").pass, "only that part");
    let mut leak = calm();
    leak.last.commit_mb = leak.before.commit_mb + g.churn_growth_mb + 1.0;
    let got = parts(&g, &leak);
    assert!(!part(&got, "rounds_commit_mb").pass);
    assert!(!budget::all_pass(&got));
}

#[test]
fn the_reading_names_what_the_fill_and_the_open_panel_added() {
    let line = reading(&calm());
    assert!(line.contains("20.0 MB working set"), "{line}");
    assert!(line.contains("never a fail"), "{line}");
}

#[test]
fn the_pages_round_up_and_an_empty_list_still_shows_one() {
    assert_eq!(pages_for(10, 5), 2);
    assert_eq!(pages_for(11, 5), 3);
    assert_eq!(pages_for(0, 5), 1);
    assert_eq!(
        pages_for(3, 0),
        3,
        "a zero page size counts as one, as the product's does"
    );
}

#[test]
fn the_fill_holds_only_when_rows_and_page_count_match_every_entry() {
    let n = load().expect("loads").g8_clip.entries;
    let per = 5;
    assert!(holds(n, per, Some(pages_for(n, per))));
    assert!(!holds(n, per, Some(pages_for(n, per) - 1)), "a page short");
    assert!(!holds(n, per, None), "no page count over many pages");
    assert!(!holds(n, 0, Some(1)), "an empty panel");
    assert!(!holds(n, 2, Some(1)), "a part page holds too few");
    assert!(
        holds(3, 3, None) && holds(3, 3, Some(1)),
        "one page shows every row"
    );
    assert!(!holds(3, 4, Some(1)), "more rows than copies is no match");
}
