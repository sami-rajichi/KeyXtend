//! The key page's line buttons in each language (ADR-0049): each Save and Paste reads short and names its line for
//! readers, and an empty Paste says the clipboard holds no text.

use super::cloud::read_as_written;

#[test]
fn each_lines_save_reads_short_and_names_its_line_for_readers() {
    read_as_written(&[
        ("cloud.line-save", ["Save", "Enregistrer", "حفظ"]),
        (
            "cloud.line-save-key",
            ["Save the key", "Enregistrer la clé", "حفظ المفتاح"],
        ),
        (
            "cloud.line-save-address",
            ["Save the address", "Enregistrer l’adresse", "حفظ العنوان"],
        ),
        (
            "cloud.line-save-model",
            ["Save the model", "Enregistrer le modèle", "حفظ النموذج"],
        ),
    ]);
}

#[test]
fn each_lines_paste_names_its_line_for_readers_and_an_empty_paste_says_so() {
    read_as_written(&[
        (
            "cloud.line-paste-key",
            ["Paste the key", "Coller la clé", "لصق المفتاح"],
        ),
        (
            "cloud.line-paste-address",
            ["Paste the address", "Coller l’adresse", "لصق العنوان"],
        ),
        (
            "cloud.line-paste-model",
            ["Paste the model", "Coller le modèle", "لصق النموذج"],
        ),
        (
            "cloud.paste-empty",
            [
                "The clipboard holds no text.",
                "Le presse-papiers ne contient aucun texte.",
                "لا يوجد نص في الحافظة.",
            ],
        ),
    ]);
}
