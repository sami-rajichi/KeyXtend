//! The language in front as the panel names it: a title on the chip, French in lower case inside a sentence, Arabic
//! after «للغة» (owner, P11).

use super::{coded, heard, home, json, page, panel, panel_json};
use crate::texts::rig;
use kx_module_api::ports::{Listening, ModelSide, ModelsSheet, VoiceBar, VoiceLang, VoiceSheet};
use serde_json::Value;

/// The slow line for a language in front named `name` with code `iso`, on words in `setting` over a layout in `layout`.
fn slow_line(layout: &str, setting: &str, iso: &str, name: &str) -> Value {
    let (rig, _face, _host) = rig("en", layout, setting);
    let heard = heard(&rig);
    let slow = ModelsSheet {
        rows: vec![],
        best: None,
        untested: false,
        slow: true,
        folder: None,
        offer: None,
        side: ModelSide::Local,
        band: None,
    };
    let mut shown = panel(VoiceSheet::Models(slow));
    shown.lang = coded(iso);
    name.clone_into(&mut shown.lang.name);
    panel_json(&rig, &heard, shown)["sheet"]["slow"]["text"].clone()
}

#[test]
fn the_slow_line_names_the_language_in_the_words_own_way() {
    let fr = "Les modèles locaux en français seront très lents sur ce PC.";
    assert_eq!(slow_line("fr", "auto", "fr", "Français"), fr);
    let pinned = "Les modèles locaux en anglais seront très lents sur ce PC.";
    assert_eq!(slow_line("en", "fr", "en", "English"), pinned);
    let ar = "ستكون النماذج المحلية للغة الإنجليزية بطيئة جدًا على هذا الحاسوب.";
    assert_eq!(slow_line("en", "ar", "en", "English"), ar);
    let own = "ستكون النماذج المحلية للغة العربية بطيئة جدًا على هذا الحاسوب.";
    assert_eq!(slow_line("ar", "auto", "ar", "العربية"), own);
}

/// The language `name` with code `iso`, as the system names it.
fn system(iso: &str, name: &str) -> VoiceLang {
    let mut lang = coded(iso);
    name.clone_into(&mut lang.name);
    lang
}

/// The chip's name on the bar and the panel for `lang`, on words in `setting` over a layout in `layout`.
fn chips(layout: &str, setting: &str, lang: &VoiceLang) -> [Value; 2] {
    let (rig, _face, _host) = rig("en", layout, setting);
    let heard = heard(&rig);
    let bar = VoiceBar::Listening(Listening {
        lang: lang.clone(),
        elapsed_ms: 0,
        nothing: false,
        wave_ms: 0,
        warn: None,
    });
    heard.view(page(1, bar, None));
    let on_bar = json(&rig)["bar"]["lang"]["name"].clone();
    let mut shown = panel(home());
    shown.lang = lang.clone();
    [
        on_bar,
        panel_json(&rig, &heard, shown)["lang"]["name"].clone(),
    ]
}

#[test]
fn the_chip_names_the_language_as_a_title_and_keeps_the_systems_name_for_others() {
    let french = system("fr", "français");
    assert_eq!(chips("fr", "auto", &french), ["Français", "Français"]);
    let german = system("de", "Deutsch");
    assert_eq!(chips("en", "auto", &german), ["Deutsch", "Deutsch"]);
    let arabic = system("ar", "العربية");
    assert_eq!(chips("ar", "auto", &arabic), ["العربية", "العربية"]);
}
