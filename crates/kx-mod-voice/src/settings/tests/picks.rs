//! The picks (plan edge case 19): the check takes any text pick, so the kernel never resets the others, and a bad
//! pick is dropped alone and named.

use super::*;

/// A `picks` table of `pairs`.
fn picks(pairs: &[(&str, &str)]) -> toml::Value {
    let table = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), toml::Value::from(*v)));
    toml::Value::Table(table.collect())
}

#[test]
fn any_text_pick_passes_the_check_so_a_bad_one_never_resets_the_others() {
    for table in [
        picks(&[]),
        picks(&[("fr", "canary-v2-q8"), ("ar", "gone-model")]),
        picks(&[("fr-FR", "model.exe"), ("en", "")]),
    ] {
        assert!(taken(&with("picks", table.clone())), "{table}");
    }
    let number = toml::Table::from_iter([("fr".into(), 3.into())]);
    assert!(unread(&with("picks", toml::Value::Table(number))));
}

#[test]
fn a_bad_pick_is_dropped_alone_and_named() {
    let good = [
        ("fr", "canary-v2-q8"),
        ("ar", "cohere-q8"),
        ("de", "whisper-turbo-q5km"),
        ("fil", "qwen3-0.6b-q8"),
        ("en", "ggml-small-q5_1.bin"),
        ("it", "my-model.gguf"),
    ];
    let bad = [
        ("es", "gone-model"),
        ("bg", "cohere-arabic-q8"),
        ("pt", r"..\x.gguf"),
        ("ro", "C:x.gguf"),
        ("nl", "model.exe"),
        ("pl", "NUL.gguf"),
        ("sv", ""),
        ("fr-FR", "canary-v2-q8"),
        ("x y", "canary-v2-q8"),
    ];
    let mut s = VoiceSettings::from_defaults().unwrap();
    s.picks = good
        .iter()
        .chain(&bad)
        .map(|(l, p)| ((*l).to_owned(), (*p).to_owned()))
        .collect();
    let dropped = s.drop_bad_picks();
    let named = [
        "picks", "picks.bg", "picks.es", "picks.nl", "picks.pl", "picks.pt", "picks.ro", "picks.sv",
    ];
    assert_eq!(
        dropped, named,
        "a code that is no ISO code names the whole key"
    );
    let mut kept: Vec<&str> = good.iter().map(|(l, _)| *l).collect();
    kept.sort_unstable();
    assert_eq!(s.picks.keys().map(String::as_str).collect::<Vec<_>>(), kept);
    assert!(s.drop_bad_picks().is_empty(), "a second read drops nothing");
}
