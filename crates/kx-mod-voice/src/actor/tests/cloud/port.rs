//! The cloud port and the row's limits: with no port a cloud dictation ends with the fault note and keeps its
//! recording, a listen-only run cuts pieces no longer than the row's size and time caps both allow, and `ElevenLabs`,
//! Gemini and `AssemblyAI` dictations ask in their own shapes with the key under their row's header.

use kx_module_api::ports::{CloudLimits, CloudShape, SOUND_BYTES_PER_S, SpeechAsk};

use super::*;
use crate::services;
use crate::services::piece_s;
use crate::services::tests::shipped::{entry, pick};

/// Shipped row `id` on at its terms, picked for English with its first model, and its key set.
fn on_first_model(fake: &Fakes, host: &mut FakeHost, id: &str) {
    let row = services::table().unwrap().get(id).unwrap();
    host.put("cloud", true.into());
    host.put("services", table(&entry(row, "")));
    host.put("picks", table(&format!("en = \"{}\"", pick(row, 0))));
    let keys = Keys::new(Arc::new(fake.shelf.clone()), false);
    keys.set(id, secret(&key())).unwrap();
}

/// Shipped row `id`'s first model id.
fn first_id(id: &str) -> &'static str {
    &services::table().unwrap().get(id).unwrap().models[0].id
}

/// `ElevenLabs` on and keyed, picked for English with its first model.
pub(super) fn eleven(fake: &Fakes, host: &mut FakeHost) {
    on_first_model(fake, host, "elevenlabs");
}

#[test]
fn an_elevenlabs_dictation_asks_in_its_own_shape_with_its_own_key_header() {
    let mut rig = Rig::build(eleven);
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("hello"), CloudAnswer::text("world")]);
    rig.dictate();
    let runs = rig.fake.cloud.calls().runs;
    let [run] = &runs[..] else {
        panic!("one run, not {}", runs.len())
    };
    let row = services::table()
        .ok()
        .and_then(|t| t.get("elevenlabs"))
        .unwrap();
    assert!(matches!(run.shape, CloudShape::Eleven(_)), "its own shape");
    assert_eq!(
        run.shape, row.shape,
        "the row's form, as the table gives it"
    );
    let entry = &rig.actor.settings().services["elevenlabs"];
    assert_eq!(
        Some(run.url.clone()),
        row.url(&entry.address),
        "its address"
    );
    assert_eq!(run.header, row.header, "the row's header");
    let model = run.model.as_ref().map(|m| m.expose().as_str());
    assert_eq!(
        (model, run.lang.as_str()),
        (
            Some(first_id("elevenlabs")),
            row.models[0].code("en").unwrap_or_default()
        )
    );
    assert_eq!(rig.typed_texts(), ["hello world"]);
}

/// Gemini on and keyed, picked for English with its first model.
fn gemini(fake: &Fakes, host: &mut FakeHost) {
    on_first_model(fake, host, "gemini");
}

#[test]
fn a_gemini_dictation_asks_in_its_own_shape_with_its_bcp_47_code_and_its_own_key_header() {
    let mut rig = Rig::build(gemini);
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("hello"), CloudAnswer::text("world")]);
    rig.dictate();
    let runs = rig.fake.cloud.calls().runs;
    let [run] = &runs[..] else {
        panic!("one run, not {}", runs.len())
    };
    let row = services::table()
        .ok()
        .and_then(|t| t.get("gemini"))
        .unwrap();
    assert_eq!(run.shape, CloudShape::Gemini, "its own shape, with no form");
    let entry = &rig.actor.settings().services["gemini"];
    assert_eq!(
        Some(run.url.clone()),
        row.url(&entry.address),
        "its address"
    );
    assert_eq!(run.header, row.header, "the row's header");
    assert_eq!(
        run.limits, row.limits,
        "the row's limits, under its request cap"
    );
    let model = run.model.as_ref().map(|m| m.expose().as_str());
    let sent = (model, run.lang.as_str());
    let code = row.models[0].code("en");
    assert_eq!(sent, (Some(first_id("gemini")), code.unwrap_or_default()));
    assert_eq!(rig.typed_texts(), ["hello world"]);
}

/// `AssemblyAI` on and keyed, picked for English with its first model.
fn assembly(fake: &Fakes, host: &mut FakeHost) {
    on_first_model(fake, host, "assemblyai");
}

#[test]
fn an_assemblyai_dictation_asks_in_its_own_shape_with_its_model_header_and_its_rows_key_header() {
    let mut rig = Rig::build(assembly);
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("hello"), CloudAnswer::text("world")]);
    rig.dictate();
    let runs = rig.fake.cloud.calls().runs;
    let [run] = &runs[..] else {
        panic!("one run, not {}", runs.len())
    };
    let row = services::table()
        .ok()
        .and_then(|t| t.get("assemblyai"))
        .unwrap();
    assert!(matches!(run.shape, CloudShape::Assembly { .. }));
    assert_eq!(run.shape, row.shape, "the row's model header");
    assert_eq!((&run.header, run.limits), (&row.header, row.limits));
    let model = run.model.as_ref().map(|m| m.expose().as_str());
    assert_eq!(
        (model, run.lang.as_str()),
        (
            Some(first_id("assemblyai")),
            row.models[0].code("en").unwrap_or_default()
        )
    );
    assert_eq!(rig.typed_texts(), ["hello world"]);
}

/// Limits of `max_mb` and `max_s`, the rest at their least.
fn limits(max_mb: u32, max_s: u16) -> CloudLimits {
    CloudLimits {
        max_mb,
        max_s,
        per_min: *CloudLimits::PER_MIN.start(),
        at_once: *CloudLimits::AT_ONCE.start(),
    }
}

#[test]
fn with_no_cloud_port_a_cloud_dictation_ends_with_the_fault_note_and_keeps_the_recording() {
    let mut rig = Rig::build(groq);
    rig.actor.ports.cloud = None;
    rig.dictate();
    assert!(rig.fake.cloud.calls().runs.is_empty(), "nothing sent");
    assert!(rig.asks().is_empty(), "nothing typed");
    assert_eq!(rig.bar(), cloud_note(CloudWhy::Fault));
    assert!(rig.fake.speech.runs()[0].held, "the recording stays");
}

#[test]
fn a_rows_size_cap_cuts_pieces_shorter_than_its_time_cap() {
    let per_s = SOUND_BYTES_PER_S;
    let small = limits(*CloudLimits::MAX_MB.start(), *CloudLimits::MAX_S.end());
    let cut = piece_s(&small);
    assert!(
        cut < *SpeechAsk::PIECE_S.end(),
        "read from the row, below the clamp"
    );
    assert!(
        usize::from(cut) * per_s <= small.piece_bytes(),
        "each piece fits"
    );
    assert!(
        usize::from(cut + 1) * per_s > small.piece_bytes(),
        "as long as fits"
    );
    let timed = limits(*CloudLimits::MAX_MB.end(), *CloudLimits::MAX_S.start());
    assert_eq!(
        u16::from(piece_s(&timed)),
        timed.max_s,
        "the time cap rules"
    );
}
