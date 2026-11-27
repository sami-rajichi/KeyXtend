//! Snip's texts (P12 design "What the user sees"): every key, hint, reason and note its capture windows name in every
//! language, the windows' titles and the tray row, and no key in Snip's files that the code does not name.

use crate::files::keys_of;
use crate::{all, lang};
use kx_module_api::ports::{BarKey, Dim, Handle, Keep, RecKey, SnipNote, StripKey};
use kx_ui::LANGS;
use kx_ui::i18n::keys::{
    SNIP_COUNT, SNIP_DIM, SNIP_DONE, SNIP_HANDLE, SNIP_HINT, SNIP_HINT_CORNER, SNIP_JOIN,
    SNIP_LENGTH, SNIP_LIVE, SNIP_NOTE, SNIP_REC, SNIP_SIZE, SNIP_SPEED_TIMES, SNIP_STRIP,
    SNIP_STRIP_PLAY_SAY, SNIP_STRIP_WAIT, SNIP_TIME, SNIP_WAY, SNIP_WINDOWS, TRAY_SNIP,
    snip_bar_id, snip_dim_id, snip_handle_id, snip_keep_id, snip_note_id, snip_rec_id,
    snip_strip_id,
};
use std::collections::BTreeSet;

/// The ways a hint tells how to pick: every bar key but Settings and Close.
fn ways() -> impl Iterator<Item = BarKey> {
    BarKey::ALL
        .into_iter()
        .filter(|k| !matches!(k, BarKey::Settings | BarKey::Close))
}

/// Each of `ids` after `prefix`.
fn of(prefix: &str, ids: impl IntoIterator<Item = &'static str>) -> Vec<String> {
    ids.into_iter().map(|id| format!("{prefix}{id}")).collect()
}

/// Every key Snip's capture windows name in `snip/*.ftl`.
fn keys() -> BTreeSet<String> {
    let bar = of(SNIP_WAY, BarKey::ALL.map(snip_bar_id));
    let hints = of(SNIP_HINT, ways().map(snip_bar_id));
    let strip = of(SNIP_STRIP, StripKey::ALL.map(snip_strip_id));
    let rec = of(SNIP_REC, RecKey::ALL.map(snip_rec_id));
    let done = of(SNIP_DONE, Keep::ALL.map(snip_keep_id));
    let dims = of(SNIP_DIM, Dim::ALL.map(snip_dim_id));
    let notes = of(SNIP_NOTE, SnipNote::ALL.map(snip_note_id));
    let handles = of(SNIP_HANDLE, Handle::ALL.map(snip_handle_id));
    let one = [
        SNIP_HINT_CORNER,
        SNIP_LENGTH,
        SNIP_COUNT,
        SNIP_LIVE,
        SNIP_JOIN,
        SNIP_TIME,
        SNIP_SIZE,
        SNIP_SPEED_TIMES,
        SNIP_STRIP_PLAY_SAY,
        SNIP_STRIP_WAIT,
    ];
    let all = [
        bar,
        hints,
        strip,
        rec,
        done,
        dims,
        notes,
        handles,
        crate::snipedit::keys(),
    ]
    .into_iter()
    .flatten();
    all.chain(one.map(str::to_owned)).collect()
}

#[test]
fn every_snip_key_title_and_the_tray_row_have_their_text_in_every_language() {
    let titles = SNIP_WINDOWS
        .into_iter()
        .chain([TRAY_SNIP])
        .map(str::to_owned);
    for t in all() {
        let all = keys().into_iter().chain(titles.clone());
        let missing: Vec<String> = all.filter(|k| !t.has(k)).collect();
        assert!(missing.is_empty(), "{}: {missing:?}", t.lang().tag);
    }
}

#[test]
fn snips_files_hold_only_the_keys_the_code_names() {
    let group = SNIP_WAY.split_once('.').map(|(g, _)| g).unwrap_or_default();
    for l in &LANGS {
        let named: BTreeSet<String> = l.files().flat_map(keys_of).collect();
        let ours: BTreeSet<String> = named
            .into_iter()
            .filter(|k| k.split_once('.').is_some_and(|(g, _)| g == group))
            .collect();
        assert_eq!(ours, keys(), "{}", l.tag);
    }
}

