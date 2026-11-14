//! The table refuses a broken row or order, so a bad edit fails its test instead of reaching a download.

use super::*;

/// The compiled-in text.
const TEXT: &str = include_str!("../../../models.toml");
/// The first row's address and fingerprint, which the edits below break.
const LINK: &str = "https://huggingface.co/handy-computer/whisper-small-gguf/resolve/4afdc7942ecf1997f359deae48a63506b062eac4/whisper-small-Q5_K_M.gguf";
const SHA: &str = "326cd00c3e7217c751667c7c1600eaf7e0de174e186ca2c16b4bf590251c3c3b";
/// The first row's commit.
const REV: &str = "4afdc7942ecf1997f359deae48a63506b062eac4";
/// The start of the English order.
const EN: &str = "\"parakeet-v3-q8\", \"whisper-small-q5km\", \"cohere-arabic-q8\"";

/// The table with each pair's first `from` replaced by its `to`, in turn.
fn edits(pairs: &[(&str, &str)]) -> Result<Table, TableError> {
    let text = pairs.iter().fold(TEXT.to_owned(), |text, (from, to)| {
        assert!(text.contains(from), "{from}");
        text.replacen(from, to, 1)
    });
    Table::parse(&text)
}

/// The row error one edit gives.
fn row_err(from: &str, to: &str) -> Option<RowError> {
    match edits(&[(from, to)]) {
        Err(TableError::Row { why, .. }) => Some(why),
        _ => None,
    }
}

/// The language and order error the edits give.
fn order_err(pairs: &[(&str, &str)]) -> Option<(String, OrderError)> {
    match edits(pairs) {
        Err(TableError::Order { lang, why }) => Some((lang, why)),
        _ => None,
    }
}

#[test]
fn the_shipped_text_parses() {
    assert_eq!(Table::parse(TEXT).map(|t| t.rows().len()), Ok(10));
}

#[test]
fn a_bad_fingerprint_is_refused() {
    let (upper, long) = (SHA.to_uppercase(), format!("{SHA}00"));
    for bad in [&SHA[1..], "zz", upper.as_str(), long.as_str()] {
        assert_eq!(row_err(SHA, bad), Some(RowError::Print), "{bad}");
    }
}

#[test]
fn an_address_that_is_not_https_at_a_fixed_revision_on_a_listed_host_is_refused() {
    let bad = [
        LINK.replace(REV, "main"),
        LINK.replace(REV, &REV[..7]),
        LINK.replace("huggingface.co", "example.com"),
        LINK.replace("Q5_K_M.gguf", "Q8_0.gguf"),
        LINK.replace("https://", "http://"),
        "https://huggingface.co/a b".to_owned(),
    ];
    for link in bad {
        assert_eq!(row_err(LINK, &link), Some(RowError::Link), "{link}");
    }
    let none = format!("links = [\"{LINK}\"]");
    assert_eq!(row_err(&none, "links = []"), Some(RowError::Link));
    let hosts = "hosts = [\"huggingface.co\", \"us.aws.cdn.hf.co\"]";
    assert_eq!(row_err(hosts, "hosts = []"), Some(RowError::Link));
}

#[test]
fn a_bad_field_is_refused_with_its_rule() {
    let file = "file = \"whisper-small-Q5_K_M.gguf\"";
    let origin = "origin = \"https://github.com/openai/whisper\"";
    let cases = [
        ("bytes = 193749056", "bytes = 0", RowError::Bytes),
        (
            "licence = \"MIT\"",
            "licence = \"GPL-3.0\"",
            RowError::Licence,
        ),
        (
            "runner = \"transcribe\"",
            "runner = \"whisper\"",
            RowError::Runner,
        ),
        (file, "file = \"whisper-small.exe\"", RowError::File),
        (file, "file = \"..\\\\x.gguf\"", RowError::File),
        (file, "file = \"NUL.gguf\"", RowError::File),
        ("cpu_mb = 1344", "cpu_mb = 0", RowError::Memory),
        ("card_mb = 384", "card_mb = 70000", RowError::Memory),
        ("name = \"Whisper Small\"", "name = \"\"", RowError::Name),
        ("credit = \"OpenAI\"", "credit = \"\"", RowError::Credit),
        (origin, "origin = \"http://x\"", RowError::Credit),
    ];
    for (from, to, why) in cases {
        assert_eq!(row_err(from, to), Some(why), "{to}");
    }
}

