//! The language choice, the dot rule, the fallback and missing keys, on small test files.

use super::*;

/// A small English file: a plain message, a group and a text with an argument.
static EN: Lang = Lang {
    tag: "en",
    rtl: false,
    ftl: "plain = Plain\ngroup =\n    .name = Named\n    .only-en = English only\nhello = Hello { $who }\n",
};

/// A small French file that lacks `group.only-en` and has a broken line before its last entry.
static FR: Lang = Lang {
    tag: "fr",
    rtl: false,
    ftl: "plain = Simple\ngroup =\n    .name = Nommé\nbroken line\nafter = Après\n",
};

#[test]
fn auto_follows_the_windows_language() {
    assert_eq!(pick(AUTO, "fr-FR").tag, "fr");
    assert_eq!(pick(AUTO, "ar-TN").tag, "ar");
    assert_eq!(pick(AUTO, "en-GB").tag, "en");
}

#[test]
fn auto_falls_back_to_english() {
    for os in ["de-DE", "zh-Hant-TW", "", "not a tag"] {
        assert_eq!(pick(AUTO, os).tag, "en", "{os:?}");
    }
}

#[test]
fn auto_reads_odd_but_real_tags() {
    assert_eq!(pick(AUTO, "fr_CA").tag, "fr");
    assert_eq!(pick(AUTO, "AR-tn").tag, "ar");
    assert_eq!(pick(AUTO, "fr-FR-u-ca-gregory").tag, "fr");
}

#[test]
fn a_supported_setting_wins_over_windows() {
    assert_eq!(pick("ar", "fr-FR").tag, "ar");
    assert_eq!(pick("en", "fr-FR").tag, "en");
}

#[test]
fn an_unknown_setting_follows_windows() {
    assert_eq!(pick("xx", "fr-FR").tag, "fr");
    assert_eq!(pick("", "ar-TN").tag, "ar");
}

#[test]
fn the_table_starts_with_english_and_marks_arabic_right_to_left() {
    let tags: Vec<&str> = LANGS.iter().map(|l| l.tag).collect();
    assert_eq!(tags, ["en", "fr", "ar"]);
    assert!(LANGS.iter().all(|l| l.rtl == (l.tag == "ar")));
    assert_eq!(find("fr").map(|l| l.tag), Some("fr"));
    assert!(find(AUTO).is_none());
}

#[test]
fn a_dotted_key_is_an_attribute_and_a_plain_key_a_message() {
    let t = I18n::of(&EN, &EN);
    assert_eq!(t.text("plain"), "Plain");
    assert_eq!(t.text("group.name"), "Named");
}

#[test]
fn a_group_without_its_own_text_shows_its_key() {
    let t = I18n::of(&EN, &EN);
    assert_eq!(t.text("group"), "group");
    assert_eq!(t.text("group.nothing"), "group.nothing");
    assert_eq!(t.text("group.name.deeper"), "group.name.deeper");
}

#[test]
fn a_key_the_language_lacks_comes_from_english() {
    let t = I18n::of(&FR, &EN);
    assert_eq!(t.text("group.name"), "Nommé");
    assert_eq!(t.text("group.only-en"), "English only");
    assert!(t.has("group.name") && !t.has("group.only-en"));
    assert_eq!(t.lang().tag, "fr");
}

#[test]
fn a_key_missing_everywhere_shows_itself_and_a_name_its_id() {
    let t = I18n::of(&FR, &EN);
    assert_eq!(t.text("nowhere.at-all"), "nowhere.at-all");
    assert_eq!(t.name_of("module", "mystery"), "mystery");
    assert_eq!(t.name_of("group", "name"), "Nommé");
}

#[test]
fn a_broken_line_leaves_the_other_texts() {
    let t = I18n::of(&FR, &EN);
    assert_eq!(t.text("after"), "Après");
    assert_eq!(t.text("plain"), "Simple");
}

#[test]
fn a_missing_argument_never_panics() {
    let t = I18n::of(&EN, &EN);
    assert!(t.text("hello").starts_with("Hello"));
}

#[test]
fn texts_can_move_between_threads() {
    fn shared<T: Send + Sync>() {}
    shared::<I18n>();
}