#[test]
fn each_language_says_the_mock_ups_words() {
    let words = |tag: &str| {
        let t = lang(tag);
        BarKey::ALL.map(|k| t.text(&format!("{SNIP_WAY}{}", snip_bar_id(k))))
    };
    let en = [
        "Region",
        "Window",
        "Full screen",
        "Video region",
        "Video full screen",
        "Snip settings",
        "Close",
    ];
    assert_eq!(words("en"), en, "design/snip-text.js");
    let fr = [
        "Zone",
        "Fenêtre",
        "Plein écran",
        "Vidéo d’une zone",
        "Vidéo plein écran",
        "Réglages de la capture",
    ];
    assert_eq!(words("fr")[..6], fr, "design/snip-text.js");
    let ar = [
        "منطقة",
        "نافذة",
        "ملء الشاشة",
        "فيديو لمنطقة",
        "فيديو بملء الشاشة",
        "إعدادات اللقطة",
        "إغلاق",
    ];
    assert_eq!(words("ar"), ar, "design/snip-text.js");
    assert_eq!(
        lang("ar").text(&format!("{SNIP_STRIP}record")),
        "بدء التسجيل"
    );
    assert_eq!(lang("fr").text(&format!("{SNIP_DONE}copy")), "Copié");
}

#[test]
fn a_videos_strip_says_play_and_speed_as_the_mock_ups_video_row_did_with_the_speed_after() {
    let words = |tag: &str| ["play", "speed"].map(|k| lang(tag).text(&format!("{SNIP_STRIP}{k}")));
    assert_eq!(words("en"), ["Play", "Speed"], "design/snip-text.js ed");
    assert_eq!(words("fr"), ["Lire", "Vitesse"], "design/snip-text.js ed");
    assert_eq!(words("ar"), ["تشغيل", "السرعة"], "design/snip-text.js ed");
    let say = |tag: &str| lang(tag).text(SNIP_STRIP_PLAY_SAY);
    assert_eq!(say("en"), "Play opens the video in your own video app.");
    assert_eq!(
        say("fr"),
        "«\u{a0}Lire\u{a0}» ouvre la vidéo dans votre propre lecteur vidéo."
    );
    assert_eq!(say("ar"), "يفتح زر «تشغيل» الفيديو في مشغّل الفيديو لديك.");
    for t in all() {
        assert_eq!(
            t.text(SNIP_SPEED_TIMES),
            "%1×",
            "{}: n× in every language",
            t.lang().tag
        );
    }
}

#[test]
fn the_time_and_size_keep_their_marks_and_each_note_is_two_sentences_at_most() {
    for t in all() {
        assert_eq!(
            t.text(SNIP_TIME),
            "%1:%2",
            "{}: minutes then seconds, left to right",
            t.lang().tag
        );
        assert!(t.text(SNIP_SIZE).starts_with("%1") && t.text(SNIP_SIZE).ends_with("%2"));
        for note in SnipNote::ALL.map(snip_note_id) {
            let said = t.text(&format!("{SNIP_NOTE}{note}"));
            let sentences = said.split('.').filter(|s| !s.trim().is_empty()).count();
            assert!(sentences <= 2, "{}: {said}", t.lang().tag);
        }
    }
}

#[test]
fn each_language_names_the_frames_handles_by_the_screens_own_sides() {
    let words = |tag: &str| {
        let t = lang(tag);
        Handle::ALL.map(|h| t.text(&format!("{SNIP_HANDLE}{}", snip_handle_id(h))))
    };
    let en = [
        "Top-left corner",
        "Top edge",
        "Top-right corner",
        "Right edge",
        "Bottom-right corner",
        "Bottom edge",
        "Bottom-left corner",
        "Left edge",
    ];
    assert_eq!(words("en"), en);
    let fr = [
        "Coin supérieur gauche",
        "Bord supérieur",
        "Coin supérieur droit",
        "Bord droit",
        "Coin inférieur droit",
        "Bord inférieur",
        "Coin inférieur gauche",
        "Bord gauche",
    ];
    assert_eq!(words("fr"), fr);
    // The screen is not mirrored in Arabic, so left stays the screen's left.
    let ar = [
        "الزاوية العلوية اليسرى",
        "الحافة العلوية",
        "الزاوية العلوية اليمنى",
        "الحافة اليمنى",
        "الزاوية السفلية اليمنى",
        "الحافة السفلية",
        "الزاوية السفلية اليسرى",
        "الحافة اليسرى",
    ];
    assert_eq!(words("ar"), ar);
}
