//! A fill typed at the caret (ADR-0038 decision 4): every latch lets go first, text goes by the route a letter takes, a
//! Tab as the board's Tab key with the next part after the gap, the answer names the round, and prediction hears only
//! a break, so the word in progress ends as after a paste.

use super::{GAP, Quick, gap_ms, line, tab, typed, unicode};
use crate::assist::Said;
use crate::rig::{HELD, STATE};
use kx_module_api::Redacted;
use kx_module_api::ports::{FaceAction, FaceReply, FillPart, InjectError, Piece, Scan, TextRoute};
use kx_platform_fake::Stroke;

/// The app took none of a send.
const REFUSED: InjectError = InjectError::Refused { sent: 0, total: 1 };

#[test]
fn every_latch_lets_go_first_and_the_text_goes_as_unicode() {
    let q = Quick::quiet(None);
    q.b.rig.tap("ctrl").unwrap();
    q.b.rig.tap("shift").unwrap();
    q.b.said();
    q.fill(4, vec![line("alex@example.com")]);
    let want = [unicode("alex@example.com")];
    assert_eq!(q.strokes(), want, "no chord: Ctrl let go first");
    assert_eq!(q.heard(), [typed(4, true)]);
    let none = Said::Latches(Vec::new());
    assert_eq!(q.b.said(), [none], "the bus hears they let go");
    assert_eq!(q.b.rig.act(FaceAction::Relabel), Ok(STATE));
    assert!(!q.on("ctrl") && !q.on("shift"));
}

#[test]
fn in_stay_mode_a_fill_lets_the_latches_go_too() {
    let q = Quick::quiet(Some(("modifiers", "stay".into())));
    q.b.rig.tap("alt").unwrap();
    q.fill(1, vec![line("x")]);
    assert_eq!(q.strokes(), [unicode("x")]);
    assert!(!q.on("alt"));
}

#[test]
fn an_arabic_value_and_a_persian_one_with_its_zero_width_non_joiner_go_whole() {
    let q = Quick::quiet(None);
    let (arabic, persian) = ("كلمة السر", "می\u{200C}خواهم");
    q.fill(2, vec![line(arabic)]);
    q.fill(3, vec![line(persian)]);
    assert_eq!(q.strokes(), [unicode(arabic), unicode(persian)]);
    assert_eq!(q.heard(), [typed(2, true), typed(3, true)]);
}

#[test]
fn a_tab_goes_alone_as_the_boards_tab_key_and_the_next_part_waits_the_gap() {
    let q = Quick::quiet(None);
    q.fill(1, vec![line("alex"), FillPart::Tab, line("pässwörd")]);
    assert_eq!(q.strokes(), [unicode("alex"), tab()]);
    assert!(q.heard().is_empty(), "no answer while a part waits");
    q.relabel_in(gap_ms() - 1);
    assert!(q.strokes().is_empty(), "nothing before the gap");
    q.relabel_in(1);
    assert_eq!(
        q.strokes(),
        [unicode("pässwörd")],
        "the part right after it"
    );
    assert_eq!(q.heard(), [typed(1, true)]);
}

#[test]
fn each_tab_waits_its_own_gap_and_a_last_tab_waits_nothing() {
    let q = Quick::quiet(None);
    let parts = vec![
        line("a"),
        FillPart::Tab,
        FillPart::Tab,
        line("b"),
        FillPart::Tab,
    ];
    q.fill(3, parts);
    assert_eq!(q.strokes(), [unicode("a"), tab()]);
    q.relabel_in(gap_ms());
    assert_eq!(q.strokes(), [tab()]);
    assert!(q.heard().is_empty());
    q.relabel_in(gap_ms());
    assert_eq!(q.strokes(), [unicode("b"), tab()]);
    assert_eq!(q.heard(), [typed(3, true)]);
}

