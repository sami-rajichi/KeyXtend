//! The table, the markers, the formats a copy is read from, the bytes a write sets and the file facts, all pure.

use std::path::PathBuf;

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipPart, FileFact, ImageCodec};

use super::parts::{Formats, Kind, blank, crlf, text_of};
use super::*;

#[cfg(windows)]
mod app;
mod bytes;
mod gather;
#[cfg(windows)]
mod live;

/// Made-up format ids, as Windows would give them.
const F: Formats = Formats {
    text: 13,
    html: 0xC001,
    rtf: 0xC002,
    png: 0xC003,
    dib_v5: 17,
    dib: 8,
    files: 15,
    effect: 0xC004,
    copy: 1,
};
/// A private format nobody keeps.
const OTHER: u32 = 0xC0FF;

fn marks(present: &[&str], zero: &[&str]) -> Marks {
    let owned = |n: &[&str]| n.iter().map(ToString::to_string).collect();
    Marks::new(owned(present), owned(zero))
}

#[test]
fn the_shipped_table_parses_and_its_marker_is_one_the_watch_skips() {
    let t = Table::parse(TABLE).unwrap();
    assert!(t.marks().has(&t.marker));
    assert!(table().is_ok());
}

#[test]
fn a_table_whose_own_marker_the_watch_would_read_is_refused() {
    let own = TABLE.replace(
        r#"marker = "CanIncludeInClipboardHistory""#,
        r#"marker = "KeyXtend.Own""#,
    );
    assert_ne!(own, TABLE, "the test edits the marker line");
    assert_eq!(Table::parse(&own).unwrap_err(), TableError::Unmarked);
}

