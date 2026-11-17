//! Each row's largest piece, by size and by time, leaves room for the WAV header and the form's other parts under the
//! service's own cap, grown by a third when the shape sends Base64 (ruling 2026-11-17).

use super::*;
use kx_module_api::ports::{MAX_FIELD_VALUE, MAX_FIXED, MB, SAMPLE_BYTES, SOUND_HZ, TAG_LEN};

/// A plain PCM WAV header: the RIFF, `fmt ` and `data` chunks' heads.
const WAV_HEAD: u64 = 44;
/// A generous bound on one multipart part's boundary and headers.
const PART_HEAD: u64 = 256;
/// The parts a form sends besides its fixed fields: the model, the language and the file.
const OWN_PARTS: u64 = 3;
/// A megabyte as a service's cap reads it, the smaller decimal one.
const CAP_MB: u64 = 1_000_000;

/// A Gemini row with the sound inline as Base64, under its 20 MB request cap.
const GEMINI: &str = r#"
[[services]]
id = "gemini-test"
name = "Gemini"
shape = "gemini"
host = "generativelanguage.googleapis.com"
address = "https://generativelanguage.googleapis.com/v1beta/interactions"
header = "x-goog-api-key"
keys = "https://aistudio.google.com/apikey"
limits = { cap_mb = 20, cap_s = 3600, max_mb = 14, max_s = 420, per_min = 10, at_once = 2 }
terms = 1
offered = false
untested = true

[[services.models]]
id = "models/gemini-3.5-transcribe"
name = "Gemini 3.5 Transcribe"
langs = { ar = "ar-EG", en = "en-US", fr = "fr-FR" }

[services.facts]
read = 2026-11-17
source = "https://aistudio.google.com/docs/pricing"
place = { id = "us" }
trains = { id = "free-tier", warn = true }
keeps = { id = "limited" }
free = { id = "tier" }
light = { id = "unknown" }
heavy = { id = "unknown" }
"#;

/// The table's `room_kb`, read from its text.
fn room_kb() -> u64 {
    let raw: toml::Table = TEXT.parse().unwrap();
    raw["room_kb"]
        .as_integer()
        .and_then(|n| u64::try_from(n).ok())
        .unwrap()
}

/// The service's cap in bytes as row `id`'s limits write it.
fn cap(id: &str) -> u64 {
    let raw: toml::Table = TEXT.parse().unwrap();
    let rows = raw["services"].as_array().unwrap();
    let row = rows.iter().find(|r| r["id"].as_str() == Some(id)).unwrap();
    let mb = row["limits"]["cap_mb"]
        .as_integer()
        .and_then(|n| u64::try_from(n).ok())
        .unwrap();
    mb * CAP_MB
}

#[test]
fn the_room_covers_the_wav_header_and_the_largest_forms_other_parts() {
    let fixed = u64::try_from(MAX_FIXED).unwrap();
    let values = fixed * u64::try_from(MAX_FIELD_VALUE).unwrap()
        + 2 * u64::try_from(*TAG_LEN.end()).unwrap();
    let largest = WAV_HEAD + (fixed + OWN_PARTS) * PART_HEAD + values;
    assert!(
        room_kb() * 1024 >= largest,
        "{} KiB < {largest} bytes",
        room_kb()
    );
}

#[test]
fn every_shipped_rows_piece_by_size_and_by_time_fits_under_its_cap() {
    let room = room_kb() * 1024;
    let per_s = u64::try_from(SOUND_HZ * SAMPLE_BYTES).unwrap();
    for row in shipped().rows() {
        let cap = cap(&row.id);
        let by_size = u64::from(row.limits.max_mb) * MB;
        let by_time = u64::from(row.limits.max_s) * per_s;
        assert!(by_size + room <= cap && by_time + room <= cap, "{}", row.id);
        let piece = u64::try_from(row.limits.piece_bytes()).unwrap();
        assert!(piece + room <= cap, "{}: the relay's largest piece", row.id);
    }
}

#[test]
fn a_base64_shape_grows_its_piece_by_a_third_against_the_cap() {
    let table = with_rows(GEMINI);
    assert!(
        table.get("gemini-test").is_some_and(|r| !r.offered),
        "built, not offered"
    );
    let long = edits_of(
        &format!("{TEXT}{GEMINI}"),
        &[("max_s = 420", "max_s = 480")],
    );
    let refused = matches!(long, Err(ServicesError::Row { id, why: ServiceError::Room }) if id == "gemini-test");
    assert!(
        refused,
        "480 s of sound is 15.36 MB raw, 20.48 MB as Base64"
    );
    let as_form = format!("{TEXT}{GEMINI}").replace(
        "shape = \"gemini\"",
        "shape = \"form\"\nform = { lang = \"language\", list = false }",
    );
    let as_form = edits_of(&as_form, &[("max_s = 420", "max_s = 480")]);
    assert!(
        as_form.is_ok(),
        "the same piece fits a form's cap: {as_form:?}"
    );
}

#[test]
fn a_cap_in_seconds_bounds_a_pieces_seconds() {
    let short = edits_of(
        &format!("{TEXT}{GEMINI}"),
        &[("cap_s = 3600", "cap_s = 300")],
    );
    let refused = matches!(short, Err(ServicesError::Row { id, why: ServiceError::Room }) if id == "gemini-test");
    assert!(refused, "420 s past a 300 s cap");
}
