//! The voice port's newer notes and lines in each language: a bad download, a name another file holds, a file still
//! checked, a failed Compare run, the language note and the slow-PC line.

use super::named;
use crate::lang;
use kx_module_api::ports::BarHint;
use kx_ui::i18n::keys::vars::LANG;
use kx_ui::i18n::keys::{VOICE_SLOW, VOICE_WORD};

#[test]
fn the_ports_newer_notes_and_lines_read_as_written_in_each_language() {
    let table = [
        (
            "note-print",
            [
                "A download did not pass its check, so it was deleted.",
                "Un téléchargement n’a pas réussi sa vérification\u{a0}; il a été supprimé.",
                "لم يجتز الملف المُنزَّل الفحص، فحُذف.",
            ],
        ),
        (
            "note-taken",
            [
                "Another file in the models folder has this model’s name, so nothing was downloaded.",
                "Un autre fichier du dossier des modèles porte déjà ce nom\u{a0}; rien n’a été téléchargé.",
                "يحمل ملف آخر في مجلد النماذج اسم هذا النموذج، فلم يُنزَّل شيء.",
            ],
        ),
        (
            "note-checking",
            [
                "The chosen file is still being checked. Try again in a moment.",
                "Le fichier choisi est encore en cours de vérification. Réessayez dans un instant.",
                "ما زال الملف المختار قيد الفحص. حاول مرة أخرى بعد قليل.",
            ],
        ),
        ("mark-run-failed", ["Failed", "Échec", "فشل"]),
    ];
    for (id, words) in table {
        for (tag, word) in ["en", "fr", "ar"].into_iter().zip(words) {
            let got = lang(tag).text(&format!("{VOICE_WORD}{id}"));
            assert_eq!(got, word, "{tag} {id}");
        }
    }
}

#[test]
fn the_language_note_and_the_slow_line_name_the_language() {
    let named_in = [("en", "English"), ("fr", "français"), ("ar", "العربية")];
    let stranger = [
        "The chosen model does not know English. Pick another in the settings.",
        "Le modèle choisi n’écrit pas en français. Choisissez-en un autre dans les réglages.",
        "النموذج المختار لا يدعم اللغة العربية. اختر نموذجًا آخر من الإعدادات.",
    ];
    let slow = [
        "Local models for English will be very slow on this PC.",
        "Les modèles locaux en français seront très lents sur ce PC.",
        "ستكون النماذج المحلية للغة العربية بطيئة جدًا على هذا الحاسوب.",
    ];
    for (((tag, name), stranger), slow) in named_in.into_iter().zip(stranger).zip(slow) {
        let t = lang(tag);
        assert_eq!(t.voice_hint(&BarHint::Language(named(name))), stranger);
        assert_eq!(t.voice_with(VOICE_SLOW, &[(LANG, name.to_owned())]), slow);
    }
}

#[test]
fn a_stopped_typing_and_a_refused_copy_read_as_written_in_each_language() {
    let table = [
        (
            BarHint::Stopped,
            [
                "Typing stopped part way. Copy the whole text from here or from History.",
                "La saisie s’est arrêtée en cours de route. Copiez le texte entier ici ou depuis l’historique.",
                "توقفت كتابة النص قبل اكتمالها. انسخه كاملًا من هنا أو من السجل.",
            ],
        ),
        (
            BarHint::CopyFailed,
            [
                "The text could not be copied. Try again.",
                "Le texte n’a pas pu être copié. Réessayez.",
                "تعذّر نسخ النص. حاول مرة أخرى.",
            ],
        ),
    ];
    for (hint, words) in table {
        for (tag, word) in ["en", "fr", "ar"].into_iter().zip(words) {
            assert_eq!(lang(tag).voice_hint(&hint), word, "{tag} {hint:?}");
        }
    }
}
