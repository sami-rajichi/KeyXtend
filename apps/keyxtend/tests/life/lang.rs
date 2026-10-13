//! The language key, the space bar, the legends and the words on keys always show one layout: the one of the app in
//! front (ADR-0020).

use crate::rig::{Rig, char_key};
use kx_mod_layouts::{LayoutsSettings, label};
use kx_module_api::ports::{FaceAction, LayoutSource, Scan};
use kx_ui::{I18n, find};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// The top letter row's first key: `q` in English, `a` in French, `ض` in Arabic.
const TOP_LEFT: u16 = 0x10;
/// The Esc key's word: Esc in English and Arabic, Échap in French.
const ESC: &str = "label.esc";

#[test]
fn the_language_shown_and_the_legends_come_from_the_layout_in_front() {
    let rig = Rig::new();
    let mut boot = rig.load();
    boot.start().unwrap();
    let short = LayoutsSettings::from_defaults().unwrap().short;
    let key = char_key(TOP_LEFT);
    let ids = rig.layouts.installed();
    let mut words = BTreeSet::new();
    for &id in ids.iter().chain(ids.iter().rev()) {
        rig.layouts.set_foreground(Some(id));
        let _ = boot.hub.act(FaceAction::Relabel);
        let state: Value = serde_json::from_str(&boot.hub.state_json()).unwrap();
        let want = label(id, &rig.layouts.facts(id), &short);
        let typed = &rig.layouts.keys(id, &[Scan::new(TOP_LEFT)])[0].base;
        assert_eq!(state["lang"]["cur"]["short"], want.short.as_str(), "{id}");
        assert_eq!(state["lang"]["space"], json!(want.name), "{id}");
        assert_eq!(state["keys"][&key]["name"], typed.as_str(), "{id}");
        assert_eq!(
            state["keys"][&key]["main"],
            typed.as_str(),
            "{id}: the drawn legend"
        );
        let word = boot.hub.i18n().text(ESC);
        let own = I18n::new(find(&want.iso).unwrap()).text(ESC);
        assert_eq!(word, own, "{id}: the word on the key");
        words.insert(word);
    }
    assert!(words.len() > 1, "the words switch: {words:?}");
}
