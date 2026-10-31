//! The clipboard panel's texts: every key in every language, sizes with the language's decimal mark, the storage line,
//! a row's time and meta line, and the plural forms, Arabic's six included (ADR-0017).

mod when;

use crate::{CONTROLS, all, lang};
use kx_module_api::ports::{ClipKind, MB, PanelNote, Room, Weekday};
use kx_ui::I18n;
use kx_ui::i18n::keys::{
    CLIP_BACK, CLIP_CHARS, CLIP_CLOSE, CLIP_CONFIRM, CLIP_DATE, CLIP_DAY, CLIP_DELETE,
    CLIP_DELETE_ALL, CLIP_EMPTY, CLIP_EXPIRES, CLIP_FILTER, CLIP_FILTERS, CLIP_INSERT,
    CLIP_INSERT_ONE, CLIP_JOIN, CLIP_KEPT, CLIP_KIND, CLIP_META, CLIP_META_BARE, CLIP_NO,
    CLIP_NOTE, CLIP_OPEN_FOLDER, CLIP_PIN, CLIP_PINNED, CLIP_PREVIEW, CLIP_SEARCH, CLIP_SIZE,
    CLIP_SIZE_GB, CLIP_SIZE_SMALL, CLIP_STORAGE, CLIP_TICKED, CLIP_TIME, CLIP_TITLE, CLIP_TODAY,
    CLIP_UNPIN, CLIP_WHEN, CLIP_YES, CLIP_YESTERDAY, MODULE, NUM_DECIMAL, NUM_GROUP, WINDOW_CLIP,
    kind_id, note_id,
};
use std::ops::RangeInclusive;
use when::when;

/// Bytes in a gigabyte, as Windows counts sizes.
const GB: u64 = MB << 10;
/// 2.41 MB.
const SIZE: u64 = 2_527_068;
/// The notices the clipboard module tells, and its name.
const OUTSIDE: [&str; 3] = [
    "notice.clip-damaged",
    "notice.clip-locked",
    "module.clipboard",
];
/// Arabic-Indic digits, which no text writes (ADR-0017).
const INDIC: RangeInclusive<char> = '\u{660}'..='\u{669}';
/// The counts that reach every Arabic plural form.
const COUNTS: [usize; 7] = [0, 1, 2, 3, 11, 100, 103];
/// A no-break space, which French sets before `?`.
const NBSP: char = '\u{a0}';

/// The clipboard keys the code names one by one.
const FIXED: [&str; 36] = [
    WINDOW_CLIP,
    NUM_DECIMAL,
    NUM_GROUP,
    CLIP_TITLE,
    CLIP_KEPT,
    CLIP_SEARCH,
    CLIP_CLOSE,
    CLIP_PREVIEW,
    CLIP_PIN,
    CLIP_UNPIN,
    CLIP_DELETE,
    CLIP_DELETE_ALL,
    CLIP_INSERT,
    CLIP_INSERT_ONE,
    CLIP_BACK,
    CLIP_CONFIRM,
    CLIP_YES,
    CLIP_NO,
    CLIP_EMPTY,
    CLIP_TODAY,
    CLIP_YESTERDAY,
    CLIP_WHEN,
    CLIP_TIME,
    CLIP_DATE,
    CLIP_SIZE,
    CLIP_SIZE_GB,
    CLIP_SIZE_SMALL,
    CLIP_STORAGE,
    CLIP_OPEN_FOLDER,
    CLIP_META,
    CLIP_META_BARE,
    CLIP_CHARS,
    CLIP_EXPIRES,
    CLIP_TICKED,
    CLIP_PINNED,
    CLIP_JOIN,
];

/// Every clipboard key the code names, the groups' ids included.
fn keys() -> Vec<String> {
    let ids = CLIP_FILTERS
        .iter()
        .map(|(_, id)| format!("{CLIP_FILTER}{id}"));
    let ids = ids.chain(ClipKind::ALL.map(|k| format!("{CLIP_KIND}{}", kind_id(k))));
    let ids = ids.chain(PanelNote::ALL.map(|n| format!("{CLIP_NOTE}{}", note_id(n))));
    let days = (1..=Weekday::ALL.len()).map(|n| format!("{CLIP_DAY}{n}"));
    let all = FIXED.iter().chain(&OUTSIDE).map(|k| (*k).to_owned());
    all.chain(ids).chain(days).collect()
}

