//! Tests of `judge`.

use super::*;

fn seen(kind: Kind, shielded: bool) -> Seen {
    Seen { kind, shielded }
}

#[test]
fn nothing_is_judged_while_no_capture_window_shows() {
    let keyboard = [seen(Kind::Keyboard, false), seen(Kind::Other, false)];
    assert!(faults(SnipKeyboard::Hidden, &keyboard).is_empty());
}

#[test]
fn during_a_capture_only_the_frozen_screen_may_be_bare() {
    let frozen = seen(Kind::Frozen, false);
    let ok = [frozen, seen(Kind::Capture, true), seen(Kind::Bubble, true)];
    assert!(faults(SnipKeyboard::Bubble, &ok).is_empty());
    let bare = [frozen, seen(Kind::Capture, false)];
    assert_eq!(faults(SnipKeyboard::Bubble, &bare), [BARE]);
    let bubble = [seen(Kind::Capture, true), seen(Kind::Bubble, false)];
    assert_eq!(faults(SnipKeyboard::Bubble, &bubble), [BARE]);
}

#[test]
fn the_keyboard_steps_aside_and_hidden_shows_nothing_but_the_capture() {
    let kb = [seen(Kind::Capture, true), seen(Kind::Keyboard, true)];
    assert_eq!(faults(SnipKeyboard::Bubble, &kb), [KEYBOARD]);
    for other in [Kind::Bubble, Kind::Other] {
        let shown = [seen(Kind::Frozen, false), seen(other, true)];
        assert_eq!(faults(SnipKeyboard::Hidden, &shown), [SHOWS], "{other:?}");
    }
    let ours = [seen(Kind::Capture, true), seen(Kind::Other, true)];
    assert!(
        faults(SnipKeyboard::Bubble, &ours).is_empty(),
        "shielded, as the aside shields ours"
    );
}

#[test]
fn a_length_passes_within_its_slack_either_way() {
    assert!(within(5_000, 5_000, 500));
    assert!(within(5_500, 5_000, 500) && within(4_500, 5_000, 500));
    assert!(!within(5_501, 5_000, 500) && !within(4_499, 5_000, 500));
}

#[test]
fn the_exact_checks_say_why_they_skip_on_another_scale() {
    assert_eq!(off_scale(120, 120), None);
    let why = off_scale(96, 120).expect("skipped");
    assert!(why.contains("100 %") && why.contains("125 %"), "{why}");
}

#[test]
fn memory_is_back_when_neither_figure_grew_past_the_limit() {
    let m = |working_mb, commit_mb| Memory {
        working_mb,
        commit_mb,
    };
    assert!(
        back(m(50.0, 60.0), m(57.9, 52.0), 8.0)
            .iter()
            .all(|p| p.pass)
    );
    let grown = back(m(50.0, 60.0), m(51.0, 68.5), 8.0);
    assert_eq!(grown.map(|p| p.pass), [true, false]);
}

#[test]
fn a_part_off_scale_is_skipped_with_its_reason_and_one_at_scale_stands() {
    let v = serde_json::json!({ "ok": true });
    assert_eq!(at_scale(v.clone(), None), v);
    let off = at_scale(v, Some("the screen is at 100 %"));
    assert_eq!(off[crate::winpanel::SKIPPED], true);
    assert_eq!(off["why"], "the screen is at 100 %");
}

#[test]
fn the_keyboard_is_out_when_its_place_holds_little_of_it_and_it_never_showed_with_the_freeze() {
    assert!(keyboard_out(Some(0.2), false, 0.9));
    assert!(!keyboard_out(Some(0.95), false, 0.9), "its pixels stayed");
    assert!(
        !keyboard_out(Some(0.2), true, 0.9),
        "it showed with the frozen screen"
    );
    assert!(!keyboard_out(None, false, 0.9), "no place to judge");
}

#[test]
fn the_bubble_is_out_by_its_place_against_the_grab_and_against_its_own_drawing() {
    let shares = (0.001, 0.9);
    let way = SnipKeyboard::Bubble;
    let seen = |clear, like| {
        Some(BubbleSeen {
            clear,
            like,
            drawing: None,
        })
    };
    let ok = |way, b| bubble(way, b, shares)["ok"] == true;
    assert!(ok(SnipKeyboard::Hidden, None), "no bubble while hidden");
    assert!(!ok(way, None), "no bubble showed");
    assert!(ok(way, seen(Some(0.0), Some(0.1))));
    assert!(
        ok(way, seen(Some(0.0), None)),
        "clear of the keyboard, no drawing"
    );
    assert!(
        !ok(way, seen(Some(0.5), Some(0.1))),
        "the grab changed there"
    );
    assert!(
        ok(way, seen(None, Some(0.1))),
        "over the keyboard, by its drawing"
    );
    assert!(
        !ok(way, seen(None, Some(0.95))),
        "its drawing is in the picture"
    );
    assert!(!ok(way, seen(None, None)), "nothing to compare");
}

