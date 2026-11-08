//! The Translate link: English, French and Arabic text encoded so it decodes back to itself, hidden marks dropped,
//! the cut at a whole word within both caps, and the target's code (plan edge cases 9 and 10).

use super::*;

/// The shipped Google row.
fn google() -> &'static Site {
    sites().unwrap().get("google").unwrap()
}

/// Google's link for `text` into French at `chars` characters, as text.
fn link(text: &str, chars: u16) -> Option<String> {
    google()
        .link(text, "fr", chars)
        .map(|l| l.expose().to_owned())
}

/// The decoded text a link carries.
pub(super) fn sent(link: &str) -> String {
    let start = link.find("&text=").unwrap() + "&text=".len();
    let end = link[start..].find('&').map_or(link.len(), |n| start + n);
    decode(&link[start..end])
}

/// `text` with each `%XX` turned back into its byte, read as UTF-8.
fn decode(text: &str) -> String {
    let (mut bytes, mut rest) = (Vec::new(), text.as_bytes());
    while let Some((&b, tail)) = rest.split_first() {
        if b == b'%' {
            let hex = std::str::from_utf8(&tail[..2]).unwrap();
            bytes.push(u8::from_str_radix(hex, 16).unwrap());
            rest = &tail[2..];
        } else {
            bytes.push(b);
            rest = tail;
        }
    }
    String::from_utf8(bytes).unwrap()
}

/// Enough characters for every short text here.
const CHARS: u16 = 1000;

#[test]
fn english_text_goes_into_the_pattern_encoded() {
    let want = "https://translate.google.com/?sl=auto&tl=fr&text=Hello%20world&op=translate";
    assert_eq!(link("Hello world", CHARS).as_deref(), Some(want));
    let marks = link("a-b.c_d~e!f&g=h?i#j/k+l", CHARS).unwrap();
    assert!(
        marks.contains("text=a-b.c_d~e%21f%26g%3Dh%3Fi%23j%2Fk%2Bl&op"),
        "{marks}"
    );
}

#[test]
fn french_accents_are_encoded_as_utf8_and_decode_back() {
    let text = "Ça va, élève ? Où est l’œuvre";
    let got = link(text, CHARS).unwrap();
    assert!(
        got.contains("text=%C3%87a%20va%2C%20%C3%A9l%C3%A8ve%20%3F"),
        "{got}"
    );
    assert_eq!(sent(&got), text);
}

#[test]
fn arabic_is_encoded_byte_by_byte_and_decodes_back() {
    let got = link("مرحبا بالعالم", CHARS).unwrap();
    let marhaba = "%D9%85%D8%B1%D8%AD%D8%A8%D8%A7";
    assert!(got.contains(&format!("text={marhaba}%20")), "{got}");
    assert_eq!(sent(&got), "مرحبا بالعالم");
    let harakat = "كَتَبَ الوَلَدُ";
    assert_eq!(sent(&link(harakat, CHARS).unwrap()), harakat);
}

#[test]
fn mixed_text_keeps_every_script_in_its_order() {
    let text = "Copy نسخ Copier 2026 — ترجمة";
    assert_eq!(sent(&link(text, CHARS).unwrap()), text);
}

#[test]
fn hidden_marks_are_dropped_at_every_position() {
    let marks = [
        '\u{202e}',
        '\u{200b}',
        '\u{7}',
        '\u{feff}',
        '\u{61c}',
        '\u{2066}',
        '\u{ad}',
        '\u{e0061}',
    ];
    let word = "مرحبا";
    let plain = link(word, CHARS);
    for mark in marks {
        for at in 0..=word.chars().count() {
            let mut text: Vec<char> = word.chars().collect();
            text.insert(at, mark);
            let marked: String = text.into_iter().collect();
            assert_eq!(link(&marked, CHARS), plain, "{mark:?} at {at}");
        }
    }
}

#[test]
fn breaks_and_tabs_become_spaces_so_words_stay_apart() {
    assert_eq!(
        sent(&link("مرحبا\nبالعالم", CHARS).unwrap()),
        "مرحبا بالعالم"
    );
    assert_eq!(
        sent(&link("one\ttwo\r\nthree", CHARS).unwrap()),
        "one two  three"
    );
    assert_eq!(sent(&link("un\u{2028}deux", CHARS).unwrap()), "un deux");
}

#[test]
fn an_empty_blank_or_hidden_only_text_gives_no_link() {
    for text in [
        "",
        "   ",
        "\n\t \u{a0}",
        "\u{200b}\u{202e}\u{feff}",
        " \u{2028} ",
    ] {
        assert_eq!(link(text, CHARS), None, "{text:?}");
    }
}

#[test]
fn the_text_is_trimmed() {
    assert_eq!(
        sent(&link("  \u{200b} bonjour \n ", CHARS).unwrap()),
        "bonjour"
    );
}

#[test]
fn a_long_text_is_cut_at_a_whole_word_within_translate_chars() {
    let text = "word ".repeat(300);
    let got = sent(&link(&text, 52).unwrap());
    assert_eq!(
        got,
        "word ".repeat(10).trim_end(),
        "52 would end inside a word"
    );
    let at_space = sent(&link(&text, 54).unwrap());
    assert_eq!(
        at_space,
        "word ".repeat(11).trim_end(),
        "the 55th is a space"
    );
}

