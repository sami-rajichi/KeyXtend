//! Every name, label and tip the face draws, every mark it names, every module the app ships and the kernel, and
//! every language-key design has its text in English, French and Arabic; every mark it names is one the theme's mark
//! fonts are checked to draw.

use crate::fakes::{ports, tray};
use crate::{board, designs};
use keyxtend::boot::modules;
use kx_kernel::KERNEL;
use kx_mod_keyboard::legend::{HARAKA_NAMES, mark_names, shown};
use kx_mod_keyboard::view::{EMPTY_NAME, LONE_TIP, SCROLL_OFF_TIP};
use kx_theme::assets::{MARK_CIRCLE, MARKS};
use kx_ui::i18n::keys::{LANG_KEY, MODULE};
use kx_ui::{Hub, I18n, LANGS};
use std::collections::BTreeSet;

/// The keys of `wanted` some shipped language has no text for, with that language's tag.
fn missing(wanted: &BTreeSet<String>) -> Vec<(&'static str, &String)> {
    let langs = LANGS.iter().map(|l| (l.tag, I18n::new(l)));
    let gaps = langs.flat_map(|(tag, t)| {
        let absent: Vec<&String> = wanted.iter().filter(|k| !t.has(k)).collect();
        absent.into_iter().map(move |k| (tag, k))
    });
    gaps.collect()
}

#[test]
fn every_name_label_and_tip_the_face_draws_has_its_text() {
    let fixed = [LONE_TIP, EMPTY_NAME, SCROLL_OFF_TIP];
    let mut wanted: BTreeSet<String> = fixed.map(str::to_owned).into();
    for design in designs() {
        let view = board(design);
        for k in &view.keys {
            let tip = k.tip.map(str::to_owned);
            wanted.extend([k.name.clone(), k.label.clone(), tip].into_iter().flatten());
        }
        wanted.insert(view.dpad.name.to_owned());
    }
    assert!(
        wanted.len() > LANGS.len(),
        "the board names its keys: {wanted:?}"
    );
    assert_eq!(missing(&wanted), Vec::new());
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
    let ids = modules(&hub)
        .into_iter()
        .map(|m| m.manifest().id)
        .chain([KERNEL]);
    let wanted: BTreeSet<String> = ids.map(|id| format!("{MODULE}.{id}")).collect();
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