/// 182 MB kept and `free` bytes free on `D:`, written with a right-to-left mark as an OS may give it.
fn room(used: u64, free: u64) -> Room {
    Room {
        used,
        free,
        drive: "\u{200f}D:".to_owned(),
    }
}

/// `n` written as `t` counts with `key`.
fn counted(t: &I18n, key: &str) -> Vec<String> {
    COUNTS.iter().map(|n| t.counted(key, *n)).collect()
}

#[test]
fn every_clip_key_has_its_text_in_every_language() {
    for t in all() {
        let missing: Vec<String> = keys().into_iter().filter(|k| !t.has(k)).collect();
        assert!(missing.is_empty(), "{}: {missing:?}", t.lang().tag);
    }
    assert_eq!(lang("en").name_of(MODULE, "clipboard"), "Clipboard history");
}

#[test]
fn a_rows_size_is_megabytes_with_two_decimals_and_the_languages_mark() {
    let cases = [
        ("en", "2.41 MB", "< 0.01 MB", "1,024.00 MB"),
        (
            "fr",
            "2,41\u{a0}Mo",
            "<\u{a0}0,01\u{a0}Mo",
            "1\u{202f}024,00\u{a0}Mo",
        ),
        (
            "ar",
            "2,41 ميغابايت",
            "أقل من 0,01 ميغابايت",
            "1.024,00 ميغابايت",
        ),
    ];
    for (tag, size, small, big) in cases {
        let t = lang(tag);
        assert_eq!(t.clip_size(SIZE), size, "{tag}");
        assert_eq!(t.clip_size(MB / 100), small, "{tag}: just under 0.01 MB");
        assert_eq!(t.clip_size(0), small, "{tag}");
        assert_eq!(t.clip_size(GB), big, "{tag}: a row stays in MB");
    }
    assert_eq!(lang("fr").clip_size(MB / 100 + 1), "0,01\u{a0}Mo");
    let most = "17,592,186,044,416.00 MB";
    assert_eq!(
        lang("en").clip_size(u64::MAX),
        most,
        "every three digits grouped"
    );
}

#[test]
fn the_storage_line_writes_megabytes_below_one_gigabyte_and_gigabytes_from_it() {
    let line = |tag: &str, used, free| lang(tag).clip_room(&room(used, free), GB);
    assert_eq!(
        line("en", 182 * MB, 412 * GB),
        "History: 182.00 MB of 1.00 GB · 412.00 GB free on D:"
    );
    assert_eq!(
        line("fr", GB + GB / 2, 412 * GB),
        "Historique\u{a0}: 1,50\u{a0}Go sur 1,00\u{a0}Go · 412,00\u{a0}Go libres sur D:"
    );
    assert_eq!(
        line("ar", 182 * MB, GB + GB / 2),
        "المحفوظات: 182,00 ميغابايت من 1,00 غيغابايت · 1,50 غيغابايت متوفرة على D:"
    );
    assert!(line("en", 0, 0).starts_with("History: 0.00 MB of"));
    let under = lang("en").clip_room(&room(GB - 1, 1_863 * GB), GB);
    assert!(
        under.starts_with("History: 1.00 GB of"),
        "rounded, then in GB: {under}"
    );
    assert!(under.ends_with("1,863.00 GB free on D:"), "{under}");
}
#[test]
fn arabic_counts_the_question_and_the_characters_in_its_cldr_forms() {
    let ar = lang("ar");
    let question = [
        "لا توجد عناصر غير مثبّتة لحذفها.",
        "هل تريد حذف عنصر واحد؟ تبقى العناصر المثبّتة.",
        "هل تريد حذف عنصرين؟ تبقى العناصر المثبّتة.",
        "هل تريد حذف 3 عناصر؟ تبقى العناصر المثبّتة.",
        "هل تريد حذف 11 عنصرًا؟ تبقى العناصر المثبّتة.",
        "هل تريد حذف 100 عنصر؟ تبقى العناصر المثبّتة.",
        "هل تريد حذف 103 عناصر؟ تبقى العناصر المثبّتة.",
    ];
    assert_eq!(counted(&ar, CLIP_CONFIRM), question);
    let chars = [
        "لا أحرف",
        "حرف واحد",
        "حرفان",
        "3 أحرف",
        "11 حرفًا",
        "100 حرف",
        "103 أحرف",
    ];
    assert_eq!(counted(&ar, CLIP_CHARS), chars);
    let kept: Vec<String> = [1, 2, 3, 7].map(|d| ar.clip_kept(d)).into();
    let want = [
        "تُحفَظ يومًا واحدًا",
        "تُحفَظ يومين",
        "تُحفَظ 3 أيام",
        "تُحفَظ 7 أيام",
    ];
    assert_eq!(kept, want);
}

