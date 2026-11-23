//! A cloud dictation over the whole app (ADR-0047): the speech leaves only at Stop, through the cloud port the app
//! provides, and the keyboard types the service's text; a key set on its line or pasted is sealed on the voice shelf
//! and reaches neither the ui's view nor the keyboard; a hidden run offers no service and reads no key.

mod typed;
mod watched;

use super::{ENGLISH, INSERTS, done, listen, rig, speak, start, stop};
use crate::rig::{APP, Rig, TODAY, WAIT, hear};
use keyxtend::boot::Boot;
use keyxtend::hidden::Run;
use kx_mod_voice::cloud::pick_text;
use kx_mod_voice::keys::{KEYS, Keys};
use kx_mod_voice::services::{Service, table};
use kx_module_api::ports::{
    BlobName, ClipPart, CloudButton, CloudLine, SOUND_BYTES_PER_S, SealedShelf, ServiceSheet,
    TextAsk, TextDone, VoiceAct, VoiceClick, VoiceOpen, VoiceSheet, VoiceTab, VoiceView,
};
use kx_module_api::{Bus, Redacted, Secret, Zeroizing};
use kx_platform_fake::CloudAnswer;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use watched::Watched;

/// Tenths in a second.
const TENTHS: usize = 10;
/// A tenth of a second of sound, as one piece.
const PIECE: usize = SOUND_BYTES_PER_S / TENTHS;
/// The texts the service gives for the two pieces.
const TEXTS: [&str; 2] = ["hello", "world"];
/// What the keyboard is asked to type of them.
const TYPED: &str = "hello world";

/// A test's own key, built as it runs so no key-like text sits in the source.
fn key(test: &str) -> String {
    ["kx", "life", test].join("-")
}

/// `text` as a secret.
fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// The service the tests switch on: the first on the Cloud page, as the shipped table orders them.
fn service() -> &'static Service {
    table().unwrap().offered().next().unwrap()
}

/// Switches `service` on with its own terms confirmed on the rig's day, and picks its first model that knows English
/// for English, in an order each start keeps.
fn switch_on(boot: &mut Boot) {
    let (s, (year, month, day)) = (service(), TODAY);
    let model = s.models.iter().find(|m| m.code(ENGLISH).is_some()).unwrap();
    let on = format!("{year:04}-{month:02}-{day:02}");
    let entry = format!("{} = {{ on = {on}, terms = {} }}", s.id, s.terms);
    let pick = format!("{ENGLISH} = \"{}\"", pick_text(&s.id, &model.id));
    let table = |t: &str| toml::Value::Table(t.parse().unwrap());
    let keys = [
        ("cloud", true.into()),
        ("services", table(&entry)),
        ("picks", table(&pick)),
    ];
    for (key, value) in keys {
        boot.kernel
            .set_setting(kx_mod_voice::ID, key, value)
            .unwrap();
    }
}

/// The service's key blob on the rig's voice shelf, as the fake keeps it.
fn sealed(rig: &Rig) -> Option<Vec<u8>> {
    let (folder, name) = (
        BlobName::new(KEYS).unwrap(),
        BlobName::new(&service().id).unwrap(),
    );
    rig.voice.blob(&folder, &name)
}

/// Checks that no `keys` show in the ui's voice view or the keyboard's face, that no text the keyboard was asked to
/// type in `asks` holds one, and that none went to the clipboard.
fn unseen(rig: &Rig, boot: &Boot, asks: &[TextAsk], keys: &[String]) {
    let (view, face) = (boot.hub.voice_json(), boot.hub.board_json());
    let writes = rig.writer.writes();
    let parts = writes.iter().flatten();
    let texts: Vec<&str> = parts
        .filter_map(|p| match p {
            ClipPart::Text(t) => Some(t.expose().as_str()),
            _ => None,
        })
        .collect();
    for key in keys {
        assert!(!view.contains(key.as_str()), "the ui's view");
        assert!(!face.contains(key.as_str()), "the keyboard's face");
        assert!(asks.iter().all(|a| !a.text.expose().contains(key.as_str())));
        assert!(texts.iter().all(|t| !t.contains(key.as_str())), "no copy");
    }
}

/// Waits for a view whose panel `shows`; its round.
fn shown(views: &Receiver<VoiceView>, shows: impl Fn(&VoiceSheet) -> bool) -> u32 {
    loop {
        let view = views.recv_timeout(WAIT).unwrap();
        if view.0.panel.as_ref().is_some_and(|p| shows(&p.sheet)) {
            return view.0.round;
        }
    }
}

/// Clicks `click` on the view of `round`, as the ui does, then waits for a panel that `shows`; its round.
fn click(
    bus: &Bus,
    views: &Receiver<VoiceView>,
    (round, click): (u32, VoiceClick),
    shows: impl Fn(&VoiceSheet) -> bool,
) -> u32 {
    bus.publish(&VoiceAct { round, click }).unwrap();
    shown(views, shows)
}

/// Opens the Voice panel on the service's key page, its row the first; the round of that view.
fn key_page(bus: &Bus, views: &Receiver<VoiceView>) -> u32 {
    bus.publish(&VoiceOpen).unwrap();
    let round = shown(views, |_| true);
    let cloud = VoiceClick::Open(VoiceTab::Cloud);
    let round = click(bus, views, (round, cloud), |s| {
        matches!(s, VoiceSheet::Cloud(_))
    });
    let ticked =
        |s: &VoiceSheet| matches!(s, VoiceSheet::Cloud(c) if c.acts.contains(&CloudButton::Facts));
    let round = click(bus, views, (round, VoiceClick::Row(0)), ticked);
    let lines = VoiceClick::Service(CloudButton::Lines);
    click(bus, views, (round, lines), |s| {
        matches!(s, VoiceSheet::Service(_))
    })
}

