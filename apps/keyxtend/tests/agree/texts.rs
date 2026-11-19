//! Every name, label and tip the face draws, with the shortcuts layer hidden or on any page, every text a shortcut's
//! tip is made of, every mark it names, every module the app ships and the kernel, every language-key design and every
//! selection keycap, every voice option, bar place and cloud fact has its text in English, French and Arabic, and the
//! Dictation key says its name; every mark it names is one the theme's mark fonts are checked to draw.

use crate::fakes::{ports, tray};
use crate::{designs, pages, views};
use keyxtend::boot::modules;
use keyxtend::hidden::Run;
use kx_kernel::KERNEL;
use kx_mod_keyboard::legend::{HARAKA_NAMES, mark_names, shown};
use kx_mod_keyboard::spec::NEXT_ID;
use kx_mod_keyboard::view::{
    EMPTY_NAME, FILL_OFF_TIP, LABEL_PREFIX, LONE_TIP, NAME_PREFIX, NO_SHORTCUTS_TIP, POWER_OFF_TIP,
    PREP_NAME, SCROLL_OFF_TIP, SHORTCUT_NAME_PREFIX,
};
use kx_module_api::ports::{Fact, FactValue};
use kx_platform_fake::DEFAULT_LANGUAGE;
use kx_theme::assets::{MARK_CIRCLE, MARKS};
use kx_ui::i18n::keys::{
    COMBO_JOIN, COMBO_THEN, KEYS_TIP, LABEL, LANG_KEY, LAYER_PAGE, MODULE, VOICE_PREPARING,
    fact_key,
};
use kx_ui::{Hub, I18n, LANGS};
use std::collections::BTreeSet;

/// The keys of `wanted` some shipped language has no text for, with that language's tag.
pub(crate) fn missing(wanted: &BTreeSet<String>) -> Vec<(&'static str, &String)> {
    let langs = LANGS.iter().map(|l| (l.tag, I18n::new(l)));
    let gaps = langs.flat_map(|(tag, t)| {
        let absent: Vec<&String> = wanted.iter().filter(|k| !t.has(k)).collect();
        absent.into_iter().map(move |k| (tag, k))
    });
    gaps.collect()
}

#[test]
fn every_name_label_and_tip_the_face_draws_has_its_text() {
    let fixed = [
        LONE_TIP,
        EMPTY_NAME,
        SCROLL_OFF_TIP,
        NO_SHORTCUTS_TIP,
        POWER_OFF_TIP,
        FILL_OFF_TIP,
        PREP_NAME,
        KEYS_TIP,
        COMBO_JOIN,
        COMBO_THEN,
        LAYER_PAGE,
    ];
    let mut wanted: BTreeSet<String> = fixed.map(str::to_owned).into();
    for view in designs().into_iter().flat_map(views) {
        for k in &view.keys {
            let tip = k.tip.map(str::to_owned);
            wanted.extend([k.name.clone(), k.label.clone(), tip].into_iter().flatten());
        }
        wanted.insert(view.dpad.name.to_owned());
    }
    let named = wanted
        .iter()
        .filter(|k| k.starts_with(SHORTCUT_NAME_PREFIX));
    assert_eq!(named.count(), pages().concat().len(), "every shortcut");
    assert!(wanted.contains(&format!("{NAME_PREFIX}{NEXT_ID}")), "Next");
    assert_eq!(missing(&wanted), Vec::new());
}

#[test]
fn a_shortcuts_tip_reads_key_labels_from_the_group_the_board_names_them_in() {
    assert_eq!(format!("{LABEL}."), LABEL_PREFIX);
}

#[test]
fn the_dictation_keys_preparing_name_is_the_voice_texts_own() {
    assert_eq!(PREP_NAME, VOICE_PREPARING);
}

#[test]
fn every_mark_the_face_names_has_its_text() {
    let names = HARAKA_NAMES.map(|(c, _)| mark_names(&c.to_string()));
    let wanted: BTreeSet<String> = names.into_iter().flatten().collect();
    assert_eq!(wanted.len(), HARAKA_NAMES.len());
    assert_eq!(missing(&wanted), Vec::new());
}

