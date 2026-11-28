//! The texts of Snip's editor, close question and panel (P12 design "The editor", "Close", "The Snip panel"): every key
//! they name in every language, the mock-up's words, and the question's title.

use crate::{all, lang};
use kx_module_api::ports::{
    AskKey, CropKey, EditKey, FolderKey, Ink, Keep, SnipKeyboard, SnipTab, Tool, VideoRow,
};
use kx_ui::i18n::keys::{
    SNIP_ANSWER, SNIP_ASK, SNIP_COLOUR, SNIP_CROP, SNIP_EDIT, SNIP_EDIT_OWN, SNIP_EDITOR, SNIP_INK,
    SNIP_LESS, SNIP_MORE, SNIP_PANEL_FOLDER, SNIP_PANEL_FOLDER_NONE, SNIP_PANEL_KEYBOARD,
    SNIP_PANEL_OFF, SNIP_PANEL_ON, SNIP_PANEL_ROW, SNIP_PANEL_SAY, SNIP_PANEL_TAB, SNIP_ROW_OWN,
    SNIP_STEPPED, SNIP_STRIP, SNIP_TEXT_SIZE, SNIP_THICK, SNIP_TOOL, SNIP_WHY, SNIP_WHYS,
    WINDOW_SNIP_ASK, snip_ask_id, snip_crop_id, snip_edit_id, snip_folder_id, snip_ink_id,
    snip_keep_id, snip_keyboard_id, snip_row_id, snip_tab_id, snip_tool_id,
};

/// Each of `ids` after `prefix`.
fn of<'a>(prefix: &'a str, ids: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    ids.into_iter().map(|id| format!("{prefix}{id}")).collect()
}

/// Every key the editor, the question and the panel name in `snip/*.ftl`.
pub fn keys() -> Vec<String> {
    let edit = EditKey::ALL
        .map(snip_edit_id)
        .into_iter()
        .chain(SNIP_EDIT_OWN);
    let rows = VideoRow::ALL
        .map(snip_row_id)
        .into_iter()
        .chain(SNIP_ROW_OWN);
    let choices = SnipKeyboard::ALL.map(snip_keyboard_id);
    let lists = [
        of(SNIP_TOOL, Tool::ALL.map(snip_tool_id)),
        of(SNIP_INK, Ink::ALL.map(snip_ink_id)),
        of(SNIP_EDIT, edit),
        of(SNIP_CROP, CropKey::ALL.map(snip_crop_id)),
        of(SNIP_LESS, SNIP_STEPPED),
        of(SNIP_MORE, SNIP_STEPPED),
        of(SNIP_WHY, SNIP_WHYS),
        of(SNIP_ANSWER, AskKey::ALL.map(snip_ask_id)),
        of(SNIP_PANEL_TAB, SnipTab::ALL.map(snip_tab_id)),
        of(SNIP_PANEL_ROW, rows),
        of(SNIP_PANEL_KEYBOARD, choices),
        of(SNIP_PANEL_SAY, choices),
        of(SNIP_PANEL_FOLDER, FolderKey::ALL.map(snip_folder_id)),
    ];
    let one = [
        SNIP_EDITOR,
        SNIP_COLOUR,
        SNIP_THICK,
        SNIP_TEXT_SIZE,
        SNIP_ASK,
        SNIP_PANEL_FOLDER_NONE,
        SNIP_PANEL_OFF,
        SNIP_PANEL_ON,
    ];
    let all = lists.into_iter().flatten();
    all.chain(one.map(str::to_owned)).collect()
}

/// The editor's tools as `design/snip-text.js` names them in English.
const TOOLS_EN: [&str; 11] = [
    "Crop",
    "Pen",
    "Highlighter",
    "Box",
    "Circle",
    "Arrow",
    "Line",
    "Text",
    "Cover",
    "Steps",
    "Eraser",
];
/// Its first nine tools in French.
const TOOLS_FR: [&str; 9] = [
    "Rogner",
    "Stylo",
    "Surligneur",
    "Rectangle",
    "Cercle",
    "Flèche",
    "Ligne",
    "Texte",
    "Masque",
];
/// Its tools in Arabic.
const TOOLS_AR: [&str; 11] = [
    "اقتصاص",
    "قلم",
    "قلم تمييز",
    "مستطيل",
    "دائرة",
    "سهم",
    "خط",
    "نص",
    "إخفاء",
    "ترقيم",
    "ممحاة",
];

