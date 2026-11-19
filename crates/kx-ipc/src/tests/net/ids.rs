//! A run's model and language as both sides take them: real ids pass, a character that could break a request's body or
//! address is refused, `ElevenLabs` and Gemini refuse a run with no model before any frame, and `ElevenLabs`' shape
//! holds only its language and fixed fields.

use super::*;
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
    round_trips(send_as(groq_form(), None, "fr"));
    for shape in [eleven(), NetShape::Gemini] {
        refused(&send_as(shape.clone(), None, "fr"), BadField);
        round_trips(send_as(shape.clone(), Some(MODEL), "fr"));
        let read = |model| NetAsk::read(&mut &bare_as(&shape, model)[..]);
        assert_eq!(
            read(Some(MODEL)),
            Ok(send_as(shape.clone(), Some(MODEL), "fr"))
        );
        assert_eq!(read(None), Err(BadField), "{shape:?}");
    }
}

/// `shape`'s own bytes in a frame: its code, then its form if it has one.
fn shape_bytes(shape: &NetShape<'static>) -> Vec<u8> {
    let test = |s: &NetShape<'static>| {
        let call = call(s.clone(), NetHeader::Bearer);
        net_ask_bytes(&NetAsk::Test { id: 1, call })
    };
    let own = test(shape);
    let len = own.len() + 1 - test(&NetShape::Gemini).len();
    own[HEADER_LEN..HEADER_LEN + len].to_vec()
}

/// A send's frame for `model` with Groq's shape swapped for `shape`, as a writer that skipped the model rule would
/// send it.
fn bare_as(shape: &NetShape<'static>, model: Option<&'static str>) -> Vec<u8> {
    let groq = shape_bytes(&groq_form());
    let sent = net_ask_bytes(&send_as(groq_form(), model, "fr"));
    let rest = &sent[HEADER_LEN + groq.len()..];
    frame(Kind::NetSend, 1, &[&shape_bytes(shape)[..], rest].concat())
}

#[test]
fn a_shape_code_no_shape_has_is_refused_when_read_the_dropped_2_too() {
    let test = NetAsk::Test {
        id: 1,
        call: call(NetShape::Gemini, NetHeader::Bearer),
    };
    let good = net_ask_bytes(&test);
    assert_eq!(NetAsk::read(&mut &good[..]), Ok(test));
    // Code 2 was a fourth shape's until ADR-0050 dropped it; its byte now reads as no shape at all.
    for code in [0, 2, 5, u8::MAX] {
        let bytes = patch(good.clone(), 0, &[code]);
        assert_eq!(read_net_ask(&mut &bytes[..]), Err(BadField), "{code}");
    }
}

#[test]
fn elevenlabs_shape_sends_only_its_language_and_fixed_fields_and_refuses_a_forms_layout() {
    let NetShape::Eleven(own) = eleven() else {
        unreachable!("its own form")
    };
    let as_form = form("language_code", false, own.fixed.clone());
    // An upload form's list, paywall and region flags are the three bytes `ElevenLabs` never sends.
    assert_eq!(
        shape_bytes(&eleven()).len() + 3,
        shape_bytes(&as_form).len()
    );
    let test = |shape| NetAsk::Test {
        id: 1,
        call: call(shape, NetHeader::Bearer),
    };
    round_trips(test(eleven()));
    let code = shape_bytes(&eleven())[0];
    let layout = patch(net_ask_bytes(&test(as_form)), 0, &[code]);
    assert!(read_net_ask(&mut &layout[..]).is_err(), "a form's layout");
    let spaced = NetOwn {
        lang: Cow::Borrowed("language code"),
        ..own.clone()
    };
    refused(&test(NetShape::Eleven(spaced)), BadField);
    let many = NetOwn {
        fixed: vec![field("a", "1"); MAX_FIXED + 1],
        ..own
    };
    refused(&test(NetShape::Eleven(many)), IpcError::TooLong);
}
