//! Quick-fill's worded parts in each language: a failed fill's note, a place's last write, a found box, the waiting
//! footer, and the counts English and French read with one and other.

use super::{PLACE, when};
use crate::lang;
use kx_module_api::ports::{FillNote, NoteCounts};
use kx_ui::I18n;
use kx_ui::i18n::keys::{FILL_FOREIGN, FILL_FOUND, FILL_NOTE, FILL_VALUES, fill_note_id};

#[test]
fn a_failed_fill_says_it_did_not_finish_never_that_nothing_was_typed() {
    let key = format!("{FILL_NOTE}{}", fill_note_id(FillNote::NotTyped));
    let want = [
        ("en", "The fill did not finish. Check what was typed."),
        (
            "fr",
            "Le remplissage ne s’est pas terminé. Vérifiez le texte saisi.",
        ),
        ("ar", "لم تكتمل التعبئة. تحقّق مما كُتب."),
    ];
    for (tag, text) in want {
        assert_eq!(lang(tag).text(&key), text, "{tag}");
    }
    assert!(!lang("en").text(&key).to_lowercase().contains("nothing"));
}

#[test]
fn a_places_state_reads_today_yesterday_or_the_date() {
    let en = lang("en");
    assert_eq!(en.fill_updated(&when(0)), "Updated today 10:42");
    assert_eq!(en.fill_updated(&when(1)), "Updated yesterday 10:42");
    assert_eq!(en.fill_updated(&when(5)), "Updated 01/11/2026");
    let fr = lang("fr");
    assert_eq!(fr.fill_updated(&when(0)), "Sauvegardé aujourd’hui à 10:42");
    assert_eq!(fr.fill_updated(&when(9)), "Sauvegardé le 01/11/2026");
    let ar = lang("ar");
    assert_eq!(ar.fill_updated(&when(1)), "آخر تحديث أمس 10:42");
}

#[test]
fn a_found_box_names_its_date_and_place() {
    let found = lang("en").fill_found(FILL_FOUND, PLACE, &when(0));
    assert_eq!(
        found,
        "Found your backup from 01/11/2026 on OneDrive. Bring it back?"
    );
    let foreign = lang("ar").fill_found(FILL_FOREIGN, PLACE, &when(0));
    let want =
        "توجد في OneDrive نسخة احتياطية من كمبيوتر آخر بتاريخ 01/11/2026. هل تريد إضافة عناصرها؟";
    assert_eq!(foreign, want);
    let undated = lang("ar").fill_foreign_undated(PLACE);
    let want = "توجد في OneDrive نسخة احتياطية من كمبيوتر آخر. هل تريد إضافة عناصرها؟";
    assert_eq!(undated, want);
    assert_eq!(lang("en").fill_waiting("E:"), "Backup waiting for E:");
}

#[test]
fn english_and_french_count_with_one_and_other() {
    let (en, fr) = (lang("en"), lang("fr"));
    assert_eq!(en.fill_fields("Person", 6), "Person · 6 fields");
    assert_eq!(en.fill_fields("Note", 1), "Note · 1 field");
    assert_eq!(en.fill_fields("Other", 0), "Other · no fields");
    assert_eq!(fr.fill_fields("Personne", 1), "Personne · 1 champ");
    assert_eq!(en.counted(FILL_VALUES, 1), "1 value");
    assert_eq!(fr.counted(FILL_VALUES, 1_234), "1\u{202f}234 valeurs");
    let imported = |t: &I18n, added, skipped| t.fill_imported(NoteCounts { added, skipped });
    assert_eq!(
        imported(&en, 1, 0),
        "1 entry came in. Entries or values left out: 0."
    );
    assert_eq!(
        imported(&en, 12, 3),
        "12 entries came in. Entries or values left out: 3."
    );
    assert_eq!(
        imported(&fr, 12, 3),
        "12 éléments importés. Éléments ou valeurs ignorés\u{a0}: 3."
    );
}
