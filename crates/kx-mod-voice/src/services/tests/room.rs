//! Each row's largest piece, by size and by time, leaves room for the WAV header and the form's other parts under the
//! service's own cap, grown by a third when the shape sends Base64 (ruling 2026-11-17), so Gemini's largest piece goes
//! in one request under its 20 MB cap [N6].

use super::*;
use kx_module_api::ports::{
    MAX_FIELD_VALUE, MAX_FIXED, MB, SAMPLE_BYTES, SOUND_HZ, SpeechAsk, TAG_LEN,
};

/// A plain PCM WAV header: the RIFF, `fmt ` and `data` chunks' heads.
const WAV_HEAD: u64 = 44;
/// A generous bound on one multipart part's boundary and headers, and on Gemini's JSON names and marks, which the
/// worker's gemini tests pin at 156 bytes.
const PART_HEAD: u64 = 256;
/// The parts a form sends besides its fixed fields: the model, the language and the file.
const OWN_PARTS: u64 = 3;
/// A megabyte as a service's cap reads it, the smaller decimal one.
const CAP_MB: u64 = 1_000_000;
/// Base64 writes four characters for each three bytes.
const B64_IN: u64 = 3;
const B64_OUT: u64 = 4;

/// The table's `room_kb`, read from its text.
fn room_kb() -> u64 {
    let raw: toml::Table = TEXT.parse().unwrap();
    raw["room_kb"]
        .as_integer()
        .and_then(|n| u64::try_from(n).ok())
        .unwrap()
}

/// The service's cap in bytes as row `id`'s limits write it, in either file of the table.
fn cap(id: &str) -> u64 {
    let rows = |text: &str| {
        let raw: toml::Table = text.parse().unwrap();
        raw["services"].as_array().unwrap().clone()
    };
    let all = [rows(TEXT), rows(OWN)].concat();
    let row = all.iter().find(|r| r["id"].as_str() == Some(id)).unwrap();
    let mb = row["limits"]["cap_mb"]
        .as_integer()
        .and_then(|n| u64::try_from(n).ok())
        .unwrap();
    mb * CAP_MB
}

/// `bytes` as `row` sends them: a third larger for Gemini, whose shape carries the sound as Base64.
fn sent(row: &Service, bytes: u64) -> u64 {
    if row.shape == CloudShape::Gemini {
        bytes.div_ceil(B64_IN) * B64_OUT
    } else {
        bytes
    }
}

/// The table with the second file edited.
fn own_edits(pairs: &[(&str, &str)]) -> Result<Services, ServicesError> {
    Services::parse(TEXT, &edits_text(OWN, pairs), LANGS)
}

/// True when `got` refuses Gemini's row for the room its piece leaves.
fn roomless(got: &Result<Services, ServicesError>) -> bool {
    matches!(got, Err(ServicesError::Row { id, why: ServiceError::Room }) if id == "gemini")
}

#[test]
fn the_room_covers_the_wav_header_and_the_largest_forms_other_parts() {
    let fixed = u64::try_from(MAX_FIXED).unwrap();
    let tag = u64::try_from(*TAG_LEN.end()).unwrap();
    let values = fixed * u64::try_from(MAX_FIELD_VALUE).unwrap() + 2 * tag;
    let largest = WAV_HEAD + (fixed + OWN_PARTS) * PART_HEAD + values;
    let room = room_kb() * 1024;
    assert!(room >= largest, "{} KiB < {largest} bytes", room_kb());
    let json = WAV_HEAD.div_ceil(B64_IN) * B64_OUT + PART_HEAD + 2 * tag;
    assert!(room >= json, "Gemini's JSON and its WAV header as Base64");
}

#[test]
fn every_shipped_rows_piece_by_size_and_by_time_fits_under_its_cap() {
    let room = room_kb() * 1024;
    let per_s = u64::try_from(SOUND_HZ * SAMPLE_BYTES).unwrap();
    for row in shipped().rows() {
        let cap = cap(&row.id);
        let by_size = sent(row, u64::from(row.limits.max_mb) * MB);
        let by_time = sent(row, u64::from(row.limits.max_s) * per_s);
        assert!(by_size + room <= cap && by_time + room <= cap, "{}", row.id);
        let piece = sent(row, u64::try_from(row.limits.piece_bytes()).unwrap());
        assert!(piece + room <= cap, "{}: the relay's largest piece", row.id);
    }
}

#[test]
fn geminis_largest_piece_goes_in_one_request_under_its_cap() {
    let row = shipped().get("gemini").unwrap();
    let wav = u64::try_from(row.limits.piece_bytes()).unwrap() + WAV_HEAD;
    let tag = u64::try_from(*TAG_LEN.end()).unwrap();
    let body = wav.div_ceil(B64_IN) * B64_OUT + PART_HEAD + 2 * tag;
    assert!(body <= cap("gemini"), "{body} bytes in one request [N6]");
}

#[test]
fn a_base64_shape_grows_its_piece_by_a_third_against_the_cap() {
    assert!(own_edits(&[]).is_ok(), "the shipped row fits");
    let long = own_edits(&[("max_s = 420", "max_s = 480")]);
    assert!(
        roomless(&long),
        "480 s of sound is 15.36 MB raw, 20.48 MB as Base64"
    );
    let form = "shape = \"form\"\nform = { lang = \"language\", list = false }";
    let as_form = own_edits(&[("shape = \"gemini\"", form), ("max_s = 420", "max_s = 480")]);
    assert!(
        as_form.is_ok(),
        "the same piece fits a form's cap: {as_form:?}"
    );
}

#[test]
fn a_cap_in_seconds_bounds_a_pieces_seconds() {
    let short = own_edits(&[("cap_s = 3600", "cap_s = 300")]);
    assert!(roomless(&short), "420 s past a 300 s cap");
}

#[test]
fn a_rows_longest_piece_is_whole_seconds_its_caps_take_within_the_runs_range() {
    let per_s = SOUND_HZ * SAMPLE_BYTES;
    for row in table().unwrap().rows() {
        let secs = piece_s(&row.limits);
        assert!(SpeechAsk::PIECE_S.contains(&secs), "{}", row.id);
        assert!(
            usize::from(secs) * per_s <= row.limits.piece_bytes(),
            "{}",
            row.id
        );
    }
}
