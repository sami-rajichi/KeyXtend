//! The site table: the shipped row as agreed, and every fault refused with its error, never a panic.

use super::*;

/// A good one-site table with `key` set to `value`, written as TOML.
fn row(key: &str, value: &str) -> String {
    let mut keys = [
        ("id", r#""demo""#),
        ("name", r#""Demo""#),
        ("pattern", r#""https://demo.example/?to={to}&q={text}""#),
        ("limit", "100"),
        ("whole", r#"["pt-BR"]"#),
        ("fallback", r#""en""#),
        ("link_max", "500"),
        ("alias", r#"{ nb = "no" }"#),
    ];
    if let Some(slot) = keys.iter_mut().find(|(k, _)| *k == key) {
        slot.1 = value;
    }
    let lines: Vec<String> = keys.iter().map(|(k, v)| format!("{k} = {v}")).collect();
    format!("[[sites]]\n{}\n", lines.join("\n"))
}

#[test]
fn the_shipped_table_holds_google_as_agreed() {
    let table = sites().unwrap();
    let google = table.first().unwrap();
    assert_eq!(
        (google.id.as_str(), google.name.as_str()),
        ("google", "Google Translate")
    );
    let pattern = "https://translate.google.com/?sl=auto&tl={to}&text={text}&op=translate";
    assert_eq!(google.pattern, pattern);
    assert_eq!(google.limit, 5000);
    assert_eq!(google.whole, ["zh-CN", "zh-TW"]);
    assert_eq!((google.fallback.as_str(), google.link_max), ("en", 8000));
    assert!(google.link_max <= MAX_LINK_BYTES);
    assert_eq!(table.get("google"), Some(google));
    let aliases: Vec<(&str, &str)> = (google.alias.iter())
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let want = [
        ("fil", "tl"),
        ("nb", "no"),
        ("nn", "no"),
        ("zh-HK", "zh-TW"),
        ("zh-Hans", "zh-CN"),
        ("zh-Hant", "zh-TW"),
        ("zh-MO", "zh-TW"),
        ("zh-SG", "zh-CN"),
    ];
    assert_eq!(aliases, want);
}

#[test]
fn each_alias_maps_a_language_tag_to_a_language_tag() {
    for alias in [
        r#"{ "zh_HK" = "zh-TW" }"#,
        r#"{ nb = "" }"#,
        r#"{ "" = "no" }"#,
        r#"{ fil = "t l" }"#,
    ] {
        assert_eq!(
            Sites::parse(&row("alias", alias)),
            Err(SiteError::Tag),
            "{alias}"
        );
    }
    assert!(
        Sites::parse(&row("alias", "{}")).is_ok(),
        "no alias is fine"
    );
}

#[test]
fn a_good_table_reads_and_finds_its_sites_by_id() {
    let two = format!("{}{}", row("id", r#""one""#), row("id", r#""two""#));
    let table = Sites::parse(&two).unwrap();
    assert_eq!(table.first().map(|s| s.id.as_str()), Some("one"));
    assert_eq!(table.get("two").map(|s| s.limit), Some(100));
    assert_eq!(table.get("three"), None);
    assert_eq!(table.get(""), None);
}

#[test]
fn a_table_that_does_not_parse_or_lists_nothing_is_refused() {
    assert!(matches!(Sites::parse("[[sites]"), Err(SiteError::Parse(_))));
    assert!(matches!(
        Sites::parse(&row("limit", "70000")),
        Err(SiteError::Parse(_))
    ));
    let spare = format!("{}extra = 1\n", row("id", r#""demo""#));
    assert!(matches!(Sites::parse(&spare), Err(SiteError::Parse(_))));
    assert_eq!(Sites::parse("sites = []"), Err(SiteError::Empty));
}

#[test]
fn an_empty_or_repeated_id_is_refused() {
    assert_eq!(Sites::parse(&row("id", r#""""#)), Err(SiteError::Id));
    let twice = format!("{}{}", row("id", r#""a""#), row("id", r#""a""#));
    assert_eq!(Sites::parse(&twice), Err(SiteError::Id));
}

#[test]
fn a_pattern_must_be_https_with_each_place_once() {
    let bad = [
        r#""http://demo.example/?to={to}&q={text}""#,
        r#""HTTPS://demo.example/?to={to}&q={text}""#,
        r#""https://{to}.example/?q={text}""#,
        r#""https://demo.example/?q={text}""#,
        r#""https://demo.example/?to={to}""#,
        r#""https://demo.example/?to={to}&q={text}&r={text}""#,
        r#""https://demo.example/?to={to}{to}&q={text}""#,
        r#""https://demo.example/?to={to}&q={text}&x=a b""#,
        r#""https://démo.example/?to={to}&q={text}""#,
    ];
    for pattern in bad {
        assert_eq!(
            Sites::parse(&row("pattern", pattern)),
            Err(SiteError::Pattern),
            "{pattern}"
        );
    }
}

#[test]
fn whole_tags_and_the_fallback_must_be_language_tags() {
    for whole in [r#"["zh_CN"]"#, r#"[""]"#, r#"["zh-"]"#] {
        assert_eq!(
            Sites::parse(&row("whole", whole)),
            Err(SiteError::Tag),
            "{whole}"
        );
    }
    for fallback in [r#""""#, r#""auto ""#, r#""e1""#] {
        assert_eq!(
            Sites::parse(&row("fallback", fallback)),
            Err(SiteError::Tag),
            "{fallback}"
        );
    }
    assert!(
        Sites::parse(&row("whole", "[]")).is_ok(),
        "no whole tag is fine"
    );
}

#[test]
fn the_limit_and_the_links_room_are_checked() {
    assert_eq!(Sites::parse(&row("limit", "0")), Err(SiteError::Limit));
    let over = (MAX_LINK_BYTES + 1).to_string();
    assert_eq!(Sites::parse(&row("link_max", &over)), Err(SiteError::Room));
    assert_eq!(
        Sites::parse(&row("link_max", "40")),
        Err(SiteError::Room),
        "no room for text"
    );
    assert!(Sites::parse(&row("link_max", &MAX_LINK_BYTES.to_string())).is_ok());
}
