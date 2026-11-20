//! A run's model and language as both sides take them: ids in the services' shapes pass, a character that could break a request's body or
//! address is refused, `ElevenLabs`, Gemini and `AssemblyAI` refuse a run with no model before any frame, `ElevenLabs`'
//! shape holds only its language and fixed fields, Gemini's only its code, and `AssemblyAI`'s only its model header.

use super::*;
use crate::ShapeCode;
use crate::code::Code;
use IpcError::BadField;
use net_wire::{TAG, URL};

/// Made-up model ids in the shapes services use: dashes, a slash, an underscore, a dot and capitals.
const IDS: [&str; 8] = [
    "model-large-v3-turbo",
    "transcribe",
    "vendor/model-large-v3",
    "model-mini-transcribe-3-0",
    "model_v3",
    "model-3.5-transcribe",
    "Model-Transcribe-2",
    "model-3-6-pro",
];
/// A made-up header for the model, as the `AssemblyAI` shape names one.
const MODEL_HEADER: &str = "X-Kx-Model";

/// The model a read refusal changes in place.
const MDL: &str = "ab/cd";
/// The language a read refusal changes in place.
const LANG: &str = "fr-CA";

/// A plain upload form.
fn plain_form() -> NetShape<'static> {
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
        round_trips(send_as(plain_form(), Some(id), "ar-SA"));
    }
    for lang in ["fr", "ar-SA", "french", "zh_Hans"] {
        round_trips(send_as(plain_form(), Some(MODEL), lang));
    }
}

#[test]
fn a_model_or_language_that_could_break_a_body_or_an_address_is_refused_both_ways() {
    let ids = [
        "a\"b", "a\\b", "a?b", "a#b", "a%b", "a&b", "a b", "/ab", "ab/", "a//b", "a/../b",
    ];
    for id in ids {
        refused(&send_as(plain_form(), Some(id), "fr"), BadField);
    }
    for lang in ["fr/CA", "fr.CA", "fr:CA", "fr\"", "fr?C", "fr%C", "fr#C"] {
        refused(&send_as(plain_form(), Some(MODEL), lang), BadField);
    }
    let good = net_ask_bytes(&send_as(plain_form(), Some(MDL), LANG));
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
fn elevenlabs_gemini_and_assemblyai_refuse_a_run_with_no_model_both_ways() {
    round_trips(send_as(plain_form(), None, "fr"));
    for shape in [eleven(), NetShape::Gemini, assembly()] {
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

/// A send's frame for `model` with the plain form swapped for `shape`, as a writer that skipped the model rule would
/// send it.
fn bare_as(shape: &NetShape<'static>, model: Option<&'static str>) -> Vec<u8> {
    let plain = shape_bytes(&plain_form());
    let sent = net_ask_bytes(&send_as(plain_form(), model, "fr"));
    let rest = &sent[HEADER_LEN + plain.len()..];
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
    for code in [0, 2, 6, u8::MAX] {
        let bytes = patch(good.clone(), 0, &[code]);
        assert_eq!(read_net_ask(&mut &bytes[..]), Err(BadField), "{code}");
    }
}

#[test]
fn geminis_shape_is_its_code_alone_and_refuses_another_shapes_layout() {
    let test = |shape| NetAsk::Test {
        id: 1,
        call: call(shape, NetHeader::Named(Cow::Borrowed("x-goog-api-key"))),
    };
    let sent = net_ask_bytes(&test(NetShape::Gemini));
    assert_eq!(sent[HEADER_LEN], ShapeCode::Gemini.code());
    // The call's address follows the code at once, after only its own length, so the shape carries no field.
    let url = sent
        .windows(ADDRESS.len())
        .position(|w| w == ADDRESS.as_bytes());
    assert_eq!(url, Some(HEADER_LEN + 1 + URL.width));
    round_trips(test(NetShape::Gemini));
    let code = ShapeCode::Gemini.code();
    for other in [plain_form(), eleven()] {
        let layout = patch(net_ask_bytes(&test(other.clone())), 0, &[code]);
        assert!(read_net_ask(&mut &layout[..]).is_err(), "{other:?}");
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

#[test]
fn assemblyais_shape_sends_only_its_model_header_and_refuses_a_bad_name_or_another_layout() {
    let test = |shape| NetAsk::Test {
        id: 1,
        call: call(shape, NetHeader::Bearer),
    };
    let sent = net_ask_bytes(&test(assembly()));
    assert_eq!(sent[HEADER_LEN], ShapeCode::Assembly.code());
    // The header's name follows the code, then the call's address, so the shape carries no other field.
    let name = HEADER_LEN + 1 + TAG.width;
    assert_eq!(
        &sent[name..name + MODEL_HEADER.len()],
        MODEL_HEADER.as_bytes()
    );
    let url = sent
        .windows(ADDRESS.len())
        .position(|w| w == ADDRESS.as_bytes());
    assert_eq!(url, Some(name + MODEL_HEADER.len() + URL.width));
    round_trips(test(assembly()));
    let named = |name| NetShape::Assembly {
        model_header: Cow::Borrowed(name),
    };
    for bad in ["X Kx-Model", "X-Kx:Model"] {
        refused(&test(named(bad)), BadField);
    }
    assert!(test(named("")).write(&mut Vec::new()).is_err(), "no name");
    refused_swaps(&sent, b"X-Kx-Model", &[b"X Kx-Model", b"X-Kx:Model"]);
    let code = ShapeCode::Assembly.code();
    for other in [plain_form(), eleven(), NetShape::Gemini] {
        let layout = patch(net_ask_bytes(&test(other.clone())), 0, &[code]);
        assert!(read_net_ask(&mut &layout[..]).is_err(), "{other:?}");
    }
}
