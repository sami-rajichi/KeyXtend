//! Nothing typed on a line is ever learned (ADR-0040 decision 6): no word and no email reaches a learner or a blob, and
//! no blob is even marked changed.

use super::{aimed, in_box, search};
use crate::rig::{Rig, saved};
use kx_mod_predict::email::Emails;
use kx_mod_predict::score::Learned;
use kx_mod_predict::{Msg, emails_blob};
use kx_module_api::ports::{FieldKind, PanelLine};

/// What a line holds through a run: words ended by spaces, then an email, open and ended.
const TEXTS: [&str; 4] = [
    "zebra ",
    "zebra quartz ",
    "alex@zebra.org",
    "alex@zebra.org ",
];

/// The emails `rig` stopped and saved.
fn kept_emails(rig: &mut Rig) -> Vec<String> {
    rig.stop();
    let bytes = rig.fakes.store.blob(&emails_blob().unwrap());
    bytes.map_or_else(Vec::new, |b| {
        Emails::from_bytes(&b, usize::MAX).unwrap().list().to_vec()
    })
}

#[test]
fn words_and_emails_on_a_search_mark_no_blob_changed_and_save_nothing() {
    let mut rig = aimed("");
    for text in TEXTS {
        rig.line(search(text));
        assert_eq!(rig.wait(), None, "no blob is marked changed: {text}");
    }
    rig.send(Msg::Pressed(false));
    assert_eq!(rig.wait(), None, "a click leaves nothing to learn");
    rig.line(PanelLine::None);
    assert_eq!(rig.wait(), None, "nor does the aim leaving");
    let saves = rig.fakes.store.saves();
    rig.stop();
    assert_eq!(
        rig.fakes.store.saves(),
        saves,
        "nothing to save at the stop"
    );
}

#[test]
fn a_word_on_a_search_never_reaches_the_learner_even_after_the_stop() {
    let mut rig = aimed("");
    for text in TEXTS {
        rig.line(search(text));
    }
    rig.typed("again ");
    rig.line(PanelLine::None);
    rig.typed("again ");
    rig.stop();
    let learner = saved(&rig.fakes, "en").unwrap();
    assert_eq!(learner.view(None).count("zebra"), 0);
    assert_eq!(learner.view(None).count("quartz"), 0);
    assert_eq!(learner.view(None).count("again"), 1, "the app's word only");
}

#[test]
fn an_email_on_a_search_never_saves_by_a_click_a_focus_or_the_aim_leaving() {
    let mut rig = aimed("alex@zebra.org");
    rig.send(Msg::Pressed(false));
    rig.focus(FieldKind::Text);
    rig.line(search("alex@zebra.org "));
    rig.line(PanelLine::None);
    assert!(kept_emails(&mut rig).is_empty());
}

#[test]
fn a_domain_chip_on_a_search_teaches_nothing_when_its_text_comes() {
    let mut rig = aimed("alex@");
    rig.pick(0);
    let (_, rest) = rig.typed_words().remove(0);
    rig.line(search(&format!("alex@{rest}")));
    rig.line(PanelLine::None);
    assert!(kept_emails(&mut rig).is_empty());
}

#[test]
fn the_same_text_as_a_late_piece_teaches_nothing() {
    let mut rig = aimed("alex@");
    rig.pick(0);
    let (_, rest) = rig.typed_words().remove(0);
    rig.typed(&rest);
    rig.line(search(&format!("alex@{rest}")));
    assert!(kept_emails(&mut rig).is_empty());
}

#[test]
fn an_email_left_open_in_the_app_is_learned_as_the_aim_moves_to_a_line() {
    let mut rig = in_box("en", &[], "");
    rig.typed("nour@y.com");
    rig.line(search(""));
    assert_ne!(rig.wait(), None, "the user left it, as with a click");
    rig.line(PanelLine::None);
    assert_eq!(kept_emails(&mut rig), ["nour@y.com"]);
}

#[test]
fn a_word_left_open_in_the_app_ends_unlearned_as_at_a_click() {
    let mut rig = in_box("en", &[], "");
    rig.typed("nou");
    rig.line(search(""));
    assert_eq!(rig.wait(), None);
    rig.fakes.field.set_before(Some("nou"));
    rig.line(PanelLine::None);
    rig.typed(" ");
    assert_eq!(rig.wait(), None, "read back, so it is never learned");
}
