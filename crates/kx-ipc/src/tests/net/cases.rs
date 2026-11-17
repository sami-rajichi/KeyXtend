//! The net tests' cases and helpers: one ask of each shape and kind, one note of each kind and fault, and the readers
//! each side uses.

use super::super::*;
use crate::net::code::FaultCode;
use crate::{
    NetAsk, NetCall, NetFault, NetField, NetForm, NetHeader, NetLimits, NetNote, NetRun, NetShape,
    Wiped,
};
use zeroize::Zeroizing;

/// A key no output may show.
pub(super) const KEY: &str = "gsk_live-7731-zebra";
/// Sound the tests send, an even length so it is whole samples; no output may show it.
pub(super) const SPOKEN: &[u8] = b"spoken words";
/// Groq's address, which no output may show, as a typed address is the user's.
pub(super) const GROQ: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
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
    })
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

/// Groq's limits on its free plan.
pub(super) fn limits() -> NetLimits {
    NetLimits {
        max_mb: 25,
        max_s: 3_600,
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

/// A run sent to each shape: Groq, `OpenAI`'s list, `OpenRouter`'s keep-nothing field, Azure with no model,
/// `ElevenLabs` and Gemini.
fn sends() -> Vec<NetAsk<'static>> {
    let zdr = vec![field("provider", r#"{"zdr":true}"#)];
    let named = |name| NetHeader::Named(Cow::Borrowed(name));
    let azure = NetAsk::Send {
        id: 4,
        call: call(NetShape::Azure, named("Ocp-Apim-Subscription-Key")),
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
        send(3, call(form("language", false, zdr), NetHeader::Bearer)),
        azure,
        send(5, call(NetShape::Eleven, named("xi-api-key"))),
        send(6, call(NetShape::Gemini, named("x-goog-api-key"))),
    ]
}

/// One ask of each kind and shape.
pub(super) fn every_net_ask() -> Vec<NetAsk<'static>> {
    let mut asks = sends();
    asks.extend([
        NetAsk::Sound {
            id: 7,
            piece: 0,
            bytes: Wiped::Lent(SPOKEN),
        },
        NetAsk::Sound {
            id: 8,
            piece: 2,
            bytes: Wiped::Kept(Zeroizing::new(SPOKEN.to_vec())),
        },
        NetAsk::Test {
            id: 9,
            call: groq(),
        },
        NetAsk::Cancel { id: 10 },
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