#[test]
fn every_mark_the_face_names_is_one_the_mark_fonts_are_checked_for_on_their_circle() {
    let checked = |c: char| MARKS.iter().any(|r| r.contains(&c));
    for (c, id) in HARAKA_NAMES {
        assert!(checked(c), "{id} is not in kx-theme's MARKS");
    }
    for c in MARKS.into_iter().flatten() {
        let drawn = shown(&c.to_string());
        assert!(drawn.starts_with(MARK_CIRCLE), "U+{:04X}", u32::from(c));
    }
}

#[test]
fn every_shipped_module_and_the_kernel_have_a_name() {
    let (hub, _requests) = Hub::new(ports(tray())).unwrap();
    let ids = modules(&hub, DEFAULT_LANGUAGE, &Run::Normal)
        .into_iter()
        .map(|m| m.manifest().id)
        .chain([KERNEL]);
    let wanted: BTreeSet<String> = ids.map(|id| format!("{MODULE}.{id}")).collect();
    assert_eq!(missing(&wanted), Vec::new());
}

/// Every keycap the selection module's shipped settings can show has its word and done word, and the strip its reader
/// name (P10).
#[cfg(feature = "mod-selection")]
#[test]
fn every_keycap_the_selection_module_shows_has_its_texts() {
    use kx_mod_selection::{At, SelSettings, chips};
    use kx_module_api::ports::SelKind;
    use kx_ui::i18n::keys::{SEL_DONE, SEL_NAME, SEL_WORD, sel_chip_id};
    let rules = SelSettings::from_defaults().unwrap().rules(true);
    let places = [At::Selection, At::Caret { clip: true }];
    let shown = SelKind::ALL.into_iter().flat_map(|kind| {
        places
            .into_iter()
            .flat_map(move |at| chips(&rules, kind, at))
    });
    let named = shown.flat_map(|c| [SEL_WORD, SEL_DONE].map(|p| format!("{p}{}", sel_chip_id(c))));
    let wanted: BTreeSet<String> = named.chain([SEL_NAME.to_owned()]).collect();
    assert_eq!(
        wanted.len(),
        2 * kx_module_api::ports::SelChip::ALL.len() + 1,
        "every keycap shows"
    );
    assert_eq!(missing(&wanted), Vec::new());
}

#[test]
fn every_language_key_design_has_a_name() {
    let value = |d| serde_json::to_value(d).unwrap();
    let names = designs().map(|d| format!("{LANG_KEY}.{}", value(d).as_str().unwrap()));
    let wanted: BTreeSet<String> = names.into_iter().collect();
    assert_eq!(wanted.len(), designs().len());
    assert_eq!(missing(&wanted), Vec::new());
}

/// The voice module's Options rows, from the app's folder.
const VOICE_OPTIONS: &str = "../../crates/kx-mod-voice/options.toml";
/// An Options row's kind whose value is one of the bar's places.
const PLACE_KIND: &str = "place";

/// Each `(key, kind)` of the voice module's Options rows.
fn voice_rows() -> Vec<(String, String)> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(VOICE_OPTIONS);
    let table: toml::Table = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let field = |r: &toml::Value, k: &str| r[k].as_str().unwrap_or_default().to_owned();
    let rows = table["rows"].as_array().cloned().unwrap_or_default();
    rows.iter()
        .map(|r| (field(r, "key"), field(r, "kind")))
        .collect()
}

/// Every row of voice typing's Options page has its label, and each of the bar's places its name, as the setting
/// spells it (P11).
#[test]
fn every_voice_option_and_bar_place_has_its_text() {
    use kx_module_api::ports::BarPlace;
    use kx_ui::i18n::keys::{VOICE_OPTION, VOICE_PLACE, place_id};
    let rows = voice_rows();
    assert!(
        rows.iter().any(|(_, kind)| kind == PLACE_KIND),
        "the table is read"
    );
    let labels = rows
        .iter()
        .map(|(key, _)| format!("{VOICE_OPTION}{}", key.replace('_', "-")));
    let places = BarPlace::ALL.map(|p| {
        assert_eq!(
            serde_json::to_value(p).unwrap(),
            place_id(p),
            "the setting's name"
        );
        format!("{VOICE_PLACE}{}", place_id(p))
    });
    let wanted: BTreeSet<String> = labels.chain(places).collect();
    assert_eq!(missing(&wanted), Vec::new());
}