#[test]
fn english_and_french_count_with_one_and_other_and_say_when_nothing_is_left() {
    let (en, fr) = (lang("en"), lang("fr"));
    assert_eq!(
        en.counted(CLIP_CONFIRM, 0),
        "No unpinned entries to delete."
    );
    assert_eq!(
        en.counted(CLIP_CONFIRM, 1),
        "Delete 1 entry? Pinned ones stay."
    );
    assert_eq!(
        en.counted(CLIP_CONFIRM, 103),
        "Delete 103 entries? Pinned ones stay."
    );
    let one = format!("Supprimer 1 élément{NBSP}? Les éléments épinglés restent.");
    assert_eq!(fr.counted(CLIP_CONFIRM, 1), one);
    assert_eq!(
        fr.counted(CLIP_CONFIRM, 0),
        "Aucun élément non épinglé à supprimer."
    );
    assert_eq!(
        (en.clip_kept(1), en.clip_kept(7)),
        ("Kept for 1 day".to_owned(), "Kept for 7 days".to_owned())
    );
}

#[test]
fn counts_group_their_thousands_and_insert_reads_its_count_or_plain() {
    assert_eq!(
        lang("fr").counted(CLIP_CHARS, 1234),
        "1\u{202f}234 caractères"
    );
    assert_eq!(lang("en").counted(CLIP_CHARS, 1234), "1,234 characters");
    assert_eq!(lang("ar").counted(CLIP_CHARS, 1234), "1.234 حرفًا");
    let insert = [
        ("en", "Insert (3)"),
        ("fr", "Insérer (3)"),
        ("ar", "إدراج (3)"),
    ];
    for (tag, want) in insert {
        assert_eq!(lang(tag).counted(CLIP_INSERT, 3), want, "{tag}");
    }
    let one = [("en", "Insert"), ("fr", "Insérer"), ("ar", "إدراج")];
    for (tag, want) in one {
        assert_eq!(
            lang(tag).text(CLIP_INSERT_ONE),
            want,
            "{tag}: the preview's own"
        );
    }
}

#[test]
fn every_language_writes_western_digits_and_its_own_decimal_mark() {
    let marks = [("en", ".", ","), ("fr", ",", "\u{202f}"), ("ar", ",", ".")];
    for (tag, mark, group) in marks {
        let t = lang(tag);
        assert_eq!(t.text(NUM_DECIMAL), mark, "{tag}");
        assert_eq!(
            t.text(NUM_GROUP),
            group,
            "{tag}: CLDR's mark with Western digits"
        );
        let texts = [
            t.clip_size(SIZE),
            t.clip_room(&room(182 * MB, 412 * GB), GB),
            t.clip_when(&when(4, (2026, 10, 27), 9, 5)),
            t.counted(CLIP_CHARS, 1234),
        ];
        for text in texts {
            assert!(!text.chars().any(|c| INDIC.contains(&c)), "{tag}: {text}");
        }
    }
}

/// The clipboard texts `t` fills in, its drive and app names wrapped in every bidi control, for the bidi check.
pub fn filled(t: &I18n) -> Vec<String> {
    let controls: String = CONTROLS.iter().collect();
    let marked = Room {
        drive: format!("{controls}D:{controls}"),
        ..room(182 * MB, 412 * GB)
    };
    let app = format!("{controls}Chrome{controls}");
    let when = t.clip_when(&when(4, (2026, 10, 27), 9, 5));
    let size = t.clip_size(SIZE);
    let counts = COUNTS
        .iter()
        .flat_map(|n| [t.counted(CLIP_CONFIRM, *n), t.counted(CLIP_CHARS, *n)]);
    let mut out = vec![
        t.clip_room(&marked, GB),
        t.clip_meta(Some(&app), &when, &size),
        t.clip_kept(7),
    ];
    out.extend(counts);
    out.extend([when, size]);
    out
}
