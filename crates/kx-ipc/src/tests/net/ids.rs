//! A run's model and language as both sides take them: real ids pass, a character that could break a request's body or
//! address is refused, and `ElevenLabs` and Gemini refuse a run with no model before any frame.

use super::*;
use crate::ShapeCode;
use crate::code::Code;
use IpcError::BadField;

/// Real model ids from the services' own pages.
const IDS: [&str; 7] = [
    "whisper-large-v3-turbo",
    "gpt-transcribe",
    "openai/whisper-large-v3",
    "voxtral-mini-transcribe-3-0",
    "scribe_v3",
    "gemini-3.5-transcribe",
    "MAI-Transcribe-2",
];

/// The model a read refusal changes in place.
const MDL: &str = "ab/cd";
/// The language a read refusal changes in place.
const LANG: &str = "fr-CA";

/// Groq's upload form.
fn groq_form() -> NetShape<'static> {
    form("language", false, Vec::new())
}

/// A send to `shape` of one piece, for `model` in `lang`.
fn send_as(
    shape: NetShape<'static>,
    model: Option<&'static str>,
    lang: &'static str,
) -> NetAsk<'static> {
    let run = NetRun {
        model: model.map(Cow::Borrowed),
        lang: Cow::Borrowed(lang),
        ..run(vec![spoken()])
    };
    NetAsk::Send {
        id: 1,
        call: call(shape, NetHeader::Bearer),
        run,
    }
}

/// Checks that `ask` reads back as itself.
fn round_trips(ask: NetAsk<'static>) {
    assert_eq!(NetAsk::read(&mut &net_ask_bytes(&ask)[..]), Ok(ask));
}

/// Checks that `good` with its first `from` changed to each of `tos` is refused when read.
fn refused_swaps(good: &[u8], from: &[u8], tos: &[&[u8]]) {
    let at = good.windows(from.len()).position(|w| w == from).unwrap();
    for to in tos {
        let mut bytes = good.to_vec();
        bytes[at..at + to.len()].copy_from_slice(to);
        assert_eq!(read_net_ask(&mut &bytes[..]), Err(BadField), "{to:?}");
    }
}

#[test]
fn real_model_ids_and_language_codes_round_trip() {
    for id in IDS {
        round_trips(send_as(groq_form(), Some(id), "ar-SA"));
    }
    for lang in ["fr", "ar-SA", "french", "zh_Hans"] {
        round_trips(send_as(groq_form(), Some(MODEL), lang));
    }
}

#[test]
fn a_model_or_language_that_could_break_a_body_or_an_address_is_refused_both_ways() {
    let ids = [
        "a\"b", "a\\b", "a?b", "a#b", "a%b", "a&b", "a b", "/ab", "ab/", "a//b", "a/../b",
    ];
    for id in ids {
        refused(&send_as(groq_form(), Some(id), "fr"), BadField);
    }
    for lang in ["fr/CA", "fr.CA", "fr:CA", "fr\"", "fr?C", "fr%C", "fr#C"] {
        refused(&send_as(groq_form(), Some(MODEL), lang), BadField);
    }
    let good = net_ask_bytes(&send_as(groq_form(), Some(MDL), LANG));
    refused_swaps(
        &good,
        MDL.as_bytes(),
        &[b"a\"/cd", b"ab//d", b"ab/..", b"ab/c?", b"/b/cd"],
    );
    refused_swaps(
        &good,
        LANG.as_bytes(),
        &[b"fr/CA", b"fr.CA", b"fr:CA", b"fr\"CA"],
    );
}

#[test]
fn elevenlabs_and_gemini_refuse_a_run_with_no_model_both_ways() {
    for shape in [NetShape::Eleven, NetShape::Gemini] {
        refused(&send_as(shape.clone(), None, "fr"), BadField);
        round_trips(send_as(shape, Some(MODEL), "fr"));
    }
    for shape in [NetShape::Azure, groq_form()] {
        round_trips(send_as(shape, None, "fr"));
    }
    let azure = net_ask_bytes(&send_as(NetShape::Azure, None, "fr"));
    for shape in [ShapeCode::Eleven, ShapeCode::Gemini] {
        let bytes = patch(azure.clone(), 0, &[shape.code()]);
        assert_eq!(read_net_ask(&mut &bytes[..]), Err(BadField), "{shape:?}");
    }
}
