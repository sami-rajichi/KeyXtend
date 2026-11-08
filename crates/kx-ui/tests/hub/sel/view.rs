//! The strip's JSON: each keycap's id, word and done word in the keyboard's language, worded again when that language
//! or the `[ui] language` setting changes, the legend as two flags, the strip's reader name and the selection screen's
//! scale.

use super::{heard, sel, spot, strip, ticked, usual, view};
use crate::Rig;
use kx_module_api::ports::{BASE_DPI, KeyboardFace, Legend, SelChip};
use kx_ui::i18n::keys::{SEL_NAME, SEL_WORD, sel_chip_id};
use kx_ui::{AUTO, I18n, UiSettings, Wake, find};
use serde_json::Value;
use std::sync::Arc;

/// The text of `key` in shipped language `tag`.
fn word(tag: &str, key: &str) -> String {
    I18n::new(find(tag).unwrap()).text(key)
}

#[test]
fn the_json_names_each_chip_with_its_word_and_done_word_the_legend_and_the_scale() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&view(1)).unwrap();
    let shown = sel(&rig);
    let chips = shown["chips"].as_array().unwrap();
    let ids: Vec<&str> = chips.iter().map(|c| c["id"].as_str().unwrap()).collect();
    assert_eq!(ids, SelChip::ALL.map(sel_chip_id), "Arabic flips text only");
    let words: Vec<&str> = chips.iter().map(|c| c["word"].as_str().unwrap()).collect();
    assert_eq!(words, ["Copy", "Cut", "Paste", "Translate"]);
    let done: Vec<&str> = chips.iter().map(|c| c["done"].as_str().unwrap()).collect();
    assert_eq!(done, ["Copied", "Cut", "Pasted", "Opening"]);
    assert_eq!(shown["name"], "Selection actions");
    assert_eq!(shown["acted"], Value::Null);
    assert_eq!(shown["scale"], 1.0);
    let scaled = strip(2, spot(100, 300, BASE_DPI * 5 / 4), None, Legend::Both);
    bus.publish(&scaled).unwrap();
    assert_eq!(sel(&rig)["scale"], 1.25, "the selection screen's scale");
}

#[test]
fn the_json_shows_icons_words_or_both_as_the_view_says() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    for (round, legend) in (1..).zip(Legend::ALL) {
        bus.publish(&strip(round, usual(), None, legend)).unwrap();
        let read = sel(&rig);
        let flags = (read["icons"].as_bool(), read["words"].as_bool());
        let want = (Some(legend.icons()), Some(legend.words()));
        assert_eq!(flags, want, "{legend:?}");
    }
}

#[test]
fn the_words_follow_the_keyboards_language() {
    for tag in ["fr", "ar"] {
        let (rig, _host) = crate::words::speaking(tag, 4);
        let (bus, _sent, _subs) = heard(&rig);
        bus.publish(&ticked(1, SelChip::Translate)).unwrap();
        let shown = sel(&rig);
        let want = SelChip::ALL.map(|c| word(tag, &format!("{SEL_WORD}{}", sel_chip_id(c))));
        let chips = shown["chips"].as_array().unwrap();
        let got: Vec<&str> = chips.iter().map(|c| c["word"].as_str().unwrap()).collect();
        assert_eq!(got, want, "{tag}");
        assert_eq!(shown["name"], word(tag, SEL_NAME), "{tag}");
    }
}

#[test]
fn a_new_language_rewords_a_kept_strip() {
    let (rig, face, _host) = crate::texts::rig("en", "en", AUTO);
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&view(1)).unwrap();
    assert_eq!(sel(&rig)["chips"][0]["word"], "Copy");
    rig.wakes();
    crate::texts::switch(&rig, &face, "fr");
    assert!(rig.wakes().contains(&Wake::Sel), "QML words it again");
    assert_eq!(sel(&rig)["chips"][0]["word"], "Copier");
}

#[test]
fn a_new_language_setting_rewords_a_kept_strip() {
    let (rig, face, mut host) = crate::texts::rig("en", "en", AUTO);
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&view(1)).unwrap();
    assert_eq!(sel(&rig)["chips"][0]["word"], "Copy");
    rig.wakes();
    let mut settings = UiSettings::from_defaults().unwrap();
    "fr".clone_into(&mut settings.language);
    let shown: Arc<dyn KeyboardFace> = face;
    rig.hub.attach(settings, shown, host.handle());
    assert!(rig.wakes().contains(&Wake::Sel), "a [ui] language change");
    assert_eq!(sel(&rig)["chips"][0]["word"], "Copier");
}