#[test]
fn the_hold_timer_and_any_click_send_a_due_part_too() {
    let q = Quick::quiet(None);
    q.fill(5, vec![line("a"), FillPart::Tab, line("b")]);
    q.strokes();
    assert_eq!(q.b.rig.tick_in(gap_ms()), FaceReply::default());
    assert_eq!(q.strokes(), [unicode("b")]);
    q.fill(6, vec![FillPart::Tab, line("c")]);
    q.b.rig.fakes.clock.advance(GAP);
    q.b.rig.tap("c-0x0010").unwrap();
    assert_eq!(
        q.strokes(),
        [tab(), unicode("c"), unicode("q")],
        "the fill first"
    );
    assert_eq!(q.heard(), [typed(5, true), typed(6, true)]);
}

#[test]
fn a_part_the_app_refuses_ends_the_fill_with_no_and_nothing_after_it_goes() {
    let q = Quick::quiet(None);
    let parts = vec![
        line("a"),
        FillPart::Tab,
        line("b"),
        FillPart::Tab,
        line("c"),
    ];
    q.fill(7, parts);
    assert_eq!(q.strokes(), [unicode("a"), tab()]);
    q.b.rig.fakes.injector.set_error(Some(REFUSED));
    q.relabel_in(gap_ms());
    assert_eq!(q.heard(), [typed(7, false)]);
    q.b.rig.fakes.injector.set_error(None);
    q.relabel_in(1_000);
    assert!(
        q.strokes().is_empty() && q.heard().is_empty(),
        "nothing after it"
    );
}

#[test]
fn a_fill_the_app_refuses_at_once_answers_no_and_waits_for_nothing() {
    let q = Quick::quiet(None);
    q.b.rig
        .fakes
        .injector
        .set_error(Some(InjectError::Unmappable));
    q.fill(8, vec![line("a"), FillPart::Tab, line("b")]);
    assert_eq!(q.heard(), [typed(8, false)]);
    assert!(q.pieces().is_empty(), "nothing reached the app");
    q.b.rig.fakes.injector.set_error(None);
    q.relabel_in(1_000);
    assert!(q.strokes().is_empty());
}

#[test]
fn word_prediction_hears_one_break_and_never_a_value_a_tab_or_a_word() {
    let q = Quick::quiet(None);
    q.b.rig.tap("c-0x0010").unwrap();
    assert_eq!(q.pieces(), [Piece::Text(Redacted::new("q".into()))]);
    q.fill(9, vec![line("alex"), FillPart::Tab, line("secret")]);
    assert_eq!(q.pieces(), [Piece::Break], "the word in progress ends");
    q.relabel_in(gap_ms());
    assert!(q.pieces().is_empty(), "the rest tells nothing");
    assert_eq!(q.heard(), [typed(9, true)]);
}

#[test]
fn a_waiting_dead_key_refuses_the_fill_so_no_accent_joins_it() {
    let q = Quick::quiet(None);
    q.b.rig.switch("fr-fr");
    q.b.rig.press("c-0x001A").unwrap();
    assert_eq!(q.strokes(), [Stroke::Tap(Scan::new(0x1A))], "a wait starts");
    q.b.rig.tap("shift").unwrap();
    q.fill(10, vec![line("e")]);
    assert!(q.strokes().is_empty());
    assert_eq!(q.heard(), [typed(10, false)]);
    assert!(q.on("shift"), "a refused fill touches nothing");
}

#[test]
fn a_fill_ends_a_held_key_so_it_never_repeats_after() {
    let q = Quick::quiet(None);
    assert_eq!(q.b.rig.press("c-0x0010"), Ok(HELD));
    q.fill(11, vec![line("a")]);
    q.strokes();
    assert_eq!(q.b.rig.tick_in(1_000), FaceReply::default());
    assert!(q.strokes().is_empty(), "the hold ended with the fill");
}

#[test]
fn in_a_key_app_text_goes_as_key_presses_as_a_letter_does() {
    let q = Quick::quiet(None);
    q.b.rig.fakes.injector.set_target(Some("scrcpy.exe"));
    q.fill(12, vec![line("ab")]);
    let keys = Stroke::Text(Redacted::new("ab".into()), TextRoute::Keys);
    assert_eq!(q.strokes(), [keys]);
}