/// Every fact the services table writes on a card, with its group.
fn table_facts() -> Vec<(&'static str, Fact)> {
    use kx_ui::i18n::keys::{
        FACT_BILLS, FACT_FREE, FACT_KEEPS, FACT_PRICE, FACT_TRAINS, FACT_WARN, FACT_WHERE,
    };
    let table = kx_mod_voice::services::table().unwrap();
    let mut out = Vec::new();
    for f in table.rows().iter().map(|s| &s.facts) {
        let shown: [(&str, Option<&Fact>); 8] = [
            (FACT_WHERE, Some(&f.place)),
            (FACT_TRAINS, Some(&f.trains)),
            (FACT_KEEPS, Some(&f.keeps)),
            (FACT_FREE, Some(&f.free)),
            (FACT_PRICE, Some(&f.light)),
            (FACT_PRICE, Some(&f.heavy)),
            (FACT_WARN, f.warn.as_ref()),
            (FACT_BILLS, f.bills.as_ref()),
        ];
        out.extend(
            shown
                .into_iter()
                .filter_map(|(g, f)| f.map(|f| (g, f.clone()))),
        );
    }
    out
}

/// Every fact the services table writes on a card has its sentence in its group, so a card never shows a key (P11 cloud,
/// carry 2).
#[test]
fn every_fact_in_the_services_table_has_its_text() {
    let facts = table_facts();
    let wanted: BTreeSet<String> = facts.iter().map(|(g, f)| fact_key(g, &f.id)).collect();
    assert!(wanted.len() > 1, "the table is read");
    assert_eq!(missing(&wanted), Vec::new());
}

/// Every warning fact of the services table has its row's short mark, so a row never shows a key (P11 cloud).
#[test]
fn every_warning_fact_in_the_services_table_has_its_rows_mark() {
    use kx_ui::i18n::keys::{FACT_WARN, warn_mark_key};
    let facts = table_facts();
    let warns = facts.iter().filter(|(g, _)| *g == FACT_WARN);
    let wanted: BTreeSet<String> = warns.map(|(_, f)| warn_mark_key(&f.id)).collect();
    assert_eq!(missing(&wanted), Vec::new());
}

/// Every fact of the services table reads whole in each language once filled: no key, placeholder or marker is left,
/// and a service's own words end the sentence after a colon, as kx-ui's `own` checks for its samples.
#[test]
fn every_fact_in_the_services_table_reads_whole_in_every_language() {
    let facts = table_facts();
    let colon = |lead: &str| lead.trim_end().ends_with(':');
    for l in &LANGS {
        let t = I18n::new(l);
        for (group, fact) in &facts {
            let said = t.voice_fact(group, fact);
            let (key, text) = (fact_key(group, &fact.id), &said.text);
            let left =
                text.contains('{') || text.contains(&key) || text.chars().any(char::is_control);
            assert!(!left, "{} {key}: {text}", l.tag);
            let named = (fact.args.iter()).any(|a| matches!(a.value, FactValue::Text(_)));
            let last = matches!(&said.parts[..], [lead, own] if own.own && colon(&lead.text));
            assert!(!named || last, "{} {key}: {text}", l.tag);
        }
    }
}

/// The Dictation key and the name readers hear for it say the same word in every language (P11 design).
#[test]
fn the_dictation_key_says_the_same_word_as_its_name() {
    use kx_mod_keyboard::spec::Role;
    use kx_mod_keyboard::view::LABEL_PREFIX;
    use kx_mod_keyboard::{BoardSpec, LangKey};
    let roles = BoardSpec::shipped().unwrap().roles();
    let voice: Vec<String> = roles
        .into_iter()
        .filter(|r| r.1 == Role::Voice)
        .map(|r| r.0)
        .collect();
    assert_eq!(voice.len(), 1, "one Dictation key");
    let view = crate::board(LangKey::Cycle);
    let key = view.keys.iter().find(|k| k.id == voice[0]).unwrap();
    let name = key.name.clone().unwrap();
    let label = key
        .label
        .clone()
        .unwrap_or_else(|| format!("{LABEL_PREFIX}{}", key.id));
    for l in &LANGS {
        let t = I18n::new(l);
        assert_eq!(t.text(&name), t.text(&label), "{}", l.tag);
    }
}
