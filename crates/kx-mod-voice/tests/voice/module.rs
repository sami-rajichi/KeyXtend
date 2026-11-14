//! The module's start over the shared fake host: it reads `[voice]`, drops a bad pick alone with the settings notice,
//! and stays idle until its actor arrives.

use std::sync::{Arc, Mutex};

use kx_mod_voice::{ID, MANIFEST, VoiceModule, VoiceSettings};
use kx_module_api::{Host, Module, ModuleCx, Notice, NoticeArg, Subscription, args, keys};
use kx_test_support::host::FakeHost;

/// The notices published on `host`'s bus while the guard lives.
fn notices(host: &FakeHost) -> (Arc<Mutex<Vec<Notice>>>, Subscription) {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&heard);
    let guard = host
        .bus()
        .subscribe(move |n: &Notice| sink.lock().unwrap().push(n.clone()));
    (heard, guard.unwrap())
}

/// A `picks` table of `pairs`.
fn picks(pairs: &[(&str, &str)]) -> toml::Value {
    let table = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), toml::Value::from(*v)));
    toml::Value::Table(table.collect())
}

#[test]
fn it_starts_on_the_defaults_and_stops() {
    let mut host = FakeHost::new(&MANIFEST);
    let (heard, _guard) = notices(&host);
    let mut module = VoiceModule::new();
    assert_eq!(module.manifest().id.as_str(), "voice");
    assert!(module.start(&mut ModuleCx::new(&mut host)).is_ok());
    assert_eq!(host.lookups.get(), 0, "it asks for no service yet");
    assert!(heard.lock().unwrap().is_empty() && host.sink.writes().is_empty());
    module.stop();
}

#[test]
fn a_bad_pick_falls_back_alone_with_the_notice_and_is_written_back() {
    let mut host = FakeHost::new(&MANIFEST);
    host.put(
        "picks",
        picks(&[("fr", "whisper-medium-q5"), ("ar", "cohere-q8")]),
    );
    let (heard, _guard) = notices(&host);
    assert!(
        VoiceModule::new()
            .start(&mut ModuleCx::new(&mut host))
            .is_ok()
    );
    let mut want = Notice::about(keys::SETTINGS_RESET, ID);
    want.args
        .push((args::KEYS, NoticeArg::List(vec!["picks.fr".into()])));
    assert_eq!(*heard.lock().unwrap(), [want]);
    let kept = picks(&[("ar", "cohere-q8")]);
    let written = toml::Table::from_iter([("picks".to_owned(), kept)]);
    assert_eq!(host.sink.writes(), [(ID, written)]);
    let saved = host.sink.read_as::<VoiceSettings>().unwrap();
    assert_eq!(saved.picks.get("ar").map(String::as_str), Some("cohere-q8"));
    assert_eq!(saved.picks.len(), 1);
}

#[test]
fn a_bad_section_fails_the_start() {
    let mut host = FakeHost::new(&MANIFEST);
    host.put("job_mb", 1.into());
    let mut module = VoiceModule::new();
    assert!(module.start(&mut ModuleCx::new(&mut host)).is_err());
}
