//! Translate once the user said yes: one read at the click, cut at a whole word within `translate_chars`, the text
//! encoded into the site's link with the target's code, and a failure told (plan edge cases 1, 9 and 10).

use kx_module_api::keys;
use kx_module_api::ports::{ChipsView, LinkError, SelChip, SelError, SelKind};
use kx_test_support::host::FakeHost;
use toml::Value;

use super::rig::Rig;
use super::spots::sel;
use crate::settings::TRANSLATE_OK_KEY;

/// The site's page for `text`, already encoded, in `to`.
fn page(to: &str, text: &str) -> String {
    format!("https://translate.google.com/?sl=auto&tl={to}&text={text}&op=translate")
}

/// A rig whose user already said yes to Translate, with `setup` after.
fn agreed(setup: impl FnOnce(&mut FakeHost)) -> Rig {
    Rig::build(|h| {
        h.put(TRANSLATE_OK_KEY, Value::Boolean(true));
        setup(h);
    })
}

/// Clicks Translate on a read-only selection holding `text`; returns the links opened.
fn translate(rig: &mut Rig, text: &str) -> Vec<String> {
    rig.watch.set_text(text);
    rig.focus(sel(SelKind::ReadOnly, 40));
    assert!(rig.reads().is_empty(), "no read before the click");
    rig.act(SelChip::Translate);
    let opened = rig.web.opened();
    opened.iter().map(|l| l.expose().to_owned()).collect()
}

#[test]
fn translate_reads_once_at_the_click_and_opens_the_sites_page_in_the_system_language() {
    let mut rig = agreed(|_| {});
    let links = translate(&mut rig, "hello world");
    assert_eq!(links, [page("fr", "hello%20world")]);
    assert_eq!(rig.watch.calls().reads, 1);
    assert_eq!(
        rig.watch.calls().stills,
        1,
        "the box is checked right before"
    );
    let round = rig.round();
    let tick = matches!(rig.view(), ChipsView::Shown { round: r, acted: Some(SelChip::Translate), .. } if r == round);
    assert!(tick, "{:?}", rig.heard());
    assert!(rig.notices().is_empty(), "no question once agreed");
}

#[test]
fn a_set_target_wins_over_the_system_language() {
    let mut rig = agreed(|h| h.put("translate_to", Value::String("ar".into())));
    assert_eq!(translate(&mut rig, "hello"), [page("ar", "hello")]);
}

#[test]
fn arabic_and_french_are_encoded_and_hidden_marks_dropped() {
    let mut rig = agreed(|_| {});
    let links = translate(&mut rig, "مرحبا\u{200b} café");
    let text = "%D9%85%D8%B1%D8%AD%D8%A8%D8%A7%20caf%C3%A9";
    assert_eq!(links, [page("fr", text)]);
}

#[test]
fn a_long_selection_is_cut_within_translate_chars_at_a_whole_word() {
    let mut rig = agreed(|h| h.put("translate_chars", Value::Integer(50)));
    let long = "alpha bravo charlie delta echo foxtrot golf hotelier india";
    let links = translate(&mut rig, long);
    let kept = "alpha%20bravo%20charlie%20delta%20echo%20foxtrot%20golf";
    assert_eq!(links, [page("fr", kept)], "never half of hotelier");
    assert_eq!(
        rig.reads(),
        [51],
        "one character past the cap shows where the word ends"
    );
}

#[test]
fn a_blank_selection_opens_nothing_and_tells_nothing() {
    let mut rig = agreed(|_| {});
    assert!(translate(&mut rig, " \n\t\u{200b} ").is_empty());
    assert_eq!(
        rig.reads().len(),
        1,
        "the one read at the click finds it blank"
    );
    assert!(rig.notices().is_empty());
    assert_eq!(rig.view(), ChipsView::Hidden);
}

#[test]
fn a_failed_read_says_the_page_did_not_open() {
    let mut rig = agreed(|_| {});
    rig.watch.fail_reads(SelError::TimedOut);
    assert!(translate(&mut rig, "hello").is_empty());
    assert_eq!(rig.keys(), [keys::TRANSLATE_FAILED]);
    assert_eq!(rig.view(), ChipsView::Hidden);
}

#[test]
fn a_refused_open_says_the_page_did_not_open_and_shows_no_tick() {
    let mut rig = agreed(|_| {});
    rig.web.set_error(Some(LinkError::Refused(5)));
    assert_eq!(translate(&mut rig, "hello").len(), 1);
    assert_eq!(rig.keys(), [keys::TRANSLATE_FAILED]);
    assert_eq!(rig.view(), ChipsView::Hidden);
}

#[test]
fn an_open_whose_thread_did_not_start_says_the_page_did_not_open() {
    let mut rig = agreed(|_| {});
    rig.no_threads();
    assert!(translate(&mut rig, "hello").is_empty());
    assert_eq!(rig.keys(), [keys::TRANSLATE_FAILED]);
    assert_eq!(rig.view(), ChipsView::Hidden);
}

#[test]
fn a_selection_gone_by_the_click_is_never_read() {
    let mut rig = agreed(|_| {});
    rig.watch.set_still(Some(false));
    assert!(translate(&mut rig, "hello").is_empty());
    assert!(rig.reads().is_empty());
    assert_eq!(rig.keys(), [keys::TRANSLATE_FAILED]);
}
