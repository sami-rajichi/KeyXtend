//! Snip's wording as the app's other texts say it (Task 21 review): Arabic clicks "on" a target and counts down
//! "after", French says "boutons" and its own spaces, and English keeps straight apostrophes.

use crate::lang;
use kx_ui::i18n::keys::{
    SNIP_ANSWER, SNIP_COUNT, SNIP_DIM, SNIP_HINT, SNIP_HINT_CORNER, SNIP_LESS, SNIP_MORE,
    SNIP_NOTE, SNIP_PANEL_ROW,
};

/// The ways a hint names, each after `SNIP_HINT`.
const HINTS: [&str; 5] = ["region", "window", "full", "video-region", "video-full"];
/// How the app's Arabic asks for a click on something (fill/ar.ftl: "انقر على «جديد»").
const AR_CLICK_ON: [&str; 2] = ["انقر على ", "انقر الآن على "];

#[test]
fn an_arabic_hint_clicks_on_its_target_and_the_countdown_starts_after() {
    let ar = lang("ar");
    let mut hints: Vec<String> = HINTS.map(|w| ar.text(&format!("{SNIP_HINT}{w}"))).into();
    hints.push(ar.text(SNIP_HINT_CORNER));
    for said in hints {
        assert!(AR_CLICK_ON.iter().any(|on| said.starts_with(on)), "{said}");
    }
    assert_eq!(ar.text(SNIP_COUNT), "يبدأ التسجيل بعد");
}

#[test]
fn arabic_names_the_session_and_each_step_without_doubt() {
    let ar = lang("ar");
    assert_eq!(
        ar.text(&format!("{SNIP_DIM}hidden")),
        "هذه الجلسة لا تحفظ أي ملفات، لذا لا يمكن حفظ اللقطة."
    );
    let thick = [SNIP_LESS, SNIP_MORE].map(|p| ar.text(&format!("{p}thick")));
    assert_eq!(thick, ["أقل سُمكًا", "أكثر سُمكًا"]);
    let rows = ["folder", "longest"].map(|r| ar.text(&format!("{SNIP_PANEL_ROW}{r}")));
    assert_eq!(rows, ["تُفتح نافذة الحفظ في", "أقصى مدة للفيديو (بالدقائق)"]);
}

#[test]
fn french_keeps_buttons_out_of_videos() {
    let fr = lang("fr");
    assert_eq!(
        fr.text(&format!("{SNIP_DIM}bare")),
        "Sur ce PC, la capture ne peut pas garder ses propres boutons hors des vidéos."
    );
    let bare = fr.text(&format!("{SNIP_NOTE}bare"));
    assert!(
        bare.starts_with(
            "La capture n’a pas pu garder ses boutons hors de la vidéo et s’est arrêtée."
        ),
        "{bare}"
    );
    assert_eq!(
        fr.text(&format!("{SNIP_NOTE}speed-failed")),
        "La vitesse de la vidéo n’a pas pu être changée\u{a0}; elle reste telle quelle."
    );
}

#[test]
fn english_dont_save_takes_a_straight_apostrophe_as_the_rest() {
    assert_eq!(lang("en").text(&format!("{SNIP_ANSWER}drop")), "Don't save");
}