#[test]
fn bad_languages_are_refused() {
    let arabic = "langs = { ar = \"ar\", en = \"en\" }";
    let bad = [
        "{}",
        "{ \"fr-FR\" = \"fr\" }",
        "{ ar = \"\" }",
        "{ ar = \"ع\" }",
    ];
    for langs in bad {
        let to = format!("langs = {langs}");
        assert_eq!(row_err(arabic, &to), Some(RowError::Langs), "{langs}");
    }
}

#[test]
fn a_duplicate_or_odd_id_is_refused() {
    let small = "id = \"whisper-small-q5km\"";
    let twice = edits(&[("id = \"whisper-turbo-q5km\"", small)]);
    assert!(matches!(twice, Err(TableError::Id(id)) if id == "whisper-small-q5km"));
    let odd = edits(&[(small, "id = \"Whisper Small\"")]);
    assert!(matches!(odd, Err(TableError::Id(_))));
}

#[test]
fn an_order_that_breaks_a_rule_is_refused() {
    let en = |why| Some(("en".to_owned(), why));
    let tail = "\"qwen3-0.6b-q8\", \"nemotron-0.6b-q8\",\n]\n\n[[orders]]\nlang = \"fr\"";
    let first = EN.replace("parakeet-v3-q8", "nemotron-0.6b-q8");
    let last = tail.replace("nemotron-0.6b-q8", "parakeet-v3-q8");
    assert_eq!(
        order_err(&[(EN, &first), (tail, &last)]),
        en(OrderError::Lead)
    );
    let short = "\"parakeet-v3-q8\", \"whisper-small-q5km\"";
    assert_eq!(order_err(&[(EN, short)]), en(OrderError::Missing));
    let gone = format!("\"gone-q8\", {EN}");
    assert_eq!(order_err(&[(EN, &gone)]), en(OrderError::Unknown));
    let twice = format!("\"parakeet-v3-q8\", {EN}");
    assert_eq!(order_err(&[(EN, &twice)]), en(OrderError::Twice));
    let ar = "\"cohere-q8\", \"whisper-large-v3-q5km\"";
    let stranger = "\"cohere-q8\", \"canary-v2-q8\", \"whisper-large-v3-q5km\"";
    let ar_err = Some(("ar".to_owned(), OrderError::Stranger));
    assert_eq!(order_err(&[(ar, stranger)]), ar_err);
    assert_eq!(
        order_err(&[("lang = \"fr\"", "lang = \"en\"")]),
        en(OrderError::Lang)
    );
}

#[test]
fn a_general_order_that_misses_a_row_or_lets_nemotron_lead_is_refused() {
    let general = "    \"parakeet-v3-q8\", \"cohere-arabic-q8\", \"cohere-q8\", \"whisper-turbo-q5km\", \"canary-v2-q8\", \"whisper-large-v3-q5km\",\n    \"qwen3-1.7b-q8\", \"whisper-small-q5km\", \"qwen3-0.6b-q8\", \"nemotron-0.6b-q8\",";
    let missing = general.replace(" \"qwen3-0.6b-q8\",", "");
    let err = |to: &str| edits(&[(general, to)]).err();
    assert_eq!(
        err(&missing),
        Some(TableError::General(OrderError::Missing))
    );
    let rest = general.replace(" \"nemotron-0.6b-q8\",", "");
    let nemotron_first = format!("    \"nemotron-0.6b-q8\",{rest}");
    assert_eq!(
        err(&nemotron_first),
        Some(TableError::General(OrderError::Lead))
    );
}