#[test]
fn a_single_long_word_is_cut_at_a_whole_character() {
    let got = sent(&link(&"a".repeat(100), 50).unwrap());
    assert_eq!(got, "a".repeat(50));
    // Ba with a kasra is one letter of two characters: the cut never strips the mark from its letter.
    let word = "بِ".repeat(60);
    let cut = sent(&link(&word, 51).unwrap());
    assert_eq!(cut, "بِ".repeat(25));
}

#[test]
fn a_long_arabic_text_fits_the_link_cap_and_ends_at_a_whole_word() {
    let text = "مرحبا بالعالم ".repeat(600);
    let got = link(&text, *crate::settings::TRANSLATE_CHARS.end()).unwrap();
    let cap = google().link_max;
    assert!(got.len() <= cap, "{} bytes", got.len());
    let longest_word = "%D8%A8".len() * "بالعالم".chars().count() + "%20".len();
    assert!(
        got.len() > cap - longest_word,
        "the cut uses the room: {} bytes",
        got.len()
    );
    let kept = sent(&got);
    assert!(text.starts_with(&kept));
    assert!(text[kept.len()..].starts_with(' '), "ends at a whole word");
    assert!(
        kept.chars().count() < 5000,
        "cut earlier than translate_chars"
    );
}

#[test]
fn the_sites_limit_caps_translate_chars() {
    let table = Sites::parse(
        "[[sites]]\nid = \"small\"\nname = \"Small\"\n\
         pattern = \"https://small.example/?tl={to}&text={text}\"\n\
         limit = 60\nwhole = []\nfallback = \"en\"\nlink_max = 4000\nalias = {}\n",
    )
    .unwrap();
    let site = table.first().unwrap();
    let got = site.link(&"a".repeat(100), "fr", CHARS).unwrap();
    assert_eq!(sent(got.expose()).len(), 60);
}

#[test]
fn the_target_follows_translate_to_else_the_os_language() {
    assert_eq!(google().target("fr", "ar-TN"), "fr");
    assert_eq!(google().target("", "ar-TN"), "ar");
    assert_eq!(google().target("", "fr-FR"), "fr");
    assert_eq!(google().target("FR-ca", ""), "fr");
    assert_eq!(google().target("", "es-419"), "es");
    assert_eq!(
        google().target("", "fr_FR"),
        "fr",
        "an OS tag may use an underscore"
    );
}

#[test]
fn whole_tags_keep_the_sites_spelling() {
    assert_eq!(google().target("", "zh-CN"), "zh-CN");
    assert_eq!(google().target("zh-tw", "fr"), "zh-TW");
    assert_eq!(google().target("", "zh"), "zh");
}

#[test]
fn aliases_map_the_whole_tag_then_language_and_script_then_language() {
    let to = |os: &str| google().target("", os);
    assert_eq!(to("zh-HK"), "zh-TW", "the whole tag");
    assert_eq!(to("zh-MO"), "zh-TW");
    assert_eq!(to("zh-SG"), "zh-CN");
    assert_eq!(to("zh-Hant-TW"), "zh-TW", "language and script");
    assert_eq!(to("zh-Hant-HK"), "zh-TW");
    assert_eq!(to("zh-Hans-SG"), "zh-CN");
    assert_eq!(to("zh-Hant"), "zh-TW");
    assert_eq!(to("nb-NO"), "no", "the language");
    assert_eq!(to("nn-NO"), "no");
    assert_eq!(to("fil-PH"), "tl");
    assert_eq!(to("NB"), "no", "any case");
    assert_eq!(to("zh_HK"), "zh-TW", "an OS tag with an underscore");
    assert_eq!(google().target("fil", "fr-FR"), "tl", "translate_to too");
    assert_eq!(to("sr-Latn-RS"), "sr", "no alias: the language");
}

#[test]
fn an_empty_or_unusable_tag_gives_the_sites_fallback() {
    for os in ["", "-fr", "123", "é", &"x".repeat(TAG_CHARS + 1)] {
        assert_eq!(google().target("", os), "en", "{os:?}");
    }
}

#[test]
fn the_target_code_goes_into_the_link() {
    let got = google().link("你好", "zh-TW", CHARS).unwrap();
    assert!(
        got.expose().contains("&tl=zh-TW&text=%E4%BD%A0%E5%A5%BD&"),
        "{}",
        got.expose()
    );
}

#[test]
fn a_language_tag_is_letters_and_hyphens_and_short() {
    for good in ["fr", "fr-FR", "zh-Hant-TW", "ar"] {
        assert!(tag_ok(good), "{good}");
    }
    let long = "a".repeat(TAG_CHARS + 1);
    for bad in [
        "",
        "fr_FR",
        "fr--FR",
        "-fr",
        "fr-",
        "fr1",
        "é",
        "f r",
        long.as_str(),
    ] {
        assert!(!tag_ok(bad), "{bad}");
    }
}

mod cut;
