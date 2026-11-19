//! The net tests' cases and helpers: one ask of each shape and kind, one note of each kind and fault, and the readers
//! each side uses.

use super::super::*;
use crate::net::code::FaultCode;
use crate::{
    NetAsk, NetCall, NetFault, NetField, NetForm, NetHeader, NetLimits, NetNote, NetOwn, NetRun,
    NetShape, Wiped,
};
use zeroize::Zeroizing;

/// A key no output may show.
pub(super) const KEY: &str = "gsk_live-7731-zebra";
/// Sound the tests send, an even length so it is whole samples; no output may show it.
pub(super) const SPOKEN: &[u8] = b"spoken words";
/// Groq's address, which no output may show, as a typed address is the user's.
pub(super) const GROQ: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
/// `OpenAI`'s error code for a country or region it does not serve [O5].
pub(super) const REGION: &str = "unsupported_country_region_territory";
/// A model id no output may show, as a typed one is the user's.
pub(super) const MODEL: &str = "whisper-large-v3-turbo";
/// Words no `Debug` may show: the key, the address, the model, the sound and the texts.
pub(super) const SECRETS: &[&str] = &[KEY, "groq", MODEL, "spoken", "bonjour", "مرحبا"];

pub(super) fn net_ask_bytes(ask: &NetAsk) -> Vec<u8> {
    let mut out = Vec::new();
    ask.write(&mut out).unwrap();
    out
}

pub(super) fn net_note_bytes(note: &NetNote) -> Vec<u8> {
    let mut out = Vec::new();
    note.write(&mut out).unwrap();
    out
}

pub(super) fn read_net_ask(r: &mut dyn Read) -> Result<(), IpcError> {
    NetAsk::read(r).map(drop)
}

pub(super) fn read_net_note(r: &mut dyn Read) -> Result<(), IpcError> {
    NetNote::read(r).map(drop)
}

pub(super) fn text(s: &str) -> Zeroizing<String> {
    Zeroizing::new(s.to_string())
}

/// An upload form naming its language `lang`, as a list when `list`, with `fixed` fields.
pub(super) fn form(
    lang: &'static str,
    list: bool,
    fixed: Vec<NetField<'static>>,
) -> NetShape<'static> {
    NetShape::Form(NetForm {
        lang: Cow::Borrowed(lang),
        list,
        fixed,
        paywall: false,
        region: None,
    })
}

/// `shape` with its 403 read as a paywall, as Together's and `OpenRouter`'s rows say.
pub(super) fn paywalled(shape: NetShape<'static>) -> NetShape<'static> {
    match shape {
        NetShape::Form(form) => NetShape::Form(NetForm {
            paywall: true,
            ..form
        }),
        other => other,
    }
}

/// `shape` with the error code `code` read as a region refusal, as `OpenAI`'s row says.
pub(super) fn regioned(shape: NetShape<'static>, code: &'static str) -> NetShape<'static> {
    match shape {
        NetShape::Form(form) => NetShape::Form(NetForm {
            region: Some(Cow::Borrowed(code)),
            ..form
        }),
        other => other,
    }
}

/// A one-letter form whose replies name `code` their region refusal.
pub(super) fn region_form(code: &'static str) -> NetShape<'static> {
    regioned(form("l", false, Vec::new()), code)
}

/// `ElevenLabs`' shape, naming its language field and asking for no sound tags, as its row says.
pub(super) fn eleven() -> NetShape<'static> {
    NetShape::Eleven(NetOwn {
        lang: Cow::Borrowed("language_code"),
        fixed: vec![field("tag_audio_events", "false")],
    })
}

/// `AssemblyAI`'s shape, naming the header its model goes in, as its row says.
pub(super) fn assembly() -> NetShape<'static> {
    NetShape::Assembly {
        model_header: Cow::Borrowed("X-AAI-Model"),
    }
}

/// A fixed field.
pub(super) fn field(name: &'static str, value: &'static str) -> NetField<'static> {
    NetField {
        name: Cow::Borrowed(name),
        value: Cow::Borrowed(value),
    }
}

/// A call to Groq with `shape` and `header`, lending the key.
pub(super) fn call(shape: NetShape<'static>, header: NetHeader<'static>) -> NetCall<'static> {
    NetCall {
        shape,
        url: Cow::Borrowed(GROQ),
        header,
        key: Wiped::Lent(KEY.as_bytes()),
        wait_s: 60,
    }
}

/// Groq's call: its form and `Authorization: Bearer`.
pub(super) fn groq() -> NetCall<'static> {
    call(form("language", false, Vec::new()), NetHeader::Bearer)
}

/// Groq's limits on its free plan, at most ten minutes a request.
pub(super) fn limits() -> NetLimits {
    NetLimits {
        max_mb: 25,
        max_s: 600,
        per_min: 20,
        at_once: 2,
    }
}