#[test]
fn a_missing_drawing_keeps_its_cause_a_late_call_apart_from_a_blank_one() {
    let (shares, way) = ((0.001, 0.9), SnipKeyboard::Bubble);
    let late = "PrintWindow did not end within 5000 ms";
    let seen = |clear, why: &str| {
        Some(BubbleSeen {
            clear,
            like: None,
            drawing: Some(why.into()),
        })
    };
    let v = bubble(way, seen(None, late), shares);
    assert_eq!(
        (&v["ok"], &v["drawing"]),
        (&serde_json::json!(false), &serde_json::json!(late))
    );
    let v = bubble(way, seen(Some(0.0), BLANK), shares);
    assert_eq!(
        (&v["ok"], &v["drawing"]),
        (&serde_json::json!(true), &serde_json::json!(BLANK))
    );
}

#[test]
fn windows_settle_only_when_there_are_some_and_each_agrees() {
    assert!(settled(&[true, true], true));
    assert!(settled(&[false], false));
    assert!(!settled(&[true, false], true));
    assert!(!settled(&[], true), "none showed");
}

#[test]
fn the_summary_counts_skipped_parts_with_the_first_reason() {
    let skip = serde_json::json!({ "skipped": true, "why": "scale" });
    let copies = [
        serde_json::json!({ "region": skip, "edit": skip, "full": { "ok": true }, "ok": true }),
        serde_json::json!({ "full": { "ok": true }, "ok": true }),
    ];
    assert_eq!(skipped_note(&copies), Some("2 skipped: scale".to_string()));
    assert_eq!(skipped_note(&copies[1..]), None);
}

#[test]
fn every_picture_windows_history_could_keep_must_have_left_it() {
    let (most, small, big) = (4.0, Some(1_000), Some(8_000_000));
    let ok =
        |went: Option<&[bool]>, bytes: &[Option<usize>]| history(went, bytes, most)["ok"] == true;
    assert!(ok(None, &[small]), "history off");
    assert!(
        ok(Some(&[true, false]), &[small, big]),
        "too big to be kept"
    );
    assert!(
        ok(Some(&[true]), &[Some(4_000_000)]),
        "at the limit it is kept"
    );
    assert!(
        !ok(Some(&[false]), &[Some(4_000_000)]),
        "at the limit it must go"
    );
    assert!(
        ok(Some(&[false]), &[Some(4_194_304)]),
        "4 MiB is past 4 decimal MB"
    );
    assert!(
        !ok(Some(&[false, true]), &[small, big]),
        "a small picture stayed"
    );
    assert!(
        ok(Some(&[false]), &[None]),
        "nothing read back, nothing owed"
    );
    assert!(!ok(Some(&[true]), &[small, small]), "a span unanswered");
    let v = history(Some(&[true, false]), &[small, small], most);
    assert_eq!(
        (&v["went"], &v["missed"]),
        (&serde_json::json!(1), &serde_json::json!(1))
    );
}

#[test]
fn a_window_of_ours_under_any_frozen_screen_is_named_with_the_topmost_screen() {
    use Layer::{Above, Else, Frozen};
    assert!(under(&[Above, Above, Frozen, Else, Frozen]).is_empty());
    assert_eq!(under(&[Frozen, Above, Frozen, Above]), [(1, 0), (3, 0)]);
    assert_eq!(under(&[Else, Above, Frozen, Else, Above]), [(4, 2)]);
    assert!(under(&[Above, Else]).is_empty(), "no frozen screen");
    assert!(under(&[]).is_empty());
}

#[test]
fn a_frozen_screen_of_ours_stays_frozen_though_the_check_names_every_window_of_ours() {
    assert_eq!(layer(true, true), Layer::Frozen, "ours names it too");
    assert_eq!(layer(true, false), Layer::Frozen);
    assert_eq!(layer(false, true), Layer::Above);
    assert_eq!(layer(false, false), Layer::Else);
}

#[test]
fn an_order_without_a_frozen_screen_on_each_screen_falls_short() {
    use Layer::{Above, Else, Frozen};
    assert_eq!(short(&[Above, Else], 1), Some(0), "no frozen screen");
    assert_eq!(
        short(&[], 0),
        Some(0),
        "at least one, though Windows names no screen"
    );
    assert_eq!(short(&[Above, Frozen, Else], 2), Some(1), "one of two");
    assert_eq!(short(&[Frozen, Above, Frozen], 2), None);
    assert_eq!(short(&[Frozen], 1), None);
}

#[test]
fn a_kind_is_lacking_until_a_window_of_it_shows() {
    let need = [Kind::Frozen, Kind::Capture, Kind::Bubble];
    let shown = [Kind::Capture, Kind::Frozen, Kind::Capture];
    assert_eq!(lacks(&need, &shown), [Kind::Bubble]);
    let all = [Kind::Bubble, Kind::Capture, Kind::Frozen];
    assert!(lacks(&need, &all).is_empty());
    assert_eq!(lacks(&[Kind::Keyboard], &[]), [Kind::Keyboard]);
}

#[test]
fn a_report_names_ours_by_title_and_any_other_window_by_its_class_only() {
    assert_eq!(named(true, "Snip bar", "Qt"), "'Snip bar'");
    let other = named(false, "Save As", "#32770");
    assert_eq!(other, "[#32770]", "never another program's title");
    assert_eq!(named(true, "", "Qt"), "[Qt]", "an untitled one of ours");
}
