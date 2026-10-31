//! Tests of `p8cfg`.

use super::*;
use crate::config::load;

/// The shipped file, whose every key the tests remove in turn.
const SHIPPED: &str = include_str!("../../kx-gates-p8.toml");

/// A setting spoiled, and the key its error must name.
type Spoil = (fn(&mut GatesConfig), &'static str);

/// Runs each spoil on a fresh copy of the shipped settings and checks its error names its key.
fn refused(spoils: &[Spoil]) {
    let good = load().expect("loads");
    for (i, (spoil, key)) in spoils.iter().enumerate() {
        let mut c = good.clone();
        spoil(&mut c);
        let e = check(&c).expect_err(&format!("case {i} passed"));
        assert!(e.contains(key), "case {i}: {e}");
    }
}

/// Section `name` of the file read as its settings, or the reader's error.
fn read_section(name: &str, value: toml::Value) -> Result<(), String> {
    let e = |e: toml::de::Error| e.to_string();
    match name {
        "g20_product" => value.try_into::<G20Product>().map(drop).map_err(e),
        "g8_clip" => value.try_into::<G8Clip>().map(drop).map_err(e),
        other => Err(format!("no reader for [{other}]")),
    }
}

#[test]
fn the_shipped_settings_pass_with_a_copy_per_language_and_two_ticks() {
    let c = load().expect("loads");
    assert_eq!(check(&c), Ok(()));
    let langs: Vec<&str> = c
        .g20_product
        .copies
        .iter()
        .map(|k| k.lang.as_str())
        .collect();
    assert_eq!(langs, ["en", "fr", "ar"]);
    let ticks = c.g20_product.searches.iter().filter(|s| s.tick).count();
    assert_eq!(ticks, JOINED);
    assert_eq!(c.g8_clip.entries, 1000, "the owner's 1,000 entries");
}

#[test]
fn every_key_in_the_file_is_read_and_a_missing_one_is_an_error_naming_it() {
    let file: toml::Table = SHIPPED.parse().expect("parses");
    for (name, section) in &file {
        let keys = section.as_table().expect("a section");
        assert_eq!(read_section(name, section.clone()), Ok(()), "[{name}]");
        for key in keys.keys() {
            let mut less = keys.clone();
            less.remove(key);
            let e = read_section(name, toml::Value::Table(less)).expect_err(key);
            assert!(e.contains(key.as_str()), "[{name}] {key}: {e}");
        }
        let mut more = keys.clone();
        more.insert("stray".into(), toml::Value::Integer(1));
        let e = read_section(name, toml::Value::Table(more)).expect_err("unknown");
        assert!(e.contains("stray"), "[{name}]: {e}");
    }
}

#[test]
fn g20_copies_the_panel_would_not_show_as_they_are_are_refused() {
    refused(&[
        (|c| c.g20_product.copies.clear(), "g20_product.copies"),
        (
            |c| c.g20_product.copies[0].text.clear(),
            "g20_product.copies",
        ),
        (
            |c| c.g20_product.copies[0].text = "a\nb".into(),
            "g20_product.copies",
        ),
        (
            |c| c.g20_product.copies[0].text = "a\u{200f}b".into(),
            "g20_product.copies",
        ),
        (
            |c| c.g20_product.copies[0].text = "x".repeat(10_000),
            "g20_product.copies",
        ),
        (
            |c| c.g20_product.copies[0].lang = "EN".into(),
            "g20_product.copies",
        ),
        (
            |c| c.g20_product.copies[1].lang = "en".into(),
            "g20_product.copies",
        ),
    ]);
}

#[test]
fn g20_hidden_texts_and_the_skipped_app_must_be_plain_and_apart() {
    refused(&[
        (|c| c.g20_product.marked.clear(), "g20_product.marked"),
        (
            |c| c.g20_product.cleared = "a\rb".into(),
            "g20_product.cleared",
        ),
        (|c| c.g20_product.skipped.clear(), "g20_product.skipped"),
        (
            |c| {
                c.g20_product
                    .cleared
                    .clone_from(&c.g20_product.copies[0].text);
            },
            "g20_product",
        ),
        (|c| c.g20_product.skip_app.clear(), "g20_product.skip_app"),
        (
            |c| c.g20_product.skip_app = "a\nb".into(),
            "g20_product.skip_app",
        ),
    ]);
}

#[test]
fn g20_searches_that_find_nothing_or_tick_wrong_are_refused() {
    refused(&[
        (|c| c.g20_product.searches.clear(), "g20_product.searches"),
        (
            |c| c.g20_product.searches[1].typed.clear(),
            "g20_product.searches",
        ),
        (
            |c| c.g20_product.searches[1].typed = "12".into(),
            "g20_product.searches",
        ),
        (
            |c| c.g20_product.searches[1].typed = "tea".into(),
            "g20_product.searches",
        ),
        (
            |c| c.g20_product.searches[1].typed = "gates".into(),
            "g20_product.searches",
        ),
        (
            |c| c.g20_product.searches[1].lang = "x".into(),
            "g20_product.searches",
        ),
        (
            |c| c.g20_product.searches[0].tick = false,
            "g20_product.searches",
        ),
        (
            |c| {
                c.g20_product.searches[1].tick = true;
                c.g20_product.searches[2].tick = false;
            },
            "g20_product.searches",
        ),
    ]);
}

#[test]
fn g20_ids_names_and_waits_out_of_range_are_refused() {
    refused(&[
        (|c| c.g20_product.group.clear(), "g20_product.group"),
        (|c| c.g20_product.group = "a.b".into(), "g20_product.group"),
        (|c| c.g20_product.row.clear(), "g20_product.row"),
        (|c| c.g20_product.insert.clear(), "g20_product.insert"),
        (
            |c| c.g20_product.insert_name = "clip.nowhere".into(),
            "g20_product.insert_name",
        ),
        (
            |c| c.g20_product.count_var = "nothing".into(),
            "g20_product.insert_name",
        ),
        (
            |c| c.g20_product.join_name = "clip.nowhere".into(),
            "g20_product.join_name",
        ),
        (|c| c.g20_product.ready_ms = 0, "g20_product.ready_ms"),
        (|c| c.g20_product.copy_gap_ms = 0, "g20_product.copy_gap_ms"),
        (
            |c| c.g20_product.panel_wait_ms = c.mouse_guard.limit_ms + 1,
            "g20_product.panel_wait_ms",
        ),
        (|c| c.g20_product.settle_ms = 0, "g20_product.settle_ms"),
    ]);
}

#[test]
fn a_backup_a_gate_deletes_the_owners_backup_or_a_bad_marker_is_refused() {
    refused(&[
        (
            |c| c.g20_product.marker = "a/b".into(),
            "g20_product.marker",
        ),
        (
            |c| c.g20_product.marker = kx_platform::CLIP_DIR.into(),
            "g20_product.marker",
        ),
        (|c| c.g20_product.backup.clear(), "g20_product.backup"),
        (
            |c| c.g20_product.backup = "a/b.json".into(),
            "g20_product.backup",
        ),
        (
            |c| c.g20_product.backup = crate::out::RESULTS.into(),
            "g20_product.backup",
        ),
        (
            |c| c.g20_product.backup = "g20-keyxtend-1.json".into(),
            "g20_product.backup",
        ),
        (
            |c| c.g20_product.backup.clone_from(&c.owner_settings.backup),
            "g20_product.backup",
        ),
    ]);
}

#[test]
fn g8_clip_values_out_of_range_are_refused() {
    refused(&[
        (|c| c.g8_clip.entries = 0, "g8_clip.entries"),
        (|c| c.g8_clip.entries = 1_000_000, "g8_clip.entries"),
        (|c| c.g8_clip.forget_tail = 0, "g8_clip.forget_tail"),
        (
            |c| c.g8_clip.forget_tail = c.g8_clip.entries,
            "g8_clip.forget_tail",
        ),
        (|c| c.g8_clip.image_every = 0, "g8_clip.image_every"),
        (|c| c.g8_clip.image_px.clear(), "g8_clip.image_px"),
        (|c| c.g8_clip.image_px[0] = 0, "g8_clip.image_px"),
        (|c| c.g8_clip.image_px[0] = 1_000_000, "g8_clip.image_px"),
        (|c| c.g8_clip.text_chars.clear(), "g8_clip.text_chars"),
        (|c| c.g8_clip.text_chars[0] = 3, "g8_clip.text_chars"),
        (
            |c| c.g8_clip.text_chars[0] = 100_000_000,
            "g8_clip.text_chars",
        ),
        (|c| c.g8_clip.prefix.clear(), "g8_clip.prefix"),
        (|c| c.g8_clip.prefix = "x".repeat(300), "g8_clip.prefix"),
        (|c| c.g8_clip.rounds = 0, "g8_clip.rounds"),
        (|c| c.g8_clip.pages = 0, "g8_clip.pages"),
        (|c| c.g8_clip.search.clear(), "g8_clip.search"),
        (
            |c| c.g8_clip.search_lang = "x".into(),
            "g8_clip.search_lang",
        ),
    ]);
}

#[test]
fn g8_clip_waits_and_page_text_out_of_range_are_refused() {
    refused(&[
        (|c| c.g8_clip.copy_gap_ms = 0, "g8_clip.copy_gap_ms"),
        (
            |c| c.g8_clip.settle_ms = c.mouse_guard.limit_ms + 1,
            "g8_clip.settle_ms",
        ),
        (|c| c.g8_clip.panel_wait_ms = 0, "g8_clip.panel_wait_ms"),
        (|c| c.g8_clip.pause_ms = 0, "g8_clip.pause_ms"),
        (
            |c| c.g8_clip.count_wait_ms = c.mouse_guard.limit_ms + 1,
            "g8_clip.count_wait_ms",
        ),
        (
            |c| c.g8_clip.page_name = "page.nowhere".into(),
            "g8_clip.page_name",
        ),
        (
            |c| c.g8_clip.page_var = "nothing".into(),
            "g8_clip.page_name",
        ),
    ]);
}

#[test]
fn p8_runs_past_the_mouse_guard_are_refused_unless_it_is_off() {
    let good = load().expect("loads");
    let head = good.fit.guard_headroom_ms;
    let (g20, clip) = (runs::g20_ms(&good) + head, runs::clip_ms(&good) + head);
    assert!(clip > g20, "g8 clip copies a thousand times");
    let mut c = good.clone();
    c.mouse_guard.limit_ms = g20 - 1;
    let e = check(&c).expect_err("G20 runs too long");
    assert!(e.contains("g20_product"), "{e}");
    c.mouse_guard.limit_ms = g20;
    let e = check(&c).expect_err("g8 clip runs too long");
    assert!(e.contains("g8_clip"), "{e}");
    c.mouse_guard.on = false;
    assert_eq!(check(&c), Ok(()), "nothing is held with the guard off");
}

mod ids;
