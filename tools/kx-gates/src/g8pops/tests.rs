//! Tests of G8's churn rounds: what counts as a pop-up, the bubble swap, the list skip, the theme cycle and the buttons.

use super::*;

#[test]
fn a_pop_up_counts_by_a_window_new_since_the_step_began_not_by_how_many_show() {
    let base: HashSet<isize> = [1, 2, 3].into();
    assert!(!any_new(&base, &[1, 2, 3]), "nothing new");
    assert!(!any_new(&base, &[1, 2]), "a banner that hid is no pop-up");
    assert!(
        any_new(&base, &[1, 2, 9]),
        "a tip that showed as a banner hid"
    );
}

#[test]
fn a_minimise_counts_a_swap_or_a_miss_and_stops_with_its_cause_when_no_bubble_showed() {
    assert_eq!(
        swapped(false, true),
        Ok(true),
        "the bubble took the keyboard's place"
    );
    assert_eq!(swapped(true, true), Ok(false), "a slow hide is a miss");
    for keyboard in [true, false] {
        let e = swapped(keyboard, false).expect_err("nothing to press back");
        assert!(
            e.contains("no bubble") && e.contains("churn_settle_ms"),
            "{e}"
        );
    }
}

#[test]
fn the_list_step_runs_only_in_the_list_design_with_two_layouts_and_says_why_not() {
    assert_eq!(skip_list(LangKey::List, 2), None);
    let one = skip_list(LangKey::List, 1).expect("one layout opens no list");
    assert!(one.contains("one layout"), "{one}");
    let carousel = skip_list(LangKey::Carousel, 3).expect("the Carousel has no list");
    assert!(carousel.contains("not the List design"), "{carousel}");
}

#[test]
fn a_theme_cycle_that_ends_elsewhere_stops_the_churn_and_says_how_to_undo_it() {
    assert_eq!(back_to("native", "native"), Ok(()));
    let e = back_to("native", "et66").expect_err("the owner's theme moved");
    assert!(e.contains("native") && e.contains("et66"), "{e}");
    assert!(e.contains(THEME_LEFT), "{e}");
}

/// The windows whose buttons the bubble step presses.
// Workspace paths from `tools/kx-gates/src/g8pops`; a moved file breaks the build here, never silently.
const TOP_BAR_QML: &str = include_str!("../../../../crates/kx-ui/qml/TopBar.qml");
const BUBBLE_QML: &str = include_str!("../../../../crates/kx-ui/qml/Bubble.qml");

#[test]
fn the_bubble_step_presses_the_top_bars_minimise_and_the_bubbles_own_button() {
    let g9 = crate::config::load().expect("kx-gates.toml loads").g9;
    let hide = g9.put_away.strip_prefix("bar.").expect("a top-bar id");
    assert!(TOP_BAR_QML.contains(&format!("part: \"{hide}\"")), "{hide}");
    let (group, part) = g9.bring_back.split_once('.').expect("group.part");
    let uid = format!("uid(\"{group}\", \"{part}\")");
    assert!(BUBBLE_QML.contains(&uid), "{uid}");
}

#[test]
fn each_hover_counts_toward_the_total_as_before_and_toward_its_own_key() {
    let mut seen = Seen::default();
    let hit = Hover {
        hit: true,
        shown_ms: Some(520),
    };
    let hid = Hover {
        hit: false,
        shown_ms: Some(510),
    };
    seen.tip(0, hit);
    seen.tip(2, hid);
    seen.tip(0, hit);
    assert_eq!(seen.tips, 2, "only a tip showing at the rest's end counts");
    let ids = ["bksp", "enter", "shift"].map(String::from);
    let counts: Vec<_> = seen
        .keys
        .report(&ids)
        .iter()
        .map(|k| (k.hits, k.hid))
        .collect();
    assert_eq!(counts, [(2, 0), (0, 0), (0, 1)]);
}