/// A French run of pieces of `sizes` bytes for the turbo model.
pub(super) fn run(sizes: Vec<u32>) -> NetRun<'static> {
    NetRun {
        model: Some(Cow::Borrowed(MODEL)),
        lang: Cow::Borrowed("fr"),
        limits: limits(),
        tries: 2,
        sizes,
    }
}

/// `SPOKEN`'s length, one piece.
pub(super) fn spoken() -> u32 {
    u32::try_from(SPOKEN.len()).unwrap()
}

/// A run sent with `call`, of one piece of `SPOKEN`.
pub(super) fn send(id: u32, call: NetCall<'static>) -> NetAsk<'static> {
    NetAsk::Send {
        id,
        call,
        run: run(vec![spoken()]),
    }
}

/// A run sent to each shape: Groq, `OpenAI`'s list, `OpenRouter`'s keep-nothing field and paywall with a region code,
/// a form with no model and its key in a named header, `ElevenLabs`, Gemini and `AssemblyAI`.
fn sends() -> Vec<NetAsk<'static>> {
    let zdr = vec![field("provider", r#"{"zdr":true}"#)];
    let named = |name| NetHeader::Named(Cow::Borrowed(name));
    let unnamed = NetAsk::Send {
        id: 4,
        call: call(form("language", false, Vec::new()), named("x-api-key")),
        run: NetRun {
            model: None,
            lang: Cow::Borrowed("ar-SA"),
            ..run(vec![2, 4, spoken()])
        },
    };
    vec![
        send(1, groq()),
        send(
            2,
            call(form("languages", true, Vec::new()), NetHeader::Bearer),
        ),
        send(
            3,
            call(
                regioned(paywalled(form("language", false, zdr)), REGION),
                NetHeader::Bearer,
            ),
        ),
        unnamed,
        send(5, call(eleven(), named("xi-api-key"))),
        send(6, call(NetShape::Gemini, named("x-goog-api-key"))),
        send(7, call(assembly(), NetHeader::Bearer)),
    ]
}

/// One ask of each kind and shape.
pub(super) fn every_net_ask() -> Vec<NetAsk<'static>> {
    let mut asks = sends();
    asks.extend([
        NetAsk::Sound {
            id: 8,
            piece: 0,
            bytes: Wiped::Lent(SPOKEN),
        },
        NetAsk::Sound {
            id: 9,
            piece: 2,
            bytes: Wiped::Kept(Zeroizing::new(SPOKEN.to_vec())),
        },
        NetAsk::Test {
            id: 10,
            call: groq(),
        },
        NetAsk::Cancel { id: 11 },
        NetAsk::Bye,
    ]);
    asks
}

/// Every fault, the allowance with no renewal told.
pub(super) fn every_fault() -> Vec<NetFault> {
    let all = FaultCode::ALL.iter();
    all.map(|&code| NetFault::of(code, None)).collect()
}

/// One note of each kind: texts in two scripts and an empty one, the end, each fault and a told renewal.
pub(super) fn every_net_note() -> Vec<NetNote> {
    let piece = |index, s| NetNote::Text {
        id: 1,
        index,
        text: text(s),
    };
    let mut notes = vec![
        piece(0, "bonjour à tous"),
        piece(1, "مرحبا بكم"),
        piece(2, ""),
        NetNote::Done { id: 1 },
    ];
    let faults = every_fault().into_iter();
    notes.extend(faults.map(|fault| NetNote::Fault { id: 2, fault }));
    notes.push(NetNote::Fault {
        id: 3,
        fault: NetFault::Allowance {
            renews_s: Some(5_400),
        },
    });
    notes
}

/// `bytes` with the payload bytes from `at` set to `value`.
pub(super) fn patch(mut bytes: Vec<u8>, at: usize, value: &[u8]) -> Vec<u8> {
    let at = HEADER_LEN + at;
    bytes[at..at + value.len()].copy_from_slice(value);
    bytes
}

/// Checks that writing `ask` is refused with `err` and writes nothing.
pub(super) fn refused(ask: &NetAsk, err: IpcError) {
    let mut out = Vec::new();
    assert_eq!(ask.write(&mut out), Err(err), "{ask:?}");
    assert!(out.is_empty());
}

/// Checks that each `(at, value)` patch of `good` is refused with `err` when read.
pub(super) fn refused_reads(good: &[u8], reads: &[(usize, &[u8])], err: IpcError) {
    for &(at, value) in reads {
        let bytes = patch(good.to_vec(), at, value);
        assert_eq!(read_net_ask(&mut &bytes[..]), Err(err), "{at} {value:?}");
    }
}