#[test]
fn a_table_with_an_empty_name_or_limit_is_refused() {
    for (from, to) in [
        (r#"png = "PNG""#, r#"png = " ""#),
        ("name_chars = 80", "name_chars = 0"),
    ] {
        let text = TABLE.replace(from, to);
        assert_ne!(text, TABLE, "{from}");
        assert_eq!(Table::parse(&text).unwrap_err(), TableError::Empty, "{to}");
    }
}

#[test]
fn a_table_with_an_unknown_key_is_refused() {
    let text = format!("extra = 1\n{TABLE}");
    assert!(matches!(Table::parse(&text), Err(TableError::Parse(_))));
}

#[test]
fn a_present_marker_skips_a_copy_whatever_else_it_holds() {
    let m = marks(
        &["Clipboard Viewer Ignore"],
        &["CanIncludeInClipboardHistory"],
    );
    assert!(m.marked(|n| n == "Clipboard Viewer Ignore", |_| Some(1)));
    assert!(!m.marked(|n| n == "HTML Format", |_| None), "plain copy");
}

#[test]
fn a_number_marker_skips_only_when_it_holds_zero() {
    let m = marks(&[], &["CanIncludeInClipboardHistory"]);
    assert!(m.marked(|_| false, |_| Some(0)));
    assert!(!m.marked(|_| false, |_| Some(1)));
    assert!(!m.marked(|_| false, |_| None), "absent");
}

#[test]
fn a_number_marker_that_cannot_be_named_or_read_counts_as_zero() {
    use super::marks::number;
    assert_eq!(number(None, |_| false, |_| None), Some(0), "not registered");
    assert_eq!(
        number(Some(1), |_| true, |_| None),
        Some(0),
        "there, unreadable"
    );
    assert_eq!(number(Some(1), |_| true, |_| Some(7)), Some(7), "read");
    assert_eq!(number(Some(1), |_| false, |_| Some(7)), None, "absent");
}

#[test]
fn marker_names_ignore_case_as_windows_does() {
    let m = marks(
        &["Clipboard Viewer Ignore"],
        &["CanIncludeInClipboardHistory"],
    );
    assert!(m.has("clipboard viewer ignore") && m.has("CANINCLUDEINCLIPBOARDHISTORY"));
    assert!(!m.has("PNG"));
}

#[test]
fn without_drops_one_marker_and_keeps_the_rest() {
    let m = Table::parse(TABLE).unwrap().marks();
    let fewer = m.without("canincludeinclipboardhistory");
    assert!(!fewer.has("CanIncludeInClipboardHistory"));
    assert_eq!(fewer.names().count(), m.names().count() - 1);
    assert!(
        m.names()
            .all(|n| fewer.has(n) || n == "CanIncludeInClipboardHistory")
    );
}

#[test]
fn the_parts_follow_the_clipboards_order_with_each_kind_once() {
    let held = [OTHER, F.html, F.text, F.rtf, F.files, F.text];
    let kinds: Vec<Kind> = F.pick(&held).iter().map(|w| w.kind).collect();
    assert_eq!(kinds, [Kind::Html, Kind::Text, Kind::Rtf, Kind::Files]);
}

#[test]
fn the_one_image_is_png_before_dibv5_before_dib() {
    let images = |held: &[u32]| -> Vec<(Kind, u32)> {
        F.pick(held).iter().map(|w| (w.kind, w.format)).collect()
    };
    let png = (Kind::Image(ImageCodec::Png), F.png);
    let dib = |format| (Kind::Image(ImageCodec::Dib), format);
    assert_eq!(images(&[F.dib, F.dib_v5, F.png]), [png]);
    assert_eq!(images(&[F.png, F.dib]), [png]);
    assert_eq!(images(&[F.dib, F.dib_v5]), [dib(F.dib_v5)]);
    assert_eq!(images(&[F.dib]), [dib(F.dib)]);
}

#[test]
fn the_image_takes_the_place_of_the_first_image_format() {
    let kinds: Vec<Kind> = F
        .pick(&[F.html, F.dib, F.text, F.png])
        .iter()
        .map(|w| w.kind)
        .collect();
    let png = Kind::Image(ImageCodec::Png);
    assert_eq!(kinds, [Kind::Html, png, Kind::Text]);
}

#[test]
fn formats_nobody_keeps_give_no_parts() {
    assert!(F.pick(&[OTHER, F.effect]).is_empty());
    assert!(F.pick(&[]).is_empty());
}

#[test]
fn a_lone_line_feed_becomes_cr_lf_and_the_rest_stays() {
    assert_eq!(crlf("a\nb"), "a\r\nb");
    assert_eq!(crlf("a\r\nb\n\n"), "a\r\nb\r\n\r\n");
    assert_eq!(crlf("\nلا\r"), "\r\nلا\r");
    assert_eq!(crlf(""), "");
}

#[test]
fn only_white_space_and_nuls_are_blank() {
    for text in ["", " ", "\t\r\n", "\0\0", " \u{a0}\u{3000}"] {
        assert!(blank(text), "{text:?}");
    }
    assert!(!blank(" a "));
    assert!(!blank("\u{200b}"), "a zero-width space is no white space");
}

#[test]
fn text_stops_at_the_first_nul() {
    let units = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    assert_eq!(text_of(&units("ab\0cd"), 2).as_deref(), Some("ab"));
    assert_eq!(text_of(&units("لا€"), 3).as_deref(), Some("لا€"));
    assert_eq!(
        text_of(&units("\u{1F600}"), 1).as_deref(),
        Some("\u{1F600}")
    );
    assert_eq!(text_of(&units("abc"), usize::MAX).as_deref(), Some("abc"));
}

#[test]
fn text_past_the_cap_is_refused() {
    let units = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    assert_eq!(text_of(&units("abc"), 2), None);
    assert_eq!(text_of(&units("abcdefgh\0"), 2), None);
}

#[test]
fn file_facts_see_a_file_and_then_its_deletion() {
    let path = std::env::temp_dir().join(format!("kx-clip-facts-{}.txt", std::process::id()));
    std::fs::write(&path, b"12345").unwrap();
    let paths = Redacted::new(vec![path.clone(), std::env::temp_dir()]);
    assert_eq!(facts(&paths), [FileFact::Size(5), FileFact::Size(0)]);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(facts(&paths), [FileFact::Gone, FileFact::Size(0)]);
}

#[cfg(windows)]
#[test]
fn a_path_on_a_drive_that_is_not_there_is_away_not_gone() {
    let Some(free) = free_drive() else {
        return;
    };
    let path = PathBuf::from(format!(r"{free}:\kx-test\gone.txt"));
    assert_eq!(facts(&Redacted::new(vec![path])), [FileFact::Away]);
}

/// A drive letter no drive uses now; `None` when every letter is taken.
#[cfg(windows)]
#[allow(unsafe_code, reason = "A plain query of the drive letters in use.")]
fn free_drive() -> Option<char> {
    // SAFETY: a plain query with no arguments.
    let used = unsafe { windows::Win32::Storage::FileSystem::GetLogicalDrives() };
    let letters = ('A'..='Z').zip(0..u32::BITS);
    letters
        .filter(|&(_, bit)| used & (1 << bit) == 0)
        .last()
        .map(|(c, _)| c)
}

#[test]
fn no_paths_have_no_facts() {
    assert!(facts(&Redacted::new(Vec::<PathBuf>::new())).is_empty());
}

#[test]
fn a_text_part_writes_as_text() {
    let part = ClipPart::Text(Redacted::new("x".to_owned()));
    let w = F.encode(&part);
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].0, F.text);
}