#[test]
fn the_editor_question_and_panel_say_the_mock_ups_words() {
    let words =
        |tag: &str| Tool::ALL.map(|t| lang(tag).text(&format!("{SNIP_TOOL}{}", snip_tool_id(t))));
    assert_eq!(words("en"), TOOLS_EN, "design/snip-text.js ed");
    assert_eq!(words("fr")[..9], TOOLS_FR, "design/snip-text.js ed");
    assert_eq!(words("ar"), TOOLS_AR, "design/snip-text.js ed");
    let ask = |tag: &str| lang(tag).text(SNIP_ASK);
    assert_eq!(ask("en"), "This snip is not saved. Save it before closing?");
    assert_eq!(ask("ar"), "هذه اللقطة غير محفوظة. هل تريد حفظها قبل الإغلاق؟");
    assert_eq!(
        lang("fr").text(&format!("{SNIP_ANSWER}drop")),
        "Ne pas enregistrer"
    );
    assert_eq!(
        lang("ar").text(&format!("{SNIP_PANEL_KEYBOARD}hidden")),
        "مخفية"
    );
    let why = |w: &str| lang("en").text(&format!("{SNIP_WHY}{w}"));
    assert_eq!(why("least"), "It can't go any lower.");
    assert_eq!(why("most"), "It can't go any higher.");
}

#[test]
fn crops_keys_say_apply_and_cancel_in_words_no_other_key_uses() {
    let words = |tag: &str| {
        let t = lang(tag);
        CropKey::ALL.map(|k| t.text(&format!("{SNIP_CROP}{}", snip_crop_id(k))))
    };
    assert_eq!(words("en"), ["Apply", "Cancel"], "the owner's words");
    assert_eq!(
        words("fr"),
        ["Appliquer", "Abandonner"],
        "Undo keeps Annuler"
    );
    assert_eq!(words("ar"), ["تطبيق", "إلغاء"]);
}

#[test]
fn no_two_of_the_editors_keys_share_a_name_for_readers_or_voice() {
    let keys = [
        of(SNIP_TOOL, Tool::ALL.map(snip_tool_id)),
        of(
            SNIP_EDIT,
            EditKey::ALL
                .map(snip_edit_id)
                .into_iter()
                .chain(SNIP_EDIT_OWN),
        ),
        of(SNIP_CROP, CropKey::ALL.map(snip_crop_id)),
        of(SNIP_STRIP, Keep::ALL.map(snip_keep_id)),
        of(SNIP_INK, Ink::ALL.map(snip_ink_id)),
    ];
    for t in all() {
        let mut names: Vec<String> = keys.iter().flatten().map(|k| t.text(k)).collect();
        names.sort();
        let twins: Vec<&[String]> = names.windows(2).filter(|w| w[0] == w[1]).collect();
        assert!(twins.is_empty(), "{}: {twins:?}", t.lang().tag);
    }
}

#[test]
fn the_panels_texts_are_one_family() {
    let panel = [
        SNIP_PANEL_TAB,
        SNIP_PANEL_ROW,
        SNIP_PANEL_KEYBOARD,
        SNIP_PANEL_SAY,
        SNIP_PANEL_FOLDER,
        SNIP_PANEL_FOLDER_NONE,
        SNIP_PANEL_OFF,
        SNIP_PANEL_ON,
    ];
    let stray: Vec<&str> = panel
        .into_iter()
        .filter(|k| !k.starts_with("snip.panel-"))
        .collect();
    assert!(stray.is_empty(), "the plan's snip.panel-*: {stray:?}");
}

#[test]
fn each_reason_is_one_sentence() {
    for t in all() {
        for why in SNIP_WHYS {
            let said = t.text(&format!("{SNIP_WHY}{why}"));
            let sentences = said
                .split(['.', '؟', '?'])
                .filter(|s| !s.trim().is_empty())
                .count();
            assert_eq!(sentences, 1, "{}: {said}", t.lang().tag);
        }
        assert!(!t.text(WINDOW_SNIP_ASK).is_empty());
    }
}