/// Waits for the key page to show a key of `n` characters as dots; the round of that view.
fn dots(views: &Receiver<VoiceView>, n: usize) -> u32 {
    let want = Redacted::new(u32::try_from(n).unwrap());
    shown(
        views,
        |s| matches!(s, VoiceSheet::Service(ServiceSheet { key, .. }) if *key == want),
    )
}

/// The app over `rig` as `run` says, with the service's `key` sealed on a watched voice shelf, started with Type it
/// and the service switched on for English.
fn keyed(rig: &Rig, run: &Run, key: &str) -> (Boot, Arc<Watched>) {
    let keys = Keys::new(Arc::new(rig.voice.clone()), false);
    keys.set(&service().id, secret(key)).unwrap();
    let shelf = Arc::new(Watched::new(rig.voice.clone()));
    let mut services = rig.services();
    services.voice_shelf = Arc::clone(&shelf) as Arc<dyn SealedShelf>;
    let mut boot = rig.load_with(run, services);
    start(&mut boot, INSERTS[0].0);
    switch_on(&mut boot);
    (boot, shelf)
}

#[test]
fn a_cloud_dictation_leaves_only_at_stop_and_the_keyboard_types_the_services_text() {
    let rig = rig();
    let used = key("dictate");
    rig.cloud
        .script(TEXTS.iter().map(|t| CloudAnswer::text(t)).collect());
    rig.speech.set_speech(vec![vec![0; PIECE]; TEXTS.len()]);
    let (mut boot, shelf) = keyed(&rig, &Run::Normal, &used);
    let bus = boot.kernel.bus();
    let ((_asks, asks), (_done, typed)) = (hear::<TextAsk>(&bus), hear::<TextDone>(&bus));
    listen(&rig, &boot);
    assert!(rig.speech.runs()[0].ask.model.is_none(), "listen-only");
    assert!(
        rig.cloud.calls().runs.is_empty(),
        "nothing while it listens"
    );
    stop(&rig, &boot);
    assert!(
        rig.cloud.calls().runs.is_empty(),
        "nothing before the hand-over"
    );
    assert!(rig.speech.tell(done(0)), "the run hands its speech over");
    let typed = typed.recv_timeout(WAIT).unwrap();
    assert!(typed.ok && typed.count > 0, "{typed:?}");
    let runs = rig.cloud.calls().runs;
    assert_eq!(runs.len(), 1, "one upload");
    assert_eq!((runs[0].lang.as_str(), runs[0].sizes.len()), (ENGLISH, 2));
    assert!(shelf.key_uses() > 0, "the key was read for it");
    // The keyboard answers inside its own handler, so the ask can reach this later listener after the answer.
    let first = asks.recv_timeout(WAIT).expect("the keyboard was asked");
    let asked: Vec<TextAsk> = std::iter::once(first).chain(asks.try_iter()).collect();
    assert!(
        asked[0].text.expose().starts_with(TYPED),
        "the service's text"
    );
    unseen(&rig, &boot, &asked, &[used]);
    boot.kernel.stop_all();
}

#[test]
fn a_key_set_on_its_line_or_pasted_is_sealed_and_reaches_neither_the_ui_nor_the_keyboard() {
    let rig = rig();
    let mut boot = rig.load();
    boot.start().unwrap();
    switch_on(&mut boot);
    let bus = boot.kernel.bus();
    let ((_asks, asks), (_views, views)) = (hear::<TextAsk>(&bus), hear::<VoiceView>(&bus));
    let round = key_page(&bus, &views);
    let (typed, pasted) = (key("line"), key("pasted"));
    let set = VoiceClick::SetKey(secret(&typed));
    bus.publish(&VoiceAct { round, click: set }).unwrap();
    let round = dots(&views, typed.len());
    assert_eq!(sealed(&rig), Some(typed.clone().into_bytes()), "sealed");
    unseen(&rig, &boot, &[], std::slice::from_ref(&typed));
    rig.clip.set_text(Some(pasted.as_str()));
    let paste = VoiceClick::Paste(CloudLine::Key);
    bus.publish(&VoiceAct {
        round,
        click: paste,
    })
    .unwrap();
    dots(&views, pasted.len());
    assert_eq!(sealed(&rig), Some(pasted.clone().into_bytes()), "sealed");
    let asked: Vec<TextAsk> = asks.try_iter().collect();
    unseen(&rig, &boot, &asked, &[typed, pasted]);
    boot.kernel.stop_all();
}

#[test]
fn a_hidden_run_offers_no_service_and_reads_no_key_so_the_local_model_writes_it() {
    let rig = rig();
    let (mut boot, shelf) = keyed(&rig, &Run::hidden(APP.into()), &key("hidden"));
    let typed: TextDone = speak(&rig, &mut boot);
    assert!(typed.ok && typed.count > 0, "{typed:?}");
    assert!(rig.speech.runs()[0].ask.model.is_some(), "the local model");
    let calls = rig.cloud.calls();
    assert!(
        calls.runs.is_empty() && calls.tests.is_empty(),
        "nothing sent"
    );
    assert_eq!(
        shelf.key_uses(),
        0,
        "no key listed, read, saved or removed (ADR-0037)"
    );
    boot.kernel.stop_all();
}
