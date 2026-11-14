//! A `TextAsk` (design "Typing"): it erases paced by the keyboard's `erase_gap_ms`, as a picked word does, then types
//! each character paced by its gap, or in one piece with no gap, by the route a letter takes, keeps every latch, tells
//! prediction one break before its answer, and is refused whole when it holds a hidden mark, a dead key waits, a fill
//! waits or the module is stopped.

use super::{BREAK, Mic, done, erase_gap, ms};
use crate::fill::{GAP, login};
use crate::held::{A, times};
use crate::told::BKSP;
use crate::typing::{tap, text, unicode};
use kx_module_api::Module;
use kx_module_api::ports::{FillType, TextRoute};
use std::thread;

/// The French dead `^`.
const CIRCUMFLEX: &str = "c-0x001A";

/// An ask's gap that differs from the keyboard's own `erase_gap_ms`.
const GAP_MS: u16 = 25;

#[test]
fn a_text_ask_erases_paced_by_erase_gap_ms_then_types_paced_by_its_gap() {
    let m = Mic::quiet(None);
    m.ask(7, 2, "ab", GAP_MS);
    let want = [tap(BKSP), tap(BKSP), unicode("a"), unicode("b")];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want);
    let me = thread::current().id();
    let waits = m.b.rig.fakes.pause.waits();
    let gap = ms(u64::from(GAP_MS));
    let want = [(me, erase_gap()), (me, erase_gap()), (me, gap)];
    assert_eq!(waits, want, "after each piece but the last");
    assert_eq!(
        m.heard(),
        [BREAK, done(7, 2, true)],
        "one break, then the count"
    );
}

#[test]
fn with_no_erase_gap_the_backspaces_go_at_once_whatever_the_asks_gap() {
    let m = Mic::quiet(Some(("erase_gap_ms", 0.into())));
    m.ask(1, 3, "ab", GAP_MS);
    let want = [tap(BKSP), tap(BKSP), tap(BKSP), unicode("a"), unicode("b")];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want);
    let me = thread::current().id();
    let waits = m.b.rig.fakes.pause.waits();
    let gap = ms(u64::from(GAP_MS));
    assert_eq!(waits, [(me, gap)], "between the letters only");
}

#[test]
fn an_unpaced_text_goes_in_one_piece_with_no_wait() {
    let m = Mic::quiet(None);
    m.ask(1, 0, "bonjour", 0);
    assert_eq!(m.b.rig.fakes.injector.strokes(), [unicode("bonjour")]);
    assert!(m.b.rig.fakes.pause.waits().is_empty());
    assert_eq!(m.heard(), [BREAK, done(1, 7, true)]);
}

#[test]
fn arabic_is_typed_and_counted_in_whole_characters_marks_included() {
    let m = Mic::quiet(None);
    let t = "مَرْحَبًا";
    m.ask(2, 0, t, 10);
    let each: Vec<_> = t.chars().map(|c| unicode(&c.to_string())).collect();
    assert_eq!(m.b.rig.fakes.injector.strokes(), each);
    let count = u32::try_from(t.chars().count()).unwrap();
    assert_eq!(m.heard(), [BREAK, done(2, count, true)]);
}

#[test]
fn a_text_goes_as_key_presses_to_an_app_in_key_apps() {
    let m = Mic::quiet(None);
    m.b.rig.fakes.injector.set_target(Some("scrcpy.exe"));
    m.ask(1, 0, "ok", 0);
    assert_eq!(
        m.b.rig.fakes.injector.strokes(),
        [text("ok", TextRoute::Keys)]
    );
}

#[test]
fn a_text_leaves_every_latch_as_it_was() {
    let m = Mic::quiet(None);
    m.b.rig.tap("ctrl").unwrap();
    m.b.rig.tap("shift").unwrap();
    m.b.said();
    m.ask(1, 1, "ab", 0);
    let want = [tap(BKSP), unicode("ab")];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want, "nothing held");
    assert!(m.b.said().is_empty(), "the latched set never changed");
    assert!(m.b.key("ctrl")["on"] == true && m.b.key("shift")["on"] == true);
}

#[test]
fn a_one_shot_shift_latched_before_a_text_still_shifts_the_next_letter() {
    let m = Mic::quiet(None);
    m.b.rig.tap("shift").unwrap();
    m.ask(1, 0, "ab", 0);
    m.b.rig.tap(A).unwrap();
    let want = [unicode("ab"), unicode("A")];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want);
}

#[test]
fn prediction_never_hears_the_text_or_its_erases() {
    let m = Mic::quiet(None);
    m.ask(4, 3, "hello world", 10);
    assert_eq!(m.heard(), [BREAK, done(4, 11, true)]);
}

#[test]
fn an_ask_with_nothing_to_do_answers_yes_and_tells_nothing() {
    let m = Mic::quiet(None);
    m.ask(5, 0, "", 10);
    assert!(m.b.rig.fakes.injector.strokes().is_empty());
    assert_eq!(m.heard(), [done(5, 0, true)]);
}

#[test]
fn a_text_with_a_hidden_mark_types_nothing_and_answers_no() {
    let m = Mic::quiet(None);
    m.ask(3, 2, "a\nb", 0);
    assert!(m.b.rig.fakes.injector.strokes().is_empty());
    assert_eq!(m.heard(), [done(3, 0, false)]);
}

#[test]
fn a_dead_key_waiting_refuses_the_text_whole() {
    let m = Mic::quiet(None);
    m.b.rig.switch("fr-fr");
    m.b.rig.tap(CIRCUMFLEX).unwrap();
    m.b.rig.fakes.injector.clear();
    m.heard();
    m.ask(1, 1, "e", 0);
    assert!(m.b.rig.fakes.injector.strokes().is_empty());
    assert_eq!(m.heard(), [done(1, 0, false)]);
}

#[test]
fn while_the_module_is_stopped_nothing_types_and_the_answer_is_no() {
    let mut m = Mic::quiet(None);
    m.b.rig.module.stop();
    m.ask(2, 1, "ab", 0);
    assert!(m.b.rig.fakes.injector.strokes().is_empty());
    assert_eq!(m.heard(), [done(2, 0, false)]);
}

#[test]
fn while_a_fill_waits_its_gap_nothing_types_and_the_answer_is_no() {
    let m = Mic::quiet(None);
    let fill = FillType {
        round: 1,
        parts: login(),
        tab_gap: GAP,
    };
    m.b.send(&fill);
    m.b.rig.fakes.injector.clear();
    m.heard();
    m.ask(2, 0, "ab", 0);
    assert!(m.b.rig.fakes.injector.strokes().is_empty());
    assert_eq!(m.heard(), [done(2, 0, false)]);
}

#[test]
fn a_held_key_ends_before_the_text_and_repeats_no_more() {
    let m = Mic::quiet(None);
    m.b.rig.press(A).unwrap();
    m.ask(1, 0, "b", 0);
    let (delay, _) = times();
    assert!(!m.b.rig.tick_in(delay).holding, "the burst ended");
    let want = [unicode("a"), unicode("b")];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want);
}

#[test]
fn a_new_ask_after_a_done_one_types_again() {
    let m = Mic::quiet(None);
    m.ask(1, 0, "ab", 0);
    m.ask(2, 2, "cd", 0);
    let want = [unicode("ab"), tap(BKSP), tap(BKSP), unicode("cd")];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want);
    let heard = [BREAK, done(1, 2, true), BREAK, done(2, 2, true)];
    assert_eq!(m.heard(), heard);
}
